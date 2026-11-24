use crate::app::MyApp;
use crate::pim_entra_role_assignment::PimEntraRoleAssignment;
use crate::portal_page::PortalPage;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::EntraPimRoleAssignmentsListMineRequest;
use cloud_terrastodon_azure::EntraPimRoleDefinitionListRequest;
use cloud_terrastodon_azure::EntraRoleEligibilityScheduleListMineRequest;
use cloud_terrastodon_azure::RoleEligibilitySchedule;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::WrapErr;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;

/// PIM responses for the active account within one desktop tab.
#[derive(Default)]
pub(crate) struct PimRoleInventory {
    pub entra: Loadable<Vec<PimEntraRoleAssignment>>,
    pub azure: Loadable<Vec<Arc<RoleEligibilitySchedule>>>,
}

impl PimRoleInventory {
    pub fn is_loading(&self) -> bool {
        matches!(self.entra, Loadable::Loading { .. })
            || matches!(self.azure, Loadable::Loading { .. })
    }

    pub fn can_refresh(&self, page: PortalPage) -> bool {
        match page {
            PortalPage::PimEntraRoles => !matches!(self.entra, Loadable::Loading { .. }),
            PortalPage::PimAzureResources => !matches!(self.azure, Loadable::Loading { .. }),
            _ => false,
        }
    }
}

impl MyApp {
    pub(crate) fn load_pim_roles(&mut self, page: PortalPage, refresh: bool) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        if authentication.auth_source == AzureBearerAuthSource::WorkloadIdentity {
            return;
        }
        match page {
            PortalPage::PimEntraRoles => self.queue_pim_roles(
                refresh,
                "Load my Entra PIM assignments",
                |inventory| &mut inventory.entra,
                async move |executor| {
                    let (assignments, definitions) = executor
                        .join2(
                            async |child| {
                                EntraPimRoleAssignmentsListMineRequest { authentication }
                                    .execute(&child)
                                    .with_invalidation(refresh)
                                    .await
                                    .wrap_err("List my Entra PIM assignments")
                            },
                            async |child| {
                                EntraPimRoleDefinitionListRequest { authentication }
                                    .execute(&child)
                                    .with_invalidation(refresh)
                                    .await
                                    .wrap_err("List Entra PIM role definitions")
                            },
                        )
                        .await?;
                    let definitions = definitions
                        .into_iter()
                        .map(|definition| (definition.id, definition))
                        .collect::<HashMap<_, _>>();
                    eyre::Ok(
                        assignments
                            .into_iter()
                            .map(|assignment| PimEntraRoleAssignment {
                                definition: definitions
                                    .get(&assignment.role_definition_id)
                                    .cloned(),
                                assignment,
                            })
                            .collect(),
                    )
                },
            ),
            PortalPage::PimAzureResources => self.queue_pim_roles(
                refresh,
                "Load my Azure resource role eligibilities",
                |inventory| &mut inventory.azure,
                async move |executor| {
                    EntraRoleEligibilityScheduleListMineRequest { authentication }
                        .execute(&executor)
                        .with_invalidation(refresh)
                        .await
                        .wrap_err("List my Azure resource role eligibilities")
                },
            ),
            _ => {}
        }
    }

    fn queue_pim_roles<T, F>(
        &mut self,
        refresh: bool,
        description: &str,
        field: fn(&mut PimRoleInventory) -> &mut Loadable<T>,
        load: impl FnOnce(CloudTerrastodonExecutor) -> F + Send + 'static,
    ) where
        T: Debug + Send + 'static,
        F: Future<Output = Result<T>> + Send + 'static,
    {
        match field(&mut self.azure.pim_roles) {
            Loadable::Loading { .. } => return,
            Loadable::NotLoaded => {}
            _ if !refresh => return,
            _ => {}
        }

        let generation = self.azure.account_generation;
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, T>::new()
            .description(description)
            .setter(move |app, result| {
                if let Some(content) = app.azure_content_mut(generation) {
                    *field(&mut content.pim_roles) = result;
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                load(executor).await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );

        if let Err(error) = queued {
            *field(&mut self.azure.pim_roles) = Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            };
        }
    }
}
