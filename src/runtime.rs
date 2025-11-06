use anyhow::Result;
use std::path::Path;

use nanvix::log;
use nanvix::registry::Registry;
use nanvix::sandbox_cache::SandboxCacheConfig;
use nanvix::terminal::Terminal;

/// Supported workload types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    JavaScript,
    Python,
}

impl WorkloadType {
    /// Get the binary name for this workload type
    pub fn binary_name(&self) -> &'static str {
        match self {
            WorkloadType::JavaScript => "qjs",
            WorkloadType::Python => "python3",
        }
    }

    /// Get the file extensions associated with this workload type
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            WorkloadType::JavaScript => &["js", "mjs"],
            WorkloadType::Python => &["py"],
        }
    }

    /// Detect workload type from file extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let extension = path.as_ref().extension()?.to_str()?.to_lowercase();

        match extension.as_str() {
            "js" | "mjs" => Some(WorkloadType::JavaScript),
            "py" => Some(WorkloadType::Python),
            _ => None,
        }
    }
}

/// Runtime configuration for hyperlight-nanvix
#[derive(Clone)]
pub struct RuntimeConfig {
    /// Optional custom syscall table
    pub syscall_table: Option<std::sync::Arc<nanvix::sandbox::SyscallTable<()>>>,
    /// Directory for storing logs
    pub log_directory: String,
    /// Directory for temporary files
    pub tmp_directory: String,
    /// Directory containing toolchain binaries
    pub toolchain_binary_directory: String,
}

impl std::fmt::Debug for RuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field(
                "syscall_table",
                &self.syscall_table.as_ref().map(|_| "SyscallTable<()>"),
            )
            .field("log_directory", &self.log_directory)
            .field("tmp_directory", &self.tmp_directory)
            .field(
                "toolchain_binary_directory",
                &self.toolchain_binary_directory,
            )
            .finish()
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            syscall_table: None,
            log_directory: "/tmp/hyperlight-nanvix".to_string(),
            tmp_directory: "/tmp/hyperlight-nanvix".to_string(),
            toolchain_binary_directory: "/tmp/hyperlight-nanvix/toolchain".to_string(),
        }
    }
}

impl RuntimeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_syscall_table(
        mut self,
        table: std::sync::Arc<nanvix::sandbox::SyscallTable<()>>,
    ) -> Self {
        self.syscall_table = Some(table);
        self
    }

    pub fn with_log_directory<S: Into<String>>(mut self, dir: S) -> Self {
        self.log_directory = dir.into();
        self
    }

    pub fn with_tmp_directory<S: Into<String>>(mut self, dir: S) -> Self {
        self.tmp_directory = dir.into();
        self
    }

    pub fn with_toolchain_directory<S: Into<String>>(mut self, dir: S) -> Self {
        self.toolchain_binary_directory = dir.into();
        self
    }
}

