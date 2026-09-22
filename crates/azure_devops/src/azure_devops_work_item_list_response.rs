use facet::Facet;

#[derive(Debug, Facet)]
pub(crate) struct WorkItemListResponse<T> {
    pub value: Vec<T>,
}
