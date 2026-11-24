use crate::app::MyApp;
use crate::azure_devops_project_data::AzureDevOpsProjectData;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use std::time::Instant;

impl MyApp {
    pub(crate) fn can_refresh_azure_devops_project_in(
        &self,
        content: &PortalAzureContent,
        organization: &AzureDevOpsOrganizationUrl,
        project: AzureDevOpsProjectId,
        view: PortalAzureDevOpsProjectView,
    ) -> bool {
        if view == PortalAzureDevOpsProjectView::Overview {
            return self.can_refresh_azure_devops_projects_in(content, organization);
        }
        self.ready_azure_devops_authentication_in(content, organization)
            .is_some_and(|authentication| {
                !matches!(
                    content.azure_devops_projects.section(
                        organization,
                        authentication,
                        project,
                        view
                    ),
                    Loadable::Loading { .. }
                )
            })
    }

    pub(crate) fn load_azure_devops_project_section(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
        project: AzureDevOpsProjectId,
        view: PortalAzureDevOpsProjectView,
        refresh: bool,
    ) {
        if view == PortalAzureDevOpsProjectView::Overview {
            self.load_azure_devops_projects_for(organization, refresh);
            return;
        }
        let Some(authentication) = self.ready_azure_devops_authentication_for(organization) else {
            return;
        };
        if matches!(
            self.azure
                .azure_devops_projects
                .section(organization, authentication, project, view),
            Loadable::Loading { .. }
        ) {
            return;
        }

        let generation = self.azure.account_generation;
        let requested_organization = organization.clone();
        let work_organization = organization.clone();
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, _>::new()
            .description(format!(
                "Load Azure DevOps {} for {}/{}",
                view.label(),
                organization.name(),
                self.azure_devops_project_name_in(&self.azure, organization, project)
                    .map_or_else(|| project.to_string(), str::to_owned),
            ))
            .setter(move |app, data| {
                if let Some(content) = app.azure_content_mut(generation) {
                    content.azure_devops_projects.publish_section(
                        requested_organization.clone(),
                        authentication,
                        project,
                        view,
                        data,
                    );
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                AzureDevOpsProjectData::load(
                    view,
                    &work_organization,
                    project,
                    authentication,
                    refresh,
                    &executor,
                )
                .await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = queued {
            self.azure.azure_devops_projects.publish_section(
                organization.clone(),
                authentication,
                project,
                view,
                Loadable::Failed {
                    error: error.into(),
                    started_at,
                    finished_at: Instant::now(),
                },
            );
        }
    }
}
