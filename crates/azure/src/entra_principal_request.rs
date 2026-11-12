use crate::EntraGroupGetRequest;
use crate::EntraServicePrincipalRequest;
use crate::EntraUserGetRequest;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphBatchResponse;
use crate::MicrosoftGraphBatchResponseEntryBody;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraPrincipal;
use cloud_terrastodon_azure_types::EntraPrincipalLookup;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// Resolve a principal by its known kind, or probe Graph v1.0 entity routes with a [JSON batch](https://learn.microsoft.com/graph/json-batching).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraPrincipalRequest {
    pub lookup: EntraPrincipalLookup,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraPrincipalRequest {
    /// The batch ID identifies the principal schema; successful Graph bodies do
    /// not consistently contain an OData discriminator. Decode only inside the
    /// canonical principal initializer, so a memory hit retains the typed record.
    fn decode_batch(
        &self,
        batch: MicrosoftGraphBatchResponse<ArbitraryJson>,
    ) -> eyre::Result<EntraPrincipal> {
        let mut errors = Vec::new();
        for response in batch.responses {
            match &response.body {
                MicrosoftGraphBatchResponseEntryBody::Success(body) => {
                    return match response.id.as_str() {
                        "user" => Ok(EntraPrincipal::from(facet_json::from_str::<EntraUser>(
                            body.as_ref(),
                        )?)),
                        "group" => Ok(EntraPrincipal::from(facet_json::from_str::<EntraGroup>(
                            body.as_ref(),
                        )?)),
                        "service-principal" => Ok(EntraPrincipal::from(facet_json::from_str::<
                            EntraServicePrincipal,
                        >(
                            body.as_ref()
                        )?)),
                        other => {
                            eyre::bail!(
                                "Unexpected Microsoft Graph principal request id '{other}'"
                            );
                        }
                    };
                }
                MicrosoftGraphBatchResponseEntryBody::Error { error } => {
                    errors.push(format!("{}: {}", response.id, error.message));
                }
            }
        }
        eyre::bail!(
            "No Entra principal found for object id '{}'. {}",
            self.lookup.object_id(),
            errors.join("; ")
        );
    }
}

impl CacheInvalidationKeys for EntraPrincipalRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        let mut keys = vec![self.cache_key()?];
        match self.lookup {
            EntraPrincipalLookup::User(user_id) => keys.push(
                EntraUserGetRequest {
                    lookup: user_id.into(),
                    authentication: self.authentication,
                }
                .cache_key()?,
            ),
            EntraPrincipalLookup::Group(group_id) => keys.push(
                EntraGroupGetRequest {
                    group_id,
                    authentication: self.authentication,
                }
                .cache_key()?,
            ),
            EntraPrincipalLookup::ServicePrincipal(service_principal_id) => keys.push(
                EntraServicePrincipalRequest {
                    service_principal_id,
                    authentication: self.authentication,
                }
                .cache_key()?,
            ),
            EntraPrincipalLookup::Unknown(_) => {}
        }
        Ok(keys)
    }
}

