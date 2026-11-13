#![cfg(test)]

use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectState;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectVisibility;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use cloud_terrastodon_rest::SerializableRestResponse;
use eyre::Result;
use facet::Facet;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

// This is the persisted REST identity wire format, not an alternate transport.
// Seeding a separate cache owner also verifies that diagnostic output type names
// do not control decoding when the production adapter restores leaf artifacts.
#[derive(Facet)]
struct PersistedRestFingerprint {
    service: String,
    method: String,
    url: String,
    body: Option<String>,
    headers: BTreeMap<String, Vec<String>>,
}

fn project(name: &str, id: &str) -> Result<Arc<AzureDevOpsProject>> {
    Ok(Arc::new(AzureDevOpsProject {
        abbreviation: None,
        default_team_image_url: None,
        description: None,
        id: id.parse()?,
        last_update_time: "2020-01-01T00:00:00Z".parse()?,
        name: name.parse()?,
        revision: 1,
        state: AzureDevOpsProjectState::WellFormed,
        url: format!("https://dev.azure.com/offline-projects/_apis/projects/{id}"),
        visibility: AzureDevOpsProjectVisibility::Private,
    }))
}

fn request(
    org: &str,
    auth_selection: AzureDevOpsAuthSelection,
) -> Result<AzureDevOpsProjectsListRequest<'static>> {
    Ok(AzureDevOpsProjectsListRequest {
        org_url: Cow::Owned(org.parse()?),
        auth_selection,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "test artifact seeding specifies a page and its observation independently"
)]
async fn seed_page(
    root: &Path,
    request: &AzureDevOpsProjectsListRequest<'_>,
    page: usize,
    continuation: Option<&str>,
    next: Option<&str>,
    value: Vec<Arc<AzureDevOpsProject>>,
    auth: &TestAuthenticationProvider,
    scope: &CancellationScope,
) -> Result<()> {
    let mut query = vec![("api-version", "7.1")];
    if let Some(token) = continuation {
        query.push(("continuationToken", token));
    }
    let fingerprint = PersistedRestFingerprint {
        service: "AzureDevOps".to_owned(),
        method: "GET".to_owned(),
        url: request
            .org_url
            .api_url("dev.azure.com", "_apis/projects", &query)?
            .to_string(),
        body: None,
        headers: BTreeMap::new(),
    };
    let cache = WorkCache::new(root)?;
    let entry = cache.artifacts::<SerializableRestResponse>(
        ExecutorKind::Rest,
        request.cache_key()?.join(page.to_string())?,
        CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
        auth.admit(&request.auth_selection.into())?,
        NonEmptyString::new(
            blake3::hash(&facet_json::to_vec(&fingerprint)?)
                .to_hex()
                .to_string(),
        )?,
        ArtifactRetention::Full,
    )?;
    let mut headers = HeaderMap::new();
    if let Some(token) = next {
        headers.insert("x-ms-continuationtoken", token.parse()?);
    }
    let body = AzureDevOpsCollectionResponse {
        count: value.len().try_into()?,
        value,
    };
    let response =
        SerializableRestResponse::new(StatusCode::OK, &headers, facet_json::to_string(&body)?);
    let stored = entry
        .begin_attempt(BTreeMap::new(), scope)
        .await?
        .expect("full retention")
        .finish(
            ArtifactSet {
                payload: facet_json::to_vec(&response)?,
                diagnostics: BTreeMap::new(),
            },
            ArtifactObservation::now(),
            scope,
        )
        .await?;
    assert!(entry.publish(&stored, scope).await?);
    Ok(())
}

