/// Original root, ancestor, entry, and authentication generations for one attempt.
///
/// Every ancestor is represented, including absent generation-zero records.
/// Stamps are meaningful only with their owning cache and concrete identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CacheGenerationStamp {
    pub root: u64,
    pub ancestors: Vec<u64>,
    pub exact: u64,
    pub authentication: u64,
}