impl HasCacheKey for EntraPrincipalRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "principal",
            self.authentication.tenant_id.to_string().as_str(),
            // Keep the selected lookup route visible in the cache hierarchy.
            match self.lookup {
                EntraPrincipalLookup::User(_) => "user",
                EntraPrincipalLookup::Group(_) => "group",
                EntraPrincipalLookup::ServicePrincipal(_) => "service-principal",
                EntraPrincipalLookup::Unknown(_) => "unknown",
            },
            self.lookup.object_id().to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraPrincipalRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<EntraPrincipal>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
            let selection = self.authentication.into();
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    cache_key.clone(),
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        let principal = match self.lookup {
                            EntraPrincipalLookup::User(user_id) => Ok(EntraUserGetRequest { lookup: (user_id).into(), authentication: self.authentication }.execute(&executor)
                            .await?
                            .into()),
                            EntraPrincipalLookup::Group(group_id) => Ok(EntraGroupGetRequest { group_id: (group_id), authentication: self.authentication }.execute(&executor)
                            .await?
                            .into()),
                            EntraPrincipalLookup::ServicePrincipal(service_principal_id) => {
                                Ok(EntraServicePrincipalRequest { service_principal_id: (service_principal_id), authentication: self.authentication }.execute(&executor)
                                .await?
                                .into())
                            }
                            EntraPrincipalLookup::Unknown(object_id) => {
                                // One batch probes three different entity schemas. Keep its
                                // successful bodies opaque until the response ID selects
                                // the corresponding principal decoder below.
                                let mut batch = MicrosoftGraphBatchRequest::<ArbitraryJson>::new(self.authentication)
                                .cancellation_safe();
                                batch.cache(cache_key);
                                batch.add(MicrosoftGraphBatchRequestEntry::new_get(
                                    "user".to_string(),
                                    format!("https://graph.microsoft.com/v1.0/users/{object_id}"),
                                ));
                                batch.add(MicrosoftGraphBatchRequestEntry::new_get(
                                    "group".to_string(),
                                    format!("https://graph.microsoft.com/v1.0/groups/{object_id}"),
                                ));
                                batch.add(MicrosoftGraphBatchRequestEntry::new_get(
                                    "service-principal".to_string(),
                                    format!("https://graph.microsoft.com/v1.0/servicePrincipals/{object_id}"),
                                ));

                                self.decode_batch(batch.send::<ArbitraryJson>(&executor).await?)
                            }
                        }?;
                        Ok(WorkOutput::new(principal, ArtifactObservation::now()))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraPrincipalRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraPrincipalRequest => Arc<EntraPrincipal>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MicrosoftGraphBatchResponseEntry;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_azure_types::EntraUserId;
    use cloud_terrastodon_azure_types::uuid::Uuid;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_execution::TestAuthenticationProvider;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    fn request() -> EntraPrincipalRequest {
        EntraPrincipalRequest {
            lookup: EntraPrincipalLookup::Unknown(Uuid::nil()),
            authentication: test_authentication(),
        }
    }

    fn user_batch(response_id: &str) -> eyre::Result<MicrosoftGraphBatchResponse<ArbitraryJson>> {
        let user = EntraUser {
            business_phones: Vec::new(),
            display_name: "Synthetic principal".to_owned(),
            external_user_state: None,
            external_user_state_change_date_time: None,
            given_name: None,
            id: EntraUserId::new(Uuid::nil()),
            job_title: None,
            mail: None,
            other_mails: Vec::new(),
            mobile_phone: None,
            office_location: None,
            preferred_language: None,
            surname: None,
            user_principal_name: "synthetic@example.invalid".parse()?,
        };
        Ok(MicrosoftGraphBatchResponse {
            responses: vec![Arc::new(MicrosoftGraphBatchResponseEntry {
                id: response_id.to_owned(),
                status: 200,
                headers: HashMap::new(),
                body: MicrosoftGraphBatchResponseEntryBody::Success(ArbitraryJson::try_from_facet(
                    &user,
                )?),
            })],
        })
    }

    async fn initialize_principal(
        executor: &CloudTerrastodonExecutor,
        request: EntraPrincipalRequest,
        batch: MicrosoftGraphBatchResponse<ArbitraryJson>,
        decodes: &AtomicUsize,
    ) -> eyre::Result<Arc<EntraPrincipal>> {
        let selection = request.authentication.into();
        executor
            .cached_work(
                ExecutorKind::InProcess,
                request.cache_key()?,
                CacheOperationOwner::of::<EntraPrincipalRequest>(),
                &selection,
                NonEmptyString::new(facet_json::to_string(&request)?)?,
                WorkReuse::ReusableRead,
                move |_child| async move {
                    decodes.fetch_add(1, Ordering::SeqCst);
                    let observation = ArtifactObservation::now();
                    Ok(WorkOutput::new(request.decode_batch(batch)?, observation))
                },
            )?
            .await
    }

    #[tokio::test]
    async fn unknown_principal_decode_failures_retry_and_hits_share_the_typed_result()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let authentication = Arc::new(TestAuthenticationProvider::offline(
                    "synthetic-principal-test",
                )?);
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    authentication.clone(),
                    scope,
                );
                let decodes = AtomicUsize::new(0);
                drop(
                    initialize_principal(&executor, request(), user_batch("group")?, &decodes)
                        .await
                        .expect_err("the group response ID must not decode a user-shaped body"),
                );

                let first =
                    initialize_principal(&executor, request(), user_batch("user")?, &decodes)
                        .await?;
                let second = request().execute(&executor).await?;
                let third =
                    initialize_principal(&executor, request(), user_batch("unexpected")?, &decodes)
                        .await?;
                assert_eq!(decodes.load(Ordering::SeqCst), 2);
                assert!(Arc::ptr_eq(&first, &second));
                assert!(Arc::ptr_eq(&first, &third));
                let known_kind = EntraPrincipalRequest {
                    lookup: EntraUserId::new(Uuid::nil()).into(),
                    ..request()
                };
                let replay = facet_json::to_string(&known_kind)?;
                assert!(
                    initialize_principal(
                        &executor,
                        known_kind,
                        user_batch("unexpected")?,
                        &decodes
                    )
                    .await
                    .is_err()
                );
                assert_eq!(
                    decodes.load(Ordering::SeqCst),
                    3,
                    "the known-kind route must not reuse an unknown-ID hit"
                );
                let known = initialize_principal(
                    &executor,
                    facet_json::from_str(&replay)?,
                    user_batch("user")?,
                    &decodes,
                )
                .await?;
                let replayed = facet_json::from_str::<EntraPrincipalRequest>(&replay)?
                    .execute(&executor)
                    .await?;
                assert!(Arc::ptr_eq(&known, &replayed));
                assert!(!Arc::ptr_eq(&first, &replayed));
                assert_eq!(decodes.load(Ordering::SeqCst), 4);
                assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 0);
                assert_eq!(
                    second
                        .as_user()
                        .expect("the user batch must select a user")
                        .id,
                    EntraUserId::new(Uuid::nil())
                );
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraPrincipalRequest,
    name = "principal",
    cached
);
