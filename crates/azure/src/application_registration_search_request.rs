use crate::MicrosoftGraphHelper;
use crate::PercentEncodeExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Facet)]
pub struct ApplicationRegistrationSearchRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub search_term: String,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> Arbitrary<'a> for ApplicationRegistrationSearchRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: Arbitrary::arbitrary(u)?,
            search_term: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn search_application_registrations<'a>(
    search_term: impl Into<String>,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> ApplicationRegistrationSearchRequest<'a> {
    ApplicationRegistrationSearchRequest {
        cancellation: cancellation.clone(),
        search_term: search_term.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl ApplicationRegistrationSearchRequest<'_> {
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

#[async_trait]
impl CacheableCommand for ApplicationRegistrationSearchRequest<'_> {
    type Output = Vec<EntraApplicationRegistration>;

    fn cache_key(&self) -> CacheKey {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            "search",
            self.auth_context.tenant_id.to_string().as_str(),
            search_hash.as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let search_term = self.search_term.trim();
        if search_term.is_empty() {
            debug!(tenant_id = %self.auth_context.tenant_id, "Skipping empty application registration search");
            return Ok(Vec::new());
        }

        debug!(
            tenant_id = %self.auth_context.tenant_id,
            search_term,
            "Searching application registrations through Microsoft Graph"
        );
        let applications: Vec<EntraApplicationRegistration> = MicrosoftGraphHelper::new(
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
            &self.cancellation,
        )
        .fetch_all(&self.cancellation)
        .await?;
        debug!(
            tenant_id = %self.auth_context.tenant_id,
            count = applications.len(),
            "Found application registrations"
        );
        Ok(applications)
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationSearchRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(ApplicationRegistrationSearchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationSearchRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ApplicationRegistrationSearchRequest<'static> => Vec<EntraApplicationRegistration>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_default_tenant_id;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_azure_types::uuid::Uuid;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;

    fn test_auth_context() -> AzureTenantAuthContext {
        AuthContext::explicit_azure_cli()
            .bind_to_azure_tenant(AzureTenantId::new(Uuid::nil()))
            .expect("default auth context should bind to a tenant")
    }

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let auth_context = test_auth_context();
                let request = search_application_registrations(
                    "Cloud Terrastodon's PIM",
                    &auth_context,
                    &cancellation,
                );

                assert!(request.url().contains("Cloud%20Terrastodon%27%27s%20PIM"));
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn url_filters_graph_application_properties() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let auth_context = test_auth_context();
                let request = search_application_registrations("pim", &auth_context, &cancellation);
                let url = request.url();

                assert!(url.contains("startswith%28displayName%2C%27pim%27%29"));
                assert!(!url.contains("appId"));
                assert!(!url.contains("id%20eq"));
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn url_filters_application_ids_only_for_uuid_search_terms() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let auth_context = test_auth_context();
                let request = search_application_registrations(
                    "00000003-0000-0000-c000-000000000000",
                    &auth_context,
                    &cancellation,
                );
                let url = request.url();

                assert!(url.contains("appId%20eq%20%2700000003-0000-0000-c000-000000000000%27"));
                assert!(url.contains("id%20eq%20%2700000003-0000-0000-c000-000000000000%27"));
                assert!(!url.contains("startswith%28displayName"));
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[tokio::test]
    #[ignore = "live Microsoft Graph smoke test; requires Azure CLI authentication"]
    async fn application_search_filters_are_accepted_by_graph() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                const MICROSOFT_GRAPH_APP_ID: &str = "00000003-0000-0000-c000-000000000000";

                let tenant_id = get_default_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                for search_term in ["Microsoft Graph", MICROSOFT_GRAPH_APP_ID] {
                    let request = search_application_registrations(
                        search_term,
                        &tenant_auth_context,
                        &cancellation,
                    );

                    // The Microsoft Graph service principal is present in customer
                    // tenants, but its application object is owned by Microsoft and
                    // is not necessarily returned by /applications in the customer
                    // tenant. This smoke test therefore verifies that both generated
                    // requests are accepted by Graph rather than requiring a result.
                    let _: Vec<EntraApplicationRegistration> = MicrosoftGraphHelper::new(
                        request.url(),
                        None,
                        &tenant_auth_context,
                        &cancellation,
                    )
                    .fetch_all(&cancellation)
                    .await?;
                }

                Ok(())
            })
            .await
    }
}
