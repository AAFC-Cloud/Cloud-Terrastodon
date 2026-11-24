use crate::pim_activation_assignment::PimActivationAssignment;
use crate::pim_assignment_status::PimAssignmentStatus;
use crate::pim_entra_role_assignment::PimEntraRoleAssignment;
use crate::portal_tab_target::PortalTabTarget;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::GovernanceRoleAssignmentMemberType;
use cloud_terrastodon_azure::GovernanceRoleAssignmentState;
use cloud_terrastodon_azure::RoleEligibilitySchedule;
use cloud_terrastodon_azure::RoleEligibilityScheduleMemberType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;

/// A role inventory row with a stable service identity and an index back to its assignment.
#[derive(Facet)]
pub(crate) struct PimRoleRow {
    pub id: String,
    pub index: usize,
    pub role: String,
    pub scope: String,
    pub scope_target: Option<PortalTabTarget>,
    pub scope_type: String,
    pub membership: &'static str,
    pub end: Option<DateTime<Utc>>,
    pub condition: Option<String>,
    pub action: &'static str,
}

impl PimRoleRow {
    pub fn entra_status(item: &PimEntraRoleAssignment, now: DateTime<Utc>) -> PimAssignmentStatus {
        let row = &item.assignment;
        let status = match row.assignment_state {
            GovernanceRoleAssignmentState::Eligible => PimAssignmentStatus::Eligible,
            GovernanceRoleAssignmentState::Active => PimAssignmentStatus::Active,
        };
        status.at(now, row.start_date_time, row.end_date_time)
    }

    pub fn entra(index: usize, item: &PimEntraRoleAssignment, tenant: &str) -> Self {
        let row = &item.assignment;
        Self {
            id: row.id.clone(),
            index,
            role: item.definition.as_ref().map_or_else(
                || row.role_definition_id.to_string(),
                |definition| definition.display_name.clone(),
            ),
            scope: tenant.to_owned(),
            scope_target: None,
            scope_type: "Directory".to_owned(),
            membership: match row.member_type {
                GovernanceRoleAssignmentMemberType::Group => "Group",
                GovernanceRoleAssignmentMemberType::Direct => "Direct",
            },
            end: row.end_date_time,
            condition: None,
            action: if PimActivationAssignment::can_activate_entra(row) {
                "Activate"
            } else {
                ""
            },
        }
    }

    pub fn azure_status(item: &RoleEligibilitySchedule, now: DateTime<Utc>) -> PimAssignmentStatus {
        PimAssignmentStatus::Eligible.at(
            now,
            Some(item.properties.start_date_time),
            item.properties.end_date_time,
        )
    }

    pub fn azure(index: usize, item: &RoleEligibilitySchedule) -> Self {
        let properties = &item.properties;
        let scope = &properties.expanded_properties.scope;
        let scope_target = match &properties.scope {
            ScopeImpl::ManagementGroup(id) => Some(PortalTabTarget::ManagementGroup(id.clone())),
            ScopeImpl::Subscription(id) => Some(PortalTabTarget::Subscription(*id)),
            ScopeImpl::ResourceGroup(id) => Some(PortalTabTarget::ResourceGroup(id.clone())),
            ScopeImpl::StorageAccount(id) => Some(PortalTabTarget::StorageAccount(id.clone())),
            scope if !matches!(scope, ScopeImpl::Unknown(_)) => {
                Some(PortalTabTarget::Resource(Box::new(scope.clone())))
            }
            _ => None,
        };
        Self {
            id: item.id.expanded_form(),
            index,
            role: properties
                .expanded_properties
                .role_definition
                .display_name
                .clone(),
            scope: if scope.display_name.is_empty() {
                properties.scope.expanded_form()
            } else {
                scope.display_name.clone()
            },
            scope_target,
            scope_type: match scope.kind.as_str() {
                "managementGroup" => "Management group".to_owned(),
                "subscription" => "Subscription".to_owned(),
                "resourceGroup" => "Resource group".to_owned(),
                _ => scope.kind.clone(),
            },
            membership: match properties.member_type {
                RoleEligibilityScheduleMemberType::Group => "Group",
                RoleEligibilityScheduleMemberType::Direct => "Direct",
                RoleEligibilityScheduleMemberType::Inherited => "Inherited",
            },
            end: properties.end_date_time,
            condition: properties.condition.clone(),
            action: if PimActivationAssignment::can_activate_azure(item) {
                "Activate"
            } else {
                "Open in portal"
            },
        }
    }
}
