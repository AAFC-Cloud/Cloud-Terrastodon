use chrono::Utc;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzurePimRoleActivateRequest;
use cloud_terrastodon_azure::AzureRoleManagementPolicyAssignmentsForRoleRequest;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_azure::EntraPimRoleActivateRequest;
use cloud_terrastodon_azure::EntraPimRoleSettingsRequest;
use cloud_terrastodon_azure::GovernanceRoleAssignment;
use cloud_terrastodon_azure::GovernanceRoleAssignmentMemberType;
use cloud_terrastodon_azure::GovernanceRoleAssignmentState;
use cloud_terrastodon_azure::ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX;
use cloud_terrastodon_azure::RoleEligibilitySchedule;
use cloud_terrastodon_azure::RoleEligibilityScheduleMemberType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::uuid::Uuid;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::ensure;
use std::sync::Arc;
use std::time::Duration;

/// A role eligibility retained from the signed-in account's inventory.
#[derive(Clone)]
pub(crate) enum PimActivationAssignment {
    Entra {
        assignment: Arc<GovernanceRoleAssignment>,
        role_name: String,
    },
    Azure(Arc<RoleEligibilitySchedule>),
}

impl PimActivationAssignment {
    pub fn can_activate_entra(assignment: &GovernanceRoleAssignment) -> bool {
        let now = Utc::now();
        matches!(
            assignment.assignment_state,
            GovernanceRoleAssignmentState::Eligible
        ) && !assignment.id.trim().is_empty()
            && assignment.start_date_time.is_none_or(|start| start <= now)
            && assignment.end_date_time.is_none_or(|end| end > now)
    }

    pub fn can_activate_azure(assignment: &RoleEligibilitySchedule) -> bool {
        let now = Utc::now();
        let properties = &assignment.properties;
        properties.start_date_time <= now
            && properties.end_date_time.is_none_or(|end| end > now)
            && properties.condition.is_none()
            && properties.condition_version.is_none()
            && matches!(
                properties.member_type,
                RoleEligibilityScheduleMemberType::Direct
                    | RoleEligibilityScheduleMemberType::Group
            )
            && assignment.id.expanded_form().eq_ignore_ascii_case(&format!(
                "{}{ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX}{}",
                properties.scope.expanded_form().trim_end_matches('/'),
                assignment.name,
            ))
    }

    pub fn can_activate(&self) -> bool {
        match self {
            Self::Entra { assignment, .. } => Self::can_activate_entra(assignment),
            Self::Azure(assignment) => Self::can_activate_azure(assignment),
        }
    }

    pub fn role_name(&self) -> &str {
        match self {
            Self::Entra { role_name, .. } => role_name,
            Self::Azure(assignment) => {
                &assignment
                    .properties
                    .expanded_properties
                    .role_definition
                    .display_name
            }
        }
    }

    pub fn scope_label(&self, authentication: AzureBearerAuthenticationSelection) -> String {
        match self {
            Self::Entra { .. } => authentication.tenant_id.to_string(),
            Self::Azure(assignment) => assignment.properties.scope.expanded_form(),
        }
    }

    pub async fn maximum_duration(
        &self,
        authentication: AzureBearerAuthenticationSelection,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Duration> {
        match self {
            Self::Entra { assignment, .. } => EntraPimRoleSettingsRequest {
                authentication,
                role_definition_id: assignment.role_definition_id,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?
            .get_maximum_grant_period(),
            Self::Azure(assignment) => {
                let policies = AzureRoleManagementPolicyAssignmentsForRoleRequest {
                    authentication,
                    scope: assignment.properties.scope.clone(),
                    role_definition_id: assignment.properties.role_definition_id.clone(),
                }
                .execute(executor)
                .with_invalidation(true)
                .await?;
                let Some(maximum) = policies
                    .iter()
                    .filter_map(|policy| policy.get_maximum_activation_duration()?.to_std())
                    .min()
                else {
                    eyre::bail!(
                        "Azure did not report a maximum activation duration for this role."
                    );
                };
                Ok(maximum)
            }
        }
    }

    pub async fn submit(
        self,
        authentication: AzureBearerAuthenticationSelection,
        duration: Duration,
        justification: String,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        // Use the same credential session to identify the user and submit the grant.
        let executor = executor.with_authentication_lease(&authentication.into())?;
        let principal_id = EntraCurrentUserRequest { authentication }
            .execute(&executor)
            .await?
            .id;
        executor.cancellation().bail_if_cancelled()?;
        ensure!(
            self.can_activate(),
            "This role is no longer eligible for activation in Cloud Terrastodon."
        );
        match self {
            Self::Entra { assignment, .. } => {
                if matches!(
                    assignment.member_type,
                    GovernanceRoleAssignmentMemberType::Direct
                ) {
                    ensure!(
                        *principal_id == assignment.subject_id,
                        "The signed-in user does not match the selected role eligibility."
                    );
                }
                EntraPimRoleActivateRequest {
                    authentication,
                    principal_id: principal_id.into(),
                    role_definition_id: assignment.role_definition_id,
                    linked_eligible_role_assignment_id: assignment.id.clone(),
                    justification,
                    duration,
                }
                .execute(&executor)
                .await
            }
            Self::Azure(assignment) => {
                let properties = &assignment.properties;
                if matches!(
                    properties.member_type,
                    RoleEligibilityScheduleMemberType::Direct
                ) {
                    ensure!(
                        *principal_id == properties.principal_id,
                        "The signed-in user does not match the selected role eligibility."
                    );
                }
                AzurePimRoleActivateRequest {
                    authentication,
                    scope: properties.scope.clone(),
                    request_id: Uuid::new_v4(),
                    principal_id: principal_id.into(),
                    role_definition_id: properties.role_definition_id.clone(),
                    role_eligibility_schedule_id: assignment.id.clone(),
                    justification,
                    duration,
                }
                .execute(&executor)
                .await
            }
        }
    }
}
