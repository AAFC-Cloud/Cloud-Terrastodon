use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

const USER_SEARCH_CACHE_DURATION: Duration = Duration::from_secs(60);

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct EntraUserSearchRequest<'a> {
    pub search_term: String,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> Arbitrary<'a> for EntraUserSearchRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            search_term: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn search_entra_users<'a>(
    search_term: impl Into<String>,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, EntraUserSearchRequest<'a>> {
    executor.execute(EntraUserSearchRequest {
        search_term: search_term.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl EntraUserSearchRequest<'_> {
    /// The invalidation root shared by all search terms in this tenant.
    pub(crate) fn tenant_cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "ms",
                "graph",
                "GET",
                "users",
                "search",
                &self.auth_context.tenant_id.to_string(),
            ])?,
            valid_for: USER_SEARCH_CACHE_DURATION,
        })
    }

    fn url(&self) -> String {
        let search_term = escape_odata_string(self.search_term.trim());
        let filter = format!(
            "startswith(displayName,'{search_term}') or startswith(givenName,'{search_term}') or startswith(surname,'{search_term}') or startswith(mail,'{search_term}') or startswith(userPrincipalName,'{search_term}')"
        );

        format!(
            "https://graph.microsoft.com/v1.0/users?$select={}&$filter={}",
            EntraUser::SELECT,
            filter.percent_encode()
        )
    }
}

impl InvalidatesOwnCacheKey for EntraUserSearchRequest<'_> {}

impl HasCacheKey for EntraUserSearchRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        self.tenant_cache_key()?.join(search_hash)
    }
}

impl ExecutableRequest for EntraUserSearchRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraUser>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let search_term = self.search_term.trim();
            if search_term.is_empty() {
                debug!(tenant_id = %self.auth_context.tenant_id, "Skipping empty Entra user search");
                return Ok(Vec::new());
            }

            debug!(
                tenant_id = %self.auth_context.tenant_id,
                search_term,
                "Searching Entra users"
            );
            let users: Vec<Arc<EntraUser>> = MicrosoftGraphHelper::new(
                self.url(),
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .fetch_all(&executor)
            .await?;
            debug!(tenant_id = %self.auth_context.tenant_id, count = users.len(), "Found Entra users");
            Ok(users)
        })
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_registry::register_thing!(EntraUserSearchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserSearchRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUserSearchRequest<'static> => Vec<Arc<EntraUser>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_current_user;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_command::CancellationToken;

    fn test_auth_context() -> AzureTenantAuthContext {
        AuthContext::explicit_azure_cli()
            .bind_to_azure_tenant(AzureTenantId::new(
                cloud_terrastodon_azure_types::uuid::Uuid::nil(),
            ))
            .expect("default auth context should bind to a tenant")
    }

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let auth_context = test_auth_context();
                let request = EntraUserSearchRequest {
                    search_term: "O'Neil & Smith".to_owned(),
                    auth_context: Cow::Borrowed(&auth_context),
                };

                assert!(request.url().contains("O%27%27Neil%20%26%20Smith"));
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn paginated_search_cache_expires() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let auth_context = test_auth_context();
                let request = EntraUserSearchRequest {
                    search_term: "Smith".to_owned(),
                    auth_context: Cow::Borrowed(&auth_context),
                };

                assert_eq!(request.cache_key()?.valid_for, Duration::from_secs(60));
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[tokio::test]
    async fn it_finds_the_current_user_by_user_principal_name() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let current_user = fetch_current_user(&auth_context, &executor).await?;
                let users = search_entra_users(
                    current_user.user_principal_name.as_str(),
                    &auth_context,
                    &executor,
                )
                .await?;

                assert!(users.iter().any(|user| {
                    user.id == current_user.id
                        && user
                            .user_principal_name
                            .eq_ignore_ascii_case(current_user.user_principal_name.as_str())
                }));
                Ok(())
            })
            .await
    }
}
