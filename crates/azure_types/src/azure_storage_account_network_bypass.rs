use crate::AzureStorageAccountNetworkBypassService;
use arbitrary::Arbitrary;
use eyre::ensure;
use facet::Facet;
use std::collections::BTreeSet;

/// Storage firewall bypass categories; the empty set represents `None`.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#bypass).
/// The wire representation is one comma-separated string, not a JSON array.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct AzureStorageAccountNetworkBypass(pub BTreeSet<AzureStorageAccountNetworkBypassService>);

impl TryFrom<String> for AzureStorageAccountNetworkBypass {
    type Error = eyre::Report;

    fn try_from(value: String) -> eyre::Result<Self> {
        if value == "None" {
            return Ok(Self(BTreeSet::new()));
        }
        let mut services = BTreeSet::new();
        for service in value.split(',').map(str::trim) {
            ensure!(
                !service.is_empty() && service != "None",
                "storage firewall bypass must be None or a nonempty combination of service categories"
            );
            services.insert(AzureStorageAccountNetworkBypassService::from(
                service.to_owned(),
            ));
        }
        Ok(Self(services))
    }
}

impl From<&AzureStorageAccountNetworkBypass> for String {
    fn from(value: &AzureStorageAccountNetworkBypass) -> Self {
        if value.0.is_empty() {
            "None".to_owned()
        } else {
            value
                .0
                .iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountNetworkBypass);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountNetworkBypass);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountNetworkBypass;
    use crate::AzureStorageAccountNetworkBypassService;

    #[test]
    fn accepts_service_combinations_without_treating_none_as_a_service() -> eyre::Result<()> {
        let value =
            AzureStorageAccountNetworkBypass::try_from("Metrics, Logging, Metrics".to_owned())?;
        assert_eq!(value.0.len(), 2);
        assert!(
            value
                .0
                .contains(&AzureStorageAccountNetworkBypassService::Logging)
        );
        assert!(
            AzureStorageAccountNetworkBypass::try_from("None".to_owned())?
                .0
                .is_empty()
        );
        for invalid in ["", "None, Logging", "Logging,"] {
            assert!(AzureStorageAccountNetworkBypass::try_from(invalid.to_owned()).is_err());
        }
        Ok(())
    }
}
