use crate::prelude::AzureTenantAlias;
use crate::prelude::AzureTenantId;
use eyre::bail;
use std::str::FromStr;

/// Tenant can be specified as the default tenant, a tenant id, or a Cloud Terrastodon tenant alias.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum AzureTenantArgument<'a> {
    #[default]
    Default,
    Id(AzureTenantId),
    IdRef(&'a AzureTenantId),
    Alias(AzureTenantAlias),
    AliasRef(&'a AzureTenantAlias),
}

impl std::fmt::Display for AzureTenantArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureTenantArgument::Default => f.write_str("default"),
            AzureTenantArgument::Id(id) => id.fmt(f),
            AzureTenantArgument::IdRef(id) => id.fmt(f),
            AzureTenantArgument::Alias(alias) => alias.fmt(f),
            AzureTenantArgument::AliasRef(alias) => alias.fmt(f),
        }
    }
}

impl From<AzureTenantId> for AzureTenantArgument<'_> {
    fn from(value: AzureTenantId) -> Self {
        AzureTenantArgument::Id(value)
    }
}

impl<'a> From<&'a AzureTenantId> for AzureTenantArgument<'a> {
    fn from(value: &'a AzureTenantId) -> Self {
        AzureTenantArgument::IdRef(value)
    }
}

impl From<AzureTenantAlias> for AzureTenantArgument<'_> {
    fn from(value: AzureTenantAlias) -> Self {
        AzureTenantArgument::Alias(value)
    }
}

impl<'a> From<&'a AzureTenantAlias> for AzureTenantArgument<'a> {
    fn from(value: &'a AzureTenantAlias) -> Self {
        AzureTenantArgument::AliasRef(value)
    }
}

impl AzureTenantArgument<'_> {
    pub fn into_owned(self) -> AzureTenantArgument<'static> {
        match self {
            AzureTenantArgument::Default => AzureTenantArgument::Default,
            AzureTenantArgument::Id(id) => AzureTenantArgument::Id(id),
            AzureTenantArgument::IdRef(id) => AzureTenantArgument::Id(*id),
            AzureTenantArgument::Alias(alias) => AzureTenantArgument::Alias(alias),
            AzureTenantArgument::AliasRef(alias) => AzureTenantArgument::Alias(alias.clone()),
        }
    }
}

impl FromStr for AzureTenantArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("default") {
            Ok(AzureTenantArgument::Default)
        } else if let Ok(id) = s.parse::<AzureTenantId>() {
            Ok(AzureTenantArgument::Id(id))
        } else if let Ok(alias) = AzureTenantAlias::try_new(s) {
            Ok(AzureTenantArgument::Alias(alias))
        } else {
            bail!(
                "'{s}' is not a valid default tenant selector, Azure tenant id, or Cloud Terrastodon tenant alias"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AzureTenantArgument;
    use std::str::FromStr;

    #[test]
    fn parses_default_selector() {
        let arg = AzureTenantArgument::from_str("default").unwrap();
        assert_eq!(arg, AzureTenantArgument::Default);
    }
}
