use blake3::Hash;
use eyre::OptionExt;
use eyre::Report;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;

/// A BLAKE3 digest of an artifact's exact bytes.
///
/// The persisted form is `blake3:` followed by 64 hexadecimal digits. Parsing
/// requires that algorithm tag; writing always uses lowercase hexadecimal.
/// This checksum detects changed bytes, not the authenticity of an artifact.
#[derive(Clone, Debug, Eq, PartialEq, Facet)]
#[facet(proxy = String, opaque)]
pub(crate) struct ArtifactDigest {
    // Private typed storage prevents invalid lengths, encoding, or algorithm tags.
    hash: Hash,
}

impl ArtifactDigest {
    /// Hash the exact bytes with BLAKE3.
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            hash: blake3::hash(bytes),
        }
    }

    /// Verify bytes against this BLAKE3 digest.
    pub fn matches(&self, bytes: &[u8]) -> bool {
        self.hash == blake3::hash(bytes)
    }
}

impl TryFrom<String> for ArtifactDigest {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        let hexadecimal = value
            .strip_prefix("blake3:")
            .ok_or_eyre("artifact digest must specify the blake3: algorithm tag")?;
        Ok(Self {
            hash: Hash::from_hex(hexadecimal).wrap_err("invalid BLAKE3 artifact digest")?,
        })
    }
}

impl From<&ArtifactDigest> for String {
    fn from(digest: &ArtifactDigest) -> Self {
        format!("blake3:{}", digest.hash.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_digest_verifies_original_bytes_and_rejects_changed_bytes() {
        let digest = ArtifactDigest::new(b"artifact bytes");
        let encoded = String::from(&digest);
        assert!(encoded.starts_with("blake3:"));

        let restored = ArtifactDigest::try_from(encoded).unwrap();
        assert!(restored.matches(b"artifact bytes"));
        assert!(!restored.matches(b"changed artifact bytes"));
    }

    #[test]
    fn decoding_rejects_missing_or_unknown_algorithms_and_malformed_hashes() {
        let digest = String::from(&ArtifactDigest::new(b"artifact bytes"));
        for invalid in [
            digest.strip_prefix("blake3:").unwrap().to_owned(),
            digest.replacen("blake3:", "sha256:", 1),
            "blake3:".to_owned(),
            "blake3:0123".to_owned(),
            format!("blake3:{}", "g".repeat(64)),
        ] {
            let encoded = facet_json::to_string(&invalid).unwrap();
            assert!(
                facet_json::from_str::<ArtifactDigest>(&encoded).is_err(),
                "accepted invalid digest {invalid:?}"
            );
        }
    }
}
