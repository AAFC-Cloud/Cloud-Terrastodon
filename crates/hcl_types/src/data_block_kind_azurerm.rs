use crate::prelude::ProviderKind;
use eyre::Result;
use eyre::eyre;
use std::str::FromStr;

/// Azure Resource Manager data block kind
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureRmDataBlockKind {
    PolicyDefinition,
    PolicySetDefinition,
    ResourceGroup,
    Other(String),
}
impl AzureRmDataBlockKind {
    pub fn supported_variants() -> Vec<AzureRmDataBlockKind> {
        vec![
            AzureRmDataBlockKind::PolicyDefinition,
            AzureRmDataBlockKind::PolicySetDefinition,
            AzureRmDataBlockKind::ResourceGroup,
        ]
    }
}
impl AsRef<str> for AzureRmDataBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::PolicyDefinition => "policy_definition",
            Self::PolicySetDefinition => "policy_set_definition",
            Self::ResourceGroup => "resource_group",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureRmDataBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeking = s.trim_start_matches(ProviderKind::AzureRM.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(eyre!("no variant matches"))
    }
}
impl std::fmt::Display for AzureRmDataBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureRM.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}
