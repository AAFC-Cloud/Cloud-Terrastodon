use crate::prelude::KeyVaultAccessPolicyCertificatePrivilege;
use crate::prelude::KeyVaultAccessPolicyKeyPrivilege;
use crate::prelude::KeyVaultAccessPolicySecretPrivilege;
use crate::prelude::PrincipalId;
use crate::serde_helpers::deserialize_default_if_null;
use crate::tenants::TenantId;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KeyVaultAccessPolicy {
    pub object_id: PrincipalId,
    pub tenant_id: TenantId,
    pub permissions: KeyVaultAccessPolicyPermissions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyVaultAccessPolicyPermissions {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub keys: Vec<KeyVaultAccessPolicyKeyPrivilege>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub secrets: Vec<KeyVaultAccessPolicySecretPrivilege>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub certificates: Vec<KeyVaultAccessPolicyCertificatePrivilege>,
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json = r#"[
            {
                "tenantId": "cd284393-fae7-4efd-aaae-735488ca3c42",
                "objectId": "a2143e3a-f6bd-431d-b835-aca79e0c2c0a",
                "permissions": {
                    "certificates": [
                        "Get",
                        "List",
                        "Update",
                        "Create",
                        "Import",
                        "Delete",
                        "Recover",
                        "Backup",
                        "Restore",
                        "ManageContacts",
                        "ManageIssuers",
                        "GetIssuers",
                        "ListIssuers",
                        "SetIssuers",
                        "DeleteIssuers",
                        "Purge"
                    ],
                    "keys": [
                        "Get",
                        "List",
                        "Update",
                        "Create",
                        "Import",
                        "Delete",
                        "Recover",
                        "Backup",
                        "Restore",
                        "GetRotationPolicy",
                        "SetRotationPolicy",
                        "Rotate",
                        "Encrypt",
                        "Decrypt",
                        "UnwrapKey",
                        "WrapKey",
                        "Verify",
                        "Sign",
                        "Purge",
                        "Release"
                    ],
                    "secrets": [
                        "Get",
                        "List",
                        "Set",
                        "Delete",
                        "Recover",
                        "Backup",
                        "Restore",
                        "Purge"
                    ]
                }
            }
        ]"#;
        let _access_policies: super::KeyVaultAccessPolicies = serde_json::from_str(json)?;
        Ok(())
    }
}
