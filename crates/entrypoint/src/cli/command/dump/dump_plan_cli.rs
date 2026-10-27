use super::dump_plan_report::DumpPlanReport;
use super::dump_plan_session::DumpPlanSession;
use super::dump_planner::DumpPlanner;
use super::output::write_dump_plan;
use crate::cli::command::excalidraw::ExcalidrawInteractiveSession;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_excalidraw::ExcalidrawScene;
use cloud_terrastodon_registry::known_functions;
use cloud_terrastodon_registry::known_things;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Plan potential read-only request chains without resolving credentials or calling APIs.
#[derive(Debug, Arbitrary, Facet)]
pub struct DumpPlanArgs {
    /// Explicit Azure DevOps organization URL or cloud organization name.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Explicit Azure tenant ID to include Azure resource/identity inventory.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantId>,
    /// Desired registry type name, repeated for each target (for example AzureDevOpsProject).
    #[facet(figue::named)]
    pub target: Vec<String>,
    /// Write an editable Excalidraw request graph to this file.
    #[facet(figue::named)]
    pub diagram: Option<PathBuf>,
    /// Edit targets in the terminal and update the diagram until Q/Esc or Ctrl-C.
    /// Requires --diagram. Q/Esc emits the final report; Ctrl-C cancels.
    #[facet(figue::named, default)]
    pub interactive: bool,
}

cloud_terrastodon_registry::register_thing!(DumpPlanArgs);
cloud_terrastodon_registry::register_arbitrary!(DumpPlanArgs);

impl DumpPlanArgs {
    pub async fn invoke(
        self,
        auth_source: AuthSource,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
        ensure!(
            self.interactive || !self.target.is_empty(),
            "provide at least one --target type name"
        );
        ensure!(
            !self.interactive || self.diagram.is_some(),
            "--interactive requires --diagram for the rendered plan"
        );
        ensure!(
            self.org.is_some() || self.tenant.is_some(),
            "provide --org and/or --tenant for the inventory scope"
        );
        let mut seeds = Vec::new();
        if self.org.is_some() {
            seeds.push(AzureDevOpsOrganizationUrl::SHAPE);
            if auth_source != AuthSource::None {
                seeds.push(AzureDevOpsAuthContext::SHAPE);
            }
        }
        if self.tenant.is_some()
            && !matches!(
                auth_source,
                AuthSource::None | AuthSource::PersonalAccessToken
            )
        {
            seeds.push(AzureTenantAuthContext::SHAPE);
        }
        let planner = DumpPlanner::new(known_functions(), known_things(), seeds);
        let plan = planner.plan(&self.target, cancellation)?;
        let mut report = DumpPlanReport {
            organization: self.org,
            tenant: self.tenant,
            auth_source,
            plan,
            diagram: self.diagram,
        };
        report.explain_assumptions();
        if self.interactive {
            let path = report.diagram.clone().expect("validated diagram path");
            let planning = DumpPlanSession::new(planner, report, self.target);
            let mut session =
                ExcalidrawInteractiveSession::open_plan(path, planning, cancellation).await?;
            session.run(cancellation).await?;
            return Ok(CliOutput::facet_with_text(
                session.into_plan_report().expect("planning session"),
                write_dump_plan,
            ));
        }
        if let Some(path) = &report.diagram {
            cancellation.bail_if_cancelled()?;
            let scene = ExcalidrawScene::from_discovery_plan(&report.plan);
            let json = scene.to_json()?;
            // Publish a complete file; cancellation before this short completion
            // unit leaves an existing destination intact.
            cancellation.with_deferral("writing the request plan diagram", |_| {
                let directory = path
                    .parent()
                    .filter(|path| !path.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
                let mut temporary = tempfile::NamedTempFile::new_in(directory)
                    .wrap_err_with(|| format!("Creating diagram beside {}", path.display()))?;
                temporary.write_all(json.as_bytes())?;
                temporary.flush()?;
                temporary
                    .persist(path)
                    .map_err(|error| error.error)
                    .wrap_err_with(|| format!("Publishing diagram {}", path.display()))?;
                Ok::<_, eyre::Report>(())
            })?;
        }
        Ok(CliOutput::facet_with_text(report, write_dump_plan))
    }
}
