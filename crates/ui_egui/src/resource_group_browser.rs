use crate::resource_group_filters::ResourceGroupFilters;

/// Filters for the resource-group inventory.
#[derive(Default)]
pub(crate) struct ResourceGroupBrowser {
    pub filters: ResourceGroupFilters,
}
