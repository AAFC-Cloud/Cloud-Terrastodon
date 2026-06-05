use crate::CacheKey;
use crate::CommandOutput;
use bstr::BString;
use bstr::ByteSlice;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::Builder;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::error;

const CONTEXT_FILE: &str = "context.txt";
const STDOUT_FILE: &str = "stdout.json";
const STDERR_FILE: &str = "stderr.json";
const STATUS_FILE: &str = "status.txt";
const TIMESTAMP_FILE: &str = "timestamp.txt";
const METADATA_FILE: &str = "metadata.json";
const BUSTED_FILE: &str = "busted";
const ERROR_FILE: &str = "error.txt";

static MEMORY_CACHE: OnceLock<Mutex<HashMap<String, CommandOutput>>> = OnceLock::new();

fn memory_cache() -> &'static Mutex<HashMap<String, CommandOutput>> {
    MEMORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub version: u8,
    pub fingerprint: String,
    pub executor_kind: String,
    pub output_type: String,
}

impl ArtifactMetadata {
    pub fn new(
        fingerprint: impl Into<String>,
        executor_kind: impl Into<String>,
        output_type: impl Into<String>,
    ) -> Self {
        Self {
            version: 1,
            fingerprint: fingerprint.into(),
            executor_kind: executor_kind.into(),
            output_type: output_type.into(),
        }
    }
}

fn cache_memory_key(cache_key: &CacheKey, fingerprint: &str) -> String {
    format!("{}::{fingerprint}", cache_key.path_on_disk().display())
}

async fn load_file(cache_dir: &Path, path: impl AsRef<Path>) -> Result<BString> {
    let path = cache_dir.join(path.as_ref());
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .await
        .context(format!("opening cache file {}", path.display()))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)
        .await
        .context(format!("reading cache file {}", path.display()))?;
    Ok(BString::from(file_contents))
}

async fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await
        .context(format!("opening file {}", path.display()))?;
    file.write_all(contents)
        .await
        .context(format!("writing file {}", path.display()))?;
    Ok(())
}

async fn append_file(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .await
        .context(format!("opening file {}", path.display()))?;
    file.write_all(contents)
        .await
        .context(format!("writing file {}", path.display()))?;
    Ok(())
}

fn validate_legacy_cache(
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    context_contents: &BString,
    input_contents: &BTreeMap<PathBuf, BString>,
) -> bool {
    if context_contents != context.as_bytes() {
        return false;
    }
    for (path, expected) in debug_inputs {
        let Some(found) = input_contents.get(path) else {
            return false;
        };
        if found != expected {
            return false;
        }
    }
    true
}

pub async fn get_cached_output(
    cache_key: &CacheKey,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    fingerprint: &str,
) -> Result<Option<CommandOutput>> {
    if cache_key.valid_for.is_zero() {
        debug!("Cache validity duration is zero, not using cache");
        return Ok(None);
    }

    let memory_key = cache_memory_key(cache_key, fingerprint);
    if let Some(output) = memory_cache()
        .lock()
        .expect("memory cache poisoned")
        .get(&memory_key)
        .cloned()
    {
        debug!("Loaded cached output from memory");
        return Ok(Some(output));
    }

    let cache_dir = cache_key.path_on_disk();
    if !cache_dir.exists() {
        debug!("Cache directory does not exist, not using cache");
        return Ok(None);
    }

    if !matches!(
        tokio::fs::try_exists(cache_dir.join(BUSTED_FILE)).await,
        Ok(false)
    ) {
        debug!("Cache is busted");
        return Ok(None);
    }

    let metadata = match load_file(&cache_dir, METADATA_FILE).await {
        Ok(contents) => Some(
            serde_json::from_slice::<ArtifactMetadata>(&contents).context(format!(
                "deserializing cache metadata at {}",
                cache_dir.display()
            ))?,
        ),
        Err(error) => {
            debug!(
                path = %cache_dir.display(),
                %error,
                "Cache metadata missing or unreadable, falling back to legacy validation"
            );
            None
        }
    };

    if let Some(metadata) = metadata {
        if metadata.fingerprint != fingerprint {
            debug!(
                path = %cache_dir.display(),
                stored = %metadata.fingerprint,
                expected = %fingerprint,
                "Not using cache due to fingerprint mismatch"
            );
            return Ok(None);
        }
    } else {
        let context_contents = load_file(&cache_dir, CONTEXT_FILE).await?;
        let mut input_contents = BTreeMap::new();
        for path in debug_inputs.keys() {
            input_contents.insert(path.clone(), load_file(&cache_dir, path).await?);
        }
        if !validate_legacy_cache(context, debug_inputs, &context_contents, &input_contents) {
            debug!(
                path = %cache_dir.display(),
                "Not using cache due to legacy context/input mismatch"
            );
            return Ok(None);
        }
    }

    let timestamp = load_file(&cache_dir, TIMESTAMP_FILE).await?;
    let timestamp_first_line = timestamp
        .lines()
        .next()
        .wrap_err("timestamp.txt contained no lines")?;
    let timestamp_first_line = timestamp_first_line
        .to_str()
        .wrap_err("failed to convert timestamp first line to string")?;
    let timestamp = DateTime::parse_from_rfc2822(timestamp_first_line)
        .wrap_err_with(|| format!("failed to parse timestamp from '{}'", timestamp_first_line))?;
    let now = Local::now();
    let time_remaining = if cache_key.valid_for == Duration::MAX {
        TimeDelta::MAX
    } else {
        timestamp + cache_key.valid_for - now.fixed_offset()
    };
    if time_remaining < TimeDelta::zero() {
        debug!(
            %timestamp,
            valid_for_seconds = cache_key.valid_for.as_secs(),
            expired_for_seconds = time_remaining.abs().num_seconds(),
            "Cache entry has expired"
        );
        return Ok(None);
    }

    let status: i32 = load_file(&cache_dir, STATUS_FILE)
        .await?
        .to_str()?
        .parse()?;
    let stdout = load_file(&cache_dir, STDOUT_FILE).await?;
    let stderr = load_file(&cache_dir, STDERR_FILE).await?;
    let output = CommandOutput {
        status,
        stdout,
        stderr,
    };

    memory_cache()
        .lock()
        .expect("memory cache poisoned")
        .insert(memory_key, output.clone());

    Ok(Some(output))
}

