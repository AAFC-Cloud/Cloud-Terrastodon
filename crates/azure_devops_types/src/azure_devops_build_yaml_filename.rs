use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// The repository path of a build definition's YAML file.
///
/// REST API (7.1): [Definitions List](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1)
/// documents the `yamlFilename` filter. The containing
/// [BuildDefinition.process response](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition)
/// is documented only through the base process schema, which omits this property.
/// Supplemental JavaScript shape: [YamlProcess.yamlFilename](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/yamlprocess#yamlfilename).
///
/// This library rejects blank values and control characters as local safety
/// constraints, without imposing undocumented provider naming restrictions.
/// Exact spelling, spaces, Unicode and provider-specific separators are preserved.
/// This is a repository path, not a local Windows path;
/// leading slashes, repository separators and file extensions are not rewritten.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildYamlFilename(String);

impl AzureDevOpsBuildYamlFilename {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "The repository path of a build definition's YAML file must not be blank"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "The repository path of a build definition's YAML file must not contain control characters"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for AzureDevOpsBuildYamlFilename {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildYamlFilename {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for AzureDevOpsBuildYamlFilename {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildYamlFilename {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildYamlFilename> for String {
    fn from(value: &AzureDevOpsBuildYamlFilename) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildYamlFilename> for String {
    fn from(value: AzureDevOpsBuildYamlFilename) -> Self {
        value.0
    }
}

impl std::fmt::Display for AzureDevOpsBuildYamlFilename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildYamlFilename {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::try_new(String::arbitrary(u)?).unwrap_or_else(|_| {
            Self::try_new("azure-pipelines.yml").expect("valid synthetic value")
        }))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildYamlFilename);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildYamlFilename);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_paths_preserve_repository_spelling_without_windows_path_rules() {
        for path in [
            "azure-pipelines.yml",
            "/pipelines/CI build.yaml",
            "config:ci.yml",
        ] {
            let value = AzureDevOpsBuildYamlFilename::try_new(path).unwrap();
            assert_eq!(value.as_str(), path);
        }
        for invalid in ["", "  ", "pipeline\u{1b}.yml"] {
            assert!(AzureDevOpsBuildYamlFilename::try_new(invalid).is_err());
        }
    }
}
