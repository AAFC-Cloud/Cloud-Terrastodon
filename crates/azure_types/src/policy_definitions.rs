use crate::prelude::PolicyDefinitionId;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub id: PolicyDefinitionId,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub mode: String,
    pub parameters: Option<HashMap<String, Value>>,
    pub policy_rule: serde_json::Value,
    pub policy_type: String,
    pub version: String,
}

impl HasScope for PolicyDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &PolicyDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("policy definition ")?;
        f.write_str(&self.name)?;
        if let Some(display_name) = self.display_name.as_deref() {
            f.write_str(" (")?;
            f.write_str(display_name)?;
            f.write_str(")")?;
        }
        Ok(())
    }
}
impl From<PolicyDefinition> for HCLImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: policy_definition.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::PolicyDefinition,
                name: policy_definition.name.sanitize(),
            },
        }
    }
}
