use hyperlight_nanvix::{RuntimeConfig, Sandbox};
use std::sync::Arc;

#[tokio::test]
async fn test_javascript_hello_world() {
    let config = RuntimeConfig::new();
    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    sandbox
        .run("guest-examples/hello.js")
        .await
        .expect("Failed to run JavaScript");
}

#[tokio::test]
async fn test_javascript_file_operations() {
    let config = RuntimeConfig::new();
    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    sandbox
        .run("guest-examples/file_ops.js")
        .await
        .expect("Failed to run JavaScript file ops");
}

#[tokio::test]
async fn test_python_hello_world() {
    let config = RuntimeConfig::new();
    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    sandbox
        .run("guest-examples/hello.py")
        .await
        .expect("Failed to run Python");
}

#[tokio::test]
async fn test_syscall_interception() {
    use hyperlight_nanvix::{SyscallAction, SyscallTable};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Custom openat handler that logs calls
    unsafe fn logging_openat_handler(
        _state: &(),
        dirfd: i32,
        pathname: *const i8,
        flags: i32,
        mode: u32,
    ) -> i32 {
        // Forward to real openat (simplified for testing)
        libc::openat(dirfd, pathname, flags, mode)
    }

    // Create syscall table with custom openat handler
    let mut syscall_table = SyscallTable::new(());
    syscall_table.openat = SyscallAction::Forward(logging_openat_handler);

    // Use unique directories to avoid conflicts between parallel tests
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config = RuntimeConfig::new()
        .with_syscall_table(Arc::new(syscall_table))
        .with_log_directory(format!("/tmp/hyperlight-syscall-test-{}", timestamp))
        .with_tmp_directory(format!("/tmp/hyperlight-syscall-tmp-{}", timestamp));

    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox with syscall table");

    sandbox
        .run("guest-examples/hello.js")
        .await
        .expect("Failed to run with syscall interception");
}

#[tokio::test]
async fn test_invalid_workload_type() {
    let config = RuntimeConfig::new();
    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    let result = sandbox.run("guest-examples/nonexistent.unknown").await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error
        .to_string()
        .contains("Could not determine workload type"));
}

#[tokio::test]
async fn test_nonexistent_file() {
    let config = RuntimeConfig::new();
    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    let result = sandbox.run("truly_nonexistent_file_12345.js").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_runtime_config_customization() {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use unique directories to avoid conflicts between parallel tests
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config = RuntimeConfig::new()
        .with_log_directory(format!("/tmp/hyperlight-test-log-{}", timestamp))
        .with_tmp_directory(format!("/tmp/hyperlight-test-tmp-{}", timestamp));

    let mut sandbox = Sandbox::new(config).expect("Failed to create sandbox with custom config");

    sandbox
        .run("guest-examples/hello.js")
        .await
        .expect("Failed with custom config");
}
