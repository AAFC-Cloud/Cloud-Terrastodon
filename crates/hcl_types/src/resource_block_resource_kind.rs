use crate::resource_block_kind_azuread::AzureAdResourceBlockKind;
use crate::resource_block_kind_azuredevops::AzureDevOpsResourceBlockKind;
use crate::resource_block_kind_azurerm::AzureRmResourceBlockKind;
use crate::resource_block_kind_other::OtherResourceBlockKind;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResourceBlockResourceKind {
    AzureAD(AzureAdResourceBlockKind),
    AzureRM(AzureRmResourceBlockKind),
    AzureDevOps(AzureDevOpsResourceBlockKind),
    Other(OtherResourceBlockKind),
}
impl std::fmt::Display for ResourceBlockResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceBlockResourceKind::AzureAD(kind) => kind.fmt(f),
            ResourceBlockResourceKind::AzureRM(kind) => kind.fmt(f),
            ResourceBlockResourceKind::AzureDevOps(kind) => kind.fmt(f),
            ResourceBlockResourceKind::Other(kind) => kind.fmt(f),
        }
    }
}
impl FromStr for ResourceBlockResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(kind) = s.parse::<AzureRmResourceBlockKind>() {
            return Ok(ResourceBlockResourceKind::AzureRM(kind));
        }
        if let Ok(kind) = s.parse::<AzureAdResourceBlockKind>() {
            return Ok(ResourceBlockResourceKind::AzureAD(kind));
        }
        if let Ok(kind) = s.parse::<AzureDevOpsResourceBlockKind>() {
            return Ok(ResourceBlockResourceKind::AzureDevOps(kind));
        }
        Ok(ResourceBlockResourceKind::Other(
            OtherResourceBlockKind::from_str(s)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceBlockReference;

    #[test]
    fn parse_azurerm_role_assignment() -> eyre::Result<()> {
        let kind: ResourceBlockResourceKind = "azurerm_role_assignment".parse()?;
        assert_eq!(
            kind,
            ResourceBlockResourceKind::AzureRM(AzureRmResourceBlockKind::RoleAssignment)
        );
        Ok(())
    }

    #[test]
    fn parse_azurerm_other() -> eyre::Result<()> {
        let kind: ResourceBlockResourceKind = "azurerm_synapse_workspace".parse()?;
        assert_eq!(
            kind,
            ResourceBlockResourceKind::AzureRM(AzureRmResourceBlockKind::Other(
                "synapse_workspace".to_owned()
            ))
        );
        Ok(())
    }

    #[test]
    fn parse_azuread_group() -> eyre::Result<()> {
        let kind: ResourceBlockResourceKind = "azuread_group".parse()?;
        assert_eq!(
            kind,
            ResourceBlockResourceKind::AzureAD(AzureAdResourceBlockKind::Group)
        );
        Ok(())
    }
    #[test]
    fn parse_azuread_other() -> eyre::Result<()> {
        let kind: ResourceBlockResourceKind = "azuread_thingy".parse()?;
        assert_eq!(
            kind,
            ResourceBlockResourceKind::AzureAD(AzureAdResourceBlockKind::Other(
                "thingy".to_owned()
            ))
        );
        Ok(())
    }
    #[test]
    fn parse_resource_reference() -> eyre::Result<()> {
        let thing = "azurerm_storage_account.bruh";
        let x: ResourceBlockReference = thing.parse()?;
        let expected = ResourceBlockReference::AzureRM {
            kind: AzureRmResourceBlockKind::StorageAccount,
            name: "bruh".to_owned(),
        };
        assert_eq!(x, expected);
        Ok(())
    }
    #[test]
    fn parse_azurerm_resource_kind() -> eyre::Result<()> {
        let thing = "azurerm_storage_account";
        let x: AzureRmResourceBlockKind = thing.parse()?;
        let expected = AzureRmResourceBlockKind::StorageAccount;
        assert_eq!(x, expected);
        Ok(())
    }
}
