use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraDirectoryObjectUrl;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraGroupMemberReference;
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

/// Add a direct group membership through [Microsoft Graph v1.0](https://learn.microsoft.com/graph/api/group-post-members?view=graph-rest-1.0).
#[derive(Arbitrary, Facet)]
pub struct EntraGroupMemberAddRequest {
    pub auth_context: AzureTenantAuthContext,

    pub group_id: EntraGroupId,
    pub member_id: EntraPrincipalId,
}

#[track_caller]
pub fn add_group_member<'exec>(
    group_id: impl Into<EntraGroupId>,
    member_id: impl Into<EntraPrincipalId>,
    auth_context: &AzureTenantAuthContext,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, EntraGroupMemberAddRequest> {
    executor.execute(EntraGroupMemberAddRequest {
        auth_context: auth_context.clone(),
        group_id: group_id.into(),
        member_id: member_id.into(),
    })
}

impl InvalidatesOwnCacheKey for EntraGroupMemberAddRequest {}

impl HasCacheKey for EntraGroupMemberAddRequest {
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
                "add",
                self.group_id.to_string().as_str(),
                self.member_id.to_string().as_str(),
            ]))?,
            valid_for: Duration::ZERO,
        })
    }
}

impl ExecutableRequest for EntraGroupMemberAddRequest {
    type Error = eyre::Report;
    type Output = ();
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = Url::parse(&format!(
                "https://graph.microsoft.com/v1.0/groups/{}/members/$ref",
                self.group_id
            ))?;
            let body = EntraGroupMemberReference {
                member: EntraDirectoryObjectUrl { id: self.member_id },
            };
            executor
                .deferred(
                    "adding a group member and invalidating its membership inventory",
                    |executor| async move {
                        let execution = RestRequest::from_method_and_url(Method::POST, url)?
                            .azure_auth_context(&self.auth_context)?
                            .body(facet_json::to_string(&body)?)
                            .cache(self.cache_key()?)
                            .receive_raw_with_decoder(
                                |response| {
                                    response.error_for_status()?;
                                    eyre::ensure!(
                                        response.status == http::StatusCode::NO_CONTENT,
                                        "adding a group member requires a 204 response"
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

cloud_terrastodon_registry::register_thing!(EntraGroupMemberAddRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMemberAddRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupMemberAddRequest => (), effects = [Write]);
