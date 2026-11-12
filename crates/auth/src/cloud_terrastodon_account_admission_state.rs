use cloud_terrastodon_cache::AuthPartition;
use std::collections::HashSet;

/// Admission and lease counters change under one lock to prevent retirement races.
#[derive(Debug)]
pub(crate) struct CloudTerrastodonAccountAdmissionState {
    pub(crate) open: bool,
    pub(crate) active: usize,
    pub(crate) partitions: HashSet<AuthPartition>,
}
