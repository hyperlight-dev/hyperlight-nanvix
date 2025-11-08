use anyhow::Result;
use hyperlight_nanvix::{cache, RuntimeConfig, Sandbox};
use nanvix::log;
use nanvix::registry::Registry;
use std::env;
use std::path::Path;

/// Default log-level (overridden by RUST_LOG environment variable if set).
const DEFAULT_LOG_LEVEL: &str = "info";

async fn setup_registry_command() -> Result<()> {
    println!("Setting up Nanvix registry...");
    
    // Check cache status first using shared cache utilities
    let kernel_cached = cache::is_binary_cached("kernel.elf");
    let qjs_cached = cache::is_binary_cached("qjs");
    let python_cached = cache::is_binary_cached("python3");
    
    if kernel_cached && qjs_cached && python_cached {
        println!("Registry already set up at ~/.cache/nanvix-registry/");
    } else {
        // Trigger registry download by requesting key binaries
        let registry = Registry::new();
        
        if !kernel_cached {
            print!("Downloading kernel.elf... ");
            let _kernel = registry.get_cached_binary("hyperlight", "single-process", "kernel.elf").await?;
            println!("done");
        } else {
            println!("kernel.elf already cached");
        }
        
        if !qjs_cached {
            print!("Downloading qjs binary... ");
            let _qjs = registry.get_cached_binary("hyperlight", "single-process", "qjs").await?;
            println!("done");
        } else {
            println!("qjs already cached");
        }
        
        if !python_cached {
            print!("Downloading python3 binary... ");
            let _python = registry.get_cached_binary("hyperlight", "single-process", "python3").await?;
            println!("done");
        } else {
            println!("python3 already cached");
        }
        
        println!("\nRegistry setup complete at ~/.cache/nanvix-registry/");
    }
    
    println!("\nTo compile C/C++ programs:");
    println!("docker pull nanvix/toolchain:latest");
    
    println!("\nCompilation examples:");
    println!("# C program:");
    println!("docker run --rm -v \"$(pwd):/mnt\" -v \"$HOME/.cache/nanvix-registry:/nanvix-registry:ro\" nanvix/toolchain:latest /bin/bash -l -c 'cd /mnt && /opt/nanvix/bin/i686-nanvix-gcc -z noexecstack -T /nanvix-registry/lib/user.ld -o hello-c hello-c.c -Wl,--start-group /nanvix-registry/lib/libposix.a /opt/nanvix/i686-nanvix/lib/libc.a -Wl,--end-group'");
    
    println!("\n# C++ program:");
    println!("docker run --rm -v \"$(pwd):/mnt\" -v \"$HOME/.cache/nanvix-registry:/nanvix-registry:ro\" nanvix/toolchain:latest /bin/bash -l -c 'cd /mnt && /opt/nanvix/bin/i686-nanvix-g++ -z noexecstack -T /nanvix-registry/lib/user.ld -o hello-cpp hello-cpp.cpp -Wl,--start-group /nanvix-registry/lib/libposix.a /opt/nanvix/i686-nanvix/lib/libc.a /opt/nanvix/i686-nanvix/lib/libstdc++.a -Wl,--end-group'");
    
    println!("\nAfter compilation, run binaries with:");
    println!("cargo run -- ./hello");
    
    Ok(())
}

async fn clear_registry_command() -> Result<()> {
    println!("Clearing Nanvix registry cache...");
    
    // Create a minimal config to instantiate the Sandbox for cache clearing
    let config = RuntimeConfig::new();
    let sandbox = Sandbox::new(config)?;
    
    match sandbox.clear_cache().await {
        Ok(()) => println!("Cache cleared successfully"),
        Err(e) => {
            eprintln!("Error clearing cache: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("Run 'cargo run -- --setup-registry' to re-download if needed.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments first
    let args: Vec<String> = env::args().collect();

    // Check for flags
    let verbose = args.contains(&"--verbose".to_string());
    let setup_registry = args.contains(&"--setup-registry".to_string());
    let clear_registry = args.contains(&"--clear-registry".to_string());

    // Handle setup-registry command
    if setup_registry {
        return setup_registry_command().await;
    }

    // Handle clear-registry command
    if clear_registry {
        return clear_registry_command().await;
    }

    // Find the script argument (first non-flag argument)
    let script_arg = args
        .iter()
        .position(|arg| !arg.starts_with("--") && !arg.ends_with("hyperlight-nanvix"));

    let script_path = if let Some(idx) = script_arg {
        Path::new(&args[idx])
    } else {
        eprintln!("Usage: {} [--verbose] <script_path>", args[0]);
        eprintln!("       {} --setup-registry", args[0]);
        eprintln!("       {} --clear-registry", args[0]);
        eprintln!("Supported file types: .js, .mjs (JavaScript), .py (Python), .elf, .o (Binary)");
        eprintln!("Options:");
        eprintln!("  --verbose         Show detailed nanvix logging");
        eprintln!("  --setup-registry  Download nanvix registry and show compilation instructions");
        eprintln!("  --clear-registry  Clear the nanvix registry cache");
        std::process::exit(1);
    };

    // Check if file exists
    if !script_path.exists() {
        eprintln!("Error: File {:?} does not exist", script_path);
        std::process::exit(1);
    }

    // Initialize nanvix logging only when --verbose is specified
    if verbose {
        log::init(
            false,
            DEFAULT_LOG_LEVEL,
            "/tmp/hyperlight-nanvix".to_string(),
        );
    }

    // Create runtime configuration
    let config = RuntimeConfig::new()
        .with_log_directory("/tmp/hyperlight-nanvix")
        .with_tmp_directory("/tmp/hyperlight-nanvix");

    // Create Sandbox instance
    let mut sandbox = Sandbox::new(config)?;

    // Run the workload
    match sandbox.run(script_path).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error running workload: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
