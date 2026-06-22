use crate::RoleManagementPolicyAssignmentId;
use eyre::Result;
use facet_json::RawJson;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

#[derive(Debug, PartialEq, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind {
    ManagementGroup,
    Subscription,
    ResourceGroup,
    Other(String),
}
crate::impl_facet_string_proxy!(
    RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind,
    value => value.to_string()
);

impl std::fmt::Display
    for RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ManagementGroup => {
                f.write_str("managementgroup")
            }
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Subscription => {
                f.write_str("subscription")
            }
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ResourceGroup => {
                f.write_str("resourcegroup")
            }
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Other(s) => {
                f.write_str(s)
            }
        }
    }
}

impl std::str::FromStr
    for RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind
{
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "managementgroup" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ManagementGroup,
            "subscription" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Subscription,
            "resourcegroup" => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ResourceGroup,
            other => RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Other(other.to_string()),
        })
    }
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum RoleManagementPolicyAssignmentPropertiesEffectiveRuleId {
    #[facet(rename = "Enablement_Admin_Eligibility")]
    EnablementAdminEligibility,
    #[facet(rename = "Expiration_Admin_Eligibility")]
    ExpirationAdminEligibility,
    #[facet(rename = "Notification_Admin_Admin_Eligibility")]
    NotificationAdminAdminEligibility,
    #[facet(rename = "Notification_Requestor_Admin_Eligibility")]
    NotificationRequestorAdminEligibility,
    #[facet(rename = "Notification_Approver_Admin_Eligibility")]
    NotificationApproverAdminEligibility,
    #[facet(rename = "Enablement_Admin_Assignment")]
    EnablementAdminAssignment,
    #[facet(rename = "Expiration_Admin_Assignment")]
    ExpirationAdminAssignment,
    #[facet(rename = "Notification_Admin_Admin_Assignment")]
    NotificationAdminAdminAssignment,
    #[facet(rename = "Notification_Requestor_Admin_Assignment")]
    NotificationRequestorAdminAssignment,
    #[facet(rename = "Notification_Approver_Admin_Assignment")]
    NotificationApproverAdminAssignment,
    #[facet(rename = "Approval_EndUser_Assignment")]
    ApprovalEnduserAssignment,
    #[facet(rename = "AuthenticationContext_EndUser_Assignment")]
    AuthenticationcontextEnduserAssignment,
    #[facet(rename = "Enablement_EndUser_Assignment")]
    EnablementEnduserAssignment,
    #[facet(rename = "Expiration_EndUser_Assignment")]
    ExpirationEnduserAssignment,
    #[facet(rename = "Notification_Admin_EndUser_Assignment")]
    NotificationAdminEnduserAssignment,
    #[facet(rename = "Notification_Requestor_EndUser_Assignment")]
    NotificationRequestorEnduserAssignment,
    #[facet(rename = "Notification_Approver_EndUser_Assignment")]
    NotificationApproverEnduserAssignment,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleManagementPolicyAssignmentProperties {
    pub scope: String,
    #[facet(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    #[facet(rename = "policyId")]
    pub policy_id: String,
    #[facet(rename = "effectiveRules")]
    pub effective_rules: Vec<RawJson<'static>>,
    #[facet(
        rename = "policyAssignmentProperties",
        opaque,
        proxy = crate::HashMapDefaultNullProxy<RawJson<'static>>
    )]
    pub policy_assignment_properties: HashMap<String, RawJson<'static>>,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleManagementPolicyAssignment {
    pub properties: RoleManagementPolicyAssignmentProperties,
    pub name: String,
    pub id: RoleManagementPolicyAssignmentId,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct RoleManagementPolicyExpirationRule {
    #[facet(rename = "ruleType")]
    rule_type: String,
    id: RoleManagementPolicyAssignmentPropertiesEffectiveRuleId,
    #[facet(rename = "maximumDuration", opaque, proxy = crate::IsoDurationProxy)]
    maximum_duration: iso8601_duration::Duration,
}

impl RoleManagementPolicyAssignment {
    pub fn get_maximum_activation_duration(&self) -> Option<Duration> {
        for rule in &self.properties.effective_rules {
            if let Ok(rule) =
                facet_json::from_str::<RoleManagementPolicyExpirationRule>(rule.as_str())
            {
                if rule.rule_type == "RoleManagementPolicyExpirationRule"
                    && rule.id
                        == RoleManagementPolicyAssignmentPropertiesEffectiveRuleId::ExpirationEnduserAssignment
                {
                    return rule.maximum_duration.to_std();
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scopes::Scope;
    use uuid::Uuid;
    #[test]
    fn it_works() -> Result<()> {
        let id = format!(
            "/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments/{}_{}",
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
        );
        RoleManagementPolicyAssignmentId::try_from_expanded(&id)?;
        let scope_kind = facet_json::from_str::<
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind,
        >("\"managementgroup\"")?;
        assert_eq!(
            scope_kind,
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::ManagementGroup
        );
        assert_eq!(facet_json::to_string(&scope_kind)?, "\"managementgroup\"");

        let scope_kind = facet_json::from_str::<
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind,
        >("\"new-scope-kind\"")?;
        assert_eq!(
            scope_kind,
            RoleManagementPolicyAssignmentPropertiesPolicyAssignmentPropertiesScopeKind::Other(
                "new-scope-kind".to_string()
            )
        );
        assert_eq!(facet_json::to_string(&scope_kind)?, "\"new-scope-kind\"");
        Ok(())
    }
}
