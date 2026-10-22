use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// An exact filename suffix, normalized to lowercase without its optional dot.
///
/// Matching uses the final suffix returned by [`std::path::Path::extension`],
/// rather than language detection: `package.json` has extension `json`.
/// Inventory histograms reserve `.` for extensionless files; it is not an
/// extension filter accepted by this type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct SourceFileExtension(String);

impl SourceFileExtension {
    /// Parse an exact suffix such as `php`, `.JS`, or `ts`.
    /// Empty suffixes, paths, whitespace, and wildcard patterns are rejected.
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        let value = value.strip_prefix('.').unwrap_or(value);
        eyre::ensure!(
            !value.is_empty()
                && !value.chars().any(|ch| {
                    ch.is_whitespace()
                        || ch.is_control()
                        || matches!(ch, '.' | '/' | '\\' | '*' | '?')
                }),
            "Expected an exact file extension such as php, js, or ts (optionally prefixed with a dot)"
        );
        Ok(Self(value.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SourceFileExtension {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for SourceFileExtension {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for SourceFileExtension {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for SourceFileExtension {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<&SourceFileExtension> for String {
    fn from(value: &SourceFileExtension) -> Self {
        value.0.clone()
    }
}

impl<'a> Arbitrary<'a> for SourceFileExtension {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let value = input.choose(&["php", "js", "ts", "rs", "json", "md", "c++", "cs"])?;
        Ok(Self((*value).to_owned()))
    }
}

cloud_terrastodon_registry::register_thing!(SourceFileExtension);
cloud_terrastodon_registry::register_arbitrary!(SourceFileExtension);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_optional_dots_and_case_without_accepting_patterns_or_paths() -> Result<()> {
        assert_eq!(SourceFileExtension::try_new(".PHP")?.as_str(), "php");
        assert_eq!(SourceFileExtension::try_new("TS")?.as_str(), "ts");
        assert_eq!(SourceFileExtension::try_new("c++")?.as_str(), "c++");
        for value in ["", ".", "*.php", "src/php", "src\\php", "tar.gz", "js "] {
            assert!(SourceFileExtension::try_new(value).is_err(), "{value:?}");
        }
        Ok(())
    }
}
