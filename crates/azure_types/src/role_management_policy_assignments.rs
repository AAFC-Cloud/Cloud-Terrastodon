use crate::RoleManagementPolicyAssignmentId;
use crate::iso8601_duration::IsoDuration;
use arbitrary::Arbitrary;
use eyre::Result;
use facet_json::RawJson;
use std::collections::HashMap;
use std::fmt;

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

impl<'a> Arbitrary<'a> for RoleManagementPolicyAssignment {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Build a coherent subscription-scoped example through the real ID
        // parser, rather than deriving arbitrary strings for validated IDs.
        let subscription = uuid::Uuid::arbitrary(u)?;
        let assignment = uuid::Uuid::arbitrary(u)?;
        let role = uuid::Uuid::arbitrary(u)?;
        let policy = uuid::Uuid::arbitrary(u)?;
        let scope = format!("/subscriptions/{subscription}");
        let name = assignment.to_string();
        let id = format!(
            "{scope}/providers/Microsoft.Authorization/roleManagementPolicyAssignments/{name}"
        )
        .parse()
        .map_err(|_| arbitrary::Error::IncorrectFormat)?;
        let mut effective_rules = Vec::new();
        if bool::arbitrary(u)? {
            let hours = u.int_in_range(1..=24u64)?;
            let rule = RoleManagementPolicyExpirationRule {
                rule_type: "RoleManagementPolicyExpirationRule".to_owned(),
                id: RoleManagementPolicyAssignmentPropertiesEffectiveRuleId::ExpirationEnduserAssignment,
                maximum_duration: std::time::Duration::from_secs(hours * 60 * 60).into(),
            };
            effective_rules.push(RawJson::from_owned(
                facet_json::to_string(&rule).map_err(|_| arbitrary::Error::IncorrectFormat)?,
            ));
        }
        let role_definition_id =
            format!("{scope}/providers/Microsoft.Authorization/roleDefinitions/{role}");
        let policy_id =
            format!("{scope}/providers/Microsoft.Authorization/roleManagementPolicies/{policy}");
        Ok(Self {
            properties: RoleManagementPolicyAssignmentProperties {
                scope,
                role_definition_id,
                policy_id,
                effective_rules,
                policy_assignment_properties: HashMap::new(),
            },
            name,
            id,
        })
    }
}

cloud_terrastodon_registry::register_arbitrary!(RoleManagementPolicyAssignment);
cloud_terrastodon_registry::register_arbitrary!(Vec<RoleManagementPolicyAssignment>);

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct RoleManagementPolicyExpirationRule {
    #[facet(rename = "ruleType")]
    rule_type: String,
    id: RoleManagementPolicyAssignmentPropertiesEffectiveRuleId,
    #[facet(rename = "maximumDuration")]
    maximum_duration: IsoDuration,
}

impl RoleManagementPolicyAssignment {
    pub fn get_maximum_activation_duration(&self) -> Option<IsoDuration> {
        for rule in &self.properties.effective_rules {
            if let Ok(rule) =
                facet_json::from_str::<RoleManagementPolicyExpirationRule>(rule.as_str())
                && rule.rule_type == "RoleManagementPolicyExpirationRule"
                && rule.id
                    == RoleManagementPolicyAssignmentPropertiesEffectiveRuleId::ExpirationEnduserAssignment
            {
                return Some(rule.maximum_duration);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scopes::Scope;
    use facet::Facet;
    use uuid::Uuid;

    #[test]
    fn registered_assignment_generator_can_produce_nonempty_coherent_json() -> Result<()> {
        let constructor = cloud_terrastodon_registry::functions_from_to(
            cloud_terrastodon_registry::ArbitraryBytes::SHAPE,
            Vec::<RoleManagementPolicyAssignment>::SHAPE,
        )
        .into_iter()
        .find(|function| {
            function
                .output_shape
                .is_shape(Vec::<RoleManagementPolicyAssignment>::SHAPE)
        })
        .expect("an exact vector generator must be registered");
        // Arbitrary's vector iterator consumes a leading continuation bool;
        // the remaining zeros generate one valid UUID-based assignment.
        let mut bytes = vec![0; 1024];
        bytes[0] = 1;
        let mut input = cloud_terrastodon_registry::ArbitraryBytes::new(bytes);
        let assignments = constructor
            .invoke_mut_boxed(&mut input)?
            .downcast::<Vec<RoleManagementPolicyAssignment>>()
            .expect("constructor output has the registered vector type");
        assert_eq!(
            assignments.len(),
            1,
            "the generator must not be an empty placeholder"
        );
        for assignment in assignments.iter() {
            Uuid::parse_str(&assignment.name)?;
            let expected_id = format!(
                "{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments/{}",
                assignment.properties.scope, assignment.name
            );
            assert_eq!(assignment.id.expanded_form(), expected_id);
            assert_eq!(
                expected_id.parse::<RoleManagementPolicyAssignmentId>()?,
                assignment.id
            );
            assert!(
                assignment
                    .properties
                    .role_definition_id
                    .starts_with(&assignment.properties.scope)
            );
            assert!(
                assignment
                    .properties
                    .policy_id
                    .starts_with(&assignment.properties.scope)
            );
        }
        let json = facet_json::to_string(assignments.as_ref())?;
        assert_eq!(
            facet_json::from_str::<Vec<RoleManagementPolicyAssignment>>(&json)?,
            *assignments
        );

        // Exercise nonempty owned raw JSON too, independently of vector length.
        let bytes = vec![1; 128];
        let assignment =
            RoleManagementPolicyAssignment::arbitrary(&mut arbitrary::Unstructured::new(&bytes))?;
        assert_eq!(assignment.properties.effective_rules.len(), 1);
        assert!(assignment.get_maximum_activation_duration().is_some());
        let json = facet_json::to_string(&assignment)?;
        assert_eq!(
            facet_json::from_str::<RoleManagementPolicyAssignment>(&json)?,
            assignment
        );
        Ok(())
    }
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
