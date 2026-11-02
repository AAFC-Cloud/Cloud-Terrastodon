use crate::AzureResourceProviderNamespace;
use crate::SubscriptionId;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// The tenant- or subscription-scoped ID in a resource provider response.
///
/// [Tenant provider API](https://learn.microsoft.com/en-us/rest/api/resources/providers/get-at-tenant-scope?view=rest-resources-2021-04-01)
/// and [subscription provider API](https://learn.microsoft.com/en-us/rest/api/resources/providers/get?view=rest-resources-2021-04-01).
/// A tenant-scoped ID has no embedded tenant UUID; the executing request supplies
/// its authentication tenant. These are provider identities, not deployed resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureResourceProviderId {
    pub subscription_id: Option<SubscriptionId>,
    pub namespace: AzureResourceProviderNamespace,
}

impl Display for AzureResourceProviderId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(subscription_id) = self.subscription_id {
            write!(f, "/subscriptions/{subscription_id}")?;
        }
        write!(f, "/providers/{}", self.namespace)
    }
}

impl FromStr for AzureResourceProviderId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let segments = value.split('/').collect::<Vec<_>>();
        match segments.as_slice() {
            ["", providers, namespace] if providers.eq_ignore_ascii_case("providers") => Ok(Self {
                subscription_id: None,
                namespace: namespace
                    .parse()
                    .wrap_err("invalid resource provider namespace")?,
            }),
            ["", subscriptions, subscription_id, providers, namespace]
                if subscriptions.eq_ignore_ascii_case("subscriptions")
                    && providers.eq_ignore_ascii_case("providers") =>
            {
                Ok(Self {
                    subscription_id: Some(
                        subscription_id
                            .parse()
                            .wrap_err("invalid resource provider subscription ID")?,
                    ),
                    namespace: namespace
                        .parse()
                        .wrap_err("invalid resource provider namespace")?,
                })
            }
            _ => {
                bail!("expected a tenant- or subscription-scoped resource provider ID: {value:?}");
            }
        }
    }
}

crate::impl_facet_string_proxy!(AzureResourceProviderId, value => value.to_string());

cloud_terrastodon_registry::register_thing!(AzureResourceProviderId);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderId);

#[cfg(test)]
mod tests {
    use super::AzureResourceProviderId;

    #[test]
    fn preserves_provider_scope_and_rejects_resource_routes() -> eyre::Result<()> {
        let tenant: AzureResourceProviderId = "/providers/Example.Provider".parse()?;
        assert_eq!(tenant.subscription_id, None);
        assert_eq!(tenant.namespace.as_str(), "Example.Provider");
        assert_eq!(tenant.to_string(), "/providers/Example.Provider");

        let subscription: AzureResourceProviderId =
            "/subscriptions/00000000-0000-0000-0000-000000000001/providers/Example.Provider"
                .parse()?;
        assert!(subscription.subscription_id.is_some());
        assert_eq!(
            subscription.to_string(),
            "/subscriptions/00000000-0000-0000-0000-000000000001/providers/Example.Provider"
        );
        for invalid in [
            "/providers/",
            "Example.Provider",
            "/subscriptions/invalid/providers/Example.Provider",
            "/providers/Example.Provider/widgets/example",
        ] {
            assert!(invalid.parse::<AzureResourceProviderId>().is_err());
        }
        Ok(())
    }
}
