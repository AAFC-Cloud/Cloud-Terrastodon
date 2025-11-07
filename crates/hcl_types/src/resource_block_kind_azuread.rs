use crate::prelude::ProviderKind;
use eyre::bail;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureAdResourceBlockKind {
    Group,
    User,
    Other(String),
}
impl AzureAdResourceBlockKind {
    pub fn known_variants() -> Vec<AzureAdResourceBlockKind> {
        vec![
            AzureAdResourceBlockKind::Group,
            AzureAdResourceBlockKind::User,
        ]
    }
}
impl AsRef<str> for AzureAdResourceBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Group => "group",
            Self::User => "user",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureAdResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = ProviderKind::AzureAD.provider_prefix();
        let Some(seeking) = s
            .strip_prefix(provider_prefix)
            .and_then(|s| s.strip_prefix("_"))
        else {
            bail!(format!(
                "String {s:?} is missing prefix {}",
                provider_prefix
            ));
        };
        for variant in Self::known_variants() {
            if variant.as_ref() == seeking {
                return Ok(variant);
            }
        }
        Ok(Self::Other(seeking.to_owned()))
    }
}

impl std::fmt::Display for AzureAdResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureAD.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}
