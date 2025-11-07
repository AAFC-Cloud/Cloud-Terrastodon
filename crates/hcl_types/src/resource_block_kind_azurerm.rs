use crate::prelude::ProviderKind;
use eyre::bail;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureRmResourceBlockKind {
    ManagementGroupPolicyAssignment,
    ResourceGroup,
    PolicyAssignment,
    PolicyDefinition,
    PolicySetDefinition,
    RoleAssignment,
    RoleDefinition,
    ContainerRegistry,
    StorageAccount,
    KeyVault,
    Other(String),
}
impl AzureRmResourceBlockKind {
    pub fn known_variants() -> Vec<AzureRmResourceBlockKind> {
        vec![
            AzureRmResourceBlockKind::ManagementGroupPolicyAssignment,
            AzureRmResourceBlockKind::ResourceGroup,
            AzureRmResourceBlockKind::PolicyAssignment,
            AzureRmResourceBlockKind::PolicyDefinition,
            AzureRmResourceBlockKind::PolicySetDefinition,
            AzureRmResourceBlockKind::RoleAssignment,
            AzureRmResourceBlockKind::RoleDefinition,
            AzureRmResourceBlockKind::StorageAccount,
        ]
    }
}
impl AsRef<str> for AzureRmResourceBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::ManagementGroupPolicyAssignment => "management_group_policy_assignment",
            Self::PolicyAssignment => "policy_assignment",
            Self::ResourceGroup => "resource_group",
            Self::PolicyDefinition => "policy_definition",
            Self::PolicySetDefinition => "policy_set_definition",
            Self::RoleAssignment => "role_assignment",
            Self::RoleDefinition => "role_definition",
            Self::StorageAccount => "storage_account",
            Self::ContainerRegistry => "container_registry",
            Self::KeyVault => "key_vault",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureRmResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = ProviderKind::AzureRM.provider_prefix();
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
impl std::fmt::Display for AzureRmResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureRM.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}
