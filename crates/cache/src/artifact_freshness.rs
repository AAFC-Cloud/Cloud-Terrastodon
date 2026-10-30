use crate::artifact_expiry::ArtifactExpiry;
use crate::artifact_freshness_wire::ArtifactFreshnessWire;
use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use std::time::Duration;

/// Persisted observation and expiry of one complete raw execution result.
///
/// Reading or decoding an artifact must not replace its observation timestamp.
/// At load, the owner converts the remaining lifetime to a monotonic deadline
/// once. Aggregate results additionally retain their child observations and
/// cannot use this record alone to establish dependency freshness.
///
/// The wire representation requires an explicit deadline or `Unbounded`
/// selection. A missing or null expiry is rejected rather than silently treated
/// as unbounded. Invalid recorded ordering is rejected during deserialization.
/// Both timestamps retain nanosecond precision through the wire representation.
#[derive(Clone, Debug, Eq, PartialEq, Facet)]
#[facet(proxy = ArtifactFreshnessWire)]
pub struct ArtifactFreshness {
    pub observed_at: DateTime<Utc>,
    /// `None` denotes explicitly unbounded validity, not a missing timestamp.
    pub expires_at: Option<DateTime<Utc>>,
}

impl TryFrom<ArtifactFreshnessWire> for ArtifactFreshness {
    type Error = Report;

    fn try_from(wire: ArtifactFreshnessWire) -> Result<Self> {
        let expires_at = match wire.expires_at {
            ArtifactExpiry::At(expiry) => Some(expiry),
            ArtifactExpiry::Unbounded => None,
        };
        ensure!(
            expires_at.is_none_or(|expiry| expiry >= wire.observed_at),
            "artifact expiry precedes its observation"
        );
        Ok(Self {
            observed_at: wire.observed_at,
            expires_at,
        })
    }
}

impl ArtifactFreshness {
    /// Record one observation with checked validity arithmetic.
    ///
    /// `Duration::MAX` is unbounded. Zero validity retains artifacts while
    /// always disabling their reuse. Other unrepresentable deadlines fail.
    pub fn new(observed_at: DateTime<Utc>, valid_for: Duration) -> Result<Self> {
        Ok(Self {
            observed_at,
            expires_at: deadline(observed_at, valid_for)?,
        })
    }

    /// Bound a caller's result without extending the stored artifact deadline.
    ///
    /// The caller's bound is measured from the original observation. This
    /// method does not mutate the record or another caller's certificate. An
    /// invalid recorded ordering or unrepresentable caller deadline fails;
    /// artifact readers treat these untrustworthy records as misses.
    pub fn effective_expires_at(&self, caller_validity: Duration) -> Result<Option<DateTime<Utc>>> {
        ensure!(
            self.expires_at
                .is_none_or(|expiry| expiry >= self.observed_at),
            "artifact expiry precedes its observation"
        );
        Ok(
            match (
                self.expires_at,
                deadline(self.observed_at, caller_validity)?,
            ) {
                (Some(recorded), Some(requested)) => Some(recorded.min(requested)),
                (Some(recorded), None) => Some(recorded),
                (None, requested) => requested,
            },
        )
    }

    /// Whether the recorded artifact is trustworthy and fresh at this instant.
    ///
    /// Future observations, malformed or expired deadlines, unrepresentable
    /// caller bounds, and zero validity are misses. This check never renews an
    /// artifact's age; the execution owner maintains monotonic age after load.
    pub fn is_reusable_at(&self, now: DateTime<Utc>, caller_validity: Duration) -> bool {
        if self.observed_at > now || caller_validity.is_zero() {
            return false;
        }
        match self.effective_expires_at(caller_validity) {
            Ok(Some(expires_at)) => expires_at > now,
            Ok(None) => true,
            Err(_) => false,
        }
    }
}

