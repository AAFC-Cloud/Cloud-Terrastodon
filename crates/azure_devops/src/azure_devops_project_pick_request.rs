use crate::fetch_all_azure_devops_projects;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;
use tracing::info;

#[must_use = "This is an interactive future request, you must .await it"]
#[derive(Arbitrary, Facet)]
pub struct AzureDevOpsProjectPickRequest {
    pub org_url: AzureDevOpsOrganizationUrl,
}

pub fn pick_azure_devops_project(
    org_url: AzureDevOpsOrganizationUrl,
) -> AzureDevOpsProjectPickRequest {
    AzureDevOpsProjectPickRequest { org_url }
}

#[async_trait]
impl CacheInvalidatable for AzureDevOpsProjectPickRequest {
    async fn invalidate(&self) -> Result<()> {
        fetch_all_azure_devops_projects(&self.org_url)
            .cache_key()
            .invalidate()
            .await
    }
}

impl CacheInvalidatableIntoFuture for AzureDevOpsProjectPickRequest {
    type WithInvalidation = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl IntoFuture for AzureDevOpsProjectPickRequest {
    type Output = Result<AzureDevOpsProjectArgument<'static>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let org_url = self.org_url;
            let projects = PickerTui::<AzureDevOpsProject>::new()
                .set_header("Azure DevOps Projects")
                .add_event_handler(move |event, sink| {
                    let org_url = org_url.clone();
                    async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            info!(organization = %org_url, "Fetching Azure DevOps projects");
                            let projects = fetch_all_azure_devops_projects(&org_url).await?;
                            sink.push(projects.into_iter().map(project_choice))?;
                            info!(organization = %org_url, "Finished fetching Azure DevOps projects");
                }
                Ok(())
            }
        })
        .pick_one_events()
                .await?;

            Ok(AzureDevOpsProjectArgument::from(projects).into_owned())
        })
    }
}

fn project_choice(project: AzureDevOpsProject) -> Choice<AzureDevOpsProject> {
    Choice {
        key: format!("{} {}", project.name, project.id),
        value: project,
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectPickRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectPickRequest);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsProjectPickRequest => AzureDevOpsProjectArgument<'static>,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use super::AzureDevOpsProjectPickRequest;
    use cloud_terrastodon_registry::shape_can_be_produced_from_defaults;
    use facet::Facet;

    #[test]
    fn registry_discovers_default_organization_dependency() {
        assert!(shape_can_be_produced_from_defaults(
            AzureDevOpsProjectPickRequest::SHAPE
        ));
    }
}
