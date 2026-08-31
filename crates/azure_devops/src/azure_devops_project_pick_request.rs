use crate::fetch_all_azure_devops_projects;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;
use tracing::info;

#[must_use = "This is an interactive future request, you must .await it"]
#[derive(Facet)]
pub struct AzureDevOpsProjectPickRequest<'a> {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectPickRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: AzureDevOpsOrganizationUrl::arbitrary(u)?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

pub fn pick_azure_devops_project<'a>(
    org_url: AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
) -> AzureDevOpsProjectPickRequest<'a> {
    AzureDevOpsProjectPickRequest {
        org_url,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for AzureDevOpsProjectPickRequest<'a> {
    async fn invalidate(&self) -> Result<()> {
        fetch_all_azure_devops_projects(&self.org_url, self.auth_context.as_ref())
            .cache_key()
            .invalidate()
            .await
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsProjectPickRequest<'a> {
    type WithInvalidation = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsProjectPickRequest<'a> {
    type Output = Result<AzureDevOpsProjectArgument<'static>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let org_url = self.org_url;
            let auth_context = self.auth_context;
            let projects = PickerTui::<AzureDevOpsProject>::new()
                .set_header("Azure DevOps Projects")
                .add_event_handler(move |event, sink| {
                    let org_url = org_url.clone();
                    let auth_context = auth_context.clone();
                    async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            info!(organization = %org_url, "Fetching Azure DevOps projects");
                            let projects =
                                fetch_all_azure_devops_projects(&org_url, auth_context.as_ref())
                                    .await?;
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectPickRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectPickRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsProjectPickRequest<'static> => AzureDevOpsProjectArgument<'static>,
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
            AzureDevOpsProjectPickRequest::<'static>::SHAPE
        ));
    }
}
