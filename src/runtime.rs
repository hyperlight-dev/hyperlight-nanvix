use anyhow::Result;
use std::path::Path;

use nanvix::log;
use nanvix::registry::Registry;
use nanvix::sandbox_cache::SandboxCacheConfig;
use nanvix::terminal::Terminal;

use crate::cache;

/// Supported workload types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    JavaScript,
    Python,
    Binary,
}

impl WorkloadType {
    /// Get the interpreter binary name for this workload type
    pub fn binary_name(&self) -> &'static str {
        match self {
            WorkloadType::JavaScript => "qjs",
            WorkloadType::Python => "python3",
            WorkloadType::Binary => "binary", // No interpreter needed for binaries
        }
    }

    /// Get the registry package name for this workload type.
    /// Returns `None` for workload types that don't require a package installation.
    pub fn package_name(&self) -> Option<&'static str> {
        match self {
            WorkloadType::JavaScript => Some("quickjs"),
            WorkloadType::Python => Some("python"),
            WorkloadType::Binary => None,
        }
    }

    /// Get the file extensions associated with this workload type
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            WorkloadType::JavaScript => &["js", "mjs"],
            WorkloadType::Python => &["py"],
            WorkloadType::Binary => &["elf", "o"],
        }
    }

    /// Detect workload type from file extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path_ref = path.as_ref();

        if let Some(extension) = path_ref.extension() {
            let ext_str = extension.to_str()?.to_lowercase();
            match ext_str.as_str() {
                "js" | "mjs" => Some(WorkloadType::JavaScript),
                "py" => Some(WorkloadType::Python),
                "elf" | "o" => Some(WorkloadType::Binary),
                _ => None,
            }
        } else {
            // Check if it's an executable binary without extension
            if path_ref.is_file() {
                // Simple heuristic: if it has no extension and is a file, treat as binary
                Some(WorkloadType::Binary)
            } else {
                None
            }
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
            .finish()
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate unique directory suffix using timestamp and PID to avoid conflicts
        // when multiple tests or instances run in parallel
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = process::id();
        let unique_suffix = format!("{}-{}", unique_id, pid);

        Self {
            syscall_table: None,
            log_directory: format!("/tmp/hyperlight-nanvix-{}", unique_suffix),
            tmp_directory: format!("/tmp/hyperlight-nanvix-{}", unique_suffix),
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
}

/// Runtime for executing workloads in Nanvix sandboxes
pub struct Runtime {
    config: RuntimeConfig,
    registry: Registry,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let registry = Registry::new(None);
        Ok(Self { config, registry })
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

        // Verify the workload file exists before proceeding
        if !workload_path.exists() {
            anyhow::bail!("Workload file not found: {:?}", workload_path);
        }

        // Use hardcoded values for machine and deployment type (hyperlight single-process)
        let machine_type = "hyperlight";
        let deployment_type = "single-process";

        // Install the required package (and its dependencies) for scripted workloads,
        // but only when the interpreter binary is not already present in the cache.
        // This avoids unnecessary I/O and network calls on the common (cached) path.
        if let Some(package_name) = workload_type.package_name() {
            if !cache::is_binary_cached(workload_type.binary_name()) {
                log::info!("Installing package '{}' and dependencies...", package_name);
                self.registry
                    .install(machine_type, deployment_type, package_name, true)
                    .await?;
            }
        }

        // Get interpreter binary (only needed for scripted workloads)
        let binary_path = if matches!(workload_type, WorkloadType::Binary) {
            // For binary workloads, we don't need an interpreter
            String::new()
        } else {
            cache::get_cached_binary_path(workload_type.binary_name())
                .await
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Failed to locate {} binary in cache or registry",
                        workload_type.binary_name()
                    )
                })?
        };

        // Get kernel path for terminal configuration
        let kernel_path = cache::get_cached_binary_path("kernel.elf")
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to locate kernel.elf in cache or registry"))?;

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
                log::warn!(
                    "Could not determine registry base directory from binary path: {}",
                    binary_path
                );
            }
            current_dir
        } else {
            None
        };

        // Configure sandbox cache
        let console_log_path = format!("{}/guest-console.log", &self.config.log_directory);
        let console_file = Some(console_log_path.clone());

        // Use tmp_directory for toolchain and snapshot paths to ensure uniqueness
        let toolchain_path = format!("{}/toolchain", &self.config.tmp_directory);
        let snapshot_path = format!("{}/snapshot.bin", &self.config.tmp_directory);

        let sandbox_cache_config = SandboxCacheConfig::new(
            nanvix::syscomm::SocketType::Unix,
            nanvix::syscomm::SocketType::Unix,
            nanvix::syscomm::SocketType::Unix,
            console_file,
            None,
            None,
            0,
            &kernel_path,
            syscall_table,
            &toolchain_path,
            &self.config.log_directory,
            false,
            &snapshot_path,
            &self.config.tmp_directory,
        );

        // Create terminal
        let mut terminal: Terminal<()> = Terminal::new(sandbox_cache_config);

        // Prepare execution paths and metadata
        let (script_args, script_name) =
            self.prepare_script_args(workload_type, Path::new(&absolute_workload_path))?;
        let effective_binary_path = match workload_type {
            WorkloadType::Python => "bin/python3".to_string(),
            WorkloadType::Binary => absolute_workload_path.clone(),
            _ => binary_path.clone(),
        };
        let effective_script_args = match workload_type {
            WorkloadType::Binary => String::new(), // No args for binary execution
            _ => script_args,
        };

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
        terminal
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
            WorkloadType::Binary => {
                // Binary files are executed directly, no script args needed
                String::new()
            }
        };

        Ok((script_args, script_name))
    }
}
