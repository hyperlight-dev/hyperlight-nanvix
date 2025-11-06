use anyhow::Result;
use std::path::Path;

pub mod cache;
pub mod runtime;

#[cfg(test)]
mod unit_tests;

use runtime::Runtime;
pub use runtime::{RuntimeConfig, WorkloadType};

// Re-export Nanvix sandbox types for syscall table configuration
pub use nanvix::sandbox::{SyscallAction, SyscallTable};

/// Main entry point for creating and running Nanvix sandboxes
pub struct Sandbox {
    runtime: Runtime,
}

impl Sandbox {
    /// Create a new Sandbox instance
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let runtime = Runtime::new(config)?;
        Ok(Self { runtime })
    }

    /// Run a workload in the sandbox
    pub async fn run<P: AsRef<Path>>(&mut self, workload_path: P) -> Result<()> {
        self.runtime.run(workload_path).await
    }

    /// Clear the binary cache to force fresh downloads on next run
    pub async fn clear_cache(&self) -> Result<()> {
        self.runtime.clear_cache().await
    }
}
