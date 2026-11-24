use crate::azure_devops_project_data::AzureDevOpsProjectData;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::sync::Arc;

type ProjectsResponse = Loadable<Arc<Vec<Arc<AzureDevOpsProject>>>>;
type ProjectSectionResponses = HashMap<
    (
        AzureDevOpsAuthSelection,
        AzureDevOpsProjectId,
        PortalAzureDevOpsProjectView,
    ),
    Loadable<AzureDevOpsProjectData>,
>;

/// Retains project lists and section results for the credential used to load them.
#[derive(Default)]
pub(crate) struct AzureDevOpsProjectInventory {
    responses:
        HashMap<AzureDevOpsOrganizationUrl, HashMap<AzureDevOpsAuthSelection, ProjectsResponse>>,
    sections: HashMap<AzureDevOpsOrganizationUrl, ProjectSectionResponses>,
}

impl AzureDevOpsProjectInventory {
    pub fn get(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
    ) -> &ProjectsResponse {
        self.responses
            .get(organization)
            .and_then(|responses| responses.get(&authentication))
            .unwrap_or(&Loadable::NotLoaded)
    }

    pub fn publish(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        response: ProjectsResponse,
    ) {
        self.responses
            .entry(organization)
            .or_default()
            .insert(authentication, response);
    }

    pub fn is_loading(&self) -> bool {
        self.responses
            .values()
            .flat_map(HashMap::values)
            .any(|response| matches!(response, Loadable::Loading { .. }))
            || self
                .sections
                .values()
                .flat_map(HashMap::values)
                .any(|response| matches!(response, Loadable::Loading { .. }))
    }

    pub fn section(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        project: AzureDevOpsProjectId,
        view: PortalAzureDevOpsProjectView,
    ) -> &Loadable<AzureDevOpsProjectData> {
        self.sections
            .get(organization)
            .and_then(|responses| responses.get(&(authentication, project, view)))
            .unwrap_or(&Loadable::NotLoaded)
    }

    pub fn publish_section(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        project: AzureDevOpsProjectId,
        view: PortalAzureDevOpsProjectView,
        response: Loadable<AzureDevOpsProjectData>,
    ) {
        self.sections
            .entry(organization)
            .or_default()
            .insert((authentication, project, view), response);
    }

    pub fn loaded(
        &self,
    ) -> impl Iterator<
        Item = (
            &AzureDevOpsOrganizationUrl,
            AzureDevOpsAuthSelection,
            &Arc<Vec<Arc<AzureDevOpsProject>>>,
        ),
    > {
        self.responses.iter().flat_map(|(organization, responses)| {
            responses
                .iter()
                .filter_map(move |(&authentication, response)| {
                    Some((organization, authentication, response.as_loaded()?))
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn organizations_and_credentials_keep_independent_project_inventory() -> eyre::Result<()> {
        let first_org: AzureDevOpsOrganizationUrl = "first-org".parse()?;
        let second_org: AzureDevOpsOrganizationUrl = "second-org".parse()?;
        let first_auth = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: "00000000-0000-0000-0000-000000000001".parse()?,
        };
        let second_auth = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
        };
        let now = Instant::now();
        let mut inventory = AzureDevOpsProjectInventory::default();
        inventory.publish(
            first_org.clone(),
            first_auth,
            Loadable::Loading { started_at: now },
        );
        inventory.publish(
            second_org.clone(),
            second_auth,
            Loadable::Loaded {
                value: Arc::new(Vec::new()),
                started_at: now,
                finished_at: now,
            },
        );

        assert!(inventory.is_loading());
        assert!(matches!(
            inventory.get(&first_org, first_auth),
            Loadable::Loading { .. }
        ));
        assert!(matches!(
            inventory.get(&first_org, second_auth),
            Loadable::NotLoaded
        ));
        assert_eq!(
            inventory
                .loaded()
                .map(|(org, auth, _)| (org, auth))
                .collect::<Vec<_>>(),
            [(&second_org, second_auth)]
        );
        Ok(())
    }

    #[test]
    fn project_section_loading_is_isolated_from_other_projects_sections_and_accounts()
    -> eyre::Result<()> {
        use PortalAzureDevOpsProjectView::Repositories;
        use PortalAzureDevOpsProjectView::Teams;

        let organization: AzureDevOpsOrganizationUrl = "first-org".parse()?;
        let first_project = "00000000-0000-0000-0000-000000000001".parse()?;
        let second_project = "00000000-0000-0000-0000-000000000002".parse()?;
        let first_auth = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: "00000000-0000-0000-0000-000000000003".parse()?,
        };
        let second_auth = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: "00000000-0000-0000-0000-000000000004".parse()?,
        };
        let mut inventory = AzureDevOpsProjectInventory::default();
        inventory.publish_section(
            organization.clone(),
            first_auth,
            first_project,
            Repositories,
            Loadable::Loading {
                started_at: Instant::now(),
            },
        );

        assert!(inventory.is_loading());
        assert!(matches!(
            inventory.section(&organization, first_auth, first_project, Repositories),
            Loadable::Loading { .. }
        ));
        for (authentication, project, view) in [
            (second_auth, first_project, Repositories),
            (first_auth, second_project, Repositories),
            (first_auth, first_project, Teams),
        ] {
            assert!(matches!(
                inventory.section(&organization, authentication, project, view),
                Loadable::NotLoaded
            ));
        }
        Ok(())
    }
}
