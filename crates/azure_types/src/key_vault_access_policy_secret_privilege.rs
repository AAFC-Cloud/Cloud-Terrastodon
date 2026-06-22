use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum KeyVaultAccessPolicySecretPrivilege {
    SecretManagementOperation(KeyVaultAccessPolicySecretManagementOperation),
    PrivilegedSecretOperation(KeyVaultAccessPolicyPrivilegedSecretOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}
crate::impl_facet_string_proxy!(KeyVaultAccessPolicySecretPrivilege, value => value.to_string());

impl std::fmt::Display for KeyVaultAccessPolicySecretPrivilege {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KeyVaultAccessPolicySecretPrivilege::SecretManagementOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicySecretPrivilege::PrivilegedSecretOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicySecretPrivilege::All(_) => "All",
        })
    }
}

impl std::str::FromStr for KeyVaultAccessPolicySecretPrivilege {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "get" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Get,
            ),
            "list" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::List,
            ),
            "set" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Set,
            ),
            "delete" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Delete,
            ),
            "recover" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Recover,
            ),
            "backup" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Backup,
            ),
            "restore" => Self::SecretManagementOperation(
                KeyVaultAccessPolicySecretManagementOperation::Restore,
            ),
            "purge" => Self::PrivilegedSecretOperation(
                KeyVaultAccessPolicyPrivilegedSecretOperation::Purge,
            ),
            "all" => Self::All(KeyVaultAccessPolicyAllPrivilege::All),
            _ => eyre::bail!("unknown key vault secret privilege {value:?}"),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet)]
#[repr(C)]
pub enum KeyVaultAccessPolicySecretManagementOperation {
    Get,
    List,
    Set,
    Delete,
    Recover,
    Backup,
    Restore,
}
impl KeyVaultAccessPolicySecretManagementOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "Get",
            Self::List => "List",
            Self::Set => "Set",
            Self::Delete => "Delete",
            Self::Recover => "Recover",
            Self::Backup => "Backup",
            Self::Restore => "Restore",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet)]
#[repr(C)]
pub enum KeyVaultAccessPolicyPrivilegedSecretOperation {
    Purge,
}
impl KeyVaultAccessPolicyPrivilegedSecretOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Purge => "Purge",
        }
    }
}
