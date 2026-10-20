use crate::EntraDirectoryObject;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphResponse;
use crate::entra_directory_objects_by_ids_request_body::EntraDirectoryObjectsByIdsRequestBody;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
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

/// Resolve IDs through Microsoft Graph v1.0 [`directoryObjects/getByIds`](https://learn.microsoft.com/en-us/graph/api/directoryobject-getbyids?view=graph-rest-1.0).
///
/// Uses Graph's default `directoryObject` search in batches of at most 1,000 IDs.
/// That default includes devices and organizational contacts, but excludes Cloud
/// Solution Provider partner references (`directoryObjectPartnerReference`).
/// Returned objects retain their types; any unmodeled type fails decoding
/// explicitly instead of appearing to be a missing ID.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Facet)]
pub struct EntraDirectoryObjectsByIdsRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub ids: Vec<Uuid>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraDirectoryObjectsByIdsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_entra_directory_objects_by_ids<'a>(
    ids: impl IntoIterator<Item = Uuid>,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> EntraDirectoryObjectsByIdsRequest<'a> {
    EntraDirectoryObjectsByIdsRequest {
        cancellation: cancellation.clone(),
        ids: ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    }
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
            self.auth_context.tenant_id.to_string().as_str(),
            // The previous key cached a user/group/service-principal filter;
            // those results cannot answer an all-types existence lookup.
            "all-types",
            ids_hash.as_str(),
        ]))
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
            debug!(tenant_id = %self.auth_context.tenant_id, "Skipping empty Entra directory object lookup");
            return Ok(Vec::new());
        }

        let cache_key = self.cache_key();
        let mut batch = MicrosoftGraphBatchRequest::<EntraDirectoryObjectsByIdsRequestBody>::new(
            self.auth_context.as_ref(),
            &self.cancellation,
        )
        .cancellation_safe();
        batch.cache(cache_key);

        for (chunk_index, ids) in ids.chunks(MAX_DIRECTORY_OBJECT_IDS).enumerate() {
            debug!(
                tenant_id = %self.auth_context.tenant_id,
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
                Some(EntraDirectoryObjectsByIdsRequestBody { ids: ids.to_vec() }),
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

        debug!(tenant_id = %self.auth_context.tenant_id, count = objects.len(), "Found Entra directory objects");
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
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use facet_json::RawJson;

    #[test]
    fn normalizes_ids_for_stable_lookup_and_cache_keys() {
        CancellationToken::new()
            .with_scope(|cancellation| {
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context = auth_context
                    .bind_to_azure_tenant(AzureTenantId::new(Uuid::nil()))
                    .expect("default auth context should bind to a tenant");
                let first_id = Uuid::from_u128(1);
                let second_id = Uuid::from_u128(2);
                let first = fetch_entra_directory_objects_by_ids(
                    [first_id, second_id, first_id],
                    &auth_context,
                    &cancellation,
                );
                let second = fetch_entra_directory_objects_by_ids(
                    [second_id, first_id],
                    &auth_context,
                    &cancellation,
                );

                assert_eq!(first.normalized_ids(), vec![first_id, second_id]);
                assert_eq!(first.cache_key().path, second.cache_key().path);
                Ok::<_, eyre::Error>(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn request_body_does_not_filter_directory_object_types() -> eyre::Result<()> {
        let body = EntraDirectoryObjectsByIdsRequestBody {
            ids: vec![Uuid::from_u128(1)],
        };
        let body = facet_json::to_string(&body)?;
        let body = facet_json::from_str::<HashMap<String, RawJson<'static>>>(&body)?;

        assert!(body.contains_key("ids"));
        assert!(!body.contains_key("types"));
        Ok(())
    }
}
