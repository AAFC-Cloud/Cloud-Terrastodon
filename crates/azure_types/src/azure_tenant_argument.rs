use crate::AzureTenantAlias;
use crate::AzureTenantId;
use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::str::FromStr;

/// An explicit Azure tenant ID or Cloud Terrastodon tenant alias.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureTenantArgument<'a> {
    Id(Cow<'a, AzureTenantId>),
    Alias(Cow<'a, AzureTenantAlias>),
}

impl std::fmt::Display for AzureTenantArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureTenantArgument::Id(id) => id.fmt(f),
            AzureTenantArgument::Alias(alias) => alias.fmt(f),
        }
    }
}

impl From<AzureTenantId> for AzureTenantArgument<'_> {
    fn from(value: AzureTenantId) -> Self {
        AzureTenantArgument::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a AzureTenantId> for AzureTenantArgument<'a> {
    fn from(value: &'a AzureTenantId) -> Self {
        AzureTenantArgument::Id(Cow::Borrowed(value))
    }
}

impl From<AzureTenantAlias> for AzureTenantArgument<'_> {
    fn from(value: AzureTenantAlias) -> Self {
        AzureTenantArgument::Alias(Cow::Owned(value))
    }
}

impl<'a> From<&'a AzureTenantAlias> for AzureTenantArgument<'a> {
    fn from(value: &'a AzureTenantAlias) -> Self {
        AzureTenantArgument::Alias(Cow::Borrowed(value))
    }
}

impl AzureTenantArgument<'_> {
    pub fn into_owned(self) -> AzureTenantArgument<'static> {
        match self {
            AzureTenantArgument::Id(id) => AzureTenantArgument::Id(Cow::Owned(id.into_owned())),
            AzureTenantArgument::Alias(alias) => {
                AzureTenantArgument::Alias(Cow::Owned(alias.into_owned()))
            }
        }
    }
}

impl<'a> FromStr for AzureTenantArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureTenantId>() {
            Ok(AzureTenantArgument::Id(Cow::Owned(id)))
        } else if let Ok(alias) = AzureTenantAlias::try_new(s) {
            Ok(AzureTenantArgument::Alias(Cow::Owned(alias)))
        } else {
            bail!("'{s}' is not a valid Azure tenant ID or Cloud Terrastodon tenant alias");
        }
    }
}

impl<'a> TryFrom<String> for AzureTenantArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureTenantArgument<'_>> for String {
    fn from(value: &AzureTenantArgument<'_>) -> Self {
        value.to_string()
    }
}

impl<'a> Arbitrary<'a> for AzureTenantArgument<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=1)? {
            0 => Self::Id(Cow::Owned(AzureTenantId::arbitrary(u)?)),
            _ => Self::Alias(Cow::Owned(AzureTenantAlias::arbitrary(u)?)),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::AzureTenantArgument;
    use crate::AzureTenantAlias;

    #[test]
    fn parses_default_as_a_tracked_alias() -> eyre::Result<()> {
        assert_eq!(
            "DeFaUlT".parse::<AzureTenantArgument>()?,
            AzureTenantArgument::from(AzureTenantAlias::try_new("default")?),
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureTenantArgument<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureTenantArgument<'static>);
