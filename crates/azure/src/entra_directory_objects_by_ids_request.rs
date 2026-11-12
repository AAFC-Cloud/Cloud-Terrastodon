use crate::EntraDirectoryObject;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphResponse;
use crate::entra_directory_objects_by_ids_request_body::EntraDirectoryObjectsByIdsRequestBody;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use http::Method;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
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
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Debug, Facet)]
pub struct EntraDirectoryObjectsByIdsRequest {
    pub ids: Vec<Uuid>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraDirectoryObjectsByIdsRequest {
    fn normalized_ids(&self) -> Vec<Uuid> {
        let mut ids = self.ids.clone();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    fn cache_key_for_ids(&self, ids: &[Uuid]) -> eyre::Result<CacheKey> {
        let mut hasher = blake3::Hasher::new();
        for id in ids {
            hasher.update(id.as_bytes());
        }
        let ids_hash = hasher.finalize().to_hex().to_string();

        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "POST",
            "directoryObjects",
            "getByIds",
            self.authentication.tenant_id.to_string().as_str(),
            // The previous key cached a user/group/service-principal filter;
            // those results cannot answer an all-types existence lookup.
            "all-types",
            ids_hash.as_str(),
        ]))
    }
}

impl InvalidatesOwnCacheKey for EntraDirectoryObjectsByIdsRequest {}

impl HasCacheKey for EntraDirectoryObjectsByIdsRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        self.cache_key_for_ids(&self.normalized_ids())
    }
}

impl ExecutableRequest for EntraDirectoryObjectsByIdsRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraDirectoryObject>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let ids = self.normalized_ids();
            if ids.is_empty() {
                debug!(tenant_id = %self.authentication.tenant_id, "Skipping empty Entra directory object lookup");
                return Ok(Vec::new());
            }

            let cache_key = self.cache_key()?;
            let mut batch =
                MicrosoftGraphBatchRequest::<EntraDirectoryObjectsByIdsRequestBody>::new(
                    self.authentication,
                )
                .cancellation_safe();
            batch.cache(cache_key);

            for (chunk_index, ids) in ids.chunks(MAX_DIRECTORY_OBJECT_IDS).enumerate() {
                debug!(
                    tenant_id = %self.authentication.tenant_id,
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
                .send::<MicrosoftGraphResponse<Arc<EntraDirectoryObject>>>(&executor)
                .await?
                .responses;
            let mut objects = Vec::new();
            for response in responses {
                objects.extend(response.body()?.value.iter().cloned());
            }

            debug!(tenant_id = %self.authentication.tenant_id, count = objects.len(), "Found Entra directory objects");
            Ok(objects)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectsByIdsRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectsByIdsRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraDirectoryObjectsByIdsRequest => Vec<Arc<EntraDirectoryObject>>, effects = [Read]);

#[cfg(test)]
#[allow(
    clippy::disallowed_types,
    reason = "These tests inspect the encoded request body before decoding its individual fields."
)]
mod tests {
    use super::*;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_command::CancellationToken;
    use facet_json::RawJson;

    #[test]
    fn normalizes_ids_for_stable_lookup_and_cache_keys() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let first_id = Uuid::from_u128(1);
                let second_id = Uuid::from_u128(2);
                let first = EntraDirectoryObjectsByIdsRequest {
                    ids: vec![first_id, second_id, first_id],
                    authentication,
                };
                let second = EntraDirectoryObjectsByIdsRequest {
                    ids: vec![second_id, first_id],
                    authentication,
                };

                assert_eq!(first.normalized_ids(), vec![first_id, second_id]);
                assert_eq!(first.cache_key()?.scope, second.cache_key()?.scope);
                eyre::Ok(())
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

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraDirectoryObjectsByIdsRequest,
    name = "entra-directory-objects-by-ids",
    cached
);
