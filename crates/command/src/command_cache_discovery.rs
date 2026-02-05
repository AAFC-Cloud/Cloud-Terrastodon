use crate::CacheKey;
use chrono::DateTime;
use chrono::Local;
use eyre::Context;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Walk the provided commands directory and discover cache entries by locating
/// `timestamp.txt` files. For each found `timestamp.txt` we parse the last line
/// as the most-recently-used timestamp and return a vector of
/// (CacheKey, DateTime<Local>) sorted by most-recently-used first.
pub async fn discover_caches(root: &PathBuf) -> Result<Vec<(CacheKey, DateTime<Local>)>> {
    let mut dirs = vec![root.clone()];
    let mut results: Vec<(CacheKey, DateTime<Local>)> = Vec::new();

    while let Some(dir) = dirs.pop() {
        let mut read_dir = tokio::fs::read_dir(&dir)
            .await
            .wrap_err_with(|| format!("failed reading directory {}", dir.display()))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .wrap_err_with(|| format!("failed reading directory {}", dir.display()))?
        {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if let Some(file_name) = path.file_name()
                && file_name == "timestamp.txt"
            {
                // Read file into a string
                let contents = tokio::fs::read_to_string(&path).await.wrap_err_with(|| {
                    format!("failed reading timestamp file {}", path.display())
                })?;

                if let Some(last_line) = contents.lines().last() {
                    // Parse last line (most-recent usage)
                    let parsed = DateTime::parse_from_rfc2822(last_line)
                        .wrap_err_with(|| {
                            format!("failed parsing timestamp in {}", path.display())
                        })?
                        .with_timezone(&Local);
                    // Use parent directory of timestamp.txt as the cache entry dir
                    if let Some(parent) = path.parent() {
                        // Compute relative path from the given root. If that fails, fall back to the absolute parent path.
                        let rel = match parent.strip_prefix(root) {
                            Ok(x) => x.to_path_buf(),
                            Err(_) => parent.to_path_buf(),
                        };

                        let mut key = CacheKey::new(rel);
                        key.valid_for = Duration::MAX;
                        results.push((key, parsed));
                    }
                }
            }
        }
    }

    Ok(results)
}
