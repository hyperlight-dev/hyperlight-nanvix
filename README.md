# hyperlight-nanvix

> ⚠️ **Note**: This wrapper currently only supports Hyperlight's KVM backend

A Hyperlight VMM wrapper for running JavaScript, Python, C, and C++ programs in Nanvix microkernel sandboxes.

## Quick Start

### Rust CLI

Run scripts directly:

```bash
# JavaScript
cargo run -- guest-examples/hello.js

# Python  
cargo run -- guest-examples/hello.py
```

### Node.js

Run from JavaScript/Node.js:

```bash
npm run build
node examples/napi.js
```

### C & C++ Programs

For compiled languages, you'll need to compile first, then run.

#### Setup (one-time)

```bash
cargo run -- --setup-registry
```

This downloads the toolchain and runtime files to `~/.cache/nanvix-registry/`.

#### Compile

```bash
cd guest-examples

# C program
docker run --rm \
    -v "$(pwd):/mnt" \
    -v "$HOME/.cache/nanvix-registry:/nanvix-registry:ro" \
    nanvix/toolchain:latest \
    /bin/bash -l -c 'cd /mnt && /opt/nanvix/bin/i686-nanvix-gcc \
        -z noexecstack -T /nanvix-registry/lib/user.ld \
        -o hello-c hello-c.c \
        -Wl,--start-group /nanvix-registry/lib/libposix.a \
        /opt/nanvix/i686-nanvix/lib/libc.a -Wl,--end-group'

# C++ program  
docker run --rm \
    -v "$(pwd):/mnt" \
    -v "$HOME/.cache/nanvix-registry:/nanvix-registry:ro" \
    nanvix/toolchain:latest \
    /bin/bash -l -c 'cd /mnt && /opt/nanvix/bin/i686-nanvix-g++ \
        -z noexecstack -T /nanvix-registry/lib/user.ld \
        -o hello-cpp hello-cpp.cpp \
        -Wl,--start-group /nanvix-registry/lib/libposix.a \
        /opt/nanvix/i686-nanvix/lib/libc.a /opt/nanvix/i686-nanvix/lib/libstdc++.a \
        -Wl,--end-group'
```

#### Run

```bash
# From the project root
cargo run -- guest-examples/hello-c
cargo run -- guest-examples/hello-cpp
```

## Library Usage

### Rust

```rust
use hyperlight_nanvix::{Sandbox, RuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RuntimeConfig::new()
        .with_log_directory("/tmp/hyperlight-nanvix")
        .with_tmp_directory("/tmp/hyperlight-nanvix");

    let mut sandbox = Sandbox::new(config)?;
    
    // Works with any supported file type
    sandbox.run("guest-examples/hello.js").await?;    // JavaScript
    sandbox.run("guest-examples/hello.py").await?;    // Python  
    sandbox.run("guest-examples/hello-c").await?;     // C binary
    sandbox.run("guest-examples/hello-cpp").await?;   // C++ binary
    
    Ok(())
}
```

### Node.js

Basic usage with the library:
```javascript
const { NanvixSandbox } = require('hyperlight-nanvix');

const sandbox = new NanvixSandbox({
    logDirectory: '/tmp/hyperlight-nanvix',
    tmpDirectory: '/tmp/hyperlight-nanvix'
});

const result = await sandbox.run('guest-examples/hello.js');
if (result.success) {
    console.log('Execution completed');
}
```

To embed in your own project:
```bash
npm run build
npm pack
npm install hyperlight-nanvix-0.1.0.tgz
```

See `examples/ai-generated-scripts/` for a complete example that generates and executes AI code safely.

## What's Available

### Languages

- **JavaScript**: `.js`, `.mjs` files (via QuickJS)
- **Python**: `.py` files (via Python 3.12)
- **C/C++**: Compiled binaries (requires Docker compilation)

### C/C++ Libraries

Standard functions work as expected:
- **I/O**: `printf`, `scanf`, `fopen`, `fclose`, `fread`, `fwrite`
- **Memory**: `malloc`, `free`
- **Strings**: `strlen`, `strcpy`, `strcmp`
- **C++**: Classes, STL containers (`std::vector`, `std::string`), `iostream`

Available libraries:
- **Core**: `libposix.a`, `libc.a`, `libm.a`
- **C++**: `libstdc++.a`, `libsupc++.a` 
- **Crypto**: `libcrypto.a`, `libssl.a` (OpenSSL)
- **Compression**: `libz.a` (zlib)
- **Math/Science**: `libopenblas.a` (OpenBLAS for linear algebra)

## Examples

Check `guest-examples/` for sample programs:
- `hello.js` - JavaScript with JSON and functions
- `hello.py` - Python with modules and data structures  
- `hello-c.c` - C program with basic operations
- `hello-cpp.cpp` - C++ program with classes and STL
- `file_ops.js` - JavaScript demonstrating file operations

## Syscall Interception

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
    sandbox.run("guest-examples/hello-c").await?;
    Ok(())
}
```

Run an example with syscall interception:

```bash
cargo run --example syscall_interception
```

## Troubleshooting

**Clear cache and re-download:**
```bash
cargo run -- --clear-registry
cargo run -- --setup-registry
```

**Clean socket files if networking issues occur:**
```bash
rm -rf /tmp/hyperlight-nanvix/*
```
