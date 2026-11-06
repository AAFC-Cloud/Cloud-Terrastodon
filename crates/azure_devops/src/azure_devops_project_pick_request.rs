use crate::fetch_all_azure_devops_projects;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::info;

#[must_use = "This is an interactive future request, you must .execute it"]
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

#[track_caller]
pub fn pick_azure_devops_project<'a>(
    org_url: AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsProjectPickRequest<'a>> {
    executor.execute(AzureDevOpsProjectPickRequest {
        org_url,
        auth_context: Cow::Borrowed(auth_context),
    })
}
impl<'a> CacheInvalidationKeys for AzureDevOpsProjectPickRequest<'a> {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![
            crate::AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(&self.org_url),
                auth_context: Cow::Borrowed(self.auth_context.as_ref()),
            }
            .cache_key()?,
        ])
    }
}

impl<'a> ExecutableRequest for AzureDevOpsProjectPickRequest<'a> {
    type Error = eyre::Report;
    type Output = AzureDevOpsProjectArgument<'static>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let org_url = self.org_url;
            let auth_context = self.auth_context;
            let projects = PickerTui::<Arc<AzureDevOpsProject>>::new()
                .set_header("Azure DevOps Projects")
                .add_event_handler(move |event, sink| {
                    let org_url = org_url.clone();
                    let auth_context = auth_context.clone();
                    let executor = executor.clone();
                    async move {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            info!(organization = %org_url, "Fetching Azure DevOps projects");
                            let projects =
                                fetch_all_azure_devops_projects(&org_url, auth_context.as_ref(), &executor)
                                    .await?;
                            sink.push(projects.into_iter().map(project_choice))?;
                            info!(organization = %org_url, "Finished fetching Azure DevOps projects");
                }
                Ok(())
            }
        })
        .pick_one_events()
                .await?;

            Ok(AzureDevOpsProjectArgument::from(projects.as_ref()).into_owned())
        })
    }
}

fn project_choice(project: Arc<AzureDevOpsProject>) -> Choice<Arc<AzureDevOpsProject>> {
    Choice {
        key: format!("{} {}", project.name, project.id),
        value: project,
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectPickRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectPickRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectPickRequest<'static> => AzureDevOpsProjectArgument<'static>,
    effects = [Read, Solicit]
);

#[cfg(test)]
mod test {
    use super::AzureDevOpsProjectPickRequest;
    use cloud_terrastodon_registry_execution::shape_can_be_produced_from_defaults;
    use facet::Facet;

    #[test]
    fn registry_discovers_default_organization_dependency() {
        assert!(shape_can_be_produced_from_defaults(
            AzureDevOpsProjectPickRequest::<'static>::SHAPE
        ));
    }
}
