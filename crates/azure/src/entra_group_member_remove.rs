use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Remove a group membership using Graph's [$ref endpoint](https://learn.microsoft.com/graph/api/group-delete-members?view=graph-rest-1.0).
/// The directory object itself is never the DELETE target.
#[derive(Debug, arbitrary::Arbitrary, facet::Facet)]
pub struct EntraGroupMemberRemoveRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub auth_context: AzureTenantAuthContext,

    pub group_id: EntraGroupId,
    pub member_id: EntraPrincipalId,
}

pub fn remove_group_member(
    group_id: impl Into<EntraGroupId>,
    member_id: impl Into<EntraPrincipalId>,
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> EntraGroupMemberRemoveRequest {
    EntraGroupMemberRemoveRequest {
        cancellation: cancellation.clone(),
        auth_context: auth_context.clone(),
        group_id: group_id.into(),
        member_id: member_id.into(),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupMemberRemoveRequest {
    type Output = ();

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "az",
                "rest",
                "tenants",
                self.auth_context.tenant_id.to_string().as_str(),
                "ad",
                "group",
                "member",
                "remove",
                self.group_id.to_string().as_str(),
                self.member_id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        // Omitting /$ref can delete the directory object itself; retain it literally.
        let url = Url::parse(&format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members/{}/$ref",
            self.group_id, self.member_id
        ))?;
        let cancellation = self.cancellation.clone();
        cancellation
            .deferred(
                "removing a group member and invalidating its membership inventory",
                |scope| async move {
                    let execution = RestRequest::from_method_and_url(Method::DELETE, url, &scope)?
                        .azure_auth_context(&self.auth_context)
                        .cache(self.cache_key())
                        .receive_raw_with_decoder(|response| {
                            response.error_for_status()?;
                            eyre::ensure!(
                                response.status == http::StatusCode::NO_CONTENT,
                                "removing a group membership requires a 204 response"
                            );
                            Ok(())
                        })
                        .await;
                    // Even a failed response can follow a successful mutation.
                    let invalidation =
                        crate::fetch_group_members(self.group_id, &self.auth_context, &scope)
                            .invalidate()
                            .await;
                    match (execution, invalidation) {
                        (Ok(()), Ok(())) => Ok(()),
                        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                        (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                            "membership inventory invalidation also failed: {invalidation:#}"
                        ))),
                    }
                },
            )
            .await?;
        Ok(())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupMemberRemoveRequest);

cloud_terrastodon_registry::register_thing!(EntraGroupMemberRemoveRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMemberRemoveRequest);
cloud_terrastodon_registry_execution::register_into_future!(EntraGroupMemberRemoveRequest => (), effects = [Write]);
