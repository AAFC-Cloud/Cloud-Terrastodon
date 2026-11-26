use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureActivityLogEvent;
use cloud_terrastodon_azure_types::AzureActivityLogPage;
use cloud_terrastodon_azure_types::AzureActivityLogScope;
use cloud_terrastodon_azure_types::ManagementGroupId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::strip_prefix_case_insensitive;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use http::Method;
use jiff::RoundMode;
use jiff::SignedDuration;
use jiff::Timestamp;
use jiff::TimestampRound;
use jiff::Unit;
use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const API_VERSION: &str = "2015-04-01";
const MANAGEMENT_GROUP_API_VERSION: &str = "2017-03-01-preview";

/// Read the scoped [Azure Monitor activity log](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01).
/// The requested period is inclusive and uses UTC. The projection omits caller,
/// claims and other fields that are unnecessary for an activity inventory.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Facet)]
pub struct AzureActivityLogListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub scope: AzureActivityLogScope,
    pub from: Timestamp,
    pub until: Timestamp,
}

impl AzureActivityLogListRequest {
    /// A 24-hour window ending at the start of the current UTC minute.
    /// The minute boundary lets consecutive opens share the same window and cache entries.
    pub fn last_day(
        authentication: AzureBearerAuthenticationSelection,
        scope: AzureActivityLogScope,
    ) -> Result<Self> {
        let until = Timestamp::now()
            .round(
                TimestampRound::new()
                    .smallest(Unit::Minute)
                    .mode(RoundMode::Floor),
            )
            .wrap_err("rounding the activity log time window")?;
        Ok(Self {
            authentication,
            scope,
            from: until.checked_sub(SignedDuration::from_hours(24))?,
            until,
        })
    }

    fn validate_page_url(url: &Url, collection_path: &str) -> Result<()> {
        // Keep the token on the same ARM collection. Azure may
        // change route segment casing when constructing its continuation URL.
        ensure!(
            url.scheme() == "https"
                && url.host_str() == Some("management.azure.com")
                && url.port_or_known_default() == Some(443)
                && url.username().is_empty()
                && url.password().is_none()
                && url.fragment().is_none()
                && url.path().eq_ignore_ascii_case(collection_path),
            "activity log continuation URL leaves the selected ARM collection"
        );
        Ok(())
    }
}

impl<'a> Arbitrary<'a> for AzureActivityLogListRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let [first, second]: [Timestamp; 2] = input.arbitrary()?;
        Ok(Self {
            authentication: input.arbitrary()?,
            scope: input.arbitrary()?,
            from: first.min(second),
            until: first.max(second),
        })
    }
}

impl HasCacheKey for AzureActivityLogListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        let mut path =
            PathBuf::from("az/arm/tenant").join(self.authentication.tenant_id.to_string());
        match &self.scope {
            AzureActivityLogScope::Subscription(id) => {
                path.extend(["subscriptions", &id.to_string()]);
            }
            AzureActivityLogScope::ResourceGroup(group) => {
                path.extend([
                    "subscriptions",
                    &group.subscription_id.to_string(),
                    "resource-groups",
                    &sanitize_windows_path_component(
                        &group.resource_group_name.to_ascii_lowercase(),
                    ),
                ]);
            }
            AzureActivityLogScope::ManagementGroup(group) => {
                path.extend([
                    "management-groups",
                    &sanitize_windows_path_component(&group.name().to_ascii_lowercase()),
                ]);
            }
            AzureActivityLogScope::Resource(id) => {
                path.extend([
                    "resources",
                    &sanitize_windows_path_component(&id.expanded_form().to_ascii_lowercase()),
                ]);
            }
        }
        let mut key = CacheKey::from_relative_path(path.join("activity-log"))?;
        key.valid_for = Duration::from_secs(60);
        Ok(key)
    }
}

impl InvalidatesOwnCacheKey for AzureActivityLogListRequest {}

