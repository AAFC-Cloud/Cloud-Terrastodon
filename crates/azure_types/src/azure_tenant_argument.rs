use crate::prelude::AzureTenantAlias;
use crate::prelude::TenantId;
use eyre::bail;
use std::str::FromStr;

/// Tenant can be specified as a tenant id or a Cloud Terrastodon tenant alias.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureTenantArgument<'a> {
    Id(TenantId),
    IdRef(&'a TenantId),
    Alias(AzureTenantAlias),
    AliasRef(&'a AzureTenantAlias),
}

impl std::fmt::Display for AzureTenantArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureTenantArgument::Id(id) => id.fmt(f),
            AzureTenantArgument::IdRef(id) => id.fmt(f),
            AzureTenantArgument::Alias(alias) => alias.fmt(f),
            AzureTenantArgument::AliasRef(alias) => alias.fmt(f),
        }
    }
}

impl From<TenantId> for AzureTenantArgument<'_> {
    fn from(value: TenantId) -> Self {
        AzureTenantArgument::Id(value)
    }
}

impl<'a> From<&'a TenantId> for AzureTenantArgument<'a> {
    fn from(value: &'a TenantId) -> Self {
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
        if let Ok(id) = s.parse::<TenantId>() {
            Ok(AzureTenantArgument::Id(id))
        } else if let Ok(alias) = AzureTenantAlias::try_new(s) {
            Ok(AzureTenantArgument::Alias(alias))
        } else {
            bail!("'{s}' is not a valid Azure tenant id or Cloud Terrastodon tenant alias")
        }
    }
}