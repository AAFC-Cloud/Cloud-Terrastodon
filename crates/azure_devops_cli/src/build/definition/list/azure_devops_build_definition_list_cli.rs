use crate::build::output;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
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
use std::sync::Arc;

/// List pipeline definitions matching the supplied filters.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionListArgs {
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
    /// Refresh cached results before listing definitions.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
    /// Filter by definition name or name pattern.
    #[facet(figue::named)]
    pub name: Option<String>,
    /// Filter by definition folder path.
    #[facet(figue::named)]
    pub path: Option<AzureDevOpsBuildFolderPathArgument>,
}

impl AzureDevOpsBuildDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let definitions = AzureDevOpsBuildDefinitionListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
            name: (self.name),
            path: (path),
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet_with_text(
            definitions,
            |definitions, writer, cancellation| {
                write_definitions(definitions, writer, cancellation)
            },
        ))
    }
}

fn write_definitions(
    definitions: &[Arc<AzureDevOpsBuildDefinitionReference>],
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    if definitions.is_empty() {
        writeln!(writer, "No build definitions found.")?;
        return Ok(());
    }
    writeln!(
        writer,
        "{} ({})",
        "Build definitions".if_supports_color(Stream::Stdout, |text| text
            .style(Style::new().cyan().bold())),
        definitions.len()
    )?;
    for definition in definitions {
        cancellation.bail_if_cancelled()?;
        writeln!(
            writer,
            "  {}  {}  {}",
            definition
                .id
                .if_supports_color(Stream::Stdout, |id| id.green()),
            output::terminal_text(definition.name.as_str())
                .if_supports_color(Stream::Stdout, |text| text.bold()),
            output::terminal_text(definition.path.as_str())
                .if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    Ok(())
}
