use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Remove a group membership using Graph's [$ref endpoint](https://learn.microsoft.com/graph/api/group-delete-members?view=graph-rest-1.0).
/// The directory object itself is never the DELETE target.
#[derive(Debug, Arbitrary, Facet)]
pub struct EntraGroupMemberRemoveRequest {
    pub auth_context: AzureTenantAuthContext,

    pub group_id: EntraGroupId,
    pub member_id: EntraPrincipalId,
}

#[track_caller]
pub fn remove_group_member<'exec>(
    group_id: impl Into<EntraGroupId>,
    member_id: impl Into<EntraPrincipalId>,
    auth_context: &AzureTenantAuthContext,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, EntraGroupMemberRemoveRequest> {
    executor.execute(EntraGroupMemberRemoveRequest {
        auth_context: auth_context.clone(),
        group_id: group_id.into(),
        member_id: member_id.into(),
    })
}

impl InvalidatesOwnCacheKey for EntraGroupMemberRemoveRequest {}

impl HasCacheKey for EntraGroupMemberRemoveRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
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
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for EntraGroupMemberRemoveRequest {
    type Error = eyre::Report;
    type Output = ();
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Omitting /$ref can delete the directory object itself; retain it literally.
            let url = Url::parse(&format!(
                "https://graph.microsoft.com/v1.0/groups/{}/members/{}/$ref",
                self.group_id, self.member_id
            ))?;
            executor
                .deferred(
                    "removing a group member and invalidating its membership inventory",
                    |executor| async move {
                        let execution = RestRequest::from_method_and_url(Method::DELETE, url)?
                            .azure_auth_context(&self.auth_context)?
                            .cache(self.cache_key()?)
                            .receive_raw_with_decoder(
                                |response| {
                                    response.error_for_status()?;
                                    eyre::ensure!(
                                        response.status == http::StatusCode::NO_CONTENT,
                                        "removing a group membership requires a 204 response"
                                    );
                                    Ok(())
                                },
                                &executor,
                            )
                            .await;
                        // Even a failed response can follow a successful mutation.
                        let invalidation = crate::fetch_group_members(
                            self.group_id,
                            &self.auth_context,
                            &executor,
                        )
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
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupMemberRemoveRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMemberRemoveRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupMemberRemoveRequest => (), effects = [Write]);
