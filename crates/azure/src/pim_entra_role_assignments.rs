use crate::fetch_current_user;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use itertools::Itertools;
use std::path::PathBuf;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct MyEntraPimRoleAssignmentsListRequest;

pub fn fetch_my_entra_pim_role_assignments() -> MyEntraPimRoleAssignmentsListRequest {
    MyEntraPimRoleAssignmentsListRequest
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for MyEntraPimRoleAssignmentsListRequest {
    type Output = Vec<GovernanceRoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "pim_roleAssignments",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let my_object_id = fetch_current_user().await?.id;
        let url = format!(
            "{}{}{}{}",
            "https://graph.microsoft.com/beta/",
            "privilegedAccess/aadroles/roleAssignments",
            format_args!(
                "?$filter=(subject/id eq '{}') and (assignmentState in ('Eligible', 'Active'))",
                my_object_id
            ),
            format_args!(
                "&$select={}",
                [
                    "assignmentState",
                    "endDateTime",
                    "id",
                    "linkedEligibleRoleAssignmentId",
                    "memberType",
                    "roleDefinitionId",
                    "startDateTime",
                    "status",
                    "subjectId",
                ]
                .into_iter()
                .join(",")
            )
        );
        #[derive(facet::Facet)]
        struct Response {
            value: Vec<GovernanceRoleAssignment>,
        }

        let request = RestRequest::new(http::Method::GET, &url)?.cache(self.cache_key());
        let mut result: Result<Response, _> = request.clone().receive().await;
        if result.is_err() {
            // single retry - sometimes this returns a gateway error
            result = request.receive().await;
        }
        Ok(result?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(MyEntraPimRoleAssignmentsListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(result) =
            expect_aad_premium_p2_license(fetch_my_entra_pim_role_assignments().await).await?
        else {
            return Ok(());
        };
        assert!(!result.is_empty());
        Ok(())
    }
}

/// Fetch the current user's eligible/active Entra PIM role assignments with an explicit token.
pub async fn fetch_my_entra_pim_role_assignments_with_graph_access_token(
    tenant_id: AzureTenantId,
    principal_id: impl Into<PrincipalId>,
    access_token: &str,
) -> Result<Vec<GovernanceRoleAssignment>> {
    let principal_id: PrincipalId = principal_id.into();
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$filter=(subject/id eq '{principal_id}') and (assignmentState in ('Eligible', 'Active'))&$select={}",
        [
            "assignmentState",
            "endDateTime",
            "id",
            "linkedEligibleRoleAssignmentId",
            "memberType",
            "roleDefinitionId",
            "startDateTime",
            "status",
            "subjectId",
        ]
        .join(",")
    );
    #[derive(facet::Facet)]
    struct Response {
        value: Vec<GovernanceRoleAssignment>,
    }

    let request = RestRequest::new(http::Method::GET, &url)?
        .tenant(tenant_id)
        .bearer_token(access_token);
    let mut result: Result<Response, _> = request.clone().receive().await;
    if result.is_err() {
        result = request.receive().await;
    }
    Ok(result?.value)
}
cloud_terrastodon_registry::register_thing!(MyEntraPimRoleAssignmentsListRequest);
cloud_terrastodon_registry::register_arbitrary!(MyEntraPimRoleAssignmentsListRequest);
cloud_terrastodon_registry::register_into_future!(MyEntraPimRoleAssignmentsListRequest => Vec<GovernanceRoleAssignment>);
