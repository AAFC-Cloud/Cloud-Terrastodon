use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::ops::Deref;
use std::str::FromStr;

/// A full Azure DevOps build-definition folder path, such as `\Team\CI`.
///
/// See Microsoft's [Folder schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#folder)
/// and [pipeline folder argument](https://learn.microsoft.com/en-us/cli/azure/pipelines?view=azure-cli-latest#az-pipelines-list).
/// Paths are rooted and backslash-separated; `\` represents root. Spelling is
/// preserved, including case. Construction and deserialization validate paths.
///
/// To keep concrete paths unambiguous, this type also rejects blank, `.` and
/// `..` segments, trailing or repeated separators, controls and wildcards.
/// These are local safety constraints; the Build API does not document an
/// exhaustive character policy or a path length limit. Restrictions for TFVC
/// or Azure Boards paths do not apply to this type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildFolderPath(String);

impl AzureDevOpsBuildFolderPath {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            value.starts_with('\\') && !value.contains('/'),
            "Build folder paths must start with a backslash and use backslash separators"
        );
        if value != "\\" {
            ensure!(
                value[1..]
                    .split('\\')
                    .all(|segment| !segment.trim().is_empty()
                        && !matches!(segment, "." | "..")
                        && !segment
                            .chars()
                            .any(|ch| ch.is_control() || matches!(ch, '*' | '?'))),
                "Build folder paths must contain concrete, nonblank segments without controls, wildcards, '.' or '..'"
            );
        }
        Ok(Self(value))
    }

    pub fn root() -> Self {
        Self("\\".to_owned())
    }

    /// Normalize CLI input to a full path before validating it.
    /// Relative paths and forward slashes are alternative spellings of the
    /// same domain value. API paths are validated strictly by `try_new`,
    /// `FromStr`, and the string proxy.
    pub fn from_selector(value: &str) -> Result<Self> {
        let value = value.replace('/', "\\");
        if value.is_empty() {
            return Ok(Self::root());
        }
        if value.starts_with('\\') {
            Self::try_new(value)
        } else {
            Self::try_new(format!("\\{value}"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0 == "\\"
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0[1..]
            .split('\\')
            .filter(|segment| !segment.is_empty())
    }

    pub fn depth(&self) -> usize {
        self.segments().count()
    }

    /// A conservative comparison key for pruning, rather than a change to
    /// the spelling sent to the API or the equality of path values.
    pub fn case_insensitive_key(&self) -> String {
        self.0.to_lowercase()
    }

    /// Compare at folder boundaries without case distinctions so pruning
    /// protects potentially occupied ancestors regardless of server spelling.
    pub fn contains(&self, other: &Self) -> bool {
        if self.is_root() {
            return true;
        }
        let key = self.case_insensitive_key();
        let other_key = other.case_insensitive_key();
        key == other_key || other_key.starts_with(&format!("{key}\\"))
    }
}

impl Deref for AzureDevOpsBuildFolderPath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for AzureDevOpsBuildFolderPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AzureDevOpsBuildFolderPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AzureDevOpsBuildFolderPath {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsBuildFolderPath {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&AzureDevOpsBuildFolderPath> for String {
    fn from(value: &AzureDevOpsBuildFolderPath) -> Self {
        value.0.clone()
    }
}

impl From<AzureDevOpsBuildFolderPath> for String {
    fn from(value: AzureDevOpsBuildFolderPath) -> Self {
        value.0
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderPath {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::from_selector(&String::arbitrary(u)?).unwrap_or_else(|_| Self::root()))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPath);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPath);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_folder_paths_normalize_cli_input_before_validation() {
        for input in [r"Team\CI", r"\Team\CI", "Team/CI", "/Team/CI"] {
            let path = AzureDevOpsBuildFolderPath::from_selector(input).unwrap();
            assert_eq!(path.as_str(), r"\Team\CI");
        }
        for input in ["", "\\", "/"] {
            assert!(
                AzureDevOpsBuildFolderPath::from_selector(input)
                    .unwrap()
                    .is_root()
            );
        }
        for invalid in ["Team/../CI", "Team//CI", "Team/", "Team/./CI", "Team/*"] {
            assert!(AzureDevOpsBuildFolderPath::from_selector(invalid).is_err());
        }
    }

    #[test]
    fn build_folder_paths_preserve_spelling_and_reject_ambiguous_paths() {
        let path = AzureDevOpsBuildFolderPath::try_new(r"\Release & tools\Ünicode").unwrap();
        assert_eq!(path.as_str(), r"\Release & tools\Ünicode");
        assert_eq!(path.depth(), 2);
        assert_eq!(
            path.segments().collect::<Vec<_>>(),
            ["Release & tools", "Ünicode"]
        );
        assert!(AzureDevOpsBuildFolderPath::root().is_root());
        assert_eq!(AzureDevOpsBuildFolderPath::root().depth(), 0);
        for invalid in [
            "",
            "/",
            "relative",
            r"\A/B",
            r"\A\",
            r"\\A",
            r"\A\\B",
            r"\.",
            r"\A\..",
            r"\A\ ",
            r"\*",
            r"\A?",
            "\\A\n",
            "\\A\0",
            "\\A\u{1b}",
        ] {
            assert!(
                AzureDevOpsBuildFolderPath::try_new(invalid).is_err(),
                "Accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn build_folder_paths_compare_at_boundaries_without_changing_api_spelling() {
        let parent = AzureDevOpsBuildFolderPath::try_new(r"\Team").unwrap();
        let child = AzureDevOpsBuildFolderPath::try_new(r"\tEAM\CI").unwrap();
        assert!(parent.contains(&child));
        assert!(!child.contains(&parent));
        assert!(!parent.contains(&AzureDevOpsBuildFolderPath::try_new(r"\TeamOther").unwrap()));
        assert!(AzureDevOpsBuildFolderPath::root().contains(&child));
        assert_ne!(
            parent,
            AzureDevOpsBuildFolderPath::try_new(r"\team").unwrap()
        );
        assert_eq!(child.as_str(), r"\tEAM\CI");
    }
}
