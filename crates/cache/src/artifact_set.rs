use crate::ArtifactFilePath;
use std::collections::BTreeMap;

/// Transport-neutral bytes retained together for one execution outcome.
/// The owning adapter interprets the payload and supplies redacted diagnostics.
#[derive(Debug)]
pub struct ArtifactSet {
    pub payload: Vec<u8>,
    pub diagnostics: BTreeMap<ArtifactFilePath, Vec<u8>>,
}
