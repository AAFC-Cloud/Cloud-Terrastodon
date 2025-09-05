use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyVaultAccessPolicySecretPrivilege {
    SecretManagementOperation(KeyVaultAccessPolicySecretManagementOperation),
    PrivilegedSecretOperation(KeyVaultAccessPolicyPrivilegedSecretOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicySecretManagementOperation {
    #[serde(alias = "Get", alias = "get", alias = "GET")]
    Get,
    #[serde(alias = "List", alias = "list", alias = "LIST")]
    List,
    #[serde(alias = "Set", alias = "set", alias = "SET")]
    Set,
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
pub enum KeyVaultAccessPolicyPrivilegedSecretOperation {
    #[serde(alias = "Purge", alias = "purge", alias = "PURGE")]
    Purge,
}
