use crate::naming::validate_policy_name;
use crate::prelude::PolicyDefinitionId;
use crate::prelude::PolicySetDefinitionId;
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
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

pub const POLICY_ASSIGNMENT_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/policyAssignments/";

#[derive(Debug, Clone)]
pub enum PolicyAssignmentId {
    Unscoped { expanded: String },
    SubscriptionScoped { expanded: String },
    ManagementGroupScoped { expanded: String },
    ResourceGroupScoped { expanded: String },
}

impl PartialEq for PolicyAssignmentId {
    fn eq(&self, other: &Self) -> bool {
        // Compare ignoring case
        self.expanded_form()
            .eq_ignore_ascii_case(other.expanded_form())
    }
}

impl Eq for PolicyAssignmentId {}

impl Hash for PolicyAssignmentId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Feed lowercase bytes into the Hasher to ensure case-insensitive hashing
        for byte in self.expanded_form().bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}
impl NameValidatable for PolicyAssignmentId {
    fn validate_name(name: &str) -> Result<()> {
        validate_policy_name(name)
    }
}
impl HasPrefix for PolicyAssignmentId {
    fn get_prefix() -> &'static str {
        POLICY_ASSIGNMENT_ID_PREFIX
    }
}
impl TryFromUnscoped for PolicyAssignmentId {
    unsafe fn new_unscoped_unchecked(expanded: &str) -> Self {
        PolicyAssignmentId::Unscoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromResourceGroupScoped for PolicyAssignmentId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        PolicyAssignmentId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromSubscriptionScoped for PolicyAssignmentId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        PolicyAssignmentId::SubscriptionScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl TryFromManagementGroupScoped for PolicyAssignmentId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        PolicyAssignmentId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for PolicyAssignmentId {
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
        ScopeImplKind::PolicyAssignment
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::PolicyAssignment(self.clone())
    }
}

impl Serialize for PolicyAssignmentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for PolicyAssignmentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = PolicyAssignmentId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

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

impl HasScope for PolicyAssignment {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &PolicyAssignment {
    fn scope(&self) -> &impl Scope {
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

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn unscoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::try_from_expanded(expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyAssignmentId::Unscoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "abc123");
        Ok(())
    }

    #[test]
    fn management_group_scoped() -> Result<()> {
        let expanded = "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyAssignments/abc123";
        let id = PolicyAssignmentId::try_from_expanded_management_group_scoped(expanded)?;
        assert_eq!(id, PolicyAssignmentId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyAssignmentId::ManagementGroupScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "abc123");
        Ok(())
    }

    #[test]
    fn subscription_scoped() -> Result<()> {
        let expanded = "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyAssignments/GC Audit ISO 27001:20133";
        let id = PolicyAssignmentId::try_from_expanded_subscription_scoped(expanded)?;
        assert_eq!(id, PolicyAssignmentId::try_from_expanded(expanded)?);
        assert_eq!(id.expanded_form(), expanded);
        assert_eq!(
            PolicyAssignmentId::SubscriptionScoped {
                expanded: expanded.to_string()
            },
            id
        );
        assert_eq!(id.short_form(), "GC Audit ISO 27001:20133");
        Ok(())
    }

    #[test]
    fn deserializes() -> Result<()> {
        for expanded in [
            "/providers/Microsoft.Authorization/policyAssignments/abc123",
            "/providers/Microsoft.Management/managementGroups/my-management-group/providers/Microsoft.Authorization/policyAssignments/abc123",
            "/subscriptions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/providers/Microsoft.Authorization/policyAssignments/GC Audit ISO 27001:20133",
        ] {
            let id: PolicyAssignmentId =
                serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
            assert_eq!(id.expanded_form(), expanded);
        }
        Ok(())
    }
}
