use anyhow::Result;
use hyperlight_nanvix::{RuntimeConfig, Sandbox};
use nanvix::log;
use std::env;
use std::path::Path;

/// Default log-level (overridden by RUST_LOG environment variable if set).
const DEFAULT_LOG_LEVEL: &str = "info";

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments first
    let args: Vec<String> = env::args().collect();

    // Check for verbose flag
    let verbose = args.contains(&"--verbose".to_string());

    // Find the script argument (first non-flag argument)
    let script_arg = args
        .iter()
        .position(|arg| !arg.starts_with("--") && !arg.ends_with("hyperlight-nanvix"));

    let script_path = if let Some(idx) = script_arg {
        Path::new(&args[idx])
    } else {
        eprintln!("Usage: {} [--verbose] <script_path>", args[0]);
        eprintln!("Supported file types: .js, .mjs (JavaScript), .py (Python)");
        eprintln!("Options:");
        eprintln!("  --verbose    Show detailed nanvix logging");
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
