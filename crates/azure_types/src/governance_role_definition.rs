use compact_str::CompactString;
use facet::Facet;

#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct GovernanceRoleDefinition {
    pub display_name: CompactString,
    pub external_id: String,
    pub resource_id: String,
    pub template_id: String,
}
