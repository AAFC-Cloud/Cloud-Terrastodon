use compact_str::CompactString;
use std::sync::LazyLock;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Version(CompactString);
impl Version {
    pub fn new(version: impl Into<CompactString>) -> Self {
        Version(version.into())
    }
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

static VERSION: LazyLock<Mutex<Version>> =
    LazyLock::new(|| Mutex::new(Version::new("unknown".to_string())));

/// Set the global version used by the application (used by UI About dialogs)
pub fn set_version(ver: impl Into<Version>) -> Result<()> {
    let ver = ver.into();
    *VERSION
        .lock()
        .map_err(|_| eyre!("application version lock is poisoned"))? = ver;
    Ok(())
}

/// Get a clone of the configured version
pub fn version() -> Result<Version> {
    Ok(VERSION
        .lock()
        .map_err(|_| eyre!("application version lock is poisoned"))?
        .clone())
}

pub fn full_version() -> Result<Version> {
    Ok(Version::new(format!(
        "{} (rev {}, built {})",
        version()?,
        crate::git_revision::git_revision()?,
        crate::build_timestamp::build_timestamp()?
    )))
}
use eyre::Result;
use eyre::eyre;
