use crate::AzureDevOpsEntraUserDescriptor;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsDescriptor {
    EntraUser(AzureDevOpsEntraUserDescriptor),
    EntraGroup(String),
    EntraServicePrincipal(String),
    AzureDevOpsGroup(String),
    Other(String),
}

impl std::fmt::Display for AzureDevOpsDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsDescriptor::EntraUser(id) => write!(f, "{id}"),
            AzureDevOpsDescriptor::EntraGroup(id) => write!(f, "{id}"),
            AzureDevOpsDescriptor::EntraServicePrincipal(id) => write!(f, "{id}"),
            AzureDevOpsDescriptor::AzureDevOpsGroup(id) => write!(f, "{id}"),
            AzureDevOpsDescriptor::Other(id) => write!(f, "{id}"),
        }
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
        } else if s.starts_with("aadsp.") {
            Ok(AzureDevOpsDescriptor::EntraServicePrincipal(s.to_string()))
        } else if s.starts_with("vssgp.") {
            Ok(AzureDevOpsDescriptor::AzureDevOpsGroup(s.to_string()))
        } else {
            Ok(AzureDevOpsDescriptor::Other(s.to_string()))
        }
    }
}

impl TryFrom<String> for AzureDevOpsDescriptor {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<&AzureDevOpsDescriptor> for String {
    fn from(value: &AzureDevOpsDescriptor) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::AzureDevOpsDescriptor;
    use crate::AzureDevOpsEntraUserDescriptor;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let descriptor = "aad.bruh";
        let _parsed: AzureDevOpsDescriptor = descriptor.parse()?;
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
    pub fn it_works_for_service_principal() -> eyre::Result<()> {
        let descriptor = "aadsp.service-principal-id";
        let parsed: AzureDevOpsDescriptor = descriptor.parse()?;
        assert_eq!(
            parsed,
            AzureDevOpsDescriptor::EntraServicePrincipal("aadsp.service-principal-id".to_string())
        );
        Ok(())
    }

    #[test]
    pub fn serializes() -> eyre::Result<()> {
        let descriptor = AzureDevOpsDescriptor::EntraUser(AzureDevOpsEntraUserDescriptor::try_new(
            "aad.bruh".to_string(),
        )?);
        let serialized = facet_json::to_string(&descriptor)?;
        assert_eq!(serialized, "\"aad.bruh\"");
        let deserialized: AzureDevOpsDescriptor = facet_json::from_str(&serialized)?;
        assert_eq!(deserialized, descriptor);
        Ok(())
    }
}
