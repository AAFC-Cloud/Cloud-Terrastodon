use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::ensure;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// The name of a [policy remediation task](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01).
///
/// Requiring one nonempty, non-dot path segment is a library composition
/// invariant. The service does not document an additional character allowlist
/// or length limit for this parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct PolicyRemediationName {
    // Private so a valid ARM path segment cannot be replaced with a path.
    value: String,
}

impl PolicyRemediationName {
    pub fn new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        ensure!(
            !value.is_empty()
                && value != "."
                && value != ".."
                && !value.contains(['/', '\\'])
                && !value.chars().any(char::is_control),
            "a remediation name must be one nonempty, non-dot path segment"
        );
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl Display for PolicyRemediationName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PolicyRemediationName {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        Self::new(value)
    }
}

crate::impl_facet_string_proxy!(PolicyRemediationName, value => value.to_string());

impl<'a> Arbitrary<'a> for PolicyRemediationName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(format!("remediation{}", u.arbitrary::<u32>()?))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationName);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationName);

#[cfg(test)]
mod tests {
    use super::PolicyRemediationName;

    #[test]
    fn rejects_route_changes_in_construction_and_wire_decoding() -> eyre::Result<()> {
        for invalid in ["", ".", "..", "parent/child", "parent\\child", "name\n"] {
            assert!(PolicyRemediationName::new(invalid).is_err());
            let encoded = facet_json::to_string(&invalid)?;
            assert!(facet_json::from_str::<PolicyRemediationName>(&encoded).is_err());
        }
        let name = PolicyRemediationName::new("remediation % café")?;
        assert_eq!(name.as_str(), "remediation % café");
        Ok(())
    }
}
