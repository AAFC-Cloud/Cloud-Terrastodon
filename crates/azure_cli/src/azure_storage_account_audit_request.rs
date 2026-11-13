use crate::AzureAuditFinding;
use crate::AzureStorageAccountAuditReport;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerListRequest;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerPublicAccess;
use cloud_terrastodon_azure::AzureStorageAccountKind;
use cloud_terrastodon_azure::AzureStorageAccountListRequest;
use cloud_terrastodon_command::CancellationReportExt;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;

/// Find accounts permitting anonymous blob access without any public containers.
///
/// [Account and container access settings](https://learn.microsoft.com/en-us/azure/storage/blobs/anonymous-read-access-prevent#about-anonymous-read-access)
/// are evaluated independently of network restrictions. This reads management-plane
/// inventories, not blob contents or anonymous endpoints (including static websites).
/// Missing settings and failed inventories remain inconclusive. An empty, successfully
/// enumerated account qualifies; a partially enumerated account never does.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureStorageAccountAuditRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl AzureStorageAccountAuditRequest {
    pub fn evaluate(
        account: Arc<AzureStorageAccount>,
        containers: eyre::Result<Vec<Arc<AzureStorageAccountBlobContainer>>>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Option<AzureAuditFinding>> {
        if matches!(account.kind, AzureStorageAccountKind::FileStorage)
            || account.properties.allow_blob_public_access == Some(false)
        {
            return Ok(None);
        }
        if account.properties.allow_blob_public_access.is_none() {
            return Ok(Some(
                AzureAuditFinding::StorageAccountAnonymousAccessInconclusive {
                    account,
                    reason: "The account's allowBlobPublicAccess setting was not returned".into(),
                },
            ));
        }
        let containers = match containers {
            Ok(containers) => containers,
            Err(error) if error.is_cancellation() => return Err(error),
            Err(error) => {
                return Ok(Some(
                    AzureAuditFinding::StorageAccountAnonymousAccessInconclusive {
                        account,
                        reason: format!("Could not enumerate every blob container: {error:#}"),
                    },
                ));
            }
        };
        let mut has_public_container = false;
        for container in &containers {
            cancellation.bail_if_cancelled()?;
            if container.id.storage_account_id != account.id {
                return Ok(Some(AzureAuditFinding::StorageAccountAnonymousAccessInconclusive {
                    account,
                    reason: "Container inventory included an identity from a different storage account".into(),
                }));
            }
            if container.properties.deleted == Some(true) {
                return Ok(Some(AzureAuditFinding::StorageAccountAnonymousAccessInconclusive {
                    account,
                    reason: "Live container inventory unexpectedly included a soft-deleted container".into(),
                }));
            }
            match &container.properties.public_access {
                AzureStorageAccountBlobContainerPublicAccess::None => {}
                AzureStorageAccountBlobContainerPublicAccess::Blob
                | AzureStorageAccountBlobContainerPublicAccess::Container => {
                    has_public_container = true
                }
                AzureStorageAccountBlobContainerPublicAccess::Unknown(value) => {
                    let reason = format!(
                        "Container {} has an unrecognized publicAccess setting: {value:?}",
                        container.name
                    );
                    return Ok(Some(
                        AzureAuditFinding::StorageAccountAnonymousAccessInconclusive {
                            account,
                            reason,
                        },
                    ));
                }
            }
        }
        if has_public_container {
            Ok(None)
        } else {
            Ok(Some(
                AzureAuditFinding::StorageAccountUnusedAnonymousAccess {
                    account,
                    containers,
                },
            ))
        }
    }
}

impl CacheInvalidatableRequest for AzureStorageAccountAuditRequest {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            // Discover current account identities from a refreshed inventory before
            // clearing their separate container-page branches. Request owners select
            // their own keys; unrelated tenants and other ARM reads stay untouched.
            let accounts = AzureStorageAccountListRequest {
                authentication: self.authentication,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?;
            accounts
                .into_iter()
                .filter(|account| !matches!(account.kind, AzureStorageAccountKind::FileStorage))
                .map_concurrent(executor, 16.try_into()?, async |child, account| {
                    AzureStorageAccountBlobContainerListRequest {
                        authentication: self.authentication,
                        storage_account_id: Cow::Borrowed(&account.id),
                    }
                    .invalidate(&child)
                    .await
                })
                .await?;
            Ok(())
        })
    }
}

impl ExecutableRequest for AzureStorageAccountAuditRequest {
    type Output = AzureStorageAccountAuditReport;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let accounts = AzureStorageAccountListRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            let accounts_observed = accounts.len();
            let observations = accounts
                .into_iter()
                .map_concurrent(&executor, 16.try_into()?, async |child, account| {
                    let containers = if account.properties.allow_blob_public_access == Some(true)
                        && !matches!(account.kind, AzureStorageAccountKind::FileStorage)
                    {
                        AzureStorageAccountBlobContainerListRequest {
                            authentication: self.authentication,
                            storage_account_id: Cow::Borrowed(&account.id),
                        }
                        .execute(&child)
                        .await
                    } else {
                        Ok(Vec::new())
                    };
                    // Cancellation must abort the audit, not become an ordinary
                    // inconclusive account finding while other work continues.
                    child.cancellation().bail_if_cancelled()?;
                    Self::evaluate(account, containers, child.cancellation())
                })
                .await?;
            Ok(AzureStorageAccountAuditReport {
                accounts_observed,
                findings: observations.into_iter().flatten().collect(),
            })
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAuditRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAuditRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureStorageAccountAuditRequest => AzureStorageAccountAuditReport, effects = [Read]);
