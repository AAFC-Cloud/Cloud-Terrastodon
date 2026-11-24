use crate::entra_signed_in_user_thumbnail_get_request::receive_entra_thumbnail;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraProfileThumbnail;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use facet::Facet;
use url::Url;

/// Read a 48-by-48 [Microsoft Graph v1.0 user profile photo](https://learn.microsoft.com/graph/api/profilephoto-get?view=graph-rest-1.0).
/// Graph requires permission to read the selected user's photo and returns 404 when it is absent.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserThumbnailGetRequest {
    pub user_id: EntraUserId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl ExecutableRequest for EntraUserThumbnailGetRequest {
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
            let url = Url::parse(&format!(
                "https://graph.microsoft.com/v1.0/users/{}/photos/48x48/$value",
                self.user_id
            ))?;
            receive_entra_thumbnail(&executor, self.authentication, url).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserThumbnailGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserThumbnailGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUserThumbnailGetRequest => Option<EntraProfileThumbnail>, effects = [Read]);
