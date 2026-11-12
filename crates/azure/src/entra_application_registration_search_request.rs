use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
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
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraApplicationRegistrationSearchRequest {
    pub search_term: String,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraApplicationRegistrationSearchRequest {
    fn url(&self) -> String {
        let search_term = escape_odata_string(self.search_term.trim());
        let filter = if Uuid::parse_str(self.search_term.trim()).is_ok() {
            format!("appId eq '{search_term}' or id eq '{search_term}'")
        } else {
            format!("startswith(displayName,'{search_term}')")
        };

        format!(
            "https://graph.microsoft.com/v1.0/applications?$filter={}",
            filter.percent_encode()
        )
    }
}

impl InvalidatesOwnCacheKey for EntraApplicationRegistrationSearchRequest {}

impl HasCacheKey for EntraApplicationRegistrationSearchRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            "search",
            self.authentication.tenant_id.to_string().as_str(),
            search_hash.as_str(),
        ]))
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
                MicrosoftGraphHelper::new(self.url(), Some(self.cache_key()?), self.authentication)
                    .fetch_all(&executor)
                    .await?;
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
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let request = EntraApplicationRegistrationSearchRequest {
                    search_term: "Cloud Terrastodon's PIM".to_owned(),
                    authentication,
                };

                assert!(request.url().contains("Cloud%20Terrastodon%27%27s%20PIM"));
                eyre::Ok(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn url_filters_graph_application_properties() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let request = EntraApplicationRegistrationSearchRequest {
                    search_term: "pim".to_owned(),
                    authentication,
                };
                let url = request.url();

                assert!(url.contains("startswith%28displayName%2C%27pim%27%29"));
                assert!(!url.contains("appId"));
                assert!(!url.contains("id%20eq"));
                eyre::Ok(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn url_filters_application_ids_only_for_uuid_search_terms() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let request = EntraApplicationRegistrationSearchRequest {
                    search_term: "00000003-0000-0000-c000-000000000000".to_owned(),
                    authentication,
                };
                let url = request.url();

                assert!(url.contains("appId%20eq%20%2700000003-0000-0000-c000-000000000000%27"));
                assert!(url.contains("id%20eq%20%2700000003-0000-0000-c000-000000000000%27"));
                assert!(!url.contains("startswith%28displayName"));
                eyre::Ok(())
            })
            .expect("uncancelled test scope");
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraApplicationRegistrationSearchRequest,
    name = "application-registration-search",
    cached
);
