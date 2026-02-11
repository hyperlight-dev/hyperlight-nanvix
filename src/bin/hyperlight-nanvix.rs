use anyhow::Result;
use clap::{Parser, Subcommand};
use hyperlight_nanvix::{cache, RuntimeConfig, Sandbox};
use nanvix::log;
use std::path::PathBuf;

/// A Hyperlight VMM wrapper with out-of-the-box support for running Nanvix microkernel guests
#[derive(Parser)]
#[command(name = "hyperlight-nanvix")]
#[command(about = "Run scripts in a Nanvix microkernel guest")]
#[command(
    after_help = "Supported file types: .js, .mjs (JavaScript), .py (Python), .elf, .o (Binary)"
)]
struct Cli {
    /// Show detailed nanvix logging
    #[arg(long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the script to run
    #[arg(value_name = "SCRIPT")]
    script_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Download nanvix registry and show compilation instructions
    SetupRegistry,
    /// Clear the nanvix registry cache
    ClearRegistry,
}

/// Default log-level (overridden by RUST_LOG environment variable if set).
const DEFAULT_LOG_LEVEL: &str = "info";

async fn setup_registry_command() -> Result<()> {
    println!("Setting up Nanvix registry...");

    // Check cache status first using local filesystem probes
    let kernel_cached = cache::is_binary_cached("kernel.elf");
    let qjs_cached = cache::is_binary_cached("qjs");
    let python_cached = cache::is_binary_cached("python3");

    if kernel_cached && qjs_cached && python_cached {
        println!("Registry already set up at ~/.cache/nanvix-registry/");
    } else {
        // Download missing binaries via get_cached_binary_path (local first, registry fallback)
        if !kernel_cached {
            print!("Downloading kernel.elf... ");
            cache::get_cached_binary_path("kernel.elf")
                .await
                .ok_or_else(|| anyhow::anyhow!("Failed to download kernel.elf"))?;
            println!("done");
        } else {
            println!("kernel.elf already cached");
        }

        if !qjs_cached {
            print!("Downloading qjs binary... ");
            cache::get_cached_binary_path("qjs")
                .await
                .ok_or_else(|| anyhow::anyhow!("Failed to download qjs"))?;
            println!("done");
        } else {
            println!("qjs already cached");
        }

        if !python_cached {
            print!("Downloading python3 binary... ");
            cache::get_cached_binary_path("python3")
                .await
                .ok_or_else(|| anyhow::anyhow!("Failed to download python3"))?;
            println!("done");
        } else {
            println!("python3 already cached");
        }

        println!("\nRegistry setup complete at ~/.cache/nanvix-registry/");
    }

    println!("\nTo compile and run C/C++ programs, see the README:");
    println!(
        "https://github.com/hyperlight-dev/hyperlight-nanvix?tab=readme-ov-file#c--c-programs"
    );

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

    println!("Run 'cargo run -- setup-registry' to re-download if needed.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(command) = cli.command {
        return match command {
            Commands::SetupRegistry => setup_registry_command().await,
            Commands::ClearRegistry => clear_registry_command().await,
        };
    }

    // Require script path for default operation
    let script_path = cli.script_path.unwrap_or_else(|| {
        eprintln!("error: the following required arguments were not provided:\n  <SCRIPT>\n");
        eprintln!("Usage: hyperlight-nanvix [OPTIONS] <SCRIPT>");
        eprintln!("       hyperlight-nanvix setup-registry");
        eprintln!("       hyperlight-nanvix clear-registry");
        eprintln!("\nFor more information, try '--help'.");
        std::process::exit(1);
    });

    // Check if file exists
    if !script_path.exists() {
        eprintln!("Error: File {:?} does not exist", script_path);
        std::process::exit(1);
    }

    // Initialize nanvix logging only when --verbose is specified
    if cli.verbose {
        log::init(
            false,
            DEFAULT_LOG_LEVEL,
            "/tmp/hyperlight-nanvix".to_string(),
            None,
        );
    }

    // Create runtime configuration
    let config = RuntimeConfig::new()
        .with_log_directory("/tmp/hyperlight-nanvix")
        .with_tmp_directory("/tmp/hyperlight-nanvix");

    // Create Sandbox instance
    let mut sandbox = Sandbox::new(config)?;

    // Run the workload
    match sandbox.run(&script_path).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error running workload: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
