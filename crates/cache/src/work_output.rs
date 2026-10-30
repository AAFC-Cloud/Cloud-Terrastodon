use crate::ArtifactObservation;

/// A canonical decoded outcome and the time its underlying result was observed.
/// Capture observation before decoding; cache insertion must not renew its age.
///
/// This represents a fresh execution, not a decoded disk-cache hit. Disk adapters
/// must preserve the admitted artifact's existing deadline and monotonic age;
/// importing that complete freshness certificate belongs to the later shared
/// transport lifecycle, rather than stamping loaded bytes with a new observation.
#[derive(Debug)]
pub struct WorkOutput<T> {
    pub value: T,
    pub observation: ArtifactObservation,
}
