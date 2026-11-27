use crate::app::MyApp;
use crate::portal_azure_content::PortalAzureContent;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::time::Instant;

impl MyApp {
    pub(crate) fn can_refresh_azure_devops_users_in(
        &self,
        content: &PortalAzureContent,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> bool {
        organization.organization_name().is_ok()
            && !self.azure_devops_license_update_in_flight(organization)
            && self
                .ready_azure_devops_authentication_in(content, organization)
                .is_some_and(|authentication| {
                    !matches!(
                        content
                            .azure_devops_users
                            .users(organization, authentication),
                        Loadable::Loading { .. }
                    )
                })
    }

    pub(crate) fn load_azure_devops_users(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
        refresh: bool,
    ) {
        if !self.can_refresh_azure_devops_users_in(&self.azure, organization) {
            return;
        }
        let Some(authentication) = self.ready_azure_devops_authentication_for(organization) else {
            return;
        };
        let generation = self.azure.account_generation;
        let requested_organization = organization.clone();
        let request = AzureDevOpsUserLicenseEntitlementListRequest {
            org_url: Cow::Owned(organization.clone()),
            auth_selection: authentication,
        };
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, _>::new()
            .description(format!(
                "Load Azure DevOps users for {}",
                organization.name()
            ))
            .setter(move |app, result| {
                if let Some(content) = app.azure_content_mut(generation) {
                    content.azure_devops_users.publish_users(
                        requested_organization.clone(),
                        authentication,
                        result,
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
            self.azure.azure_devops_users.fail_to_enqueue_users(
                organization.clone(),
                authentication,
                started_at,
                error,
            );
        }
    }
}
