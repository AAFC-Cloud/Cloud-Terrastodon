use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::path::Path;
use std::path::PathBuf;

/// A portable relative path within one artifact's input or diagnostic subtree.
///
/// Components are separated by `/`. Construction rejects traversal, absolute
/// paths, Windows special characters, and Windows device aliases. This is an
/// artifact-storage portability policy, not a service or domain URL constraint.
/// Other Unicode and exact case are preserved. Storage must create files
/// exclusively to reject duplicate paths and filesystem-specific aliases.
///
/// See [Windows filename conventions](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file)
/// and [console device names](https://learn.microsoft.com/en-us/windows/console/console-handles).
/// The portable subset also rejects all Unicode control characters and padded
/// device basenames rather than depending on an individual filesystem's rules.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Facet)]
#[facet(proxy = String)]
pub struct ArtifactFilePath {
    // Private storage keeps all paths within the portable relative-path contract.
    value: String,
}

impl ArtifactFilePath {
    /// Validate an exact slash-separated relative path.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Self::try_from(value.into())
    }

    /// Inspect the exact validated portable path.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Iterate validated path components without filesystem normalization.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.value.split('/')
    }

    /// Append the validated relative components without accessing the filesystem.
    /// The storage boundary still checks directories and files before I/O.
    pub(crate) fn join_to(&self, directory: &Path) -> PathBuf {
        let mut result = directory.to_path_buf();
        for component in self.components() {
            result.push(component);
        }
        result
    }
}

impl TryFrom<String> for ArtifactFilePath {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(!value.is_empty(), "artifact file path must not be empty");
        for component in value.split('/') {
            ensure!(
                !component.is_empty() && component != "." && component != "..",
                "artifact file path must contain nonempty nontraversal components"
            );
            ensure!(
                !component.chars().any(|character| {
                    character.is_control()
                        || matches!(character, '\\' | ':' | '<' | '>' | '\"' | '|' | '?' | '*')
                }),
                "artifact file path contains a nonportable character"
            );
            ensure!(
                !component.ends_with([' ', '.']),
                "artifact file path components must not end in a space or dot"
            );
            let base = component
                .split('.')
                .next()
                .unwrap_or(component)
                .trim_end_matches(' ');
            ensure!(
                !is_windows_device_name(base),
                "artifact file path contains a Windows device name"
            );
        }
        Ok(Self { value })
    }
}

impl From<&ArtifactFilePath> for String {
    fn from(path: &ArtifactFilePath) -> Self {
        path.value.clone()
    }
}

fn is_windows_device_name(base: &str) -> bool {
    let uppercase = base.to_ascii_uppercase();
    if matches!(
        uppercase.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) {
        return true;
    }
    let suffix = uppercase
        .strip_prefix("COM")
        .or_else(|| uppercase.strip_prefix("LPT"));
    matches!(
        suffix,
        Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")
    )
}

cloud_terrastodon_registry_values::register_thing!(ArtifactFilePath);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_paths_that_escape_or_alias_portable_artifact_names() {
        for invalid in [
            "",
            ".",
            "..",
            "/file",
            "file/",
            "a//file",
            "a/./file",
            "a/../file",
            "C:/file",
            "C:file",
            "file:stream",
            "a\\file",
            "//server/share",
            "a/has?wildcard",
            "a/star*",
            "a/less<than",
            "a/more>than",
            "a/quote\"",
            "a/pipe|",
            "a/control\n",
            "a/null\0",
            "a/trailing ",
            "a/trailing.",
            "con",
            "Con.txt",
            "con .txt",
            "a/PRN",
            "aux.bin",
            "nul.log",
            "com1",
            "COM9.ext",
            "lpt1",
            "LPT9.ext",
            "com¹.txt",
            "lpt².txt",
            "CoMiNvalid/CONOUT$",
        ] {
            assert!(
                ArtifactFilePath::new(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn preserves_relative_unicode_and_nonreserved_names() {
        for valid in [
            "payload.bin",
            "nested/payload.bin",
            "é/数据.json",
            "Mixed Case/File.json",
            ".hidden",
            "a..b",
            "com0",
            "com10",
            "lpt0",
            "lpt10",
            "console.txt",
        ] {
            let path = ArtifactFilePath::new(valid).unwrap();
            assert_eq!(path.as_str(), valid);
            assert_eq!(path.components().collect::<Vec<_>>().join("/"), valid);
            assert_eq!(String::from(&path), valid);
        }
    }

    #[test]
    fn facet_decoding_cannot_bypass_path_validation() {
        for invalid in ["../secret", "/absolute", "nul.txt", "file:stream"] {
            let encoded = facet_json::to_string(&invalid).unwrap();
            assert!(facet_json::from_str::<ArtifactFilePath>(&encoded).is_err());
        }
    }
}
