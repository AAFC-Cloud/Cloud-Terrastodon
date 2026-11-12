use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPruneReport;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPruneRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::io::Write;

/// Remove empty build definition folders after fresh checks.
/// Azure DevOps folder deletion cascades to definitions and builds. Requires
/// visibility of every definition and no concurrent pipeline or folder edits.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionFolderPruneArgs {
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Explicit project ID, name, or `default`; a project selector is required.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Restrict pruning to this folder and its descendants. Root is preserved.
    #[facet(figue::named)]
    pub path: Option<AzureDevOpsBuildFolderPathArgument>,
    /// Show which empty folders would be deleted.
    #[facet(figue::named, default = false)]
    pub dry_run: bool,
}

impl AzureDevOpsBuildDefinitionFolderPruneArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let report = AzureDevOpsBuildFolderPruneRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
            path: (path),
            dry_run: (self.dry_run),
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet_with_text(report, write_prune_report))
    }
}

fn write_prune_report(
    report: &AzureDevOpsBuildFolderPruneReport,
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    let heading = if report.dry_run {
        "Build definition folder prune (dry run)"
    } else {
        "Build definition folder prune"
    };
    writeln!(
        writer,
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
        cancellation.bail_if_cancelled()?;
        writeln!(
            writer,
            "  {}  {}",
            action.if_supports_color(Stream::Stdout, |text| text.style(action_style)),
            path.as_str()
                .if_supports_color(Stream::Stdout, |text| text.bold())
        )?;
    }
    for skipped in &report.skipped {
        cancellation.bail_if_cancelled()?;
        writeln!(
            writer,
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
            writer,
            "  {}",
            "No empty folders found.".if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    Ok(())
}