pub async fn write_output(
    parent_dir: &Path,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    output: &CommandOutput,
    metadata: &ArtifactMetadata,
) -> Result<()> {
    parent_dir.ensure_dir_exists().await?;

    let status = output.status.to_string();
    let timestamp = Local::now().to_rfc2822();
    let metadata = serde_json::to_vec_pretty(metadata)?;
    let files = [
        (CONTEXT_FILE, context.as_bytes()),
        (STDOUT_FILE, output.stdout.as_bytes()),
        (STDERR_FILE, output.stderr.as_bytes()),
        (STATUS_FILE, status.as_bytes()),
        (METADATA_FILE, metadata.as_slice()),
    ];

    let busted_path = parent_dir.join(BUSTED_FILE);
    if let Ok(true) = busted_path.try_exists() {
        tokio::fs::remove_file(&busted_path)
            .await
            .context("Removing busted cache marker")?;
    }

    for (file_name, file_contents) in files {
        write_file(&parent_dir.join(file_name), file_contents).await?;
    }

    let mut line = timestamp.into_bytes();
    line.push(b'\n');
    append_file(&parent_dir.join(TIMESTAMP_FILE), &line).await?;

    for (relative_path, contents) in debug_inputs {
        let full_path = parent_dir.join(relative_path);
        if let Some(parent) = full_path.parent() {
            parent.ensure_dir_exists().await?;
        }
        write_file(&full_path, contents.as_bytes()).await?;
    }

    Ok(())
}

pub async fn write_failure(
    cache_key: Option<&CacheKey>,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    output: &CommandOutput,
    metadata: &ArtifactMetadata,
    error_message: Option<&str>,
) -> Result<PathBuf> {
    let dir = match cache_key {
        None => AppDir::Commands.join("failed"),
        Some(cache_key) => cache_key.path_on_disk().join("failed"),
    };
    dir.ensure_dir_exists().await?;
    let dir = Builder::new()
        .prefix(Local::now().format("%Y%m%d_%H%M%S_").to_string().as_str())
        .tempdir_in(dir)?
        .keep();

    write_output(&dir, context, debug_inputs, output, metadata).await?;

    if let Some(error_message) = error_message {
        write_file(&dir.join(ERROR_FILE), error_message.as_bytes()).await?;
    }

    Ok(dir)
}

pub fn put_memory_cache_entry(cache_key: &CacheKey, fingerprint: &str, output: &CommandOutput) {
    memory_cache()
        .lock()
        .expect("memory cache poisoned")
        .insert(cache_memory_key(cache_key, fingerprint), output.clone());
}

pub fn note_cache_write_failure(error: &eyre::Error) {
    error!("Encountered problem saving cache: {:?}", error);
}
