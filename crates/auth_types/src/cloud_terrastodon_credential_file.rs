use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::path::Path;
use std::path::PathBuf;

/// Explicit absolute credential input location. Construction does not read it.
/// Unicode paths are required because account metadata has a string wire format.
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
#[facet(proxy = String)]
pub struct CloudTerrastodonCredentialFile(String);

impl TryFrom<String> for CloudTerrastodonCredentialFile {
    type Error = eyre::Report;
    fn try_from(value: String) -> Result<Self> {
        ensure!(
            Path::new(&value).is_absolute(),
            "credential file path must be absolute"
        );
        ensure!(
            !value.contains('\0'),
            "credential file path must not contain NUL"
        );
        Ok(Self(value))
    }
}
impl TryFrom<PathBuf> for CloudTerrastodonCredentialFile {
    type Error = eyre::Report;
    fn try_from(value: PathBuf) -> Result<Self> {
        Self::try_from(
            value
                .into_os_string()
                .into_string()
                .map_err(|_| eyre::eyre!("credential file path must be Unicode"))?,
        )
    }
}
impl From<&CloudTerrastodonCredentialFile> for String {
    fn from(value: &CloudTerrastodonCredentialFile) -> Self {
        value.0.clone()
    }
}
impl AsRef<Path> for CloudTerrastodonCredentialFile {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

cloud_terrastodon_registry::register_thing!(CloudTerrastodonCredentialFile);
