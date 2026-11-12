use crate::AzureBearerToken;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;

/// Minimal Graph identity proof used only while explicitly provisioning a
/// credential. The Graph service validates the access token; decoding JWT claims
/// locally is not identity verification.
/// <https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0>
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct EntraAuthenticatedUserIdentity {
    #[facet(rename = "@odata.context")]
    pub(crate) context: String,
    pub(crate) id: EntraUserId,
}

impl EntraAuthenticatedUserIdentity {
    pub(crate) async fn verify(
        token: &AzureBearerToken,
        cancellation: &CancellationScope,
    ) -> Result<EntraUserId> {
        cancellation.cancellable_scope(async |_| {
            let authorization = Authorization::bearer(token.as_str())?;
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let response = client.get("https://graph.microsoft.com/v1.0/me?$select=id")
                .header(reqwest::header::AUTHORIZATION, authorization.header().ok_or_else(|| eyre::eyre!("identity verification requires bearer authorization"))?.clone())
                .send().await.wrap_err("verifying the authenticated account identity")?;
            ensure!(response.status().is_success(), "Graph could not verify the authenticated account identity ({})", response.status());
            let bytes = response.bytes().await.wrap_err("reading the authenticated account identity")?;
            let identity: Self = facet_json::from_slice(&bytes)
                .map_err(|_| eyre::eyre!("Graph returned an invalid authenticated account identity; response suppressed"))?;
            Ok(identity.id)
        }).await
    }
}