impl ExecutableRequest for AzureActivityLogListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureActivityLogEvent>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Saved requests express the window as explicit endpoints.
            ensure!(
                self.from <= self.until,
                "activity log end time must be at or after its start time"
            );
            let mut filter = format!(
                "eventTimestamp ge '{}' and eventTimestamp le '{}'",
                self.from, self.until,
            );
            let (path, api_version) = match &self.scope {
                AzureActivityLogScope::Subscription(id) => (id.expanded_form(), API_VERSION),
                AzureActivityLogScope::ResourceGroup(group) => {
                    write!(
                        filter,
                        " and resourceGroupName eq '{}'",
                        group.resource_group_name.replace('\'', "''"),
                    )?;
                    (group.subscription_id.expanded_form(), API_VERSION)
                }
                AzureActivityLogScope::ManagementGroup(group) => {
                    // Management group activity is exposed by the preview collection.
                    // https://learn.microsoft.com/en-us/azure/azure-monitor/fundamentals/activity-log#list-management-group-level-activity-log-events
                    let path = group.expanded_form();
                    ensure!(
                        !group.name().is_empty(),
                        "activity log requires a management group ID"
                    );
                    ManagementGroupId::try_from_expanded(&path)?;
                    (path, MANAGEMENT_GROUP_API_VERSION)
                }
                AzureActivityLogScope::Resource(resource) => {
                    let resource_uri = resource.expanded_form();
                    let resource_path = resource_uri.strip_prefix('/').ok_or_else(|| {
                        eyre!("activity log requires an absolute ARM resource ID")
                    })?;
                    ensure!(
                        resource_path
                            .split('/')
                            .all(|segment| !matches!(segment, "" | "." | "..")),
                        "ARM resource ID contains an invalid path segment"
                    );
                    let path = if let Some(subscription) = self.scope.subscription_id() {
                        subscription.expanded_form()
                    } else {
                        // Tenant Activity Logs - List accepts this resourceUri filter with eventChannels.
                        // https://learn.microsoft.com/en-us/rest/api/monitor/tenant-activity-logs/list?view=rest-monitor-2015-04-01#uri-parameters
                        ensure!(
                            strip_prefix_case_insensitive(&resource_uri, "/providers/").is_ok(),
                            "activity log requires a subscription- or provider-scoped ARM resource ID"
                        );
                        filter.push_str(" and eventChannels eq 'Admin, Operation'");
                        String::new()
                    };
                    write!(
                        filter,
                        " and resourceUri eq '{}'",
                        resource_uri.replace('\'', "''"),
                    )?;
                    (path, API_VERSION)
                }
            };
            let mut url = Url::parse("https://management.azure.com")?;
            url.path_segments_mut()
                .map_err(|_| eyre!("ARM endpoint cannot contain path segments"))?
                .extend(path.split('/').filter(|segment| !segment.is_empty()))
                .extend([
                    "providers",
                    "Microsoft.Insights",
                    "eventtypes",
                    "management",
                    "values",
                ]);
            url.query_pairs_mut()
                .append_pair("api-version", api_version)
                .append_pair("$filter", &filter)
                .append_pair(
                    "$select",
                    // Azure CLI's activity-log list projection also supports category.
                    // https://learn.microsoft.com/en-us/cli/azure/monitor/activity-log#az-monitor-activity-log-list
                    "eventDataId,eventTimestamp,operationName,status,resourceId,level,category,subscriptionId,resourceGroupName",
                );

            let collection_path = url.path().to_owned();
            let page_cache = self.cache_key()?.join("rest/pages")?;
            let mut next_url = Some(url);
            let mut visited = HashSet::new();
            let mut events = Vec::new();
            while let Some(url) = next_url {
                executor.cancellation().bail_if_cancelled()?;
                Self::validate_page_url(&url, &collection_path)?;
                ensure!(
                    visited.insert(url.clone()),
                    "activity log pagination repeated a page"
                );
                let page_key =
                    page_cache.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?;
                let page = RestRequest::from_method_and_url(Method::GET, url)?
                    .authentication(self.authentication.into())
                    .cache(page_key)
                    .receive_cached::<AzureActivityLogPage>(
                        &executor,
                        CacheOperationOwner::of::<Self>(),
                    )
                    .await?;
                executor.cancellation().bail_if_cancelled()?;
                events.extend(page.value.iter().cloned());
                next_url = page.next_link.clone();
            }
            Ok(events)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureActivityLogListRequest => Vec<Arc<AzureActivityLogEvent>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuation_stays_on_the_same_subscription_collection() -> Result<()> {
        let collection = "/subscriptions/00000000-0000-0000-0000-000000000001/providers/Microsoft.Insights/eventtypes/management/values";
        let next = format!(
            "https://management.azure.com{}?$skiptoken=synthetic",
            collection.to_ascii_lowercase()
        )
        .parse()?;
        AzureActivityLogListRequest::validate_page_url(&next, collection)?;
        for destination in [
            format!("https://example.invalid{collection}?$skiptoken=synthetic"),
            format!("http://management.azure.com{collection}?$skiptoken=synthetic"),
            format!(
                "https://management.azure.com{}?$skiptoken=synthetic",
                collection.replace("000000000001", "000000000002")
            ),
        ] {
            let next = destination.parse()?;
            assert!(AzureActivityLogListRequest::validate_page_url(&next, collection).is_err());
        }
        Ok(())
    }
}
