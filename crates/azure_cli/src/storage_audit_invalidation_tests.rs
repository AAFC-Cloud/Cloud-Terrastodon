use crate::AzureStorageAccountAuditRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerListRequest;
use cloud_terrastodon_azure::AzureStorageAccountKind;
use cloud_terrastodon_azure::AzureStorageAccountListRequest;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::AzureStorageAccountSku;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_command::CancellationError;
use cloud_terrastodon_command::CancellationReportExt;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::Ordering;

// A cache-generation marker, independent of transport payloads and decoding.
// Synthetic descendants exercise scope invalidation without assuming page URLs.
async fn marker(
    executor: &CloudTerrastodonExecutor,
    authentication: AzureBearerAuthenticationSelection,
    key: CacheKey,
) -> eyre::Result<Arc<u8>> {
    executor
        .cached_work(
            ExecutorKind::InProcess,
            key,
            CacheOperationOwner::of::<u8>(),
            &authentication.into(),
            NonEmptyString::new("storage-audit-invalidation-marker")?,
            WorkReuse::ReusableRead,
            |_| async { Ok(WorkOutput::new(1, ArtifactObservation::now())) },
        )?
        .await
}

#[tokio::test]
async fn failed_fresh_account_inventory_aborts_audit_without_invalidating_other_scopes()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let authentication = AzureBearerAuthenticationSelection {
                account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
                auth_source: AzureBearerAuthSource::AzureCli,
            };
            let provider = Arc::new(TestAuthenticationProvider::offline("storage-audit-clean")?);
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
            let inventory = AzureStorageAccountListRequest { authentication }.cache_key()?;
            let inventory_page = inventory.join("synthetic/pages/1")?;
            let container_page = AzureStorageAccountBlobContainerListRequest {
                authentication,
                storage_account_id: Cow::Owned("/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount".parse()?),
            }
            .cache_key()?
            .join("synthetic/pages/1")?;
            let other_tenant = AzureStorageAccountListRequest {
                authentication: AzureBearerAuthenticationSelection {
                    tenant_id: "33333333-3333-3333-3333-333333333333".parse()?,
                    ..authentication
                },
            }
            .cache_key()?;
            let old_inventory = marker(&executor, authentication, inventory.clone()).await?;
            let old_inventory_page = marker(&executor, authentication, inventory_page.clone()).await?;
            let retained_containers = marker(&executor, authentication, container_page.clone()).await?;
            let retained_other_tenant = marker(&executor, authentication, other_tenant.clone()).await?;

            let error = AzureStorageAccountAuditRequest { authentication }
                .execute(&executor)
                .with_invalidation(true)
                .await
                .expect_err("failed inventory refresh must stop the audit before classification");
            assert!(format!("{error:#}").contains("offline test forbids credential acquisition"));
            assert!(provider.authorizations.load(Ordering::SeqCst) > 0);

            assert!(!Arc::ptr_eq(
                &old_inventory,
                &marker(&executor, authentication, inventory).await?,
            ));
            assert!(!Arc::ptr_eq(
                &old_inventory_page,
                &marker(&executor, authentication, inventory_page).await?,
            ));
            assert!(Arc::ptr_eq(
                &retained_containers,
                &marker(&executor, authentication, container_page).await?,
            ), "without a refreshed inventory the audit must not guess which container branches to invalidate");
            assert!(Arc::ptr_eq(
                &retained_other_tenant,
                &marker(&executor, authentication, other_tenant).await?,
            ));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelled_container_inventory_preserves_its_error_with_an_active_parent_scope()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let account = Arc::new(AzureStorageAccount {
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
                    allow_blob_public_access: Some(true),
                    ..Default::default()
                },
                tags: Default::default(),
            });
            let cancellation = eyre::Report::new(CancellationError::new("child inventory cancelled"))
                .wrap_err("container list stopped");
            let error = AzureStorageAccountAuditRequest::evaluate(account, Err(cancellation), &scope)
                .expect_err("cancellation must not become an inconclusive account finding");
            assert!(error.is_cancellation());
            assert!(format!("{error:#}").contains("child inventory cancelled"));
            scope.bail_if_cancelled()?;
            Ok(())
        })
        .await
}
