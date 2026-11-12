use crate::domain::DesiredGroup;
use cloud_terrastodon::auth_types::AuthSource;
use cloud_terrastodon::auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Resolve the entire selected scope before any Graph reads or credential exchanges.
pub(crate) fn preflight_tenants(
    groups: &[DesiredGroup],
    account: Option<&str>,
    source: AuthSource,
    executor: &CloudTerrastodonExecutor,
) -> Result<BTreeMap<AzureTenantId, AzureBearerAuthenticationSelection>> {
    ensure!(
        source != AuthSource::None,
        "authentication is disabled by auth source none"
    );
    groups.iter().map(|group| group.tenant_id).collect::<BTreeSet<_>>()
        .into_iter().map(|tenant| {
            executor.resolve_azure_authentication(tenant, account, source)
                .map(|selection| (tenant, selection))
                .wrap_err_with(|| format!(
                    "Tenant authentication preflight failed for {tenant}. No directory reads were started. Register a matching account or select a tenant explicitly with --only-tenant."
                ))
        }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DesireDocument;
    use crate::domain::ReconciliationMode;
    use cloud_terrastodon::auth::CloudTerrastodonAccountCatalogue;
    use cloud_terrastodon::auth::CloudTerrastodonAccountStore;
    use cloud_terrastodon::auth::CredentialAuthenticationProvider;
    use cloud_terrastodon::auth_types::CloudTerrastodonAccountCredential;
    use cloud_terrastodon::auth_types::CloudTerrastodonAccountProfile;
    use cloud_terrastodon_cache::WorkCache;
    use std::sync::Arc;
    use teamy_cancellation::CancellationToken;

    fn group(tenant: &str) -> DesiredGroup {
        DesiredGroup {
            tenant_id: tenant.parse().unwrap(),
            group_id: "33333333-3333-4333-8333-333333333333".parse().unwrap(),
            display_name: "Synthetic".parse().unwrap(),
            members_mode: ReconciliationMode::Ignore,
            members: Vec::new(),
            owners_mode: ReconciliationMode::Ignore,
            owners: Vec::new(),
        }
    }

    #[tokio::test]
    async fn mixed_tenants_fail_before_credential_exchange_and_explicit_selection_is_bounded()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let first = "11111111-1111-4111-8111-111111111111";
                let second = "22222222-2222-4222-8222-222222222222";
                let directory = tempfile::tempdir()?;
                let catalogue = CloudTerrastodonAccountCatalogue {
                    accounts: vec![CloudTerrastodonAccountProfile::new(
                        "44444444-4444-4444-8444-444444444444".parse()?,
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli {
                            tenant_id: first.parse()?,
                        },
                    )],
                    defaults: Vec::new(),
                };
                std::fs::write(
                    directory.path().join("accounts.json"),
                    facet_json::to_string(&catalogue)?,
                )?;
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::from_store(
                        CloudTerrastodonAccountStore::new(directory.path())?,
                    )),
                    scope,
                );
                let desire = DesireDocument::try_new(
                    vec![group(first), group(second)],
                    "synthetic-document".into(),
                )?;
                let error =
                    preflight_tenants(desire.groups(), Some("admin"), AuthSource::Auto, &executor)
                        .unwrap_err();
                assert!(error.to_string().contains(second));
                assert!(error.to_string().contains("No directory reads"));
                let first_tenant = first.parse()?;
                let selected = desire.select_tenant(first_tenant)?;
                assert_eq!(selected.source_digest(), "synthetic-document");
                let selections = preflight_tenants(
                    selected.groups(),
                    Some("admin"),
                    AuthSource::Auto,
                    &executor,
                )?;
                assert_eq!(selections.len(), 1);
                assert_eq!(selections[&first_tenant].tenant_id.to_string(), first);
                assert!(selected.select_tenant(second.parse()?).is_err());
                assert!(!directory.path().join("credentials").exists());
                Ok(())
            })
            .await
    }
}
