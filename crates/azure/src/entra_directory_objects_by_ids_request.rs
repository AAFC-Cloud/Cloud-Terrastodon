use crate::EntraDirectoryObject;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraDirectoryObjectType;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

const MAX_DIRECTORY_OBJECT_IDS: usize = 1_000;
const DIRECTORY_OBJECTS_BY_IDS_URL: &str =
    "https://graph.microsoft.com/v1.0/directoryObjects/getByIds";

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Facet)]
pub struct EntraDirectoryObjectsByIdsRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub ids: Vec<Uuid>,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraDirectoryObjectsByIdsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_entra_directory_objects_by_ids<'a>(
    tenant_id: AzureTenantId,
    ids: impl IntoIterator<Item = Uuid>,
    auth_context: &'a AuthContext,
) -> EntraDirectoryObjectsByIdsRequest<'a> {
    EntraDirectoryObjectsByIdsRequest {
        tenant_id,
        ids: ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[derive(Debug, Facet)]
struct EntraDirectoryObjectsByIdsRequestBody {
    ids: Vec<Uuid>,
    types: Vec<String>,
}

impl EntraDirectoryObjectsByIdsRequest<'_> {
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
impl CacheableCommand for EntraDirectoryObjectsByIdsRequest<'_> {
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
        let mut batch = MicrosoftGraphBatchRequest::<EntraDirectoryObjectsByIdsRequestBody>::new(
            self.tenant_id,
            self.auth_context.as_ref(),
        );
        batch.cache(cache_key);

        for (chunk_index, ids) in ids.chunks(MAX_DIRECTORY_OBJECT_IDS).enumerate() {
            debug!(
                tenant_id = %self.tenant_id,
                chunk_index,
                count = ids.len(),
                "Queueing Entra directory objects by object id lookup"
            );
            batch.add(MicrosoftGraphBatchRequestEntry::new(
                format!("directory-objects-{chunk_index}"),
                Method::POST,
                DIRECTORY_OBJECTS_BY_IDS_URL.to_owned(),
                HashMap::from([(
                    String::from("Content-Type"),
                    String::from("application/json"),
                )]),
                Some(Self::request_body(ids)),
            ));
        }

        let responses = batch
            .send::<MicrosoftGraphResponse<EntraDirectoryObject>>()
            .await?
            .responses;
        let mut objects = Vec::new();
        for response in responses {
            objects.extend(response.into_body()?.value);
        }

        debug!(tenant_id = %self.tenant_id, count = objects.len(), "Found Entra directory objects");
        Ok(objects)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraDirectoryObjectsByIdsRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectsByIdsRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectsByIdsRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraDirectoryObjectsByIdsRequest<'static> => Vec<EntraDirectoryObject>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ids_for_stable_lookup_and_cache_keys() {
        let auth_context = AuthContext::default();
        let first_id = Uuid::from_u128(1);
        let second_id = Uuid::from_u128(2);
        let first = fetch_entra_directory_objects_by_ids(
            AzureTenantId::new(Uuid::nil()),
            [first_id, second_id, first_id],
            &auth_context,
        );
        let second = fetch_entra_directory_objects_by_ids(
            AzureTenantId::new(Uuid::nil()),
            [second_id, first_id],
            &auth_context,
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
