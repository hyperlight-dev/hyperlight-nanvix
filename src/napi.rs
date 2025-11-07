use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::runtime::{Runtime, RuntimeConfig};

/// Node.js wrapper for hyperlight-nanvix Runtime
#[napi]
pub struct NanvixSandbox {
    runtime: Runtime,
}

/// Configuration options for creating a sandbox
#[napi(object)]
pub struct SandboxConfig {
    /// Directory for storing logs
    pub log_directory: Option<String>,
    /// Directory for temporary files
    pub tmp_directory: Option<String>,
}

/// Workload execution result
#[napi]
pub struct WorkloadResult {
    pub success: bool,
    pub error: Option<String>,
}

#[napi]
impl NanvixSandbox {
    /// Create a new sandbox instance
    #[napi(constructor)]
    pub fn new(config: Option<SandboxConfig>) -> Result<Self> {
        let runtime_config = match config {
            Some(cfg) => {
                let mut runtime_config = RuntimeConfig::new();
                if let Some(log_dir) = cfg.log_directory {
                    runtime_config = runtime_config.with_log_directory(log_dir);
                }
                if let Some(tmp_dir) = cfg.tmp_directory {
                    runtime_config = runtime_config.with_tmp_directory(tmp_dir);
                }
                runtime_config
            }
            None => RuntimeConfig::new(),
        };

        let runtime = Runtime::new(runtime_config)
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

        Ok(Self { runtime })
    }

    /// Run a workload in the sandbox
    #[napi]
    pub async fn run(&self, workload_path: String) -> Result<WorkloadResult> {
        // Run the workload using the existing runtime
        match self.runtime.run(&workload_path).await {
            Ok(()) => Ok(WorkloadResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(WorkloadResult {
                success: false,
                error: Some(format!("Workload execution failed: {}", e)),
            }),
        }
    }

    /// Clear the binary cache
    #[napi]
    pub async fn clear_cache(&self) -> Result<bool> {
        match self.runtime.clear_cache().await {
            Ok(()) => Ok(true),
            Err(e) => Err(Error::from_reason(format!("Failed to clear cache: {}", e))),
        }
    }
}
