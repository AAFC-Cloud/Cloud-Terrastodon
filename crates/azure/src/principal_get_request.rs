use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchResponseEntryBody;
use crate::fetch_entra_user;
use crate::fetch_group;
use crate::fetch_service_principal;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraPrincipal;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct PrincipalRequest<'a> {
    pub principal_id: EntraPrincipalId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for PrincipalRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_principal<'a>(
    principal_id: EntraPrincipalId,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, PrincipalRequest<'a>> {
    executor.execute(PrincipalRequest {
        principal_id,
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for PrincipalRequest<'_> {}

impl HasCacheKey for PrincipalRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "principal",
            self.auth_context.tenant_id.to_string().as_str(),
            self.principal_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for PrincipalRequest<'_> {
    type Error = eyre::Report;
    type Output = EntraPrincipal;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
            let auth_context = self.auth_context;
            match self.principal_id {
                EntraPrincipalId::UserId(user_id) => {
                    Ok(fetch_entra_user(user_id, auth_context.as_ref(), &executor)
                        .await?
                        .into())
                }
                EntraPrincipalId::GroupId(group_id) => {
                    Ok(fetch_group(group_id, auth_context.as_ref(), &executor)
                        .await?
                        .into())
                }
                EntraPrincipalId::ServicePrincipalId(service_principal_id) => Ok(
                    fetch_service_principal(service_principal_id, auth_context.as_ref(), &executor)
                        .await?
                        .into(),
                ),
                EntraPrincipalId::Unknown(object_id) => {
                    // One batch probes three different entity schemas. Keep its
                    // successful bodies opaque until the response ID selects
                    // the corresponding principal decoder below.
                    let mut batch =
                        MicrosoftGraphBatchRequest::<ArbitraryJson>::new(auth_context.as_ref())
                            .cancellation_safe();
                    batch.cache(cache_key);
                    batch.add(crate::MicrosoftGraphBatchRequestEntry::new_get(
                        "user".to_string(),
                        format!("https://graph.microsoft.com/v1.0/users/{object_id}"),
                    ));
                    batch.add(crate::MicrosoftGraphBatchRequestEntry::new_get(
                        "group".to_string(),
                        format!("https://graph.microsoft.com/v1.0/groups/{object_id}"),
                    ));
                    batch.add(crate::MicrosoftGraphBatchRequestEntry::new_get(
                        "service-principal".to_string(),
                        format!("https://graph.microsoft.com/v1.0/servicePrincipals/{object_id}"),
                    ));

                    let mut errors = Vec::new();
                    for response in batch.send::<ArbitraryJson>(&executor).await?.responses {
                        match &response.body {
                            MicrosoftGraphBatchResponseEntryBody::Success(body) => {
                                return match response.id.as_str() {
                                    "user" => {
                                        Ok(EntraPrincipal::from(facet_json::from_str::<EntraUser>(
                                            body.as_ref(),
                                        )?))
                                    }
                                    "group" => {
                                        Ok(EntraPrincipal::from(
                                            facet_json::from_str::<EntraGroup>(body.as_ref())?,
                                        ))
                                    }
                                    "service-principal" => {
                                        Ok(EntraPrincipal::from(facet_json::from_str::<
                                            EntraServicePrincipal,
                                        >(
                                            body.as_ref()
                                        )?))
                                    }
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
                        object_id,
                        errors.join("; ")
                    );
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(PrincipalRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrincipalRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(PrincipalRequest<'static> => EntraPrincipal, effects = [Read]);
