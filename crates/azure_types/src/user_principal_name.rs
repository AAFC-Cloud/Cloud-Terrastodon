use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Report;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;

/// An Entra user's sign-in name, preserving its original spelling.
///
/// Microsoft's [UPN terminology and population rules](https://learn.microsoft.com/en-us/entra/identity/hybrid/connect/plan-connect-userprincipalname)
/// describe a user-account prefix joined to a domain suffix by `@`. A UPN may
/// differ from the user's email address, and synchronization may recalculate it.
/// See also the Microsoft Graph v1.0 [userPrincipalName property](https://learn.microsoft.com/en-us/graph/api/resources/user?view=graph-rest-1.0#properties).
///
/// Construction and string deserialization require one `@`, nonempty prefix and
/// suffix, and no whitespace or control characters. Guest names containing
/// `#EXT#` are supported. This is a syntax contract for observed names and exact
/// lookups, not a complete new-user provisioning validator: tenant domain
/// verification and provisioning-specific character/length policies remain the
/// service's responsibility. No directory, credentials, or DNS are consulted.
///
/// Equality and hashing preserve exact spelling. Use [`Self::matches`] or
/// [`Self::comparison_key`] for case-insensitive lookup comparisons.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct UserPrincipalName(String);

impl UserPrincipalName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control()),
            "A user principal name must not contain whitespace or controls"
        );
        let Some((local, domain)) = value.split_once('@') else {
            bail!("A user principal name must contain an '@' separator");
        };
        ensure!(
            !local.is_empty() && !domain.is_empty() && !domain.contains('@'),
            "A user principal name must have nonempty local and domain portions and one '@' separator"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, other: &Self) -> bool {
        self.comparison_key() == other.comparison_key()
    }

    pub fn comparison_key(&self) -> String {
        self.0.to_lowercase()
    }
}

impl Display for UserPrincipalName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for UserPrincipalName {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for UserPrincipalName {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for UserPrincipalName {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl From<UserPrincipalName> for String {
    fn from(value: UserPrincipalName) -> Self {
        value.0
    }
}

impl From<&UserPrincipalName> for String {
    fn from(value: &UserPrincipalName) -> Self {
        value.0.clone()
    }
}

impl AsRef<str> for UserPrincipalName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for UserPrincipalName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> Arbitrary<'a> for UserPrincipalName {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let length = input.int_in_range(1..=32)?;
        let mut value = String::new();
        for _ in 0..length {
            value.push(char::from(*input.choose(
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-'!#^~",
            )?));
        }
        value.push_str("@example.invalid");
        Self::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(UserPrincipalName);
cloud_terrastodon_registry::register_arbitrary!(UserPrincipalName);

#[cfg(test)]
mod tests {
    use super::UserPrincipalName;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn accepts_guest_names_and_compares_without_case() {
        let guest = UserPrincipalName::try_new("Alex_example.invalid#EXT#@tenant.invalid").unwrap();
        let lower = UserPrincipalName::try_new("alex_example.invalid#ext#@tenant.invalid").unwrap();
        assert!(guest.matches(&lower));
        assert_ne!(guest, lower, "annotation spelling remains observable");
        assert_eq!(guest.as_str(), "Alex_example.invalid#EXT#@tenant.invalid");
    }

    #[test]
    fn rejects_ambiguous_or_blank_lookup_names() {
        for value in [
            "",
            "alex",
            "@tenant.invalid",
            "alex@",
            "a@b@c",
            " a@b",
            "a@b\n",
        ] {
            assert!(UserPrincipalName::try_new(value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn scalar_wire_decoding_uses_the_same_validation() {
        for invalid in ["alex", "@tenant.invalid", "a@b@c", "a@b\n"] {
            let json = facet_json::to_string(invalid).unwrap();
            assert!(facet_json::from_str::<UserPrincipalName>(&json).is_err());
        }
        let spelling = "Alex_example.invalid#EXT#@tenant.invalid";
        let json = facet_json::to_string(spelling).unwrap();
        let name = facet_json::from_str::<UserPrincipalName>(&json).unwrap();
        assert_eq!(name.as_str(), spelling);
        assert_eq!(facet_json::to_string(&name).unwrap(), json);
    }

    #[test]
    fn arbitrary_names_satisfy_the_same_invariants() {
        for seed in 0..=u8::MAX {
            let bytes = [seed; 256];
            let name = UserPrincipalName::arbitrary(&mut Unstructured::new(&bytes)).unwrap();
            assert!(UserPrincipalName::try_new(name.as_str()).is_ok());
        }
    }
}
