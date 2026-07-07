use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;
use arbitrary::Arbitrary;

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(C)]
pub enum KeyVaultAccessPolicyKeyPrivilege {
    KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation),
    CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation),
    PrivilegedKeyOperation(KeyVaultAccessPolicyPrivilegedKeyOperation),
    RotationPolicyOperation(KeyVaultAccessPolicyRotationPolicyOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}
crate::impl_facet_string_proxy!(KeyVaultAccessPolicyKeyPrivilege, value => value.to_string());

impl std::fmt::Display for KeyVaultAccessPolicyKeyPrivilege {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KeyVaultAccessPolicyKeyPrivilege::KeyManagementOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyKeyPrivilege::CryptographicOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyKeyPrivilege::PrivilegedKeyOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyKeyPrivilege::RotationPolicyOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyKeyPrivilege::All(_) => "All",
        })
    }
}

impl std::str::FromStr for KeyVaultAccessPolicyKeyPrivilege {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "get" => Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Get),
            "list" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::List)
            }
            "update" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Update)
            }
            "create" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Create)
            }
            "import" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Import)
            }
            "delete" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Delete)
            }
            "recover" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Recover)
            }
            "backup" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Backup)
            }
            "restore" => {
                Self::KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation::Restore)
            }
            "decrypt" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::Decrypt)
            }
            "encrypt" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::Encrypt)
            }
            "unwrapkey" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::UnwrapKey)
            }
            "wrapkey" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::WrapKey)
            }
            "verify" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::Verify)
            }
            "sign" => {
                Self::CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation::Sign)
            }
            "purge" => {
                Self::PrivilegedKeyOperation(KeyVaultAccessPolicyPrivilegedKeyOperation::Purge)
            }
            "release" => {
                Self::PrivilegedKeyOperation(KeyVaultAccessPolicyPrivilegedKeyOperation::Release)
            }
            "rotate" => {
                Self::RotationPolicyOperation(KeyVaultAccessPolicyRotationPolicyOperation::Rotate)
            }
            "getrotationpolicy" => Self::RotationPolicyOperation(
                KeyVaultAccessPolicyRotationPolicyOperation::GetRotationPolicy,
            ),
            "setrotationpolicy" => Self::RotationPolicyOperation(
                KeyVaultAccessPolicyRotationPolicyOperation::SetRotationPolicy,
            ),
            "all" => Self::All(KeyVaultAccessPolicyAllPrivilege::All),
            _ => eyre::bail!("unknown key vault key privilege {value:?}"),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyKeyManagementOperation {
    Get,
    List,
    Update,
    Create,
    Import,
    Delete,
    Recover,
    Backup,
    Restore,
}
impl KeyVaultAccessPolicyKeyManagementOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "Get",
            Self::List => "List",
            Self::Update => "Update",
            Self::Create => "Create",
            Self::Import => "Import",
            Self::Delete => "Delete",
            Self::Recover => "Recover",
            Self::Backup => "Backup",
            Self::Restore => "Restore",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyCryptographicOperation {
    Decrypt,
    Encrypt,
    UnwrapKey,
    WrapKey,
    Verify,
    Sign,
}
impl KeyVaultAccessPolicyCryptographicOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Decrypt => "Decrypt",
            Self::Encrypt => "Encrypt",
            Self::UnwrapKey => "UnwrapKey",
            Self::WrapKey => "WrapKey",
            Self::Verify => "Verify",
            Self::Sign => "Sign",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyPrivilegedKeyOperation {
    Purge,
    Release,
}
impl KeyVaultAccessPolicyPrivilegedKeyOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Purge => "Purge",
            Self::Release => "Release",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyRotationPolicyOperation {
    Rotate,
    GetRotationPolicy,
    SetRotationPolicy,
}
impl KeyVaultAccessPolicyRotationPolicyOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Rotate => "Rotate",
            Self::GetRotationPolicy => "GetRotationPolicy",
            Self::SetRotationPolicy => "SetRotationPolicy",
        }
    }
}
