use crate::prelude::PolicyAssignmentId;
use crate::prelude::PolicyAssignmentName;
use crate::prelude::PolicyDefinitionIdReference;
use crate::prelude::PrincipalId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::serde_helpers::deserialize_default_if_null;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use compact_str::CompactString;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyAssignment {
    pub id: PolicyAssignmentId,
    pub name: PolicyAssignmentName,
    pub location: CompactString,
    pub identity: Option<Value>,
    pub properties: PolicyAssignmentProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyAssignmentProperties {
    pub policy_definition_id: PolicyDefinitionIdReference,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub non_compliance_messages: Vec<Value>,
    pub definition_version: CompactString,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub resource_selectors: Vec<Value>,
    pub enforcement_mode: PolicyAssignmentEnforcementMode,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub display_name: CompactString,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub description: CompactString,
    pub parameters: Option<Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    pub not_scopes: Vec<String>,
    pub metadata: PolicyAssignmentMetadata,
    pub scope: ScopeImpl,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyAssignmentEnforcementMode {
    Default,
    DoNotEnforce,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyAssignmentMetadata {
    pub created_on: DateTime<Utc>,
    pub created_by: PrincipalId,
    pub assigned_by: Option<CompactString>,
    pub parameter_scopes: Option<Value>,
    pub updated_by: Option<PrincipalId>,
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
        f.write_fmt(format_args!("{:?}", &self.properties.display_name))?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<PolicyAssignment> for HCLImportBlock {
    fn from(policy_assignment: PolicyAssignment) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: policy_assignment.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::ManagementGroupPolicyAssignment,
                name: policy_assignment.id.expanded_form().sanitize(),
            },
        }
    }
}
