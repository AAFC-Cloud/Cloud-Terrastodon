use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_azure_types::EntraUserArgument;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;
use url::Url;

/// Fetch a Microsoft Graph v1.0 [user](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0)
/// by object ID or validated user principal name.
///
/// Guest UPNs are percent-encoded. Leading `$` names use Graph's documented
/// parenthesized key syntax, with embedded apostrophes escaped as OData string
/// literals. An absent user returns a typed [`cloud_terrastodon_rest::RestHttpError`]
/// with status 404; authentication, transport, and decoding failures remain errors.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct EntraUserGetRequest<'a> {
    pub lookup: EntraUserArgument<'a>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> Arbitrary<'a> for EntraUserGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            lookup: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

/// Construct a user lookup from an `EntraUserId`, `UserPrincipalName`, or
/// [`EntraUserArgument`]. Parse raw text as `EntraUserArgument` before calling.
#[track_caller]
pub fn fetch_entra_user<'a, T>(
    lookup: T,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraUserGetRequest<'a>>
where
    T: Into<EntraUserArgument<'a>>,
{
    executor.execute(EntraUserGetRequest {
        lookup: lookup.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl EntraUserGetRequest<'_> {
    fn url(&self) -> Result<Url> {
        let path = match &self.lookup {
            EntraUserArgument::ObjectId(user_id) => format!("users/{user_id}"),
            EntraUserArgument::UserPrincipalName(user_principal_name)
                if user_principal_name.as_str().starts_with('$') =>
            {
                // https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0#http-request
                // Quoted keys follow OData literal escaping before URL encoding:
                // https://learn.microsoft.com/en-us/graph/query-parameters#escaping-single-quotes
                format!(
                    "users('{}')",
                    user_principal_name
                        .as_str()
                        .replace('\'', "''")
                        .percent_encode()
                )
            }
            EntraUserArgument::UserPrincipalName(user_principal_name) => {
                format!("users/{}", user_principal_name.as_str().percent_encode())
            }
        };
        let mut url = Url::parse(&format!("https://graph.microsoft.com/v1.0/{path}"))?;
        url.query_pairs_mut()
            .append_pair("$select", EntraUser::SELECT);
        Ok(url)
    }
}

impl InvalidatesOwnCacheKey for EntraUserGetRequest<'_> {}

impl HasCacheKey for EntraUserGetRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let tenant_id = self.auth_context.tenant_id.to_string();

        match &self.lookup {
            EntraUserArgument::ObjectId(user_id) => {
                let user_id = user_id.to_string();
                CacheKey::from_relative_path(PathBuf::from_iter([
                    "ms",
                    "graph",
                    "GET",
                    "users",
                    tenant_id.as_str(),
                    user_id.as_str(),
                ]))
            }
            EntraUserArgument::UserPrincipalName(user_principal_name) => {
                let user_principal_name_component =
                    sanitize_windows_path_component(user_principal_name.as_str());
                // The spelling fingerprint keeps distinct requests separate even
                // on a case-insensitive filesystem; the identity remains readable.
                let user_principal_name_hash =
                    blake3::hash(user_principal_name.as_str().as_bytes())
                        .to_hex()
                        .to_string();
                CacheKey::from_relative_path(PathBuf::from_iter([
                    "ms",
                    "graph",
                    "GET",
                    "users",
                    "by_user_principal_name",
                    tenant_id.as_str(),
                    user_principal_name_component.as_str(),
                    user_principal_name_hash.as_str(),
                ]))
            }
        }
    }
}

impl ExecutableRequest for EntraUserGetRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<EntraUser>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            match &self.lookup {
                EntraUserArgument::ObjectId(user_id) => {
                    debug!(
                        tenant_id = %self.auth_context.tenant_id,
                        user_id = %user_id,
                        "Fetching user by object id"
                    );
                }
                EntraUserArgument::UserPrincipalName(user_principal_name) => {
                    debug!(
                        tenant_id = %self.auth_context.tenant_id,
                        user_principal_name = %user_principal_name,
                        "Fetching Entra user by user principal name"
                    );
                }
            }

            let mut request = RestRequest::from_method_and_url(Method::GET, self.url()?)?
                .azure_auth_context(&self.auth_context)?;
            request.cache_key = Some(self.cache_key()?);
            request
                .receive_cached::<EntraUser>(
                    &executor,
                    CacheOperationOwner::of::<EntraUserGetRequest<'static>>(),
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUserGetRequest<'static> => Arc<EntraUser>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn url_keeps_special_upns_in_the_user_key() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let auth = AuthContext::explicit_azure_cli().bind_to_azure_tenant(
                    AzureTenantId::new(cloud_terrastodon_azure_types::uuid::Uuid::nil()),
                )?;
                for (upn, expected_path) in [
                    ("O'Neil@example.com", "/v1.0/users/O%27Neil%40example.com"),
                    (
                        "$O'Neil#EXT#@example.com",
                        "/v1.0/users('%24O%27%27Neil%23EXT%23%40example.com')",
                    ),
                    (
                        "guest_example.com#EXT#@tenant.example",
                        "/v1.0/users/guest_example.com%23EXT%23%40tenant.example",
                    ),
                    (
                        "person+tag/alias?x&y%z@example.com",
                        "/v1.0/users/person%2Btag%2Falias%3Fx%26y%25z%40example.com",
                    ),
                ] {
                    let request = EntraUserGetRequest {
                        lookup: upn.parse()?,
                        auth_context: Cow::Borrowed(&auth),
                    };
                    let url = request.url()?;
                    assert_eq!(url.path(), expected_path);
                    assert!(url.fragment().is_none());
                    assert_eq!(
                        url.query_pairs().collect::<Vec<_>>(),
                        vec![(Cow::Borrowed("$select"), Cow::Borrowed(EntraUser::SELECT))]
                    );
                }
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }
}
