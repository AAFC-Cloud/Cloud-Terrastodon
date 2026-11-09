use cloud_terrastodon_azure_types::OptionalNonEmptyStringProxy;
use cloud_terrastodon_azure_types::ResourceGroupName;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_azure_types::SubscriptionName;
use facet::Facet;

/// Projection of the [policy-state resource query](https://learn.microsoft.com/en-us/azure/governance/policy/samples/resource-graph-samples)
/// used by the compliance picker. Blank names represent resources without a
/// resource group or an unmatched subscription-name lookup.
#[derive(Clone, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct PolicyNoncompliantResourceRow {
    #[facet(proxy = OptionalNonEmptyStringProxy)]
    pub resource_group_name: Option<ResourceGroupName>,
    pub subscription_id: SubscriptionId,
    #[facet(proxy = OptionalNonEmptyStringProxy)]
    pub subscription_name: Option<SubscriptionName>,
    pub resource_id: ScopeImpl,
}
