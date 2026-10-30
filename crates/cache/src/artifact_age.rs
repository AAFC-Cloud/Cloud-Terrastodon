use crate::ArtifactFreshness;
use chrono::DateTime;
use chrono::Utc;
use std::time::Duration;
use std::time::Instant;

/// One immutable attempt's first monotonic correlation; disk hits never reset it.
#[derive(Debug)]
pub(crate) struct ArtifactAge {
    pub freshness: ArtifactFreshness,
    pub initial_age: Duration,
    pub anchored_at: Instant,
}

impl ArtifactAge {
    pub fn reusable(
        &self,
        freshness: &ArtifactFreshness,
        validity: Duration,
        wall: DateTime<Utc>,
        monotonic: Instant,
    ) -> bool {
        if &self.freshness != freshness || !freshness.is_reusable_at(wall, validity) {
            return false;
        }
        let Some(age) = self
            .initial_age
            .checked_add(monotonic.saturating_duration_since(self.anchored_at))
        else {
            return false;
        };
        let Ok(expiry) = freshness.effective_expires_at(validity) else {
            return false;
        };
        expiry.is_none_or(|expiry| {
            (expiry - freshness.observed_at)
                .to_std()
                .is_ok_and(|maximum| age < maximum)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wall_clock_rollback_and_repeated_reads_never_renew_monotonic_age() {
        let observed = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let freshness = ArtifactFreshness::new(observed, Duration::from_secs(60)).unwrap();
        let anchor = Instant::now();
        let age = ArtifactAge {
            freshness: freshness.clone(),
            initial_age: Duration::from_secs(20),
            anchored_at: anchor,
        };
        assert!(age.reusable(
            &freshness,
            Duration::MAX,
            observed + chrono::TimeDelta::seconds(30),
            anchor + Duration::from_secs(10)
        ));
        assert!(!age.reusable(
            &freshness,
            Duration::MAX,
            observed + chrono::TimeDelta::seconds(1),
            anchor + Duration::from_secs(40)
        ));
        assert!(!age.reusable(
            &freshness,
            Duration::from_secs(25),
            observed + chrono::TimeDelta::seconds(1),
            anchor + Duration::from_secs(10)
        ));
    }
}
