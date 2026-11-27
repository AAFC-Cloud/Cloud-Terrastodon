use crate::BmcHelixServerUrl;
use arbitrary::Arbitrary;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use sha2::Digest;
use sha2::Sha256;
use std::fmt;
use std::str;
use std::str::FromStr;

/// Stable SHA-256 identity of a Helix server's canonical HTTPS root URL.
///
/// Local names may change without affecting the server or its account sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixServerId([u8; 32]);

impl BmcHelixServerId {
    pub(crate) fn for_url(server: &BmcHelixServerUrl) -> Self {
        Self(Sha256::digest(server.as_url().as_str().as_bytes()).into())
    }
}

impl fmt::Display for BmcHelixServerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for BmcHelixServerId {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        let mut digest = [0; 32];
        ensure!(
            value.len() == digest.len() * 2 && value.bytes().all(|digit| digit.is_ascii_hexdigit()),
            "Helix server ID must be a 64-digit hexadecimal SHA-256 digest"
        );
        for (byte, pair) in digest.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
            *byte = u8::from_str_radix(str::from_utf8(pair)?, 16)?;
        }
        Ok(Self(digest))
    }
}

impl TryFrom<String> for BmcHelixServerId {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<&BmcHelixServerId> for String {
    fn from(value: &BmcHelixServerId) -> Self {
        value.to_string()
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixServerId);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixServerId);
