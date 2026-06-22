use compact_str::CompactString;

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct GovernanceRoleDefinition {
    pub display_name: CompactString,
    pub external_id: String,
    pub resource_id: String,
    pub template_id: String,
}
