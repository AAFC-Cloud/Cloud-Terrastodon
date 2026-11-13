use arbitrary::Arbitrary;
use eyre::ensure;
use facet::Facet;

/// Account immutability retention period, between one and 146,000 days.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#accountimmutabilitypolicyproperties).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = i32)]
pub struct AzureStorageAccountImmutabilityRetentionDays(i32);

impl AzureStorageAccountImmutabilityRetentionDays {
    pub fn new(days: i32) -> eyre::Result<Self> {
        ensure!(
            (1..=146_000).contains(&days),
            "storage account immutability retention must be between 1 and 146000 days"
        );
        Ok(Self(days))
    }

    pub fn days(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for AzureStorageAccountImmutabilityRetentionDays {
    type Error = eyre::Report;

    fn try_from(value: i32) -> eyre::Result<Self> {
        Self::new(value)
    }
}

impl From<&AzureStorageAccountImmutabilityRetentionDays> for i32 {
    fn from(value: &AzureStorageAccountImmutabilityRetentionDays) -> Self {
        value.0
    }
}

impl<'a> Arbitrary<'a> for AzureStorageAccountImmutabilityRetentionDays {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(input.int_in_range(1..=146_000)?))
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountImmutabilityRetentionDays);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountImmutabilityRetentionDays);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountImmutabilityRetentionDays;

    #[test]
    fn retention_limits_apply_to_construction_and_wire_values() -> eyre::Result<()> {
        for days in [1, 146_000] {
            assert_eq!(
                AzureStorageAccountImmutabilityRetentionDays::new(days)?.days(),
                days
            );
            assert_eq!(
                facet_json::from_str::<AzureStorageAccountImmutabilityRetentionDays>(
                    &days.to_string()
                )?
                .days(),
                days
            );
        }
        for days in [i32::MIN, 0, 146_001, i32::MAX] {
            assert!(AzureStorageAccountImmutabilityRetentionDays::new(days).is_err());
            assert!(
                facet_json::from_str::<AzureStorageAccountImmutabilityRetentionDays>(
                    &days.to_string()
                )
                .is_err()
            );
        }
        Ok(())
    }
}
