use eyre::Context;
use eyre::Result;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use teamy_cancellation::CancellationScope;
use tokio::io::AsyncReadExt;

/// Bounded filesystem view for an explicitly opened local path.
pub(crate) enum LocalPathPreview {
    Directory {
        entries: Vec<PathBuf>,
        truncated: bool,
    },
    File {
        bytes: u64,
        text: Option<String>,
        truncated: bool,
    },
    Other,
}

impl fmt::Debug for LocalPathPreview {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory { entries, truncated } => formatter
                .debug_struct("Directory")
                .field("entry_count", &entries.len())
                .field("truncated", truncated)
                .finish(),
            Self::File {
                bytes, truncated, ..
            } => formatter
                .debug_struct("File")
                .field("bytes", bytes)
                .field("truncated", truncated)
                .finish_non_exhaustive(),
            Self::Other => formatter.write_str("Other"),
        }
    }
}

impl LocalPathPreview {
    pub async fn read(path: &Path, cancellation: &CancellationScope) -> Result<Self> {
        const MAX_DIRECTORY_ENTRIES: usize = 256;
        const MAX_PREVIEW_BYTES: usize = 64 * 1024;

        let metadata = tokio::fs::metadata(path)
            .await
            .wrap_err_with(|| format!("Inspecting {}", path.display()))?;
        cancellation.bail_if_cancelled()?;
        if metadata.is_dir() {
            let mut reader = tokio::fs::read_dir(path).await?;
            let mut entries = Vec::new();
            while entries.len() < MAX_DIRECTORY_ENTRIES {
                cancellation.bail_if_cancelled()?;
                let Some(entry) = reader.next_entry().await? else {
                    break;
                };
                entries.push(entry.path());
            }
            let truncated =
                entries.len() == MAX_DIRECTORY_ENTRIES && reader.next_entry().await?.is_some();
            entries.sort();
            Ok(Self::Directory { entries, truncated })
        } else if metadata.is_file() {
            let mut text = None;
            let mut truncated = false;
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| {
                    ["tf", "tfvars", "hcl"]
                        .iter()
                        .any(|extension| value.eq_ignore_ascii_case(extension))
                })
            {
                let mut contents = Vec::new();
                tokio::fs::File::open(path)
                    .await?
                    .take(MAX_PREVIEW_BYTES as u64 + 1)
                    .read_to_end(&mut contents)
                    .await?;
                truncated = contents.len() > MAX_PREVIEW_BYTES;
                contents.truncate(MAX_PREVIEW_BYTES);
                text = Some(String::from_utf8_lossy(&contents).into_owned());
            }
            Ok(Self::File {
                bytes: metadata.len(),
                text,
                truncated,
            })
        } else {
            Ok(Self::Other)
        }
    }
}
