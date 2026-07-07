use crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege;
use arbitrary::Arbitrary;

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[facet(proxy = String)]
#[repr(C)]
pub enum KeyVaultAccessPolicyCertificatePrivilege {
    CertificateManagementOperation(KeyVaultAccessPolicyCertificateManagementOperation),
    PrivilegedCertificateOperation(KeyVaultAccessPolicyPrivilegedCertificateOperation),
    All(KeyVaultAccessPolicyAllPrivilege),
}
crate::impl_facet_string_proxy!(KeyVaultAccessPolicyCertificatePrivilege, value => value.to_string());

impl std::fmt::Display for KeyVaultAccessPolicyCertificatePrivilege {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KeyVaultAccessPolicyCertificatePrivilege::CertificateManagementOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyCertificatePrivilege::PrivilegedCertificateOperation(operation) => {
                operation.as_str()
            }
            KeyVaultAccessPolicyCertificatePrivilege::All(_) => "All",
        })
    }
}

impl std::str::FromStr for KeyVaultAccessPolicyCertificatePrivilege {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "get" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Get,
            ),
            "list" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::List,
            ),
            "update" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Update,
            ),
            "create" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Create,
            ),
            "import" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Import,
            ),
            "delete" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Delete,
            ),
            "recover" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Recover,
            ),
            "backup" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Backup,
            ),
            "restore" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::Restore,
            ),
            "managecontacts" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::ManageContacts,
            ),
            "managecertificateauthorities" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::ManageCertificateAuthorities,
            ),
            "manageissuers" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::ManageIssuers,
            ),
            "getissuers" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::GetIssuers,
            ),
            "listissuers" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::ListIssuers,
            ),
            "setissuers" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::SetIssuers,
            ),
            "deleteissuers" => Self::CertificateManagementOperation(
                KeyVaultAccessPolicyCertificateManagementOperation::DeleteIssuers,
            ),
            "purge" => Self::PrivilegedCertificateOperation(
                KeyVaultAccessPolicyPrivilegedCertificateOperation::Purge,
            ),
            "all" => Self::All(KeyVaultAccessPolicyAllPrivilege::All),
            _ => eyre::bail!("unknown key vault certificate privilege {value:?}"),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyCertificateManagementOperation {
    Get,
    List,
    Update,
    Create,
    Import,
    Delete,
    Recover,
    Backup,
    Restore,
    ManageContacts,
    ManageCertificateAuthorities,
    ManageIssuers,
    GetIssuers,
    ListIssuers,
    SetIssuers,
    DeleteIssuers,
}
impl KeyVaultAccessPolicyCertificateManagementOperation {
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
            Self::ManageContacts => "ManageContacts",
            Self::ManageCertificateAuthorities => "ManageCertificateAuthorities",
            Self::ManageIssuers => "ManageIssuers",
            Self::GetIssuers => "GetIssuers",
            Self::ListIssuers => "ListIssuers",
            Self::SetIssuers => "SetIssuers",
            Self::DeleteIssuers => "DeleteIssuers",
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, facet::Facet, Arbitrary)]
#[repr(C)]
pub enum KeyVaultAccessPolicyPrivilegedCertificateOperation {
    Purge,
}
impl KeyVaultAccessPolicyPrivilegedCertificateOperation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Purge => "Purge",
        }
    }
}

#[cfg(test)]
mod test {
    use crate::KeyVaultAccessPolicyCertificatePrivilege;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json = r#""All""#;
        let privilege: KeyVaultAccessPolicyCertificatePrivilege = facet_json::from_str(json)?;
        match privilege {
            KeyVaultAccessPolicyCertificatePrivilege::All(all) => {
                assert_eq!(all, crate::key_vault_access_policy_all_privilege::KeyVaultAccessPolicyAllPrivilege::All)
            }
            _ => panic!("expected All variant"),
        }
        let lower: KeyVaultAccessPolicyCertificatePrivilege =
            facet_json::from_str(r#""managecontacts""#)?;
        assert_eq!(facet_json::to_string(&lower)?, r#""ManageContacts""#);
        Ok(())
    }
}
