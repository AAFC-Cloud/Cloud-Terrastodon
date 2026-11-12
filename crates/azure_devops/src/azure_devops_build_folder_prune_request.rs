use crate::AzureDevOpsBuildDefinitionListRequest;
use crate::AzureDevOpsBuildFolderDeleteRequest;
use crate::AzureDevOpsBuildFolderListRequest;
use crate::AzureDevOpsBuildFolderPruneReport;
use crate::AzureDevOpsBuildFolderPruneSkip;
use crate::azure_devops_build_folder_snapshot::AzureDevOpsBuildFolderSnapshot;
use crate::azure_devops_build_folder_validated_snapshot::AzureDevOpsBuildFolderValidatedSnapshot;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Context;
use facet::Facet;
use std::borrow::Cow;

/// Folder pruning invalidates and refreshes project-wide list caches before
/// planning and before each deletion, including during dry-run planning.
/// The API's cascading deletion has no atomic emptiness condition: callers must
/// be able to see all definitions and coordinate concurrent definition changes.
///
/// See the [folder-delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsBuildFolderPruneRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub path: Option<AzureDevOpsBuildFolderPath>,
    pub dry_run: bool,
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderPruneRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            path: Option::<AzureDevOpsBuildFolderPath>::arbitrary(u)?,
            dry_run: bool::arbitrary(u)?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsBuildFolderPruneRequest<'a> {
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
    type Output = AzureDevOpsBuildFolderPruneReport;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let scope = self.path.unwrap_or_else(AzureDevOpsBuildFolderPath::root);
            // Always inspect the complete project. User scope restricts deletion
            // candidates, rather than hiding definitions or descendant folders.
            let folders = AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&self.org_url),
                project: (self.project.clone()),
                auth_selection: self.auth_selection,
                path: (None),
            }
            .execute(&executor)
            .with_invalidation(true)
            .await?;
            let definitions = AzureDevOpsBuildDefinitionListRequest {
                org_url: Cow::Borrowed(&self.org_url),
                project: (self.project.clone()),
                auth_selection: self.auth_selection,
                name: (None),
                path: (None),
            }
            .execute(&executor)
            .with_invalidation(true)
            .await?;
            let inventory =
                AzureDevOpsBuildFolderValidatedSnapshot::new(AzureDevOpsBuildFolderSnapshot {
                    folders,
                    definitions,
                })?;
            let candidates = inventory.candidates(&scope);
            let mut report = AzureDevOpsBuildFolderPruneReport {
                dry_run: self.dry_run,
                candidates: candidates.clone(),
                deleted: Vec::new(),
                skipped: Vec::new(),
            };
            if self.dry_run {
                return Ok(report);
            }
            for candidate in candidates {
                executor.cancellation().bail_if_cancelled()?;
                // A definition may have been created since planning, and a child
                // may remain after an earlier candidate was skipped. Recheck
                // both inventories before invoking the cascading delete API.
                let fresh = async {
                    let folders = AzureDevOpsBuildFolderListRequest {
                        org_url: Cow::Borrowed(&self.org_url),
                        project: (self.project.clone()),
                        auth_selection: self.auth_selection,
                        path: (None),
                    }
                    .execute(&executor)
                    .with_invalidation(true)
                    .await?;
                    let definitions = AzureDevOpsBuildDefinitionListRequest {
                        org_url: Cow::Borrowed(&self.org_url),
                        project: (self.project.clone()),
                        auth_selection: self.auth_selection,
                        name: (None),
                        path: (None),
                    }
                    .execute(&executor)
                    .with_invalidation(true)
                    .await?;
                    AzureDevOpsBuildFolderValidatedSnapshot::new(AzureDevOpsBuildFolderSnapshot {
                        folders,
                        definitions,
                    })
                }
                .await
                .wrap_err_with(|| {
                    format!(
                        "Pruning stopped before deleting {:?}; folders already deleted: {:?}",
                        candidate, report.deleted
                    )
                })?;
                if let Some(reason) = fresh.skip_reason(&candidate) {
                    report.skipped.push(AzureDevOpsBuildFolderPruneSkip {
                        path: candidate,
                        reason,
                    });
                    continue;
                }
                AzureDevOpsBuildFolderDeleteRequest {
                    org_url: Cow::Borrowed(&self.org_url),
                    project: self.project.clone(),
                    auth_selection: self.auth_selection,
                    path: candidate.clone(),
                }
                .execute(&executor)
                .await
                .wrap_err_with(|| {
                    format!(
                        "Pruning stopped while deleting {:?}; folders already deleted: {:?}",
                        candidate, report.deleted
                    )
                })?;
                report.deleted.push(candidate);
            }
            Ok(report)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPruneRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPruneRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsBuildFolderPruneRequest<'static> => AzureDevOpsBuildFolderPruneReport,
    effects = [Read, Write]
);
