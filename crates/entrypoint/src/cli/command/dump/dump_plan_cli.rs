use super::dump_plan_report::DumpPlanReport;
use super::output::write_dump_plan;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_excalidraw::ExcalidrawScene;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::DiscoveryPlanNode;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::known_functions;
use cloud_terrastodon_registry::known_things;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::collections::HashSet;
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
            !self.target.is_empty(),
            "provide at least one --target type name"
        );
        ensure!(
            self.org.is_some() || self.tenant.is_some(),
            "provide --org and/or --tenant for the inventory scope"
        );
        let mut available = HashSet::new();
        for thing in known_things() {
            available.insert(thing.shape);
        }
        for function in known_functions() {
            available.insert(function.input_shape);
            available.insert(function.output_shape);
        }
        let mut targets = Vec::new();
        let mut unknown = Vec::new();
        let mut names = self.target;
        names.sort();
        names.dedup();
        for name in names {
            cancellation.bail_if_cancelled()?;
            ensure!(!name.trim().is_empty(), "target type names cannot be empty");
            let matches = available
                .iter()
                .copied()
                .filter(|shape| describe_shape(shape) == name)
                .collect::<Vec<_>>();
            ensure!(
                matches.len() <= 1,
                "target type name {name:?} is ambiguous in the linked registry"
            );
            if let Some(shape) = matches.first() {
                targets.push(*shape);
            } else {
                unknown.push(name);
            }
        }
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
        let mut plan = DiscoveryPlan::build(known_functions(), &seeds, &targets, cancellation)?;
        for name in unknown {
            plan.diagnostics.push(format!("Target {name} has no linked type registration or request signature. It cannot be planned until registered."));
            plan.nodes.push(DiscoveryPlanNode {
                id: plan.nodes.len(),
                label: name,
                kind: DiscoveryPlanNodeKind::Shape,
                reachable: false,
                seed: false,
                target: true,
                level: 0,
                missing_inputs: Vec::new(),
                effects: Vec::new(),
            });
        }
        plan.diagnostics.push("This is a structural plan, not an execution schedule: collections and optional fields may be empty, permissions may fail, and a live runner must preserve organization/project/tenant provenance.".into());
        if matches!(
            auth_source,
            AuthSource::Auto | AuthSource::WorkloadIdentity | AuthSource::Browser
        ) {
            plan.diagnostics.push("Authentication is a declared input assumption only; source resolution, tenant configuration, and credentials have not been checked.".into());
        }
        if self.tenant.is_some() && auth_source == AuthSource::PersonalAccessToken {
            plan.diagnostics.push("A DevOps PAT cannot supply an Azure tenant authentication context; Azure inventory remains blocked.".into());
        }
        let report = DumpPlanReport {
            organization: self.org,
            tenant: self.tenant,
            auth_source,
            plan,
            diagram: self.diagram,
        };
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
