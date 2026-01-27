use std::sync::LazyLock;
use std::sync::Mutex;

use compact_str::CompactString;

#[derive(Debug, Clone)]
pub struct GitRevision(pub CompactString);

impl GitRevision {
    pub fn new(revision: impl Into<CompactString>) -> Self {
        Self(revision.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for GitRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for GitRevision {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

static GIT_REVISION: LazyLock<Mutex<GitRevision>> = LazyLock::new(|| Mutex::new(GitRevision::new("unknown")));

/// Set the global git revision used by the application (used by UI About dialogs)
pub fn set_git_revision(rev: impl Into<GitRevision>) {
    *GIT_REVISION.lock().unwrap() = rev.into();
}

/// Get a clone of the configured git revision
pub fn git_revision() -> GitRevision {
    GIT_REVISION.lock().unwrap().clone()
}
