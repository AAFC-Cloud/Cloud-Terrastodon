use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use url::Url;

/// Search Graph v1.0 [applications](https://learn.microsoft.com/graph/api/application-list?view=graph-rest-1.0)
/// by object or client ID, or by display-name tokens.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraApplicationRegistrationSearchRequest {
    pub search_term: String,
    pub authentication: AzureBearerAuthenticationSelection,
    /// Require the full display name for a non-ID query.
    #[facet(default)]
    pub exact_display_name: bool,
}

impl EntraApplicationRegistrationSearchRequest {
    fn graph_query(&self) -> eyre::Result<MicrosoftGraphHelper> {
        let search_term = self.search_term.trim();
        let cache_key = Some(self.cache_key()?);
        let mut url = Url::parse("https://graph.microsoft.com/v1.0/applications")?;
        let filter = if let Ok(id) = Uuid::parse_str(search_term) {
            format!("appId eq '{id}' or id eq '{id}'")
        } else if self.exact_display_name {
            format!("displayName eq '{}'", escape_odata_string(search_term))
        } else {
            url.query_pairs_mut().append_pair("$count", "true");
            return MicrosoftGraphHelper::for_display_name_search(
                url,
                search_term,
                cache_key,
                self.authentication,
            );
        };
        url.query_pairs_mut().append_pair("$filter", &filter);
        Ok(MicrosoftGraphHelper::new(
            url,
            cache_key,
            self.authentication,
        ))
    }
}

impl InvalidatesOwnCacheKey for EntraApplicationRegistrationSearchRequest {}

impl HasCacheKey for EntraApplicationRegistrationSearchRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        let mut key = CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            "search",
            self.authentication.tenant_id.to_string().as_str(),
        ]))?;
        if self.exact_display_name || Uuid::parse_str(self.search_term.trim()).is_err() {
            key.valid_for = Duration::from_secs(60);
        }
        if self.exact_display_name {
            key.join("exact-display-name")?.join(search_hash)
        } else {
            key.join(search_hash)
        }
    }
}

impl ExecutableRequest for EntraApplicationRegistrationSearchRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraApplicationRegistration>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let search_term = self.search_term.trim();
            if search_term.is_empty() {
                debug!(tenant_id = %self.authentication.tenant_id, "Skipping empty application registration search");
                return Ok(Vec::new());
            }

            debug!(
                tenant_id = %self.authentication.tenant_id,
                search_term,
                "Searching application registrations through Microsoft Graph"
            );
            let applications: Vec<Arc<EntraApplicationRegistration>> =
                self.graph_query()?.fetch_all(&executor).await?;
            debug!(
                tenant_id = %self.authentication.tenant_id,
                count = applications.len(),
                "Found application registrations"
            );
            Ok(applications)
        })
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_registry::register_thing!(EntraApplicationRegistrationSearchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraApplicationRegistrationSearchRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraApplicationRegistrationSearchRequest => Vec<Arc<EntraApplicationRegistration>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_authentication::test_authentication;

    #[test]
    fn exact_display_name_escapes_odata_string_literals() -> eyre::Result<()> {
        let request = EntraApplicationRegistrationSearchRequest {
            exact_display_name: true,
            search_term: "Cloud Terrastodon's PIM".to_owned(),
            authentication: test_authentication(),
        };
        let graph = request.graph_query()?;
        let url = Url::parse(&graph.url)?;
        assert!(url.query_pairs().any(|(name, value)| {
            name == "$filter" && value == "displayName eq 'Cloud Terrastodon''s PIM'"
        }));
        assert!(graph.headers.is_none());
        Ok(())
    }

    #[test]
    fn tokenized_search_keeps_special_characters_in_one_clause() -> eyre::Result<()> {
        let request = EntraApplicationRegistrationSearchRequest {
            exact_display_name: false,
            search_term: r#"Team "Web"\Ops & Support"#.to_owned(),
            authentication: test_authentication(),
        };
        let graph = request.graph_query()?;
        let url = Url::parse(&graph.url)?;
        assert_eq!(url.query_pairs().count(), 2);
        assert!(url.query_pairs().any(|(name, value)| {
            name == "$search" && value == r#""displayName:Team \"Web\"\\Ops & Support""#
        }));
        assert!(
            url.query_pairs()
                .any(|(name, value)| name == "$count" && value == "true")
        );
        let headers = graph.headers.unwrap().to_header_map()?;
        assert_eq!(headers["ConsistencyLevel"], "eventual");
        assert_eq!(headers["Prefer"], "legacySearch=false");
        Ok(())
    }

    #[test]
    fn uuid_search_matches_application_object_and_client_ids() -> eyre::Result<()> {
        let id = "00000003-0000-0000-c000-000000000000";
        let request = EntraApplicationRegistrationSearchRequest {
            exact_display_name: true,
            search_term: id.to_owned(),
            authentication: test_authentication(),
        };
        let graph = request.graph_query()?;
        let url = Url::parse(&graph.url)?;
        assert_eq!(url.query_pairs().count(), 1);
        assert!(url.query_pairs().any(|(name, value)| {
            name == "$filter" && value == format!("appId eq '{id}' or id eq '{id}'")
        }));
        assert!(graph.headers.is_none());
        Ok(())
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraApplicationRegistrationSearchRequest,
    name = "application-registration-search",
    cached
);
