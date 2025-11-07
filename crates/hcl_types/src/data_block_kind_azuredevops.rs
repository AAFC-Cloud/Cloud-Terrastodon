use crate::prelude::ProviderKind;
use eyre::Result;
use eyre::eyre;
use std::str::FromStr;
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureDevOpsDataBlockKind {
    Other(String),
}
impl AzureDevOpsDataBlockKind {
    pub fn supported_variants() -> Vec<AzureDevOpsDataBlockKind> {
        vec![
        ]
    }
}
impl AsRef<str> for AzureDevOpsDataBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureDevOpsDataBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeking = s.trim_start_matches(ProviderKind::AzureRM.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(eyre!("no variant matches"))
    }
}
impl std::fmt::Display for AzureDevOpsDataBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureDevOps.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}