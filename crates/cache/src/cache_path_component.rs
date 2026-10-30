use cloud_terrastodon_pathing::sanitize_windows_path_component;
use std::path::Path;

/// The readable, bounded physical encoding of one cache identity component.
///
/// The readable prefix uses [`sanitize_windows_path_component`] and contains at
/// most 160 UTF-16 units, followed by `-` and the full lowercase BLAKE3 digest of
/// the original UTF-8 bytes. The result contains at most 225 UTF-16 units. The
/// digest distinguishes case and Unicode spellings even when a filesystem treats
/// their readable prefixes as aliases. Only the component length is bounded;
/// callers must also account for the complete path length.
///
/// A digest is not proof of identity. Cache storage must retain and verify the
/// original identity before accepting an existing entry at this component.
#[derive(Debug)]
pub(crate) struct CachePathComponent(
    // Private storage prevents bypassing the encoding and component-length policy.
    String,
);

impl CachePathComponent {
    /// Encode original identity text, including empty strings, into a valid
    /// component. This does not parse an already encoded component or perform I/O.
    pub fn from_identity(value: &str) -> Self {
        let encoded = sanitize_windows_path_component(value);
        let mut prefix = String::new();
        let mut units = 0;
        for character in encoded.chars() {
            if units + character.len_utf16() > 160 {
                break;
            }
            prefix.push(character);
            units += character.len_utf16();
        }
        Self(format!(
            "{prefix}-{}",
            blake3::hash(value.as_bytes()).to_hex()
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for CachePathComponent {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::CachePathComponent;

    #[test]
    fn case_and_unicode_aliases_have_distinct_physical_components() {
        for (left, right) in [("A", "a"), ("Å", "å"), ("K", "K"), ("é", "e\u{301}")] {
            assert_ne!(
                CachePathComponent::from_identity(left)
                    .as_str()
                    .to_lowercase(),
                CachePathComponent::from_identity(right)
                    .as_str()
                    .to_lowercase(),
                "{left:?} and {right:?} must remain distinct on case-insensitive filesystems",
            );
        }
        assert!(
            CachePathComponent::from_identity("Résumé")
                .as_str()
                .starts_with("Résumé-")
        );
    }

    #[test]
    fn sanitized_components_remain_bounded_and_preserve_truncated_identity() {
        for (input, prefix) in [
            ("CON.txt", "%43ON.txt-"),
            ("COM¹.txt", "%43OM¹.txt-"),
            ("..", "%2E%2E-"),
            ("", "%EMPTY-"),
            ("a b\t\n\0", "a%20b%09%0A%00-"),
            ("name.. ", "name..%20-"),
        ] {
            let component = CachePathComponent::from_identity(input);
            let component = component.as_str();
            assert!(component.starts_with(prefix));
            assert!(component.encode_utf16().count() <= 225);
            assert!(!component.chars().any(|character| {
                character.is_whitespace()
                    || character.is_control()
                    || "\\/:*?\"<>|".contains(character)
            }));
            assert!(!component.ends_with('.'));
        }

        for character in ["語", "🦀"] {
            let common = character.repeat(300);
            let left = CachePathComponent::from_identity(&format!("{common}A"));
            let right = CachePathComponent::from_identity(&format!("{common}a"));
            let (prefix, digest) = left.as_str().rsplit_once('-').unwrap();
            assert_eq!(
                prefix,
                character.repeat(160 / character.encode_utf16().count())
            );
            assert!(left.as_str().encode_utf16().count() <= 225);
            assert_eq!(digest.len(), 64);
            assert!(
                digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            assert_ne!(left.as_str().to_lowercase(), right.as_str().to_lowercase());
        }
    }

    #[test]
    fn escaped_values_remain_distinct() {
        for (left, right) in [
            ("", "%EMPTY"),
            ("a/b", "a\\b"),
            ("a/b", "a%2Fb"),
            ("a b", "a%20b"),
            ("a.b", "a/b"),
        ] {
            assert_ne!(
                CachePathComponent::from_identity(left)
                    .as_str()
                    .to_lowercase(),
                CachePathComponent::from_identity(right)
                    .as_str()
                    .to_lowercase(),
            );
        }
    }
}
