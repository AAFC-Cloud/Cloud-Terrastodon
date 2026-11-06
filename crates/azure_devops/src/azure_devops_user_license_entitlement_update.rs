use crate::fetch_azure_devops_user_license_entitlements;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseAccessLevelUpdate;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseEntitlementUpdateResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserEntitlementsPatchResponse;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1>
#[must_use = "This is an unsent request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub user: AzureDevOpsUserArgument<'a>,
    pub license_kind: AzureDevOpsLicenseType,
}

#[track_caller]
pub fn update_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
    license_kind: AzureDevOpsLicenseType,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsUserLicenseEntitlementUpdateRequest<'a>> {
    executor.execute(AzureDevOpsUserLicenseEntitlementUpdateRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        user: user.into(),
        license_kind,
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            user: AzureDevOpsUserArgument::arbitrary(u)?.into_owned(),
            license_kind: AzureDevOpsLicenseType::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        {
            let mut key = CacheKey::from_relative_path(PathBuf::from_iter([
                "az",
                "devops",
                self.org_url.name(),
                "license",
                "entitlement",
                "update-user",
                &self.user.to_string(),
            ]))?;
            key.valid_for = Duration::ZERO;
            Ok(key)
        }
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    type Error = eyre::Report;
    type Output = AzureDevOpsLicenseEntitlementUpdateResponse;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let access_level = AzureDevOpsLicenseAccessLevelUpdate::try_from(&self.license_kind)?;
            let user_id = match &self.user {
                AzureDevOpsUserArgument::Id(id) => *id.as_ref(),
                AzureDevOpsUserArgument::Email(_) => {
                    let listing = fetch_azure_devops_user_license_entitlements(
                        self.org_url.as_ref(),
                        self.auth_context.as_ref(),
                        &executor,
                    );
                    listing.invalidate().await?;
                    let users = listing.await?;
                    let mut matches = users
                        .into_iter()
                        .filter(|entry| self.user.matches(&entry.user));
                    let user = matches
                        .next()
                        .ok_or_else(|| eyre::eyre!("no entitlement matches the selected user"))?;
                    eyre::ensure!(
                        matches.next().is_none(),
                        "the user selection matches multiple entitlements"
                    );
                    user.user_id
                }
            };
            let patch = AzureDevOpsJsonPatch::new(vec![AzureDevOpsJsonPatchOperation::Replace {
                path: "/accessLevel".to_owned(),
                value: ArbitraryJson::try_from_facet(&access_level)?,
            }])?;
            let mut url =
                self.org_url
                    .api_url("vsaex.dev.azure.com", "", &[("api-version", "7.1")])?;
            url.path_segments_mut()
                .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                .pop_if_empty()
                .extend(["_apis", "userentitlements", &user_id.to_string()]);
            let cache_key = self.cache_key()?;
            // Retain the mutation result and invalidate its affected inventory as
            // one completion unit. A decoded partial failure can have side effects.
            executor
                .deferred(
                    "updating a user license and invalidating entitlements",
                    |executor| async move {
                        let response = RestRequest::from_method_and_url(Method::PATCH, url)?
                            .headers(RequestHeaders::from_header(
                                "Content-Type",
                                "application/json-patch+json",
                            )?)
                            .body(facet_json::to_string(&patch)?)
                            .cache(cache_key)
                            .azure_devops_auth_context(self.auth_context.as_ref())?
                            .receive::<AzureDevOpsUserEntitlementsPatchResponse>(&executor)
                            .await;
                        // A response-decoding failure cannot prove that PATCH had no effect.
                        let invalidation = fetch_azure_devops_user_license_entitlements(
                            self.org_url.as_ref(),
                            self.auth_context.as_ref(),
                            &executor,
                        )
                        .invalidate()
                        .await;
                        let response = match (response, invalidation) {
                            (Ok(response), Ok(())) => response,
                            (Err(error), Ok(())) | (Ok(_), Err(error)) => return Err(error),
                            (Err(error), Err(invalidation)) => {
                                return Err(error.wrap_err(format!(
                                "entitlement inventory invalidation also failed: {invalidation:#}"
                            )));
                            }
                        };
                        eyre::ensure!(
                            response.is_success
                                && response
                                    .operation_results
                                    .iter()
                                    .all(|operation| operation.is_success),
                            "Azure DevOps rejected one or more license changes: {:?}",
                            response.operation_results,
                        );
                        eyre::ensure!(
                            response.user_entitlement.id == user_id,
                            "license update returned a different user"
                        );
                        Ok(response.user_entitlement)
                    },
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> => AzureDevOpsLicenseEntitlementUpdateResponse, effects = [Write]);

#[cfg(test)]
mod tests {
    use super::AzureDevOpsLicenseEntitlementUpdateResponse;
    use cloud_terrastodon_azure_devops_types::LastAccessedDate;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use std::collections::BTreeMap;

    #[test]
    fn registered_response_constructor_generates_round_trippable_json() -> eyre::Result<()> {
        let constructor = AzureDevOpsLicenseEntitlementUpdateResponse::REGISTERED_ARBITRARY;
        let mut saw_never_accessed = false;
        let mut saw_accessed = false;
        let mut saw_empty_collection = false;
        let mut saw_populated_collection = false;

        for seed in 0..=3 {
            // The prefix varies dates and license data; the zero suffix supplies
            // valid inputs for the generated license strings.
            let mut bytes = vec![0; 256];
            bytes[..128].fill(seed);
            let mut input = ArbitraryBytes::new(bytes);
            let output = constructor.generate(&mut input)?;
            assert!(input.as_slice().len() < 256);
            let response = output
                .downcast::<AzureDevOpsLicenseEntitlementUpdateResponse>()
                .expect("registered constructor must return its exact declared response type");

            let json = facet_json::to_string(response.as_ref())?;
            let decoded: AzureDevOpsLicenseEntitlementUpdateResponse = facet_json::from_str(&json)?;
            assert_eq!(decoded.id, response.id);
            assert_eq!(decoded.date_created, response.date_created);
            assert_eq!(decoded.last_accessed_date, response.last_accessed_date);
            assert_eq!(facet_json::to_string(&decoded)?, json);

            let user: BTreeMap<String, String> = facet_json::from_str(response.user.as_str())?;
            assert_eq!(user.get("id"), Some(&response.id.to_string()));
            assert_eq!(user.get("subjectKind").map(String::as_str), Some("user"));
            assert!(user.get("displayName").is_some_and(|name| !name.is_empty()));
            for objects in [
                &response.extensions,
                &response.group_assignments,
                &response.project_entitlements,
            ] {
                assert!(objects.len() <= 3);
                saw_empty_collection |= objects.is_empty();
                saw_populated_collection |= !objects.is_empty();
                for object in objects {
                    let fields: BTreeMap<String, String> = facet_json::from_str(object.as_str())?;
                    assert!(fields.get("id").is_some_and(|id| !id.is_empty()));
                    assert!(
                        fields
                            .get("displayName")
                            .is_some_and(|name| !name.is_empty())
                    );
                }
            }

            assert!(
                (946_684_800..=4_102_444_799).contains(&response.date_created.unwrap().timestamp())
            );
            match response.last_accessed_date {
                LastAccessedDate::Never => saw_never_accessed = true,
                LastAccessedDate::Some(date) => {
                    saw_accessed = true;
                    assert!(date >= response.date_created.unwrap());
                    assert!(date.timestamp() <= 4_102_444_799);
                }
            }
        }

        assert!(saw_never_accessed && saw_accessed);
        assert!(saw_empty_collection && saw_populated_collection);
        Ok(())
    }
}
