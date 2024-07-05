use crate::resource_name_rules::validate_policy_name;
use crate::scopes::try_from_expanded_hierarchy_scoped;
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
use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::HashMap;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureRMResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderReference;
use tofu_types::prelude::TofuResourceReference;

pub const POLICY_DEFINITION_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyDefinitions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PolicyDefinitionId {
    Unscoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
}
impl NameValidatable for PolicyDefinitionId {
    fn validate_name(name: &str) -> Result<()> {
        validate_policy_name(name)
    }
}
impl HasPrefix for PolicyDefinitionId {
    fn get_prefix() -> &'static str {
        POLICY_DEFINITION_ID_PREFIX
    }
}
impl TryFromUnscoped for PolicyDefinitionId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        PolicyDefinitionId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for PolicyDefinitionId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        PolicyDefinitionId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromSubscriptionScoped for PolicyDefinitionId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        PolicyDefinitionId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl TryFromManagementGroupScoped for PolicyDefinitionId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        PolicyDefinitionId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for PolicyDefinitionId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        try_from_expanded_hierarchy_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::Unscoped { expanded } => expanded,
            Self::ResourceGroupScoped { expanded } => expanded,
            Self::SubscriptionScoped { expanded } => expanded,
            Self::ManagementGroupScoped { expanded } => expanded,
        }
    }

    fn short_form(&self) -> &str {
        self.expanded_form()
            .rsplit_once('/')
            .expect("no slash found, structure should have been validated at construction")
            .1
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::PolicyDefinition
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicyDefinition(self.clone())
    }
}

impl Serialize for PolicyDefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyDefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyDefinitionId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub id: PolicyDefinitionId,
    pub metadata: HashMap<String, Value>,
    pub mode: String,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyRule")]
    pub policy_rule: serde_json::Value,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
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
        f.write_str(&self.name)?;
        f.write_str(" (")?;
        f.write_fmt(format_args!("{:?}", self.display_name))?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<PolicyDefinition> for TofuImportBlock {
    fn from(policy_definition: PolicyDefinition) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: policy_definition.id.expanded_form().to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::PolicyDefinition,
                name: policy_definition.name.sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyDefinitionId::Unscoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyDefinitionId::ManagementGroupScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555";
        let id = PolicyDefinitionId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyDefinitionId::SubscriptionScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "55555555-5555-5555-5555-555555555555");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
            "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
            "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyDefinitions/55555555-5555-5555-5555-555555555555",
        ] {
            let id: PolicyDefinitionId =
                serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
