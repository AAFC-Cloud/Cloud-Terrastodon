use crate::prelude::PolicyAssignmentId;
use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicySetDefinitionId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyAssignment {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "enforcementMode")]
    pub enforcement_mode: String,
    pub id: PolicyAssignmentId,
    pub identity: Option<HashMap<String, Value>>,
    pub location: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    #[serde(rename = "nonComplianceMessages")]
    pub non_compliance_messages: Option<Value>,
    #[serde(rename = "notScopes")]
    pub not_scopes: Option<Vec<String>>,
    pub parameters: Option<Value>,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    pub scope: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
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
        f.write_fmt(format_args!("{:?}", &self.display_name))?;
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
                name: policy_assignment.name.sanitize(),
            },
        }
    }
}

pub enum SomePolicyDefinitionId {
    PolicyDefinitionId(PolicyDefinitionId),
    PolicySetDefinitionId(PolicySetDefinitionId),
}
impl PolicyAssignment {
    pub fn policy_definition_id(&self) -> Result<SomePolicyDefinitionId> {
        match (
            PolicySetDefinitionId::try_from_expanded(&self.policy_definition_id),
            PolicyDefinitionId::try_from_expanded(&self.policy_definition_id),
        ) {
            (Ok(a), Ok(b)) => {
                bail!(
                    "Matched both types of policy definition id, this shouldnt happen. Got {} and {}",
                    a.expanded_form(),
                    b.expanded_form()
                );
            }
            (Ok(a), Err(_)) => Ok(SomePolicyDefinitionId::PolicySetDefinitionId(a)),
            (Err(_), Ok(b)) => Ok(SomePolicyDefinitionId::PolicyDefinitionId(b)),
            (Err(a), Err(b)) => {
                bail!("Failed to determine policy definition id kind. a={a}, b={b}")
            }
        }
    }
}
