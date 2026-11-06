use facet::Facet;
/// Projection of the policy-state summary query used by the compliance picker.
#[derive(Clone, Facet)]
pub(crate) struct PolicyNoncomplianceReferenceRow {
    pub policy_definition_reference_id: String,
    pub resource_type: String,
    pub found: u32,
}
