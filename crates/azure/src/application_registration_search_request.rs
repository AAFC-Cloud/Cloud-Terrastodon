use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Arbitrary, Facet)]
pub struct ApplicationRegistrationSearchRequest {
    pub tenant_id: AzureTenantId,
    pub search_term: String,
}

pub fn search_application_registrations(
    tenant_id: AzureTenantId,
    search_term: impl Into<String>,
) -> ApplicationRegistrationSearchRequest {
    ApplicationRegistrationSearchRequest {
        tenant_id,
        search_term: search_term.into(),
    }
}

impl ApplicationRegistrationSearchRequest {
    fn url(&self) -> String {
        let search_term = escape_odata_string(self.search_term.trim());
        let filter = format!(
            "appId eq '{search_term}' or id eq '{search_term}' or startswith(displayName,'{search_term}') or startswith(uniqueName,'{search_term}') or identifierUris/any(x:startswith(x,'{search_term}'))"
        );

        format!(
            "https://graph.microsoft.com/v1.0/applications?$filter={}",
            percent_encode_query_component(&filter)
        )
    }
}

#[async_trait]
impl CacheableCommand for ApplicationRegistrationSearchRequest {
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
            self.tenant_id.to_string().as_str(),
            search_hash.as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let search_term = self.search_term.trim();
        if search_term.is_empty() {
            debug!(tenant_id = %self.tenant_id, "Skipping empty application registration search");
            return Ok(Vec::new());
        }

        debug!(
            tenant_id = %self.tenant_id,
            search_term,
            "Searching application registrations through Microsoft Graph"
        );
        let applications: Vec<EntraApplicationRegistration> =
            MicrosoftGraphHelper::new(self.tenant_id, self.url(), Some(self.cache_key()))
                .fetch_all()
                .await?;
        debug!(
            tenant_id = %self.tenant_id,
            count = applications.len(),
            "Found application registrations"
        );
        Ok(applications)
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

fn percent_encode_query_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }
    encoded
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationSearchRequest);
cloud_terrastodon_registry::register_thing!(ApplicationRegistrationSearchRequest);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationSearchRequest);
cloud_terrastodon_registry::register_into_future!(ApplicationRegistrationSearchRequest => Vec<EntraApplicationRegistration>);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_types::uuid::Uuid;

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        let request = search_application_registrations(
            AzureTenantId::new(Uuid::nil()),
            "Cloud Terrastodon's PIM",
        );

        assert!(request.url().contains("Cloud%20Terrastodon%27%27s%20PIM"));
    }

    #[test]
    fn url_filters_graph_application_properties() {
        let request = search_application_registrations(AzureTenantId::new(Uuid::nil()), "pim");
        let url = request.url();

        assert!(url.contains("appId%20eq%20%27pim%27"));
        assert!(url.contains("startswith%28displayName%2C%27pim%27%29"));
        assert!(url.contains("identifierUris%2Fany%28x%3Astartswith%28x%2C%27pim%27%29%29"));
    }
}