fn deadline(observed_at: DateTime<Utc>, valid_for: Duration) -> Result<Option<DateTime<Utc>>> {
    if valid_for == Duration::MAX {
        return Ok(None);
    }
    let validity = TimeDelta::from_std(valid_for)
        .wrap_err("artifact validity cannot be represented as a timestamp duration")?;
    observed_at
        .checked_add_signed(validity)
        .map(Some)
        .ok_or_else(|| eyre!("artifact validity exceeds the representable timestamp range"))
}

cloud_terrastodon_registry_values::register_thing!(ArtifactFreshness);

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::SecondsFormat;

    fn observation() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    #[test]
    fn narrower_callers_do_not_renew_or_extend_the_recorded_deadline() {
        let observed = observation();
        let freshness = ArtifactFreshness::new(observed, Duration::from_secs(60)).unwrap();
        assert_eq!(
            freshness
                .effective_expires_at(Duration::from_secs(10))
                .unwrap(),
            Some(observed + TimeDelta::seconds(10))
        );
        assert!(
            !freshness.is_reusable_at(observed + TimeDelta::seconds(10), Duration::from_secs(10))
        );
        for caller_validity in [Duration::from_secs(120), Duration::MAX] {
            assert_eq!(
                freshness.effective_expires_at(caller_validity).unwrap(),
                Some(observed + TimeDelta::seconds(60))
            );
            assert!(freshness.is_reusable_at(observed + TimeDelta::seconds(59), caller_validity));
            assert!(!freshness.is_reusable_at(observed + TimeDelta::seconds(60), caller_validity));
        }
        assert_eq!(freshness.observed_at, observed);
        assert_eq!(
            freshness.expires_at,
            Some(observed + TimeDelta::seconds(60))
        );
    }

    #[test]
    fn zero_validity_never_reuses_even_at_the_observation_instant() {
        let observed = observation();
        let zero = ArtifactFreshness::new(observed, Duration::ZERO).unwrap();
        assert_eq!(zero.expires_at, Some(observed));
        assert!(!zero.is_reusable_at(observed, Duration::MAX));
        let unbounded = ArtifactFreshness::new(observed, Duration::MAX).unwrap();
        assert!(!unbounded.is_reusable_at(observed, Duration::ZERO));
    }

    #[test]
    fn unbounded_records_are_explicit_and_still_accept_narrower_callers() {
        let observed = observation();
        let freshness = ArtifactFreshness::new(observed, Duration::MAX).unwrap();
        assert_eq!(freshness.expires_at, None);
        assert_eq!(freshness.effective_expires_at(Duration::MAX).unwrap(), None);
        assert!(freshness.is_reusable_at(observed + TimeDelta::days(10_000), Duration::MAX));
        assert_eq!(
            freshness
                .effective_expires_at(Duration::from_secs(30))
                .unwrap(),
            Some(observed + TimeDelta::seconds(30))
        );
        assert!(
            !freshness.is_reusable_at(observed + TimeDelta::seconds(30), Duration::from_secs(30))
        );
    }

    #[test]
    fn future_observations_and_malformed_ordering_are_not_reusable() {
        let observed = observation();
        let future = ArtifactFreshness::new(observed, Duration::MAX).unwrap();
        assert!(!future.is_reusable_at(observed - TimeDelta::nanoseconds(1), Duration::MAX));
        let malformed = ArtifactFreshness {
            observed_at: observed,
            expires_at: Some(observed - TimeDelta::seconds(1)),
        };
        assert!(malformed.effective_expires_at(Duration::MAX).is_err());
        assert!(!malformed.is_reusable_at(observed, Duration::MAX));
    }

    #[test]
    fn timestamp_overflow_is_rejected_instead_of_becoming_unbounded() {
        assert!(ArtifactFreshness::new(DateTime::<Utc>::MAX_UTC, Duration::from_secs(1)).is_err());
        let excessive = Duration::MAX - Duration::from_secs(1);
        assert!(ArtifactFreshness::new(observation(), excessive).is_err());
        let freshness = ArtifactFreshness::new(observation(), Duration::from_secs(60)).unwrap();
        assert!(freshness.effective_expires_at(excessive).is_err());
        assert!(!freshness.is_reusable_at(observation(), excessive));
        assert!(ArtifactFreshness::new(DateTime::<Utc>::MAX_UTC, Duration::MAX).is_ok());
    }

    #[test]
    fn decoding_requires_explicit_expiry_and_rejects_invalid_ordering() {
        use std::collections::BTreeMap;

        let observed = observation();
        // Build damaged cache records with typed timestamps; this exercises
        // freshness admission, not a service's JSON response schema.
        let missing_expiry = BTreeMap::from([("observed_at", observed)]);
        let encoded = facet_json::to_string(&missing_expiry).unwrap();
        assert!(facet_json::from_str::<ArtifactFreshness>(&encoded).is_err());

        let null_expiry = BTreeMap::from([("observed_at", Some(observed)), ("expires_at", None)]);
        let encoded = facet_json::to_string(&null_expiry).unwrap();
        assert!(facet_json::from_str::<ArtifactFreshness>(&encoded).is_err());

        let invalid_order = ArtifactFreshnessWire {
            observed_at: observed,
            expires_at: ArtifactExpiry::At(observed - TimeDelta::seconds(1)),
        };
        let encoded = facet_json::to_string(&invalid_order).unwrap();
        assert!(facet_json::from_str::<ArtifactFreshness>(&encoded).is_err());

        // Invalid timestamps cannot inhabit the typed wire record. Corrupt the
        // persisted representation to verify both field proxies reject them
        // before the outer freshness validator can receive the record.
        let valid = ArtifactFreshness::new(observed, Duration::from_secs(60)).unwrap();
        let encoded = facet_json::to_string(&valid).unwrap();
        for timestamp in [observed, valid.expires_at.unwrap()] {
            let invalid = encoded.replace(
                &timestamp.to_rfc3339_opts(SecondsFormat::Nanos, true),
                "invalid timestamp",
            );
            assert_ne!(invalid, encoded);
            assert!(facet_json::from_str::<ArtifactFreshnessWire>(&invalid).is_err());
            assert!(facet_json::from_str::<ArtifactFreshness>(&invalid).is_err());
        }

        let missing_observation = BTreeMap::from([("expires_at", ArtifactExpiry::Unbounded)]);
        let encoded = facet_json::to_string(&missing_observation).unwrap();
        assert!(facet_json::from_str::<ArtifactFreshness>(&encoded).is_err());

        for valid_for in [Duration::ZERO, Duration::from_secs(30), Duration::MAX] {
            let original = ArtifactFreshness::new(observed, valid_for).unwrap();
            let encoded = facet_json::to_string(&original).unwrap();
            let decoded: ArtifactFreshness = facet_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, original);
            assert_eq!(
                decoded.is_reusable_at(observed, Duration::MAX),
                !valid_for.is_zero()
            );
        }
    }

    #[test]
    fn persisted_freshness_preserves_subsecond_observations_and_deadlines() {
        for nanos in [1, 123_456_789, 999_999_999] {
            let observed = DateTime::from_timestamp(1_700_000_000, nanos).unwrap();
            for validity in [
                Duration::ZERO,
                Duration::from_nanos(1),
                Duration::from_secs(60),
                Duration::MAX,
            ] {
                let original = ArtifactFreshness::new(observed, validity).unwrap();
                let encoded = facet_json::to_string(&original).unwrap();
                let decoded: ArtifactFreshness = facet_json::from_str(&encoded).unwrap();
                assert_eq!(decoded, original);
                assert_eq!(
                    decoded.is_reusable_at(observed, validity),
                    !validity.is_zero()
                );
                if let Some(expiry) = original.expires_at {
                    assert!(!decoded.is_reusable_at(expiry, Duration::MAX));
                }
            }
        }
    }
}
