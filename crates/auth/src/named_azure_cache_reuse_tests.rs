use super::CloudTerrastodonAccountStore;
use crate::AuthContext;
use crate::BrowserSession;
use crate::CredentialAuthenticationProvider;
use crate::ResolvedAuthContext;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountPrincipal;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::AuthenticationProvider;
use eyre::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn named_azure_account_reuses_disk_artifacts_across_runs_and_sign_in_methods() -> Result<()> {
    struct SyntheticResourceGroupList;

    let directory = tempfile::tempdir()?;
    let account_root = directory.path().join("accounts");
    let cache_root = directory.path().join("cache");
    let store = CloudTerrastodonAccountStore::new(&account_root)?;
    let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
    let client_id = EntraApplicationClientId::new(Uuid::from_u128(2));

    CancellationToken::new()
        .with_async_scope(async |scope| {
            let account = store
                .add(
                    "admin".parse()?,
                    CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                    &scope,
                )
                .await?;
            store
                .register_credential(
                    account.id,
                    CloudTerrastodonAccountCredential::Browser {
                        tenant_id,
                        client_id,
                    },
                    &scope,
                )
                .await?;
            let other_account = store
                .add(
                    "reader".parse()?,
                    CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                    &scope,
                )
                .await?;
            let mut catalogue = store.load()?;
            for profile in &mut catalogue.accounts {
                profile.verify_principal(CloudTerrastodonAccountPrincipal::EntraUser(
                    EntraUserId::new(Uuid::new_v4()),
                ))?;
                for registration in &mut profile.credentials {
                    registration.session = CloudTerrastodonAccountSession::Active {
                        generation: Uuid::new_v4(),
                    };
                }
            }
            store.save(&catalogue)?;

            let azure_cli = |account_id| {
                AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                    account_id,
                    tenant_id,
                    auth_source: AzureBearerAuthSource::AzureCli,
                })
            };
            let open_provider = || -> Result<_> {
                Ok(CredentialAuthenticationProvider::from_store(
                    CloudTerrastodonAccountStore::new(&account_root)?,
                ))
            };
            let open_entry = |partition| {
                WorkCache::new(&cache_root)?.artifacts::<String>(
                    ExecutorKind::Rest,
                    CacheKey::from_relative_path("az/resource-graph/synthetic-resource-groups/0")?,
                    CacheOperationOwner::of::<SyntheticResourceGroupList>(),
                    partition,
                    NonEmptyString::new("synthetic-request-fingerprint")?,
                    ArtifactRetention::Full,
                )
            };
            let payload = b"synthetic result";
            {
                let provider = open_provider()?;
                let entry = open_entry(provider.admit(&azure_cli(account.id))?)?;
                let attempt = entry.begin_attempt(BTreeMap::new(), &scope).await?.unwrap();
                let stored = attempt
                    .finish(
                        ArtifactSet {
                            payload: payload.to_vec(),
                            diagnostics: BTreeMap::new(),
                        },
                        ArtifactObservation::now(),
                        &scope,
                    )
                    .await?;
                assert!(entry.publish(&stored, &scope).await?);
            }

            let reopened = open_provider()?;
            assert_eq!(
                open_entry(reopened.admit(&azure_cli(account.id))?)?
                    .load(&scope)
                    .await?
                    .unwrap()
                    .artifacts
                    .payload
                    .as_slice(),
                payload,
            );
            assert!(
                open_entry(reopened.admit(&azure_cli(other_account.id))?)?
                    .load(&scope)
                    .await?
                    .is_none()
            );

            // Construct the captured browser context directly to avoid touching
            // the OS credential store or making a token request.
            let profile = catalogue.by_id(account.id)?;
            let CloudTerrastodonAccountSession::Active { generation } = profile
                .registration(CloudTerrastodonCredentialSource::Browser)?
                .session
            else {
                unreachable!("test creates an active browser session");
            };
            let mut browser = ResolvedAuthContext::new(
                AuthSource::Browser,
                None,
                true,
                Some(BrowserSession {
                    tenant_id,
                    client_id,
                    refresh_token: "synthetic-refresh-token".to_owned(),
                }),
            );
            browser.account_id = account.id;
            browser.tenant_id = Some(tenant_id);
            browser.credential_store = Some(store.credential_store(
                profile,
                CloudTerrastodonCredentialSource::Browser,
                generation,
            ));
            let browser =
                CredentialAuthenticationProvider::from_accounts([AuthContext::Resolved(
                    Arc::new(browser),
                )])?;
            let browser_selection =
                AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                    account_id: account.id,
                    tenant_id,
                    auth_source: AzureBearerAuthSource::Browser,
                });
            let entry = open_entry(browser.admit(&browser_selection)?)?;
            assert_eq!(
                entry
                    .load(&scope)
                    .await?
                    .unwrap()
                    .artifacts
                    .payload
                    .as_slice(),
                payload,
            );
            eyre::Ok(())
        })
        .await
}
