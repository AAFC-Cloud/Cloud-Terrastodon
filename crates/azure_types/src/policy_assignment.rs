use crate::AzureLocationName;
use crate::AzurePolicyDefinitionParametersSupplied;
use crate::PolicyAssignmentId;
use crate::PolicyAssignmentName;
use crate::PolicyDefinitionIdReference;
use crate::PrincipalId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use compact_str::CompactString;
use facet::Facet;
use facet_json::RawJson;

#[derive(Debug, PartialEq, Eq, Facet)]
pub struct PolicyAssignment {
    pub id: PolicyAssignmentId,
    pub name: PolicyAssignmentName,
    pub location: AzureLocationName,
    pub identity: Option<RawJson<'static>>,
    pub properties: PolicyAssignmentProperties,
}

#[derive(Debug, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct PolicyAssignmentProperties {
    pub policy_definition_id: PolicyDefinitionIdReference,
    pub non_compliance_messages: Option<Vec<RawJson<'static>>>,
    pub definition_version: CompactString,
    pub resource_selectors: Option<Vec<RawJson<'static>>>,
    pub enforcement_mode: PolicyAssignmentEnforcementMode,
    pub display_name: Option<CompactString>,
    pub description: Option<CompactString>,
    pub parameters: Option<AzurePolicyDefinitionParametersSupplied>,
    pub not_scopes: Option<Vec<String>>,
    pub metadata: PolicyAssignmentMetadata,
    pub scope: ScopeImpl,
}

#[derive(Debug, PartialEq, Eq, Facet)]
#[repr(C)]
pub enum PolicyAssignmentEnforcementMode {
    Default,
    DoNotEnforce,
}

#[derive(Debug, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct PolicyAssignmentMetadata {
    pub created_on: DateTime<Utc>,
    pub created_by: PrincipalId,
    pub assigned_by: Option<CompactString>,
    pub parameter_scopes: Option<RawJson<'static>>,
    pub updated_by: Option<PrincipalId>,
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct PolicyAssignmentNonComplianceMessage {
    pub policy_definition_reference_id: CompactString,
    pub message: CompactString,
}

impl AsScope for PolicyAssignment {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &PolicyAssignment {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for PolicyAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        f.write_str(
            self.properties
                .display_name
                .as_deref()
                .unwrap_or(self.name.as_str()),
        )?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<PolicyAssignment> for HclImportBlock {
    fn from(policy_assignment: PolicyAssignment) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: policy_assignment.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::ManagementGroupPolicyAssignment,
                name: policy_assignment.id.expanded_form().sanitize(),
            },
        }
    }
}
