use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tracing::debug;

const USER_SELECT: &str = "businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName";

#[must_use = "This is a future request, you must .await it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserSearchRequest {
    pub tenant_id: AzureTenantId,
    pub search_term: String,
}

pub fn search_entra_users(
    tenant_id: AzureTenantId,
    search_term: impl Into<String>,
) -> EntraUserSearchRequest {
    EntraUserSearchRequest {
        tenant_id,
        search_term: search_term.into(),
    }
}

impl EntraUserSearchRequest {
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
impl CacheableCommand for EntraUserSearchRequest {
    type Output = Vec<EntraUser>;

    fn cache_key(&self) -> CacheKey {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "users",
            "search",
            self.tenant_id.to_string().as_str(),
            search_hash.as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let search_term = self.search_term.trim();
        if search_term.is_empty() {
            debug!(tenant_id = %self.tenant_id, "Skipping empty Entra user search");
            return Ok(Vec::new());
        }

        debug!(
            tenant_id = %self.tenant_id,
            search_term,
            "Searching Entra users"
        );
        let users: Vec<EntraUser> =
            MicrosoftGraphHelper::new(self.tenant_id, self.url(), Some(self.cache_key()))
                .fetch_all()
                .await?;
        debug!(tenant_id = %self.tenant_id, count = users.len(), "Found Entra users");
        Ok(users)
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserSearchRequest);
cloud_terrastodon_registry::register_thing!(EntraUserSearchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserSearchRequest);
cloud_terrastodon_registry::register_into_future!(EntraUserSearchRequest => Vec<EntraUser>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_current_user;
    use crate::get_test_tenant_id;

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        let request = search_entra_users(
            AzureTenantId::new(cloud_terrastodon_azure_types::uuid::Uuid::nil()),
            "O'Neil & Smith",
        );

        assert!(request.url().contains("O%27%27Neil%20%26%20Smith"));
    }

    #[tokio::test]
    async fn it_finds_the_current_user_by_user_principal_name() -> Result<()> {
        let current_user = fetch_current_user().await?;
        let users = search_entra_users(
            get_test_tenant_id().await?,
            &current_user.user_principal_name,
        )
        .await?;

        assert!(users.iter().any(|user| {
            user.id == current_user.id
                && user
                    .user_principal_name
                    .eq_ignore_ascii_case(&current_user.user_principal_name)
        }));
        Ok(())
    }
}
