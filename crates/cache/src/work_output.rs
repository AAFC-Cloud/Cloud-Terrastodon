use crate::ArtifactObservation;
use crate::work_freshness::WorkFreshness;

/// A canonical decoded outcome and the time its underlying result was observed.
/// Capture observation before decoding; cache insertion must not renew its age.
///
/// Use `new` for fresh execution or `LoadedArtifact::decoded` for disk reuse.
/// The private freshness proof prevents replacing a loaded artifact's original
/// age or generations when decoding finishes.
#[derive(Debug)]
pub struct WorkOutput<T> {
    pub value: T,
    pub(crate) freshness: WorkFreshness,
}

impl<T> WorkOutput<T> {
    pub fn new(value: T, observation: ArtifactObservation) -> Self {
        Self {
            value,
            freshness: WorkFreshness::Observed(observation),
        }
    }
}
