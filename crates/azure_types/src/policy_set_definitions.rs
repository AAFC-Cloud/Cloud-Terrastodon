use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicySetDefinitionId;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinitionPolicyDefinitionGroup {
    #[serde(rename = "additionalMetadataId")]
    pub additional_metadata_id: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinitionPolicyDefinition {
    #[serde(rename = "groupNames")]
    pub group_names: Option<Vec<String>>,
    pub parameters: Value,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: PolicyDefinitionId,
    #[serde(rename = "policyDefinitionReferenceId")]
    pub policy_definition_reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicySetDefinition {
    pub id: PolicySetDefinitionId,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, Value>>,
    pub policy_definitions: Option<Vec<PolicySetDefinitionPolicyDefinition>>,
    pub policy_definition_groups: Option<Vec<PolicySetDefinitionPolicyDefinitionGroup>>,
    pub policy_type: String,
    pub version: String,
}

impl HasScope for PolicySetDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &PolicySetDefinition {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for PolicySetDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        f.write_str(&self.name)?;
        if let Some(display_name) = self.display_name.as_deref() {
            f.write_str(" (")?;
            f.write_str(display_name)?;
            f.write_str(")")?;
        }
        Ok(())
    }
}
impl From<PolicySetDefinition> for HCLImportBlock {
    fn from(policy_definition: PolicySetDefinition) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: policy_definition.id.expanded_form().to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::PolicySetDefinition,
                name: policy_definition.name.sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scopes::{TryFromManagementGroupScoped, TryFromSubscriptionScoped};

    use super::*;
    use eyre::{bail, Result};

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicySetDefinitionId::Unscoped(_) => {},
            x => bail!("Bad: {x:?}")
        }
        assert_eq!(id.short_form(), "my-policy-set-name");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicySetDefinitionId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicySetDefinitionId::ManagementGroupScoped(_) => {},
            x => bail!("Bad: {x:?}")
        }
        assert_eq!(id.short_form(), "my-policy-set-name");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded_subscription_scoped(expanded)?;
        assert_eq!(id, PolicySetDefinitionId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        match &id {
            PolicySetDefinitionId::SubscriptionScoped(_) => {},
            x => bail!("Bad: {x:?}")
        }
        assert_eq!(id.short_form(), "my-policy-set-name");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name",
            "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name",
            "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name",
        ] {
            let id: PolicySetDefinitionId =
                serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
