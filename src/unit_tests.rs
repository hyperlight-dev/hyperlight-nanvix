#[cfg(test)]
mod unit_tests {
    use crate::runtime::{Runtime, WorkloadType};
    use crate::*;
    use std::sync::Arc;

    #[test]
    fn test_workload_type_detection() {
        assert_eq!(
            WorkloadType::from_path("script.js"),
            Some(WorkloadType::JavaScript)
        );
        assert_eq!(
            WorkloadType::from_path("module.mjs"),
            Some(WorkloadType::JavaScript)
        );
        assert_eq!(
            WorkloadType::from_path("script.py"),
            Some(WorkloadType::Python)
        );
        assert_eq!(WorkloadType::from_path("unknown.txt"), None);
    }

    #[test]
    fn test_workload_type_binary_names() {
        assert_eq!(WorkloadType::JavaScript.binary_name(), "qjs");
        assert_eq!(WorkloadType::Python.binary_name(), "python3");
    }

    #[test]
    fn test_runtime_config_builder() {
        let config = RuntimeConfig::new()
            .with_log_directory("/custom/log")
            .with_tmp_directory("/custom/tmp")
            .with_toolchain_directory("/custom/toolchain");

        assert_eq!(config.log_directory, "/custom/log");
        assert_eq!(config.tmp_directory, "/custom/tmp");
        assert_eq!(config.toolchain_binary_directory, "/custom/toolchain");
        assert!(config.syscall_table.is_none());
    }

    #[test]
    fn test_runtime_config_with_syscall_table() {
        let syscall_table = Arc::new(SyscallTable::new(()));
        let config = RuntimeConfig::new().with_syscall_table(syscall_table.clone());

        assert!(config.syscall_table.is_some());
        assert!(Arc::ptr_eq(
            config.syscall_table.as_ref().unwrap(),
            &syscall_table
        ));
    }

    #[test]
    fn test_sandbox_creation() {
        let config = RuntimeConfig::new();
        let sandbox = Sandbox::new(config);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_runtime_creation() {
        let config = RuntimeConfig::new();
        let runtime = Runtime::new(config);
        assert!(runtime.is_ok());
    }
}
