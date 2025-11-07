use eyre::bail;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ProviderKind {
    AzureRM,
    AzureAD,
    AzureDevOps,
    Other(String),
}
impl ProviderKind {
    pub fn provider_prefix(&self) -> &str {
        match self {
            ProviderKind::AzureRM => "azurerm",
            ProviderKind::AzureAD => "azuread",
            ProviderKind::AzureDevOps => "azuredevops",
            ProviderKind::Other(s) => s.as_str(),
        }
    }
    pub fn well_known_variants() -> [ProviderKind; 3] {
        [
            ProviderKind::AzureRM,
            ProviderKind::AzureAD,
            ProviderKind::AzureDevOps,
        ]
    }
}
impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}
impl FromStr for ProviderKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for char in s.chars() {
            if !char.is_alphabetic() {
                bail!("Invalid character {char} parsing provider kind {s}");
            }
        }
        for kind in ProviderKind::well_known_variants() {
            if kind.provider_prefix() == s {
                return Ok(kind);
            }
        }
        Ok(ProviderKind::Other(s.to_owned()))
    }
}
