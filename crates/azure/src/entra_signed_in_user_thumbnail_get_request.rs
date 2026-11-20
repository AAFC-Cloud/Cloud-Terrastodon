use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraProfileThumbnail;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestHttpError;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use http::StatusCode;
use std::time::Duration;
use url::Url;

/// Read the [signed-in user's Graph profile thumbnail](https://learn.microsoft.com/en-us/graph/api/profilephoto-get?view=graph-rest-1.0#example-2-get-the-48x48-photo-for-the-signed-in-user).
/// Returns `None` when Graph reports that the user has no photo.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraSignedInUserThumbnailGetRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl ExecutableRequest for EntraSignedInUserThumbnailGetRequest {
    type Error = eyre::Report;
    type Output = Option<EntraProfileThumbnail>;

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
            let url = Url::parse("https://graph.microsoft.com/v1.0/me/photos/48x48/$value")?;
            match RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .transport_timeout(Duration::from_secs(10))
                .receive_limited_bytes(&executor, 256 * 1024)
                .await
            {
                Ok(bytes) => Ok(Some(EntraProfileThumbnail { bytes })),
                Err(error)
                    if error
                        .downcast_ref::<RestHttpError>()
                        .is_some_and(|error| error.status == StatusCode::NOT_FOUND) =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraSignedInUserThumbnailGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraSignedInUserThumbnailGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraSignedInUserThumbnailGetRequest => Option<EntraProfileThumbnail>, effects = [Read]);
