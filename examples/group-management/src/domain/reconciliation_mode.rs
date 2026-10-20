/// Whether to ignore a relationship or reconcile all or only required entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(u8)]
pub enum ReconciliationMode {
    /// Leave the relationship unmanaged; its desired list must be empty.
    Ignore,
    Exact,
    Additive,
}
