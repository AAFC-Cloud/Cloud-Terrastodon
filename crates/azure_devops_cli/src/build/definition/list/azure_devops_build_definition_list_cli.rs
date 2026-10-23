use crate::build::output;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildFolderPathArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_build_definitions;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::io::Write;

/// List pipeline definitions matching the supplied filters.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionListArgs {
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    /// Tenant ID or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
        let path = self.path.map(AzureDevOpsBuildFolderPathArgument::into_path);
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let definitions = fetch_azure_devops_build_definitions(
            &self.org,
            self.project,
            &auth_context,
            self.name,
            path,
            cancellation,
        )
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
    definitions: &[AzureDevOpsBuildDefinitionReference],
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
