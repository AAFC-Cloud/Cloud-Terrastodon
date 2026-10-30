use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BuildTimestamp(Option<i64>);

impl BuildTimestamp {
    pub fn from_env(value: Option<&str>) -> Self {
        Self(value.and_then(|value| value.parse().ok()))
    }
}

impl std::fmt::Display for BuildTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(timestamp) = self
            .0
            .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
        else {
            return f.write_str("unknown build time");
        };

        write!(
            f,
            "{}",
            timestamp
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S %Z")
        )
    }
}

static BUILD_TIMESTAMP: LazyLock<Mutex<BuildTimestamp>> =
    LazyLock::new(|| Mutex::new(BuildTimestamp(None)));

/// Set the global build timestamp used by the application (used by UI About dialogs)
pub fn set_build_timestamp(timestamp: impl Into<BuildTimestamp>) -> Result<()> {
    let timestamp = timestamp.into();
    *BUILD_TIMESTAMP
        .lock()
        .map_err(|_| eyre!("application build timestamp lock is poisoned"))? = timestamp;
    Ok(())
}

/// Get a clone of the configured build timestamp
pub fn build_timestamp() -> Result<BuildTimestamp> {
    Ok(BUILD_TIMESTAMP
        .lock()
        .map_err(|_| eyre!("application build timestamp lock is poisoned"))?
        .clone())
}
use eyre::Result;
use eyre::eyre;
