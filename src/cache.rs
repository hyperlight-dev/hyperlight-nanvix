use std::path::PathBuf;

use nanvix::registry::Registry;

/// Default machine type for hyperlight-nanvix
const DEFAULT_MACHINE: &str = "hyperlight";

/// Default deployment type for hyperlight-nanvix
const DEFAULT_DEPLOYMENT: &str = "single-process";

/// Name of the nanvix-registry cache directory (matches the upstream constant).
const CACHE_DIRECTORY_NAME: &str = "nanvix-registry";

/// Return the base nanvix-registry cache directory.
///
/// Uses `dirs::cache_dir()` (e.g. `~/.cache` on Linux) and falls back to the
/// current directory when unavailable.
fn get_cache_directory() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CACHE_DIRECTORY_NAME)
}

/// Perform a pure filesystem probe for a cached binary.
///
/// Scans every `<machine>-<deployment>-*/bin/<binary_name>` path inside the
/// cache directory and returns the first match. This never hits the network.
fn find_in_local_cache(binary_name: &str) -> Option<String> {
    let cache_dir = get_cache_directory();
    let prefix = format!("{}-{}-", DEFAULT_MACHINE, DEFAULT_DEPLOYMENT);

    let entries = std::fs::read_dir(&cache_dir).ok()?;
    for entry in entries.flatten() {
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        if dir_name_str.starts_with(&prefix) && entry.file_type().is_ok_and(|ft| ft.is_dir()) {
            let candidate = entry.path().join("bin").join(binary_name);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Check if a binary exists in the local cache.
///
/// Pure filesystem probe with no network side effects.
pub fn is_binary_cached(binary_name: &str) -> bool {
    find_in_local_cache(binary_name).is_some()
}

/// Locate a cached binary, downloading it from the registry if not found locally.
///
/// First probes the local filesystem. If the binary is not present, falls back
/// to the nanvix registry which will download and cache it.
pub async fn get_cached_binary_path(binary_name: &str) -> Option<String> {
    // Try local filesystem first.
    if let Some(path) = find_in_local_cache(binary_name) {
        return Some(path);
    }

    // Fall back to the nanvix registry (downloads if needed).
    let registry = Registry::new(None);
    registry
        .get_cached_binary(DEFAULT_MACHINE, DEFAULT_DEPLOYMENT, binary_name)
        .await
        .ok()
}
