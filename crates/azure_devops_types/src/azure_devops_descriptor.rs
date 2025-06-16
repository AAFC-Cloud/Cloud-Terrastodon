use crate::prelude::AzureDevOpsEntraUserDescriptor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AzureDevOpsDescriptor {
    EntraUser(AzureDevOpsEntraUserDescriptor),
    EntraGroup(String),
    AzureDevOpsGroup(String),
    Other(String),
}

impl std::fmt::Display for AzureDevOpsDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsDescriptor::EntraUser(id) => write!(f, "{}", id),
            AzureDevOpsDescriptor::EntraGroup(id) => write!(f, "{}", id),
            AzureDevOpsDescriptor::AzureDevOpsGroup(id) => write!(f, "{}", id),
            AzureDevOpsDescriptor::Other(id) => write!(f, "{}", id),
        }
    }
}
impl Serialize for AzureDevOpsDescriptor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl FromStr for AzureDevOpsDescriptor {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("aad.") {
            Ok(AzureDevOpsDescriptor::EntraUser(
                AzureDevOpsEntraUserDescriptor::try_new(s.to_string())?,
            ))
        } else if s.starts_with("aadgp.") {
            Ok(AzureDevOpsDescriptor::EntraGroup(s.to_string()))
        } else if s.starts_with("vssgp.") {
            Ok(AzureDevOpsDescriptor::AzureDevOpsGroup(s.to_string()))
        } else {
            Ok(AzureDevOpsDescriptor::Other(s.to_string()))
        }
    }
}

impl<'de> Deserialize<'de> for AzureDevOpsDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::AzureDevOpsDescriptor;
    use crate::prelude::AzureDevOpsEntraUserDescriptor;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let descriptor = "aad.bruh";
        let parsed: AzureDevOpsDescriptor = descriptor.parse()?;
        Ok(())
    }

    #[test]
    pub fn it_works_for_group() -> eyre::Result<()> {
        let descriptor = "aadgp.bruh";
        let parsed: AzureDevOpsDescriptor = descriptor.parse()?;
        assert_eq!(
            parsed,
            AzureDevOpsDescriptor::EntraGroup("aadgp.bruh".to_string())
        );
        Ok(())
    }

    #[test]
    pub fn it_works_for_other() -> eyre::Result<()> {
        let descriptor = "other.bruh";
        let parsed: AzureDevOpsDescriptor = descriptor.parse()?;
        assert_eq!(
            parsed,
            AzureDevOpsDescriptor::Other("other.bruh".to_string())
        );
        Ok(())
    }

    #[test]
    pub fn serializes() -> eyre::Result<()> {
        let descriptor = AzureDevOpsDescriptor::EntraUser(AzureDevOpsEntraUserDescriptor::try_new(
            "aad.bruh".to_string(),
        )?);
        let serialized = serde_json::to_string(&descriptor)?;
        assert_eq!(serialized, "\"aad.bruh\"");
        let deserialized: AzureDevOpsDescriptor = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized, descriptor);
        Ok(())
    }
}