/// Runtime for executing workloads in Nanvix sandboxes
pub struct Runtime {
    config: RuntimeConfig,
    registry: Registry,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let registry = Registry::new();
        Ok(Self { config, registry })
    }

    /// Check if a binary exists in the cache and return its path if found
    async fn get_cached_binary_path(&self, binary_name: &str) -> Option<String> {
        // Check the common cache location for nanvix registry
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_path = std::path::Path::new(&home_dir)
            .join(".cache")
            .join("nanvix-registry")
            .join("bin")
            .join(binary_name);

        if tokio::fs::metadata(&cache_path).await.is_ok() {
            Some(cache_path.to_string_lossy().to_string())
        } else {
            None
        }
    }



    /// Clear the nanvix registry cache to force fresh downloads
    pub async fn clear_cache(&self) -> Result<()> {
        log::info!("Clearing nanvix registry cache...");
        self.registry.clear_cache().await?;
        log::info!("Cache cleared successfully");
        Ok(())
    }

    /// Run a workload
    pub async fn run<P: AsRef<Path>>(&self, workload_path: P) -> Result<()> {
        let workload_path = workload_path.as_ref();

        // Determine workload type from file extension
        let workload_type = WorkloadType::from_path(workload_path).ok_or_else(|| {
            anyhow::anyhow!("Could not determine workload type for {:?}", workload_path)
        })?;

        // Use hardcoded values for machine and deployment type (hyperlight single-process)
        let machine_type = "hyperlight";
        let deployment_type = "single-process";

        // Check if binaries are cached and get paths directly if available
        let binary_path = if let Some(cached_path) = self.get_cached_binary_path(workload_type.binary_name()).await {
            log::info!(
                "Using cached {} binary: {}",
                workload_type.binary_name(),
                cached_path
            );
            cached_path
        } else {
            log::info!(
                "{} not cached, downloading from registry...",
                workload_type.binary_name()
            );
            self.registry
                .get_cached_binary(machine_type, deployment_type, workload_type.binary_name())
                .await?
        };

        // Get kernel path for terminal configuration
        let kernel_path = if let Some(cached_path) = self.get_cached_binary_path("kernel.elf").await {
            log::info!("Using cached kernel binary: {}", cached_path);
            cached_path
        } else {
            log::info!("kernel.elf not cached, downloading from registry...");
            self.registry
                .get_cached_binary(machine_type, deployment_type, "kernel.elf")
                .await?
        };

        // Ensure the temporary directory exists for socket creation
        std::fs::create_dir_all(&self.config.tmp_directory)?;
        std::fs::create_dir_all(&self.config.log_directory)?;

        // Use syscall table provided by embedder, or create default one
        let syscall_table = self.config.syscall_table.clone().or_else(|| {
            use nanvix::sandbox::SyscallTable;
            Some(std::sync::Arc::new(SyscallTable::new(())))
        });

        // Convert workload path to absolute path before potentially changing directory
        let absolute_workload_path = workload_path
            .canonicalize()
            .unwrap_or_else(|_| {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(workload_path)
            })
            .to_string_lossy()
            .to_string();

        // For Python workloads, change to the registry directory
        let original_dir = if matches!(workload_type, WorkloadType::Python) {
            let current_dir = std::env::current_dir().ok();
            let registry_base = std::path::Path::new(&binary_path)
                .parent()
                .and_then(|p| p.parent());

            if let Some(base_path) = registry_base {
                if let Err(e) = std::env::set_current_dir(base_path) {
                    log::warn!(
                        "Failed to change directory to {}: {}",
                        base_path.display(),
                        e
                    );
                } else {
                    log::info!("Changed working directory to: {}", base_path.display());
                }
            } else {
                log::warn!("Could not determine registry base directory from binary path: {}", binary_path);
            }
            current_dir
        } else {
            None
        };

        // Configure sandbox cache
        let console_log_path = format!("{}/guest-console.log", &self.config.log_directory);
        let console_file = Some(console_log_path.clone());

        let sandbox_cache_config = SandboxCacheConfig::new(
            nanvix::syscomm::SocketType::Unix,
            nanvix::syscomm::SocketType::Unix,
            nanvix::syscomm::SocketType::Unix,
            console_file,
            None,
            &kernel_path,
            syscall_table,
            &self.config.toolchain_binary_directory,
            &self.config.log_directory,
            false,
            "/tmp/hyperlight-nanvix/snapshot.bin",
            &self.config.tmp_directory,
        );

        // Create terminal
        let mut terminal: Terminal<()> = Terminal::new(sandbox_cache_config);

        // Prepare execution paths
        let (effective_binary_path, effective_script_args) =
            if matches!(workload_type, WorkloadType::Python) {
                let script_args = format!("-S -I {}", absolute_workload_path);
                ("bin/python3".to_string(), script_args)
            } else {
                let (script_args, _) = self.prepare_script_args(workload_type, workload_path)?;
                (binary_path.clone(), script_args)
            };

        // Prepare execution metadata
        let script_name = workload_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid workload path: {:?}", workload_path))?
            .to_string();

        let unique_app_name = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
            .to_string();

        log::info!(
            "Running {} workload: {:?}",
            workload_type.binary_name(),
            workload_path
        );
        log::debug!("Binary path: {}", effective_binary_path);
        log::debug!("Script args: {}", effective_script_args);

        // Execute workload
        let _execution_result = terminal
            .run(
                Some(&script_name),
                Some(&unique_app_name),
                &effective_binary_path,
                &effective_script_args,
            )
            .await?;

        // Restore original working directory if we changed it for Python
        if let Some(original_dir) = original_dir {
            if let Err(e) = std::env::set_current_dir(original_dir) {
                log::warn!("Failed to restore original working directory: {}", e);
            }
        }

        Ok(())
    }

    fn prepare_script_args(
        &self,
        workload_type: WorkloadType,
        workload_path: &Path,
    ) -> Result<(String, String)> {
        let script_name = workload_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid workload path: {:?}", workload_path))?
            .to_string();

        let script_args = match workload_type {
            WorkloadType::JavaScript => {
                let mut args = workload_path.to_string_lossy().to_string();
                args.insert_str(0, "-m ");
                args
            }
            WorkloadType::Python => {
                format!("-S -I {}", workload_path.to_string_lossy())
            }
        };

        Ok((script_args, script_name))
    }
}
