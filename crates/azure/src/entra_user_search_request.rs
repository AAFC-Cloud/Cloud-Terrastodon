use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

const USER_SELECT: &str = "businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName";
const USER_SEARCH_CACHE_DURATION: Duration = Duration::from_secs(60);

#[must_use = "This is a future request, you must .await it"]
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

pub fn search_entra_users<'a>(
    search_term: impl Into<String>,
    auth_context: &'a AzureTenantAuthContext,
) -> EntraUserSearchRequest<'a> {
    EntraUserSearchRequest {
        search_term: search_term.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl EntraUserSearchRequest<'_> {
    fn url(&self) -> String {
        let search_term = escape_odata_string(self.search_term.trim());
        let filter = format!(
            "startswith(displayName,'{search_term}') or startswith(givenName,'{search_term}') or startswith(surname,'{search_term}') or startswith(mail,'{search_term}') or startswith(userPrincipalName,'{search_term}')"
        );

        format!(
            "https://graph.microsoft.com/v1.0/users?$select={USER_SELECT}&$filter={}",
            filter.percent_encode()
        )
    }
}

#[async_trait]
impl CacheableCommand for EntraUserSearchRequest<'_> {
    type Output = Vec<EntraUser>;

    fn cache_key(&self) -> CacheKey {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "users",
                "search",
                self.auth_context.tenant_id.to_string().as_str(),
                search_hash.as_str(),
            ]),
            valid_for: USER_SEARCH_CACHE_DURATION,
        }
    }

    async fn run(self) -> Result<Self::Output> {
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
        let users: Vec<EntraUser> = MicrosoftGraphHelper::new(
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_all()
        .await?;
        debug!(tenant_id = %self.auth_context.tenant_id, count = users.len(), "Found Entra users");
        Ok(users)
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserSearchRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(EntraUserSearchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserSearchRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraUserSearchRequest<'static> => Vec<EntraUser>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_current_user;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_credentials::AuthContext;

    fn test_auth_context() -> AzureTenantAuthContext {
        AuthContext::explicit_azure_cli()
            .bind_to_azure_tenant(AzureTenantId::new(
                cloud_terrastodon_azure_types::uuid::Uuid::nil(),
            ))
            .expect("default auth context should bind to a tenant")
    }

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        let auth_context = test_auth_context();
        let request = search_entra_users("O'Neil & Smith", &auth_context);

        assert!(request.url().contains("O%27%27Neil%20%26%20Smith"));
    }

    #[test]
    fn paginated_search_cache_expires() {
        let auth_context = test_auth_context();
        let request = search_entra_users("Smith", &auth_context);

        assert_eq!(request.cache_key().valid_for, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn it_finds_the_current_user_by_user_principal_name() -> Result<()> {
        let current_user = fetch_current_user().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let users = search_entra_users(&current_user.user_principal_name, &auth_context).await?;

        assert!(users.iter().any(|user| {
            user.id == current_user.id
                && user
                    .user_principal_name
                    .eq_ignore_ascii_case(&current_user.user_principal_name)
        }));
        Ok(())
    }
}
