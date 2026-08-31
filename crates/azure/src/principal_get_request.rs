use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchResponseEntryBody;
use crate::fetch_entra_user;
use crate::fetch_group;
use crate::fetch_service_principal;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct PrincipalRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub principal_id: PrincipalId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for PrincipalRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_principal<'a>(
    tenant_id: AzureTenantId,
    principal_id: PrincipalId,
    auth_context: &'a AuthContext,
) -> PrincipalRequest<'a> {
    PrincipalRequest {
        tenant_id,
        principal_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for PrincipalRequest<'_> {
    type Output = Principal;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "principal",
            self.tenant_id.to_string().as_str(),
            self.principal_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let cache_key = self.cache_key();
        let auth_context = self.auth_context;
        match self.principal_id {
            PrincipalId::UserId(user_id) => {
                Ok(
                    fetch_entra_user(self.tenant_id, user_id, auth_context.as_ref())
                        .await?
                        .into(),
                )
            }
            PrincipalId::GroupId(group_id) => {
                Ok(fetch_group(self.tenant_id, group_id, auth_context.as_ref())
                    .await?
                    .into())
            }
            PrincipalId::ServicePrincipalId(service_principal_id) => Ok(fetch_service_principal(
                self.tenant_id,
                service_principal_id,
                auth_context.as_ref(),
            )
            .await?
            .into()),
            PrincipalId::Unknown(object_id) => {
                let mut batch = MicrosoftGraphBatchRequest::<RawJson<'static>>::new(
                    self.tenant_id,
                    auth_context.as_ref(),
                );
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
                for response in batch.send::<RawJson<'static>>().await?.responses {
                    match response.body {
                        MicrosoftGraphBatchResponseEntryBody::Success(body) => {
                            return match response.id.as_str() {
                                "user" => Ok(Principal::from(facet_json::from_str::<EntraUser>(
                                    body.as_str(),
                                )?)),
                                "group" => Ok(Principal::from(facet_json::from_str::<EntraGroup>(
                                    body.as_str(),
                                )?)),
                                "service-principal" => Ok(Principal::from(facet_json::from_str::<
                                    EntraServicePrincipal,
                                >(
                                    body.as_str()
                                )?)),
                                other => eyre::bail!(
                                    "Unexpected Microsoft Graph principal request id '{other}'"
                                ),
                            };
                        }
                        MicrosoftGraphBatchResponseEntryBody::Error(error) => {
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
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PrincipalRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(PrincipalRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrincipalRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PrincipalRequest<'static> => Principal);
