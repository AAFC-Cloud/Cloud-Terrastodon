use crate::AzureDevOpsBuildFolderDeleteRequest;
use crate::AzureDevOpsBuildFolderPruneReport;
use crate::AzureDevOpsBuildFolderPruneSkip;
use crate::azure_devops_build_folder_snapshot::AzureDevOpsBuildFolderSnapshot;
use crate::azure_devops_build_folder_validated_snapshot::AzureDevOpsBuildFolderValidatedSnapshot;
use crate::fetch_azure_devops_build_definitions;
use crate::fetch_azure_devops_build_folders;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Context;
use eyre::Result;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

/// Folder pruning invalidates and refreshes project-wide list caches before
/// planning and before each deletion, including during dry-run planning.
/// The API's cascading deletion has no atomic emptiness condition: callers must
/// be able to see all definitions and coordinate concurrent definition changes.
///
/// See the [folder-delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildFolderPruneRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub path: Option<AzureDevOpsBuildFolderPath>,
    pub dry_run: bool,
}

/// Build an awaitable empty-folder pruning request for an explicit project.
/// Dry runs only inspect inventory; deletion rechecks every candidate.
///
/// See the [folder-delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1)
/// for its cascade behavior and the request type for caller preconditions.
pub fn prune_azure_devops_build_definition_folders<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    path: Option<AzureDevOpsBuildFolderPath>,
    dry_run: bool,
) -> AzureDevOpsBuildFolderPruneRequest<'a> {
    AzureDevOpsBuildFolderPruneRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
        path,
        dry_run,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderPruneRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: Option::<AzureDevOpsBuildFolderPath>::arbitrary(u)?,
            dry_run: bool::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsBuildFolderPruneRequest<'a> {
    type Output = Result<AzureDevOpsBuildFolderPruneReport>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let scope = self.path.unwrap_or_else(AzureDevOpsBuildFolderPath::root);
            // Always inspect the complete project. User scope restricts deletion
            // candidates, rather than hiding definitions or descendant folders.
            let folders = fetch_azure_devops_build_folders(
                &self.org_url,
                self.project.clone(),
                &self.auth_context,
                None,
            )
            .with_invalidation(true)
            .await?;
            let definitions = fetch_azure_devops_build_definitions(
                &self.org_url,
                self.project.clone(),
                &self.auth_context,
                None,
                None,
            )
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
                // A definition may have been created since planning, and a child
                // may remain after an earlier candidate was skipped. Recheck
                // both inventories before invoking the cascading delete API.
                let fresh = async {
                    let folders = fetch_azure_devops_build_folders(
                        &self.org_url,
                        self.project.clone(),
                        &self.auth_context,
                        None,
                    )
                    .with_invalidation(true)
                    .await?;
                    let definitions = fetch_azure_devops_build_definitions(
                        &self.org_url,
                        self.project.clone(),
                        &self.auth_context,
                        None,
                        None,
                    )
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
                    auth_context: Cow::Borrowed(&self.auth_context),
                    path: candidate.clone(),
                }
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
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildFolderPruneRequest<'static> => AzureDevOpsBuildFolderPruneReport,
    effects = [Read, Write]
);
