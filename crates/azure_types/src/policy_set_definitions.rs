use crate::naming::validate_policy_name;
use crate::prelude::PolicyDefinitionId;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use crate::scopes::TryFromResourceGroupScoped;
use crate::scopes::TryFromSubscriptionScoped;
use crate::scopes::TryFromUnscoped;
use crate::scopes::try_from_expanded_resource_container_scoped;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use serde_json::Value;
use std::collections::HashMap;

pub const POLICY_SET_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policySetDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicySetDefinitionId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
}
impl NameValidatable for PolicySetDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        validate_policy_name(name)
    }
}
impl HasPrefix for PolicySetDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_SET_DEFINITION_ID_PREFIX
    }
}
impl TryFromUnscoped for PolicySetDefinitionId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        PolicySetDefinitionId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for PolicySetDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        PolicySetDefinitionId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromSubscriptionScoped for PolicySetDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        PolicySetDefinitionId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromManagementGroupScoped for PolicySetDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        PolicySetDefinitionId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for PolicySetDefinitionId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_resource_container_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ResourceGroupScoped { expanded } => expanded,
            Self::SubscriptionScoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicySetDefinition
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicySetDefinition(self.clone())
    }
}

impl Serialize for PolicySetDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicySetDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicySetDefinitionId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

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
    use super::*;
    use eyre::Result;

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicySetDefinitionId::Unscoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "my-policy-set-name");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicySetDefinitionId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicySetDefinitionId::ManagementGroupScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "my-policy-set-name");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policySetDefinitions/my-policy-set-name";
        let id = PolicySetDefinitionId::try_from_expanded_subscription_scoped(expanded)?;
        assert_eq!(id, PolicySetDefinitionId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicySetDefinitionId::SubscriptionScoped {
                expanded: expanded.to_string()
            },
            id
        );
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
