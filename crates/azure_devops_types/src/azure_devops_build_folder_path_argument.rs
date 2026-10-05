use crate::AzureDevOpsBuildFolderPath;
use eyre::Result;
use std::ops::Deref;
use std::str::FromStr;

/// Validated build-folder CLI input accepting relative paths and forward slashes.
///
/// See Microsoft's [pipeline folder argument](https://learn.microsoft.com/en-us/cli/azure/pipelines?view=azure-cli-latest#az-pipelines-list).
/// The parser and String proxy use [`AzureDevOpsBuildFolderPath::from_selector`]
/// to normalize shorthand before validating it. API records instead use
/// [`AzureDevOpsBuildFolderPath`]'s strict canonical-path parser and String proxy.
///
/// This type stores the validated path, so conversion to the domain value is
/// infallible and command handlers do not need to parse raw strings.
#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildFolderPathArgument(AzureDevOpsBuildFolderPath);

impl AzureDevOpsBuildFolderPathArgument {
    /// Return the path already normalized and validated during construction.
    pub fn into_path(self) -> AzureDevOpsBuildFolderPath {
        self.0
    }
}

impl Deref for AzureDevOpsBuildFolderPathArgument {
    type Target = AzureDevOpsBuildFolderPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildFolderPathArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AzureDevOpsBuildFolderPathArgument {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Ok(Self(AzureDevOpsBuildFolderPath::from_selector(value)?))
    }
}

impl TryFrom<String> for AzureDevOpsBuildFolderPathArgument {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildFolderPathArgument> for String {
    fn from(value: &AzureDevOpsBuildFolderPathArgument) -> Self {
        value.0.as_str().to_owned()
    }
}

impl From<AzureDevOpsBuildFolderPathArgument> for AzureDevOpsBuildFolderPath {
    fn from(value: AzureDevOpsBuildFolderPathArgument) -> Self {
        value.into_path()
    }
}

impl From<AzureDevOpsBuildFolderPath> for AzureDevOpsBuildFolderPathArgument {
    fn from(value: AzureDevOpsBuildFolderPath) -> Self {
        Self(value)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPathArgument);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPathArgument);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_input_accepts_shorthand_while_api_paths_remain_strict() {
        let canonical = r"\Team\CI";
        for input in [r"Team\CI", canonical, "Team/CI", "/Team/CI"] {
            let argument: AzureDevOpsBuildFolderPathArgument = input.parse().unwrap();
            let path: AzureDevOpsBuildFolderPath = argument.into();

            assert_eq!(path.as_str(), canonical);
            assert_eq!(
                input.parse::<AzureDevOpsBuildFolderPath>().is_ok(),
                input == canonical
            );
            assert_eq!(
                AzureDevOpsBuildFolderPathArgument::from(path.clone()).into_path(),
                path
            );
        }
    }

    #[test]
    fn cli_input_validates_before_conversion_and_normalizes_root() {
        for input in ["", "\\", "/"] {
            let argument: AzureDevOpsBuildFolderPathArgument = input.parse().unwrap();
            assert!(argument.into_path().is_root());
        }
        for invalid in ["Team/../CI", "Team//CI", "Team/", "Team/./CI", "Team/*"] {
            assert!(
                invalid
                    .parse::<AzureDevOpsBuildFolderPathArgument>()
                    .is_err()
            );
        }
    }
}
