use cloud_terrastodon_azure::ManagementGroup;
use facet::Facet;
use std::sync::Arc;

/// The management group and its compact labels in the tenant inventory.
#[derive(Facet)]
pub(crate) struct ManagementGroupInventoryRow {
    pub group: Arc<ManagementGroup>,
    pub group_name: String,
    pub ancestors: String,
}

impl From<&Arc<ManagementGroup>> for ManagementGroupInventoryRow {
    fn from(group: &Arc<ManagementGroup>) -> Self {
        Self {
            group: Arc::clone(group),
            group_name: group.name().to_owned(),
            ancestors: group
                .management_group_ancestors_chain
                .iter()
                .map(|ancestor| ancestor.display_name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}
