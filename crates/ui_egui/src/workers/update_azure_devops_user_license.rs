use crate::app::MyApp;
use crate::azure_devops_organization_user_row::license_label;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementShowRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementUpdateRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::time::Instant;

impl MyApp {
    pub(crate) fn azure_devops_license_update_in_flight(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> bool {
        self.azure_devops_license_updates.contains(organization)
    }

    pub(crate) fn update_azure_devops_user_license(
        &mut self,
        organization: &AzureDevOpsOrganizationUrl,
        authentication: AzureDevOpsAuthSelection,
        user_id: AzureDevOpsUserId,
        expected_license: AzureDevOpsLicenseType,
        new_license: AzureDevOpsLicenseType,
    ) {
        if self.ready_azure_devops_authentication_for(organization) != Some(authentication)
            || expected_license == new_license
            || !self
                .azure_devops_license_updates
                .insert(organization.clone())
        {
            return;
        }

        let generation = self.azure.account_generation;
        let requested_organization = organization.clone();
        let work_organization = organization.clone();
        let requested_license = new_license.clone();
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, _>::new()
            .description(format!("Change Azure DevOps license in {}", organization.name()))
            .setter(move |app, result| {
                if !matches!(result, Loadable::Loading { .. }) {
                    app.azure_devops_license_updates.remove(&requested_organization);
                    for content in app.open_azure_contents_mut() {
                        content.azure_devops_users.reconcile_license_update(
                            &requested_organization,
                            authentication,
                            result.as_loaded(),
                        );
                    }
                }
                if let Some(content) = app.azure_content_mut(generation) {
                    let status = result.map(|response| {
                        if response.access_level.matches_license(&requested_license) {
                            format!("License changed to {}.", license_label(&requested_license))
                        } else {
                            format!(
                                "Azure DevOps accepted the change. It reports {} as the effective license.",
                                response.access_level.license_display_name,
                            )
                        }
                    });
                    content.azure_devops_users.publish_update(
                        requested_organization.clone(),
                        authentication,
                        status,
                    );
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                let current = AzureDevOpsUserLicenseEntitlementShowRequest {
                    org_url: Cow::Borrowed(&work_organization),
                    auth_selection: authentication,
                    user: user_id.into(),
                }
                .execute(&executor)
                .with_invalidation(true)
                .await?;
                let current_license = current.license();
                eyre::ensure!(
                    current_license == expected_license,
                    "The license changed to {} after you opened the editor. Review the current license before trying again.",
                    license_label(&current_license)
                );
                AzureDevOpsUserLicenseEntitlementUpdateRequest {
                    org_url: Cow::Owned(work_organization),
                    auth_selection: authentication,
                    user: user_id.into(),
                    license_kind: new_license,
                }
                .execute(&executor)
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
            self.azure_devops_license_updates.remove(organization);
            self.azure.azure_devops_users.publish_update(
                organization.clone(),
                authentication,
                Loadable::Failed {
                    error: error.into(),
                    started_at,
                    finished_at: Instant::now(),
                },
            );
        }
    }
}