#[tokio::test]
async fn assembled_projects_share_records_and_track_page_invalidation() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let auth = Arc::new(TestAuthenticationProvider {
                allow_anonymous: false.into(),
                ..TestAuthenticationProvider::offline("synthetic-test-session")?
            });
            let executor =
                CloudTerrastodonExecutor::new(cache.clone(), auth.clone(), scope.clone());
            let request = request(
                "offline-projects",
                AzureDevOpsAuthSelection::PersonalAccessToken {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                },
            )?;
            let first = project("First", "11111111-1111-1111-1111-111111111111")?;
            let second = project("Second", "22222222-2222-2222-2222-222222222222")?;
            seed_page(
                cache.root(),
                &request,
                0,
                None,
                Some("next"),
                vec![first.clone()],
                &auth,
                &scope,
            )
            .await?;
            seed_page(
                cache.root(),
                &request,
                1,
                Some("next"),
                None,
                vec![second.clone()],
                &auth,
                &scope,
            )
            .await?;

            let unrelated_request = self::request(
                "other-offline-projects",
                AzureDevOpsAuthSelection::PersonalAccessToken {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                },
            )?;
            seed_page(
                cache.root(),
                &unrelated_request,
                0,
                None,
                None,
                vec![first.clone()],
                &auth,
                &scope,
            )
            .await?;

            // Both calls use the production request, including pagination and the
            // real REST artifact decoder. Authorization fails before any network use.
            let (initial, joined) = executor
                .join2(
                    async |child| request.clone().execute(&child).await,
                    async |child| request.clone().execute(&child).await,
                )
                .await?;
            assert!(Arc::ptr_eq(&initial, &joined));
            assert_eq!(initial.as_ref(), &vec![first, second]);
            let warm = request.clone().execute(&executor).await?;
            assert!(Arc::ptr_eq(&initial, &warm));
            let aggregate_artifact = cache.artifacts::<Vec<Arc<AzureDevOpsProject>>>(
                ExecutorKind::InProcess,
                request.cache_key()?,
                CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
                auth.admit(&request.auth_selection.into())?,
                NonEmptyString::new(
                    blake3::hash(&facet_json::to_vec(&request)?)
                        .to_hex()
                        .to_string(),
                )?,
                ArtifactRetention::Full,
            )?;
            assert!(
                !aggregate_artifact.directory()?.exists(),
                "assembled results are memory-only"
            );

            let unrelated = unrelated_request.clone().execute(&executor).await?;
            cache
                .invalidate(&request.cache_key()?.join("1")?.scope, &scope)
                .await?;
            // The missing child retires the warm aggregate. The offline provider
            // rejects the replacement fetch before any network request can start.
            assert!(request.clone().execute(&executor).await.is_err());
            assert_eq!(
                initial[1].name.as_ref(),
                "Second",
                "held snapshots survive invalidation"
            );
            assert!(Arc::ptr_eq(
                &unrelated,
                &unrelated_request.execute(&executor).await?
            ));

            // Release every root owner before a separate fixture writer updates the
            // leaf. Holding result snapshots does not retain cache ownership.
            drop(aggregate_artifact);
            drop(executor);
            drop(cache);
            let replacement = project("Replacement", "33333333-3333-3333-3333-333333333333")?;
            seed_page(
                temporary.path(),
                &request,
                1,
                Some("next"),
                None,
                vec![replacement.clone()],
                &auth,
                &scope,
            )
            .await?;
            // A new owner rebuilds the aggregate from the retained leaf artifacts.
            let restored_executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                auth.clone(),
                scope.clone(),
            );
            let restored = request.clone().execute(&restored_executor).await?;
            assert!(!Arc::ptr_eq(&initial, &restored));
            assert_eq!(restored[0], initial[0]);
            assert_eq!(restored[1], replacement);
            assert!(Arc::ptr_eq(
                &restored,
                &request.clone().execute(&restored_executor).await?
            ));
            assert_eq!(
                facet_json::to_string(&restored)?,
                facet_json::to_string(restored.as_ref())?
            );

            auth.allow_authenticated.store(false, Ordering::SeqCst);
            assert!(
                request
                    .execute(&restored_executor)
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("synthetic policy denies authenticated selection")
            );
            assert_eq!(
                auth.authorizations.load(Ordering::SeqCst),
                1,
                "only the invalidated page requests a credential; cache hits do not"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn selected_bearer_tenants_and_pat_have_independent_aggregate_variants() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            let auth = Arc::new(TestAuthenticationProvider {
                allow_anonymous: false.into(),
                ..TestAuthenticationProvider::offline("synthetic-test-session")?
            });
            let executor =
                CloudTerrastodonExecutor::new(cache.clone(), auth.clone(), scope.clone());
            let selections = [
                AzureDevOpsAuthSelection::PersonalAccessToken {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                },
                AzureDevOpsAuthSelection::Bearer(AzureBearerAuthenticationSelection {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id: "11111111-1111-1111-1111-111111111111".parse()?,
                }),
                AzureDevOpsAuthSelection::Bearer(AzureBearerAuthenticationSelection {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
                }),
                AzureDevOpsAuthSelection::Bearer(AzureBearerAuthenticationSelection {
                    account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                    auth_source: AzureBearerAuthSource::Browser,
                    tenant_id: "11111111-1111-1111-1111-111111111111".parse()?,
                }),
            ];
            let project = project("Synthetic", "33333333-3333-3333-3333-333333333333")?;
            // Independent writers finish before this executor first acquires the root.
            for selection in selections {
                seed_page(
                    cache.root(),
                    &request("offline-projects", selection)?,
                    0,
                    None,
                    None,
                    vec![project.clone()],
                    &auth,
                    &scope,
                )
                .await?;
            }
            let mut values = Vec::new();
            for selection in selections {
                let request = request("offline-projects", selection)?;
                let serialized = facet_json::to_string(&request)?;
                let borrowed = AzureDevOpsProjectsListRequest {
                    org_url: Cow::Borrowed(request.org_url.as_ref()),
                    auth_selection: selection,
                };
                let replayed =
                    facet_json::from_str::<AzureDevOpsProjectsListRequest<'static>>(&serialized)?;
                let result = borrowed.execute(&executor).await?;
                assert!(values.iter().all(|other| !Arc::ptr_eq(other, &result)));
                assert!(Arc::ptr_eq(&result, &replayed.execute(&executor).await?));
                assert!(Arc::ptr_eq(&result, &request.execute(&executor).await?));
                values.push(result);
            }
            assert_eq!(auth.authorizations.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}

#[test]
fn project_invalidation_scopes_preserve_server_and_organization_identity() -> Result<()> {
    let selection = AzureDevOpsAuthSelection::PersonalAccessToken {
        account_id: "11111111-1111-1111-1111-111111111111".parse()?,
    };
    let cloud = request("https://dev.azure.com/offline", selection)?;
    let alias = request("https://offline.visualstudio.com", selection)?;
    assert_eq!(cloud.cache_key()?, alias.cache_key()?);
    let scopes = [
        "http://server.example/tfs/Collection",
        "https://server.example/tfs/Collection",
        "https://other.example/tfs/Collection",
        "https://server.example/instance/Collection",
        "https://server.example/tfs/Other",
        "https://server.example/tfs/With%20Space",
        "https://server.example/tfs/With%2520Space",
    ]
    .into_iter()
    .map(|org| Ok(request(org, selection)?.cache_key()?.scope))
    .collect::<Result<Vec<_>>>()?;
    for (index, scope) in scopes.iter().enumerate() {
        assert!(scopes.iter().skip(index + 1).all(|other| scope != other));
    }
    Ok(())
}
