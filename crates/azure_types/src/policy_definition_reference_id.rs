use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::ensure;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// A definition's local identity inside a policy initiative, distinct from its
/// global ARM policy definition ID.
///
/// See [remediation policy definition selection](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/remediation-structure#policy-definition-id).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = String)]
pub struct PolicyDefinitionReferenceId {
    // An empty value cannot select an initiative member.
    value: String,
}

impl PolicyDefinitionReferenceId {
    pub fn new(value: impl Into<String>) -> eyre::Result<Self> {
        let value = value.into();
        ensure!(
            !value.is_empty(),
            "a policy definition reference ID cannot be empty"
        );
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl Display for PolicyDefinitionReferenceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PolicyDefinitionReferenceId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        Self::new(value)
    }
}

crate::impl_facet_string_proxy!(PolicyDefinitionReferenceId, value => value.to_string());

impl<'a> Arbitrary<'a> for PolicyDefinitionReferenceId {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(format!("definition{}", u.arbitrary::<u32>()?))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(PolicyDefinitionReferenceId);
cloud_terrastodon_registry::register_arbitrary!(PolicyDefinitionReferenceId);
