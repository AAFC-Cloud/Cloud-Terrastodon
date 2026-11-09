/// How this invocation obtained access to one canonical cache entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkCacheStatus {
    Hit,
    Waiting,
    Initializing,
}
