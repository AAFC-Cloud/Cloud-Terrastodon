use crate::cli::azure_devops::build::output;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsBuild;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionId;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildListLimit;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildResult;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildStatus;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_builds;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use eyre::Result;
use std::fmt::Write;

/// List the latest build runs matching the supplied filters.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildListArgs {
    /// Organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project ID or name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    /// Tenant ID or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Refresh cached results before listing builds.
    #[facet(
        figue::named,
        figue::alias = "skip-cache",
        figue::alias = "clean",
        default = false
    )]
    pub no_cache: bool,
    /// Definition ID to filter by. Repeat to include multiple definitions.
    #[facet(figue::named, default)]
    pub definition: Vec<AzureDevOpsBuildDefinitionId>,
    /// Filter by API status, such as completed, inProgress, or notStarted.
    #[facet(figue::named)]
    pub status: Option<AzureDevOpsBuildStatus>,
    /// Filter by API result, such as succeeded, partiallySucceeded, or failed.
    #[facet(figue::named)]
    pub result: Option<AzureDevOpsBuildResult>,
    /// Filter by branch, such as refs/heads/main.
    #[facet(figue::named)]
    pub branch: Option<String>,
    /// Positive maximum number of recent builds to return. Defaults to 100.
    #[facet(figue::named, figue::alias = "top", default = AzureDevOpsBuildListLimit::new(100).expect("100 is a positive build limit"))]
    pub limit: AzureDevOpsBuildListLimit,
}

impl AzureDevOpsBuildListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let builds = fetch_azure_devops_builds(
            &self.org,
            self.project,
            &auth_context,
            self.definition,
            self.status,
            self.result,
            self.branch,
            Some(self.limit),
        )
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet_with_text(builds, |builds| {
            render_builds(builds)
        }))
    }
}

fn render_builds(builds: &[AzureDevOpsBuild]) -> Result<String> {
    if builds.is_empty() {
        return Ok("No builds found.\n".to_owned());
    }
    let mut rendered = String::new();
    writeln!(
        rendered,
        "{} ({})",
        "Builds".if_supports_color(Stream::Stdout, |text| text
            .style(Style::new().cyan().bold())),
        builds.len()
    )?;
    for build in builds {
        let definition = format!(
            "{} ({})",
            output::terminal_text(
                build
                    .definition
                    .name
                    .as_ref()
                    .map(|name| name.as_str())
                    .unwrap_or("(name unavailable)"),
            )
            .if_supports_color(Stream::Stdout, |text| text.bold()),
            build
                .definition
                .id
                .if_supports_color(Stream::Stdout, |id| id.green())
        );
        let status = output::terminal_text(build.status.as_str());
        let status_style = match status.to_ascii_lowercase().as_str() {
            "succeeded" | "deleted" => Style::new().green(),
            "failed" => Style::new().red(),
            "partiallysucceeded" | "inprogress" | "cancelling" | "notstarted" | "would delete"
            | "skipped" => Style::new().yellow(),
            _ => Style::new().dimmed(),
        };
        let result = output::terminal_text(build.result.as_str());
        let result_style = match result.to_ascii_lowercase().as_str() {
            "succeeded" | "deleted" => Style::new().green(),
            "failed" => Style::new().red(),
            "partiallysucceeded" | "inprogress" | "cancelling" | "notstarted" | "would delete"
            | "skipped" => Style::new().yellow(),
            _ => Style::new().dimmed(),
        };
        writeln!(
            rendered,
            "  {}  {}  {}  {} / {}  {}",
            build.id.if_supports_color(Stream::Stdout, |id| id.green()),
            output::terminal_text(build.build_number.as_str())
                .if_supports_color(Stream::Stdout, |text| text.bold()),
            definition,
            status.if_supports_color(Stream::Stdout, |text| text.style(status_style)),
            result.if_supports_color(Stream::Stdout, |text| text.style(result_style)),
            output::terminal_text(&build.source_branch)
                .if_supports_color(Stream::Stdout, |text| text.dimmed())
        )?;
    }
    Ok(rendered)
}
