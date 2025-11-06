use std::path::PathBuf;

/// Get the default cache directory for nanvix registry
pub fn get_cache_directory() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&home_dir)
        .join(".cache")
        .join("nanvix-registry")
}

/// Get the binary cache directory
pub fn get_binary_cache_directory() -> PathBuf {
    get_cache_directory().join("bin")
}

/// Check if a binary exists in the cache and return its path if found
pub async fn get_cached_binary_path(binary_name: &str) -> Option<String> {
    let cache_path = get_binary_cache_directory().join(binary_name);

    if tokio::fs::metadata(&cache_path).await.is_ok() {
        Some(cache_path.to_string_lossy().to_string())
    } else {
        None
    }
}

/// Check if a binary exists in the cache (synchronous version for setup command)
pub fn is_binary_cached(binary_name: &str) -> bool {
    let cache_path = get_binary_cache_directory().join(binary_name);
    cache_path.exists()
}