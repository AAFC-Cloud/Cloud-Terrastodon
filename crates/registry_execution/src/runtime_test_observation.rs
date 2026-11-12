use crate::RuntimeTestOutputCoverage;
use cloud_terrastodon_cache::WorkCacheStatus;
use cloud_terrastodon_execution::ExecutionEvent;
use facet::Facet;

/// Evidence about cache use during verification, without retaining service data.
/// An initializer may be a composition or a disk decode, not a network exchange.
#[derive(Debug, Default, Facet)]
pub struct RuntimeTestObservation {
    pub cache_hits: usize,
    pub cache_initializations: usize,
    pub cache_waits: usize,
    pub output_coverage: RuntimeTestOutputCoverage,
}

impl RuntimeTestObservation {
    pub fn record(&mut self, event: ExecutionEvent) {
        if let ExecutionEvent::Cache { status, .. } = event {
            match status {
                WorkCacheStatus::Hit => self.cache_hits += 1,
                WorkCacheStatus::Waiting => self.cache_waits += 1,
                WorkCacheStatus::Initializing => self.cache_initializations += 1,
            }
        }
    }
}
