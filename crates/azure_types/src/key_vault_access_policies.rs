use crate::KeyVaultAccessPolicyCertificatePrivilege;
use crate::KeyVaultAccessPolicyKeyPrivilege;
use crate::KeyVaultAccessPolicySecretPrivilege;
use crate::PrincipalId;
use crate::tenant_id::AzureTenantId;
use arbitrary::Arbitrary;

#[derive(Debug, PartialEq, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct KeyVaultAccessPolicy {
    pub object_id: PrincipalId,
    pub tenant_id: AzureTenantId,
    pub permissions: KeyVaultAccessPolicyPermissions,
}

#[derive(Debug, PartialEq, facet::Facet, Arbitrary)]
pub struct KeyVaultAccessPolicyPermissions {
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<KeyVaultAccessPolicyKeyPrivilege>)]
    pub keys: Vec<KeyVaultAccessPolicyKeyPrivilege>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<KeyVaultAccessPolicySecretPrivilege>)]
    pub secrets: Vec<KeyVaultAccessPolicySecretPrivilege>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<KeyVaultAccessPolicyCertificatePrivilege>
    )]
    pub certificates: Vec<KeyVaultAccessPolicyCertificatePrivilege>,
}

#[cfg(test)]
mod test {
    use crate::KeyVaultAccessPolicy;

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
        let access_policies: Vec<KeyVaultAccessPolicy> = facet_json::from_str(json)?;
        let permissions = &access_policies[0].permissions;
        assert_eq!(permissions.certificates.len(), 16);
        assert_eq!(permissions.keys.len(), 20);
        assert_eq!(permissions.secrets.len(), 8);
        let reparsed = facet_json::from_str::<Vec<KeyVaultAccessPolicy>>(&facet_json::to_string(
            &access_policies,
        )?)?;
        assert_eq!(access_policies, reparsed);
        Ok(())
    }

    #[test]
    pub fn null_permission_lists_default_to_empty() -> eyre::Result<()> {
        let json = r#"
        {
            "tenantId": "cd284393-fae7-4efd-aaae-735488ca3c42",
            "objectId": "a2143e3a-f6bd-431d-b835-aca79e0c2c0a",
            "permissions": {
                "certificates": null,
                "keys": null,
                "secrets": null
            }
        }
        "#;
        let access_policy: KeyVaultAccessPolicy = facet_json::from_str(json)?;
        assert!(access_policy.permissions.certificates.is_empty());
        assert!(access_policy.permissions.keys.is_empty());
        assert!(access_policy.permissions.secrets.is_empty());
        Ok(())
    }
}
