use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::path::PathBuf;
use url::Url;

/// Fetch the delegated signed-in user in the explicitly selected tenant.
///
/// [Microsoft Graph v1.0 /me](https://learn.microsoft.com/graph/api/user-get?view=graph-rest-1.0)
/// requires delegated credentials; application-only authentication has no current user.
pub async fn fetch_current_user(
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> Result<EntraUser> {
    let mut url = Url::parse("https://graph.microsoft.com/v1.0/me")?;
    url.query_pairs_mut()
        .append_pair("$select", EntraUser::SELECT);
    RestRequest::from_method_and_url(Method::GET, url, cancellation)?
        .azure_auth_context(auth_context)
        .cache(CacheKey::new(PathBuf::from_iter([
            "rest",
            "graph",
            "v1.0",
            "tenants",
            auth_context.tenant_id.to_string().as_str(),
            "me",
        ])))
        .receive()
        .await
}
/// Fetch the signed-in user from Graph using an explicitly supplied delegated token.
pub async fn fetch_current_user_with_graph_access_token(
    tenant_id: AzureTenantId,
    access_token: &str,
    cancellation: &CancellationScope,
) -> Result<EntraUser> {
    let url = format!(
        "https://graph.microsoft.com/v1.0/me?$select={}",
        EntraUser::SELECT
    );
    RestRequest::new(Method::GET, url, cancellation)?
        .tenant(tenant_id)
        .bearer_token(access_token)
        .receive()
        .await
}
