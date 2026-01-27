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
pub fn set_version(ver: impl Into<Version>) {
    *VERSION.lock().unwrap() = ver.into();
}

/// Get a clone of the configured version
pub fn version() -> Version {
    VERSION.lock().unwrap().clone()
}

pub fn full_version() -> Version {
    Version::new(format!(
        "{} (git revision: {})",
        version(),
        crate::git_revision::git_revision()
    ))
}
