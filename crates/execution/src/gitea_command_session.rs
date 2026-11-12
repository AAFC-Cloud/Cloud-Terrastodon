use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

/// Owns a private immutable tea configuration snapshot for one admitted account.
///
/// The last session owner removes the credential file. Commands select this root
/// through `XDG_CONFIG_HOME`, supported by tea's configuration implementation:
/// <https://gitea.com/gitea/tea/src/tag/v0.11.1/modules/config/config.go>.
///
/// This session is shared between the authentication provider and Gitea SDK.
/// It currently belongs to their lower-level execution contract: moving it into
/// the SDK would create a cycle because that SDK already depends on execution.
pub struct GiteaCommandSession {
    login: String,
    configuration: TempDir,
}
impl GiteaCommandSession {
    pub fn new(login: String, configuration: &str) -> Result<Self> {
        ensure!(
            !login.trim().is_empty() && !login.chars().any(char::is_control),
            "tea login profile must be nonempty and contain no control characters"
        );
        let directory = tempfile::tempdir().wrap_err("creating a private tea session")?;
        #[cfg(unix)]
        std::fs::set_permissions(
            directory.path(),
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        )?;
        std::fs::create_dir(directory.path().join("tea"))?;
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(directory.path().join("tea/config.yml"))?;
        file.write_all(configuration.as_bytes())?;
        Ok(Self {
            login,
            configuration: directory,
        })
    }
    pub fn login(&self) -> &str {
        &self.login
    }
    pub fn configuration_home(&self) -> &Path {
        self.configuration.path()
    }
}
impl std::fmt::Debug for GiteaCommandSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GiteaCommandSession")
            .field("login", &self.login)
            .finish_non_exhaustive()
    }
}
