use serde::{Deserialize, Serialize};
use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyVaultAccessPolicyCertificatePrivilege {
    CertificateManagementOperation(KeyVaultAccessPolicyCertificateManagementOperation),
    PrivilegedCertificateOperation(KeyVaultAccessPolicyPrivilegedCertificateOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyCertificateManagementOperation {
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
    #[serde(
        alias = "ManageContacts",
        alias = "managecontacts",
        alias = "MANAGECONTACTS",
        alias = "manageContacts"
    )]
    ManageContacts,
    #[serde(
        alias = "ManageCertificateAuthorities",
        alias = "managecertificateauthorities",
        alias = "MANAGECERTIFICATEAUTHORITIES",
        alias = "manageCertificateAuthorities"
    )]
    ManageCertificateAuthorities,
    #[serde(
        alias = "ManageIssuers",
        alias = "manageissuers",
        alias = "MANAGEISSUERS",
        alias = "manageIssuers"
    )]
    ManageIssuers,
    #[serde(
        alias = "GetIssuers",
        alias = "getissuers",
        alias = "GETISSUERS",
        alias = "getIssuers"
    )]
    GetIssuers,
    #[serde(
        alias = "ListIssuers",
        alias = "listissuers",
        alias = "LISTISSUERS",
        alias = "listIssuers"
    )]
    ListIssuers,
    #[serde(
        alias = "SetIssuers",
        alias = "setissuers",
        alias = "SETISSUERS",
        alias = "setIssuers"
    )]
    SetIssuers,
    #[serde(
        alias = "DeleteIssuers",
        alias = "deleteissuers",
        alias = "DELETEISSUERS",
        alias = "deleteIssuers"
    )]
    DeleteIssuers,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyVaultAccessPolicyPrivilegedCertificateOperation {
    #[serde(alias = "Purge", alias = "purge", alias = "PURGE")]
    Purge,
}

#[cfg(test)]
mod test {
    use crate::prelude::KeyVaultAccessPolicyCertificatePrivilege;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json = r#""All""#;
        let privilege: KeyVaultAccessPolicyCertificatePrivilege = serde_json::from_str(json)?;
        match privilege {
            KeyVaultAccessPolicyCertificatePrivilege::All(all) => {
                assert_eq!(all, crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege::All)
            }
            _ => panic!("expected All variant"),
        }
        Ok(())
    }
}