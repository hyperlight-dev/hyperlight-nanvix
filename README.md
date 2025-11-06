# hyperlight-nanvix

> ⚠️ **Note**: This wrapper currently only supports Hyperlight's KVM backend

A Hyperlight VMM wrapper with out-of-the-box support for running Nanvix microkernel guests.

## Features

- **JavaScript & Python Support**: Run `.js`, `.mjs`, and `.py` files in Nanvix sandboxes
- **Syscall Interception**: Custom syscall handlers using Nanvix's SyscallTable
- **Async Runtime**: Built on Tokio for non-blocking operations
- **Automatic Registry**: Downloads and caches binaries from Nanvix releases

## Quick Start

### Command Line

```bash
# JavaScript
cargo run -- guest-examples/hello.js

# Python
cargo run -- guest-examples/hello.py
```

### Library Usage

```rust
use hyperlight_nanvix::{Sandbox, RuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RuntimeConfig::new()
        .with_log_directory("/tmp/hyperlight-nanvix")
        .with_tmp_directory("/tmp/hyperlight-nanvix");

    let mut sandbox = Sandbox::new(config)?;
    sandbox.run("guest-examples/hello.js").await?;
    Ok(())
}
```

### Syscall Interception

```rust
use hyperlight_nanvix::{Sandbox, RuntimeConfig, SyscallTable, SyscallAction};
use std::sync::Arc;

unsafe fn custom_openat(
    _state: &(),
    dirfd: i32,
    pathname: *const i8,
    flags: i32,
    mode: u32,
) -> i32 {
    println!("Intercepted openat call");
    libc::openat(dirfd, pathname, flags, mode)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut syscall_table = SyscallTable::new(());
    syscall_table.openat = SyscallAction::Forward(custom_openat);

    let config = RuntimeConfig::new()
        .with_syscall_table(Arc::new(syscall_table));

    let mut sandbox = Sandbox::new(config)?;
    sandbox.run("guest-examples/hello.js").await?;
    Ok(())
}
```

## Supported File Types

- **JavaScript**: `.js`, `.mjs` (via QuickJS)
- **Python**: `.py` (via Python 3.12)

