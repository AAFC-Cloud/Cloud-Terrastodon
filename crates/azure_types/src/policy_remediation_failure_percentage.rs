use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::ensure;
use facet::Facet;

/// A remediation failure threshold represented as a fraction from 0 to 1.
///
/// The [REST percentage property](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01#remediationpropertiesfailurethreshold)
/// uses a fraction, although human-facing tools often describe it as 0–100%.
#[derive(Debug, Clone, Copy, PartialEq, Facet)]
#[facet(proxy = f32)]
pub struct PolicyRemediationFailurePercentage {
    // Private so every value remains finite and inside the service's interval.
    fraction: f32,
}

impl PolicyRemediationFailurePercentage {
    pub fn new(fraction: f32) -> eyre::Result<Self> {
        ensure!(
            (0.0..=1.0).contains(&fraction),
            "remediation failure percentage must be a finite fraction from 0 to 1"
        );
        Ok(Self { fraction })
    }

    pub fn fraction(self) -> f32 {
        self.fraction
    }
}

impl TryFrom<f32> for PolicyRemediationFailurePercentage {
    type Error = eyre::Report;

    fn try_from(value: f32) -> eyre::Result<Self> {
        Self::new(value)
    }
}

impl From<&PolicyRemediationFailurePercentage> for f32 {
    fn from(value: &PolicyRemediationFailurePercentage) -> Self {
        value.fraction
    }
}

impl<'a> Arbitrary<'a> for PolicyRemediationFailurePercentage {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(f32::from(u.arbitrary::<u16>()?) / f32::from(u16::MAX))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationFailurePercentage);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationFailurePercentage);

#[cfg(test)]
mod tests {
    use super::PolicyRemediationFailurePercentage;

    #[test]
    fn fraction_validation_rejects_percent_units_and_nonfinite_values() -> eyre::Result<()> {
        for invalid in [
            -0.01,
            1.01,
            100.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert!(PolicyRemediationFailurePercentage::new(invalid).is_err());
        }
        for valid in [0.0, 0.5, 1.0] {
            assert_eq!(
                PolicyRemediationFailurePercentage::new(valid)?.fraction(),
                valid
            );
        }
        assert!(facet_json::from_str::<PolicyRemediationFailurePercentage>("100.0").is_err());
        Ok(())
    }
}
