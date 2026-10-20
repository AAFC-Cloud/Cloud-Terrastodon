use crate::planning::GroupPlan;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon::azure_types::AzureTenantId;

mod save;
mod try_new;

/// A deterministic, reviewable plan for achieving a desire document's relationships.
///
/// This first milestone only emits plans. It is not an executable apply format;
/// a future apply command must reauthenticate and verify fresh observed state.
/// Schema version 2 represents warnings and blockers as tagged, typed records.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct DesireDocumentAchievementPlan {
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub source_digest: String,
    pub selected_tenant_id: Option<AzureTenantId>,
    pub document_tenant_ids: Vec<AzureTenantId>,
    pub excluded_tenant_ids: Vec<AzureTenantId>,
    pub groups: Vec<GroupPlan>,
}

impl DesireDocumentAchievementPlan {
    pub fn has_blockers(&self) -> bool {
        self.groups.iter().any(|group| !group.blockers.is_empty())
    }

    pub fn change_count(&self) -> usize {
        self.groups
            .iter()
            .flat_map(|group| [&group.members, &group.owners])
            .flatten()
            .map(|relationship| relationship.add.len() + relationship.remove.len())
            .sum()
    }
}
