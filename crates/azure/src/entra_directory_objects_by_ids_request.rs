use crate::EntraDirectoryObject;
use crate::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraDirectoryObjectType;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tracing::debug;

const MAX_DIRECTORY_OBJECT_IDS: usize = 1_000;
const DIRECTORY_OBJECTS_BY_IDS_URL: &str =
    "https://graph.microsoft.com/v1.0/directoryObjects/getByIds";

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, arbitrary::Arbitrary, Facet)]
pub struct EntraDirectoryObjectsByIdsRequest {
    pub tenant_id: AzureTenantId,
    pub ids: Vec<Uuid>,
}

pub fn fetch_entra_directory_objects_by_ids(
    tenant_id: AzureTenantId,
    ids: impl IntoIterator<Item = Uuid>,
) -> EntraDirectoryObjectsByIdsRequest {
    EntraDirectoryObjectsByIdsRequest {
        tenant_id,
        ids: ids.into_iter().collect(),
    }
}

#[derive(Debug, Facet)]
struct EntraDirectoryObjectsByIdsRequestBody {
    ids: Vec<Uuid>,
    types: Vec<String>,
}

impl EntraDirectoryObjectsByIdsRequest {
    fn normalized_ids(&self) -> Vec<Uuid> {
        let mut ids = self.ids.clone();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    fn cache_key_for_ids(&self, ids: &[Uuid]) -> CacheKey {
        let mut hasher = blake3::Hasher::new();
        for id in ids {
            hasher.update(id.as_bytes());
        }
        let ids_hash = hasher.finalize().to_hex().to_string();

        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "POST",
            "directoryObjects",
            "getByIds",
            self.tenant_id.to_string().as_str(),
            ids_hash.as_str(),
        ]))
    }

    fn request_body(ids: &[Uuid]) -> EntraDirectoryObjectsByIdsRequestBody {
        EntraDirectoryObjectsByIdsRequestBody {
            ids: ids.to_vec(),
            types: EntraDirectoryObjectType::PRINCIPAL_TYPES
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }
}

#[async_trait]
impl CacheableCommand for EntraDirectoryObjectsByIdsRequest {
    type Output = Vec<EntraDirectoryObject>;

    fn cache_key(&self) -> CacheKey {
        self.cache_key_for_ids(&self.normalized_ids())
    }

    async fn run(self) -> Result<Self::Output> {
        let ids = self.normalized_ids();
        if ids.is_empty() {
            debug!(tenant_id = %self.tenant_id, "Skipping empty Entra directory object lookup");
            return Ok(Vec::new());
        }

        let cache_key = self.cache_key();
        let chunk_count = ids.len().div_ceil(MAX_DIRECTORY_OBJECT_IDS);
        let mut objects = Vec::new();

        for (chunk_index, ids) in ids.chunks(MAX_DIRECTORY_OBJECT_IDS).enumerate() {
            debug!(
                tenant_id = %self.tenant_id,
                chunk_index,
                chunk_count,
                count = ids.len(),
                "Fetching Entra directory objects by object id"
            );

            let chunk_cache_key = if chunk_count == 1 {
                cache_key.clone()
            } else {
                CacheKey {
                    path: cache_key.path.join(chunk_index.to_string()),
                    valid_for: cache_key.valid_for,
                }
            };
            let body = facet_json::to_string_pretty(&Self::request_body(ids))
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            let response: MicrosoftGraphResponse<EntraDirectoryObject> =
                RestRequest::new(http::Method::POST, DIRECTORY_OBJECTS_BY_IDS_URL)?
                    .tenant(self.tenant_id)
                    .cache(chunk_cache_key)
                    .body(body)
                    .receive()
                    .await?;
            objects.extend(response.value);
        }

        debug!(tenant_id = %self.tenant_id, count = objects.len(), "Found Entra directory objects");
        Ok(objects)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraDirectoryObjectsByIdsRequest);
cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectsByIdsRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectsByIdsRequest);
cloud_terrastodon_registry::register_into_future!(EntraDirectoryObjectsByIdsRequest => Vec<EntraDirectoryObject>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ids_for_stable_lookup_and_cache_keys() {
        let first_id = Uuid::from_u128(1);
        let second_id = Uuid::from_u128(2);
        let first = fetch_entra_directory_objects_by_ids(
            AzureTenantId::new(Uuid::nil()),
            [first_id, second_id, first_id],
        );
        let second = fetch_entra_directory_objects_by_ids(
            AzureTenantId::new(Uuid::nil()),
            [second_id, first_id],
        );

        assert_eq!(first.normalized_ids(), vec![first_id, second_id]);
        assert_eq!(first.cache_key().path, second.cache_key().path);
    }

    #[test]
    fn request_body_restricts_lookup_to_supported_principal_types() -> eyre::Result<()> {
        let body = EntraDirectoryObjectsByIdsRequest::request_body(&[Uuid::from_u128(1)]);
        let body = facet_json::to_string(&body)?;

        assert!(body.contains("user"));
        assert!(body.contains("group"));
        assert!(body.contains("servicePrincipal"));
        Ok(())
    }
}
