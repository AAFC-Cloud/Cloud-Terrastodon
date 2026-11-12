#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraSecurityGroupListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraPrincipal;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

struct SyntheticPage;

// Populate the real category request contract through the public cache boundary.
// Each category consumes a synthetic page so the real principal assembly must
// retain dependencies even when every child itself is already a memory hit.
async fn seed_category<R: HasCacheKey + Facet<'static> + 'static, T: Send + Sync + 'static>(
    executor: &CloudTerrastodonExecutor,
    request: &R,
    selection: &AuthenticationSelection,
    records: Vec<Arc<T>>,
) -> Result<Arc<Vec<Arc<T>>>> {
    let key = request.cache_key()?;
    executor
        .cached_work(
            ExecutorKind::InProcess,
            key.clone(),
            CacheOperationOwner::of::<R>(),
            selection,
            NonEmptyString::new(facet_json::to_string(request)?)?,
            WorkReuse::ReusableRead,
            move |child| async move {
                let page =
                    child
                        .cached_work(
                            ExecutorKind::Rest,
                            key.join("synthetic-page")?,
                            CacheOperationOwner::of::<SyntheticPage>(),
                            selection,
                            NonEmptyString::new("synthetic-page")?,
                            WorkReuse::ReusableRead,
                            move |_| async move {
                                Ok(WorkOutput::new(records, ArtifactObservation::now()))
                            },
                        )?
                        .await?;
                Ok(WorkOutput::new(
                    page.as_ref().clone(),
                    ArtifactObservation::now(),
                ))
            },
        )?
        .await
}

fn user(name: &str) -> EntraUser {
    EntraUser {
        business_phones: Vec::new(),
        display_name: name.to_owned(),
        external_user_state: None,
        external_user_state_change_date_time: None,
        given_name: None,
        id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        job_title: None,
        mail: None,
        other_mails: Vec::new(),
        mobile_phone: None,
        office_location: None,
        preferred_language: None,
        surname: None,
        user_principal_name: "synthetic@example.invalid".parse().unwrap(),
    }
}

#[tokio::test]
async fn principal_assembly_shares_records_and_tracks_warm_child_page_dependencies() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let authentication = Arc::new(TestAuthenticationProvider::offline(
                "synthetic-test-session",
            )?);
            let executor =
                CloudTerrastodonExecutor::new(cache.clone(), authentication.clone(), scope.clone());
            let tenant_id: AzureTenantId = "22222222-2222-2222-2222-222222222222".parse()?;
            let auth_source = AzureBearerAuthSource::AzureCli;
            let selected = AzureBearerAuthenticationSelection {
                account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                tenant_id,
                auth_source,
            };
            let selection = selected.into();
            let users = EntraUserListRequest {
                authentication: selected,
            };
            let groups = EntraSecurityGroupListRequest {
                authentication: selected,
            };
            let principals = EntraServicePrincipalListRequest {
                authentication: selected,
            };
            let original = Arc::new(user("original"));
            seed_category(&executor, &users, &selection, vec![original.clone()]).await?;
            seed_category::<_, EntraGroup>(&executor, &groups, &selection, Vec::new()).await?;
            seed_category::<_, EntraServicePrincipal>(
                &executor,
                &principals,
                &selection,
                Vec::new(),
            )
            .await?;
            let request = || EntraPrincipalListRequest {
                authentication: selected,
            };
            let (first, second) = executor
                .join2(
                    |child| async move { request().execute(&child).await },
                    |child| async move { request().execute(&child).await },
                )
                .await?;
            assert!(Arc::ptr_eq(&first, &second));
            let id = EntraPrincipalId::UserId(original.id);
            let Some(EntraPrincipal::User(record)) = first.get(&id) else {
                panic!("principal assembly retains its user variant")
            };
            assert!(Arc::ptr_eq(&original, record));

            // Invalidate a transitive page, outside the principal aggregate's prefix.
            cache
                .invalidate(&users.cache_key()?.join("synthetic-page")?.scope, &scope)
                .await?;
            let replacement = Arc::new(user("replacement"));
            seed_category(&executor, &users, &selection, vec![replacement.clone()]).await?;
            let refreshed = request().execute(&executor).await?;
            assert!(!Arc::ptr_eq(&first, &refreshed));
            let Some(EntraPrincipal::User(record)) = refreshed.get(&id) else {
                panic!("principal assembly retains its user variant")
            };
            assert!(Arc::ptr_eq(&replacement, record));
            assert_eq!(original.display_name, "original");
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);

            // Aggregates have no durable selection: an independent cache owner cannot
            // skip rebuilding children, even when it points at the same root.
            let independent = CloudTerrastodonExecutor::new(
                WorkCache::new(cache.root())?,
                authentication.clone(),
                scope,
            );
            assert!(request().execute(&independent).await.is_err());
            assert!(authentication.authorizations.load(Ordering::SeqCst) > 0);
            Ok(())
        })
        .await
}
