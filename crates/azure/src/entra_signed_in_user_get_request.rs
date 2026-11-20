use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraSignedInUser;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::time::Duration;
use url::Url;

/// Read the [delegated Graph user's identity and address](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0#http-request)
/// with no durable response or diagnostic artifacts.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraSignedInUserGetRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl ExecutableRequest for EntraSignedInUserGetRequest {
    type Error = eyre::Report;
    type Output = EntraSignedInUser;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut url = Url::parse("https://graph.microsoft.com/v1.0/me")?;
            url.query_pairs_mut()
                .append_pair("$select", "id,mail,userPrincipalName");
            let bytes = RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .transport_timeout(Duration::from_secs(10))
                .receive_limited_bytes(&executor, 16 * 1024)
                .await?;
            // Facet decode errors can quote the profile fields.
            facet_json::from_slice(&bytes).map_err(|_| {
                eyre::eyre!("Microsoft Graph returned an invalid signed-in user profile")
            })
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraSignedInUserGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraSignedInUserGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraSignedInUserGetRequest => EntraSignedInUser, effects = [Read]);
