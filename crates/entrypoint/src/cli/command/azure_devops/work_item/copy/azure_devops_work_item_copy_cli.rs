use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemCopyJournal;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemCopyOptions;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemCopyPlanRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemCopyRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::ensure;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::error;

/// Copy writable fields and hierarchy. Comments/history and attachment binaries
/// are omitted and reported in the plan. External links are opt-in.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemCopyArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Recursively copy Child relations, creating each parent first.
    #[facet(figue::named, default)]
    pub deep: bool,
    /// Comma-separated ceiling on source IDs that discovery may read.
    #[facet(figue::named)]
    pub allowed_ids: Option<String>,
    /// Replacement title for the copied root.
    #[facet(figue::named)]
    pub title: Option<String>,
    /// Preserve the root's existing parent link.
    #[facet(figue::named, default)]
    pub keep_parent: bool,
    /// Preserve links outside the copied subtree (except attachments).
    #[facet(figue::named, default)]
    pub keep_external_relations: bool,
    /// Source snapshot timestamp; defaults to the current UTC time.
    #[facet(figue::named)]
    pub as_of: Option<DateTime<Utc>>,
    /// Print the complete plan without any writes.
    #[facet(figue::named, default)]
    pub dry_run: bool,
    /// Progress journal path. Defaults to a unique file in the current directory.
    #[facet(figue::named)]
    pub journal: Option<PathBuf>,
    /// Resume the saved plan in --journal. Uncertain writes require reconciliation.
    #[facet(figue::named, default)]
    pub resume: bool,
    /// Suppress notifications for newly created items and links.
    #[facet(figue::named, default)]
    pub suppress_notifications: bool,
}

impl AzureDevOpsWorkItemCopyArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        ensure!(
            !self.resume || (self.journal.is_some() && !self.dry_run),
            "Resume requires --journal and cannot be combined with --dry-run"
        );
        ensure!(
            !self.resume
                || (self.title.is_none()
                    && self.as_of.is_none()
                    && self.allowed_ids.is_none()
                    && !self.keep_parent
                    && !self.keep_external_relations
                    && !self.deep),
            "Resume uses the saved plan; omit discovery and copy-policy options"
        );
        let options = AzureDevOpsWorkItemCopyOptions {
            deep: self.deep,
            allowed_ids: self
                .allowed_ids
                .map(|ids| ids.split(',').map(|id| id.trim().parse()).collect())
                .transpose()?
                .unwrap_or_default(),
            title: self.title,
            keep_parent: self.keep_parent,
            keep_external_relations: self.keep_external_relations,
            as_of: self.as_of,
        };
        let organization =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let plan = if self.resume {
            let journal = AzureDevOpsWorkItemCopyJournal::load(
                self.journal
                    .as_deref()
                    .ok_or_else(|| eyre::eyre!("Missing journal"))?,
            )?;
            ensure!(
                journal.plan.root == self.id,
                "Journal belongs to a different root item"
            );
            journal.plan
        } else {
            AzureDevOpsWorkItemCopyPlanRequest {
                org_url: Cow::Borrowed(&organization),
                project: self.project.clone(),
                auth_context: Cow::Borrowed(&auth_context),
                root: self.id,
                options,
            }
            .await?
        };
        if self.dry_run {
            cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &plan)?;
            return Ok(());
        }
        let journal_path = self.journal.unwrap_or_else(|| {
            PathBuf::from(format!("work-item-copy-{}.json", uuid::Uuid::new_v4()))
        });
        error!(journal = %journal_path.display(), "Copy progress journal");
        let result = AzureDevOpsWorkItemCopyRequest {
            org_url: Cow::Borrowed(&organization),
            project: self.project,
            auth_context: Cow::Borrowed(&auth_context),
            plan,
            journal_path,
            resume: self.resume,
            suppress_notifications: self.suppress_notifications,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &result)?;
        Ok(())
    }
}
