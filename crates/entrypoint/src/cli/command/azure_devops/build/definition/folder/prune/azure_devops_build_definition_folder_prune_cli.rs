use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPruneReport;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prune_azure_devops_build_definition_folders;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::fmt::Write;

/// Remove empty build definition folders after fresh checks.
/// Azure DevOps folder deletion cascades to definitions and builds. Requires
/// visibility of every definition and no concurrent pipeline or folder edits.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionFolderPruneArgs {
    /// Organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Explicit project ID or name; pruning never uses the configured default.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
    /// Tenant ID or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Restrict pruning to this folder and its descendants. Root is preserved.
    #[facet(figue::named)]
    pub path: Option<AzureDevOpsBuildFolderPathArgument>,
    /// Show which empty folders would be deleted.
    #[facet(figue::named, default = false)]
    pub dry_run: bool,
}

impl AzureDevOpsBuildDefinitionFolderPruneArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let org = crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let report = prune_azure_devops_build_definition_folders(
            &org,
            self.project,
            &auth_context,
            path,
            self.dry_run,
        )
        .await?;
        Ok(CliOutput::facet_with_text(report, render_prune_report))
    }
}

fn render_prune_report(report: &AzureDevOpsBuildFolderPruneReport) -> Result<String> {
    let mut rendered = String::new();
    let heading = if report.dry_run {
        "Build definition folder prune (dry run)"
    } else {
        "Build definition folder prune"
    };
    writeln!(
        rendered,
        "{}",
        heading.if_supports_color(Stream::Stdout, |text| text
            .style(Style::new().cyan().bold()))
    )?;
    let (action, action_style, paths) = if report.dry_run {
        ("Would delete", Style::new().yellow(), &report.candidates)
    } else {
        ("Deleted", Style::new().green(), &report.deleted)
    };
    for path in paths {
        writeln!(
            rendered,
            "  {}  {}",
            action.if_supports_color(Stream::Stdout, |text| text.style(action_style)),
            path.as_str()
                .if_supports_color(Stream::Stdout, |text| text.bold())
        )?;
    }
    for skipped in &report.skipped {
        writeln!(
            rendered,
            "  {}  {}: {}",
            "Skipped".if_supports_color(Stream::Stdout, |text| text.yellow()),
            skipped
                .path
                .as_str()
                .if_supports_color(Stream::Stdout, |text| text.bold()),
            skipped
                .reason
                .as_str()
                .if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    if paths.is_empty() && report.skipped.is_empty() {
        writeln!(
            rendered,
            "  {}",
            "No empty folders found.".if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    Ok(rendered)
}
