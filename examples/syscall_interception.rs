use anyhow::Result;
use hyperlight_nanvix::{RuntimeConfig, Sandbox, SyscallAction, SyscallTable};
use std::sync::Arc;

unsafe fn logging_openat_handler(
    _state: &(),
    dirfd: i32,
    pathname: *const i8,
    flags: i32,
    mode: u32,
) -> i32 {
    let pathname_str = std::ffi::CStr::from_ptr(pathname)
        .to_str()
        .unwrap_or("<invalid>");

    eprintln!(
        ">>> INTERCEPTED openat: dirfd={}, pathname='{}', flags={}, mode={}",
        dirfd, pathname_str, flags, mode
    );

    let result = libc::openat(dirfd, pathname, flags, mode);
    if result >= 0 {
        eprintln!(">>> openat SUCCESS: fd={}", result);
    } else {
        let errno = *libc::__errno_location();
        eprintln!(">>> openat FAILED: errno={}", errno);
    }
    result
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Running guest-examples/file_ops.js with openat syscall logging...");

    let mut syscall_table = SyscallTable::new(());
    syscall_table.openat = SyscallAction::Forward(logging_openat_handler);

    let config = RuntimeConfig::new()
        .with_syscall_table(Arc::new(syscall_table))
        .with_log_directory("/tmp/hyperlight-nanvix")
        .with_tmp_directory("/tmp/hyperlight-nanvix");

    let mut sandbox = Sandbox::new(config)?;

    match sandbox.run("guest-examples/file_ops.js").await {
        Ok(()) => {
            println!("Workload completed successfully with syscall interception!");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
