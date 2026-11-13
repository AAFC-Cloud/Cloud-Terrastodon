use crate::AzureAuditFinding;
use crate::AzureStorageAccountAuditRequest;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerLeaseState;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerLeaseStatus;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerProperties;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerPublicAccess;
use cloud_terrastodon_azure::AzureStorageAccountKind;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::AzureStorageAccountPublicNetworkAccess;
use cloud_terrastodon_azure::AzureStorageAccountSku;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::CancellationToken;
use std::collections::HashMap;
use std::sync::Arc;

fn account(enabled: Option<bool>) -> eyre::Result<AzureStorageAccount> {
    Ok(AzureStorageAccount {
        id: "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount".parse()?,
        kind: AzureStorageAccountKind::StorageV2,
        resource_type: None,
        identity: None,
        system_data: None,
        extended_location: None,
        placement: None,
        zones: None,
        name: "syntheticaccount".parse()?,
        location: "eastus".into(),
        sku: AzureStorageAccountSku { name: "Standard_LRS".into(), tier: "Standard".into() },
        properties: AzureStorageAccountProperties {
            allow_blob_public_access: enabled,
            public_network_access: Some(AzureStorageAccountPublicNetworkAccess::Disabled),
            ..Default::default()
        },
        tags: HashMap::from([("Owner".into(), "synthetic-team".into())]),
    })
}

fn container(
    access: AzureStorageAccountBlobContainerPublicAccess,
) -> eyre::Result<AzureStorageAccountBlobContainer> {
    Ok(AzureStorageAccountBlobContainer {
        id: format!(
            "{}/blobServices/default/containers/synthetic",
            account(Some(true))?.id.expanded_form()
        )
        .parse()?,
        name: "synthetic".parse()?,
        resource_type: "Microsoft.Storage/storageAccounts/blobServices/containers".parse()?,
        etag: "synthetic".into(),
        system_data: None,
        properties: AzureStorageAccountBlobContainerProperties {
            version: None,
            deleted: None,
            deleted_time: None,
            remaining_retention_days: None,
            default_encryption_scope: None,
            deny_encryption_scope_override: None,
            public_access: access,
            last_modified_time: "2020-01-01T00:00:00Z".parse()?,
            lease_status: AzureStorageAccountBlobContainerLeaseStatus::Unlocked,
            lease_state: AzureStorageAccountBlobContainerLeaseState::Available,
            lease_duration: None,
            metadata: None,
            immutability_policy: None,
            legal_hold: None,
            has_legal_hold: false,
            has_immutability_policy: false,
            immutable_storage_with_versioning: None,
            enable_nfs_v3_root_squash: None,
            enable_nfs_v3_all_squash: None,
        },
    })
}

#[test]
fn storage_audit_flags_private_and_empty_accounts_regardless_of_network_restrictions()
-> eyre::Result<()> {
    CancellationToken::new().with_scope(|scope| {
        let account = Arc::new(account(Some(true))?);
        for containers in [
            vec![],
            vec![Arc::new(container(
                AzureStorageAccountBlobContainerPublicAccess::None,
            )?)],
        ] {
            let count = containers.len();
            let finding = AzureStorageAccountAuditRequest::evaluate(
                Arc::clone(&account),
                Ok(containers),
                &scope,
            )?
            .unwrap();
            let AzureAuditFinding::StorageAccountUnusedAnonymousAccess {
                account: observed,
                containers,
            } = finding
            else {
                panic!("expected unused anonymous-access finding");
            };
            assert!(Arc::ptr_eq(&account, &observed));
            assert_eq!(containers.len(), count);
            assert_eq!(
                observed.tags.get("Owner").map(String::as_str),
                Some("synthetic-team")
            );
        }
        Ok(())
    })
}

#[test]
fn storage_audit_excludes_disabled_file_only_and_accounts_with_any_public_container()
-> eyre::Result<()> {
    CancellationToken::new().with_scope(|scope| {
        assert!(
            AzureStorageAccountAuditRequest::evaluate(
                Arc::new(account(Some(false))?),
                Ok(vec![]),
                &scope
            )?
            .is_none()
        );
        let mut file_account = account(Some(true))?;
        file_account.kind = AzureStorageAccountKind::FileStorage;
        assert!(
            AzureStorageAccountAuditRequest::evaluate(
                Arc::new(file_account),
                Err(eyre::eyre!("blob inventory unavailable")),
                &scope
            )?
            .is_none()
        );
        for access in [
            AzureStorageAccountBlobContainerPublicAccess::Blob,
            AzureStorageAccountBlobContainerPublicAccess::Container,
        ] {
            let containers = vec![
                Arc::new(container(
                    AzureStorageAccountBlobContainerPublicAccess::None,
                )?),
                Arc::new(container(access)?),
            ];
            assert!(
                AzureStorageAccountAuditRequest::evaluate(
                    Arc::new(account(Some(true))?),
                    Ok(containers),
                    &scope
                )?
                .is_none()
            );
        }
        Ok(())
    })
}

#[test]
fn storage_audit_keeps_missing_settings_and_failed_inventory_inconclusive() -> eyre::Result<()> {
    CancellationToken::new().with_scope(|scope| {
        for (account, containers) in [
            (account(None)?, Ok(vec![])),
            (
                account(Some(true))?,
                Err(eyre::eyre!("a later container page was denied")),
            ),
        ] {
            assert!(matches!(
                AzureStorageAccountAuditRequest::evaluate(Arc::new(account), containers, &scope)?,
                Some(AzureAuditFinding::StorageAccountAnonymousAccessInconclusive { .. })
            ));
        }
        Ok(())
    })
}

#[test]
fn storage_audit_rejects_unknown_deleted_and_cross_account_container_observations()
-> eyre::Result<()> {
    CancellationToken::new().with_scope(|scope| {
        let unknown = container(AzureStorageAccountBlobContainerPublicAccess::Unknown(
            "future-access".into(),
        ))?;
        let mut deleted = container(AzureStorageAccountBlobContainerPublicAccess::None)?;
        deleted.properties.deleted = Some(true);
        let mut wrong_account = container(AzureStorageAccountBlobContainerPublicAccess::None)?;
        wrong_account.id.storage_account_id.storage_account_name = "otheraccount".parse()?;
        for container in [unknown, deleted, wrong_account] {
            assert!(matches!(
                AzureStorageAccountAuditRequest::evaluate(
                    Arc::new(account(Some(true))?),
                    Ok(vec![Arc::new(container)]),
                    &scope
                )?,
                Some(AzureAuditFinding::StorageAccountAnonymousAccessInconclusive { .. })
            ));
        }
        Ok(())
    })
}
