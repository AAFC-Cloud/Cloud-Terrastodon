use crate::AzureDevOpsProjectsListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerEvent;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::info;

/// Interactively select a project from an explicit organization's shared inventory.
/// Credential resolution and cancellation belong to the supplied executor.
/// The organization may be borrowed for direct execution or owned for replay.
///
/// See [Projects List (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/list?view=azure-devops-rest-7.1).
#[must_use = "This is an interactive future request, you must .execute it"]
#[derive(Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectPickRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectPickRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl CacheInvalidationKeys for AzureDevOpsProjectPickRequest<'_> {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![
            crate::AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .cache_key()?,
        ])
    }
}

impl ExecutableRequest for AzureDevOpsProjectPickRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = AzureDevOpsProjectArgument<'static>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let projects = PickerTui::<Arc<AzureDevOpsProject>>::new()
                .set_header("Azure DevOps Projects")
                .add_event_handler(async |child, event, sink| {
                        if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                            info!(organization = %self.org_url, "Fetching Azure DevOps projects");
                            let projects = AzureDevOpsProjectsListRequest { org_url: Cow::Borrowed(&self.org_url), auth_selection: (self.auth_selection) }.execute(&executor.with_cancellation_scope(child))
                            .await?;
                            sink.push(projects.iter().cloned().map(project_choice))?;
                            info!(organization = %self.org_url, "Finished fetching Azure DevOps projects");
                        }
                        Ok(())
                })
                .pick_one_events(executor.cancellation())
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
    use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
    use cloud_terrastodon_cache::CacheInvalidationKeys;
    use cloud_terrastodon_registry::RuntimeValue;
    use std::borrow::Cow;
    use std::sync::LazyLock;

    #[test]
    fn reflected_project_picker_clone_preserves_borrowing_and_invalidation() -> eyre::Result<()> {
        static ORG: LazyLock<AzureDevOpsOrganizationUrl> = LazyLock::new(|| {
            "offline-picker-clone-example"
                .parse()
                .expect("valid synthetic organization")
        });
        for borrowed in [false, true] {
            let request = AzureDevOpsProjectPickRequest {
                org_url: if borrowed {
                    Cow::Borrowed(&*ORG)
                } else {
                    Cow::Owned((*ORG).clone())
                },
                auth_selection: crate::test_executor::authentication_selection(),
            };
            let invalidation = request.cache_keys_to_invalidate()?;
            let original = RuntimeValue::from_box(Box::new(request))?;
            // Registry clone-mode invocation must preserve the borrowed URL
            // and the selected refresh scope without opening a picker.
            let cloned = original
                .try_clone()?
                .into_box::<AzureDevOpsProjectPickRequest<'static>>()?
                .downcast::<AzureDevOpsProjectPickRequest<'static>>()
                .expect("runtime cloning preserves the request type");
            assert_eq!(cloned.org_url.as_ref(), &*ORG);
            assert_eq!(matches!(cloned.org_url, Cow::Borrowed(_)), borrowed);
            assert_eq!(
                cloned.auth_selection,
                crate::test_executor::authentication_selection()
            );
            assert_eq!(cloned.cache_keys_to_invalidate()?, invalidation);
            if borrowed {
                assert!(std::ptr::eq(cloned.org_url.as_ref(), &*ORG));
            }
        }
        Ok(())
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsProjectPickRequest<'static>,
    name = "azure-devops-project-pick",
    cached
);
