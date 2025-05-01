use crate::naming::validate_policy_name;
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
#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

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
