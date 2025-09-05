use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyVaultAccessPolicyKeyPrivilege {
    KeyManagementOperation(KeyVaultAccessPolicyKeyManagementOperation),
    CryptographicOperation(KeyVaultAccessPolicyCryptographicOperation),
    PrivilegedKeyOperation(KeyVaultAccessPolicyPrivilegedKeyOperation),
    RotationPolicyOperation(KeyVaultAccessPolicyRotationPolicyOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyKeyManagementOperation {
    #[serde(alias = "Get", alias = "get", alias = "GET")]
    Get,
    #[serde(alias = "List", alias = "list", alias = "LIST")]
    List,
    #[serde(alias = "Update", alias = "update", alias = "UPDATE")]
    Update,
    #[serde(alias = "Create", alias = "create", alias = "CREATE")]
    Create,
    #[serde(alias = "Import", alias = "import", alias = "IMPORT")]
    Import,
    #[serde(alias = "Delete", alias = "delete", alias = "DELETE")]
    Delete,
    #[serde(alias = "Recover", alias = "recover", alias = "RECOVER")]
    Recover,
    #[serde(alias = "Backup", alias = "backup", alias = "BACKUP")]
    Backup,
    #[serde(alias = "Restore", alias = "restore", alias = "RESTORE")]
    Restore,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyCryptographicOperation {
    #[serde(alias = "Decrypt", alias = "decrypt", alias = "DECRYPT")]
    Decrypt,
    #[serde(alias = "Encrypt", alias = "encrypt", alias = "ENCRYPT")]
    Encrypt,
    #[serde(
        alias = "UnwrapKey",
        alias = "unwrapkey",
        alias = "UNWRAPKEY",
        alias = "unwrapKey"
    )]
    UnwrapKey,
    #[serde(
        alias = "WrapKey",
        alias = "wrapkey",
        alias = "WRAPKEY",
        alias = "wrapKey"
    )]
    WrapKey,
    #[serde(alias = "Verify", alias = "verify", alias = "VERIFY")]
    Verify,
    #[serde(alias = "Sign", alias = "sign", alias = "SIGN")]
    Sign,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyPrivilegedKeyOperation {
    #[serde(alias = "Purge", alias = "purge", alias = "PURGE")]
    Purge,
    #[serde(alias = "Release", alias = "release", alias = "RELEASE")]
    Release,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyRotationPolicyOperation {
    #[serde(alias = "Rotate", alias = "rotate", alias = "ROTATE")]
    Rotate,
    #[serde(
        alias = "GetRotationPolicy",
        alias = "getrotationpolicy",
        alias = "GETROTATIONPOLICY",
        alias = "getRotationPolicy"
    )]
    GetRotationPolicy,
    #[serde(
        alias = "SetRotationPolicy",
        alias = "setrotationpolicy",
        alias = "SETROTATIONPOLICY",
        alias = "setRotationPolicy"
    )]
    SetRotationPolicy,
}
