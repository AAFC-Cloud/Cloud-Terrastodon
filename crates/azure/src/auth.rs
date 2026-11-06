use crate::CurrentEntraUserRequest;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::sync::Arc;

/// Fetch the delegated signed-in user using the executor's selected credential.
/// See [Graph /me](https://learn.microsoft.com/graph/api/user-get?view=graph-rest-1.0).
pub async fn fetch_current_user(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<Arc<EntraUser>> {
    CurrentEntraUserRequest {
        tenant_id: auth_context.tenant_id,
        auth_source: AzureBearerAuthSource::try_from(
            auth_context
                .auth_context
                .source()
                .ok_or_else(|| eyre::eyre!("authentication source is unresolved"))?,
        )?,
    }
    .execute(executor)
    .await
}
