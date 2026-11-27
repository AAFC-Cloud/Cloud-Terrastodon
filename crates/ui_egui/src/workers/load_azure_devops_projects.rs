use crate::app::MyApp;
use crate::portal_azure_content::PortalAzureContent;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::time::Instant;

impl MyApp {
    pub(crate) fn azure_devops_project_name_in<'a>(
        &self,
        content: &'a PortalAzureContent,
        organization: &AzureDevOpsOrganizationUrl,
        project_id: AzureDevOpsProjectId,
    ) -> Option<&'a str> {
        let authentication = self.ready_azure_devops_authentication_in(content, organization)?;
        let project = content
            .azure_devops_projects
            .get(organization, authentication)
            .as_loaded()?
            .iter()
            .find(|project| project.id == project_id)?;
        Some(project.name.as_ref())
    }

    pub(crate) fn can_refresh_azure_devops_projects_in(
        &self,
        content: &PortalAzureContent,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> bool {
        self.ready_azure_devops_authentication_in(content, organization)
            .is_some_and(|authentication| {
                !matches!(
                    content
                        .azure_devops_projects
                        .get(organization, authentication),
                    Loadable::Loading { .. }
                )
            })
    }

    pub(crate) fn refresh_azure_devops_projects_for(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
    ) {
        self.load_azure_devops_projects_for(organization, true);
    }

    pub(crate) fn load_azure_devops_projects_for(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
        refresh: bool,
    ) {
        let Some(auth_selection) = self.ready_azure_devops_authentication_for(organization) else {
            return;
        };
        if matches!(
            self.azure
                .azure_devops_projects
                .get(organization, auth_selection),
            Loadable::Loading { .. }
        ) {
            return;
        }

        let account_generation = self.azure.account_generation;
        let requested_organization = organization.clone();
        let request = AzureDevOpsProjectsListRequest {
            org_url: Cow::Owned(organization.clone()),
            auth_selection,
        };
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, _>::new()
            .description(format!(
                "Load Azure DevOps projects for {}",
                organization.name()
            ))
            .setter(move |app, data| {
                if let Some(azure) = app.azure_content_mut(account_generation) {
                    azure.azure_devops_projects.publish(
                        requested_organization.clone(),
                        auth_selection,
                        data,
                    );
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                request.execute(&executor).with_invalidation(refresh).await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = queued {
            self.azure.azure_devops_projects.publish(
                organization.clone(),
                auth_selection,
                Loadable::Failed {
                    error: error.into(),
                    started_at,
                    finished_at: Instant::now(),
                },
            );
        }
    }
}
