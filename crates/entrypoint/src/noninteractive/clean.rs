use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::remove_dir_all;
use tokio::fs::try_exists;
use tracing::warn;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CleanStats {
    pub files: u64,
    pub bytes: u64,
}

impl CleanStats {
    fn add_assign(&mut self, other: Self) {
        self.files = self.files.saturating_add(other.files);
        self.bytes = self.bytes.saturating_add(other.bytes);
    }
}

pub async fn clean() -> Result<CleanStats> {
    clean_dirs(AppDir::ok_to_clean()).await
}

pub async fn clean_dirs(dirs: impl IntoIterator<Item = AppDir>) -> Result<CleanStats> {
    let mut total = CleanStats::default();

    for dir in dirs {
        let path = dir.as_path_buf();
        if !try_exists(&path).await? {
            continue;
        }

        let stats = match directory_stats(path.clone()).await {
            Ok(stats) => stats,
            Err(error) => {
                warn!(
                    directory = %dir,
                    %error,
                    "Unable to calculate cleanup size"
                );
                CleanStats::default()
            }
        };

        if let Err(e) = remove_dir_all(path).await {
            warn!("Ignoring error encountered cleaning {dir}: {e:?}");
        } else {
            total.add_assign(stats);
        }
    }

    tracing::info!(
        "Removed {} files, {} total",
        total.files,
        format_bytes(total.bytes)
    );
    Ok(total)
}

async fn directory_stats(path: PathBuf) -> io::Result<CleanStats> {
    tokio::task::spawn_blocking(move || collect_directory_stats(&path))
        .await
        .map_err(io::Error::other)?
}

fn collect_directory_stats(path: &Path) -> io::Result<CleanStats> {
    let mut stats = CleanStats::default();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            stats.add_assign(collect_directory_stats(&entry.path())?);
        } else {
            stats.files = stats.files.saturating_add(1);
            stats.bytes = stats.bytes.saturating_add(entry.metadata()?.len());
        }
    }

    Ok(stats)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes}B")
    } else {
        format!("{value:.1}{}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn formats_bytes_like_cargo() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(1024), "1.0KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0MiB");
    }
}
