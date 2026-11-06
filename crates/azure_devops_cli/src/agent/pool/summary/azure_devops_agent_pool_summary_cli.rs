use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolEntitlement;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pools;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// Print a summary of agent pools and projects that belong to each pool.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolSummaryArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsAgentPoolSummaryArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        // Fetch pools and projects once
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let pools =
            fetch_azure_devops_agent_pools(&self.org, &azure_devops_auth_context, executor).await?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context, executor)
                .await?;

        // Map project id -> project name and record project ids
        let mut project_map: HashMap<_, String> = HashMap::with_capacity(projects.len());
        let mut project_ids: Vec<AzureDevOpsProjectId> = Vec::with_capacity(projects.len());
        for p in projects {
            project_ids.push(p.id.clone());
            project_map.insert(p.id.clone(), p.name.to_string());
        }

        // Build pool -> project ids map by fetching entitlements once per project in parallel
        let mut pool_projects: HashMap<AzureDevOpsAgentPoolId, HashSet<AzureDevOpsProjectId>> =
            HashMap::new();

        // Enqueue project entitlement fetches in parallel
        let mut work: ParallelFallibleWorkQueue<(
            AzureDevOpsProjectId,
            Vec<Arc<AzureDevOpsAgentPoolEntitlement>>,
        )> = ParallelFallibleWorkQueue::new("fetching project agent pool entitlements", 8);
        for pid in project_ids.iter() {
            let pid = pid.clone();
            let org_clone = self.org.clone();
            let azure_devops_auth_context = azure_devops_auth_context.clone();
            work.enqueue({
                executor.cancellable_scope(move |executor| async move {
                    let executor = &executor;
                    let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
                        &org_clone,
                        pid.clone(),
                        &azure_devops_auth_context,
                        executor,
                    )
                    .await?;
                    Ok((pid, entitlements))
                })
            });
        }

        for (pid, entitlements) in work.join().await?.into_iter() {
            for e in entitlements.into_iter() {
                pool_projects
                    .entry(e.pool.id)
                    .or_default()
                    .insert(pid.clone());
            }
        }

        println!(
            "{}",
            format!("Found {} agent pools", pools.len()).yellow().bold()
        );

        for pool in pools.iter() {
            println!("{}", "────────────────────────────────────────".dimmed());
            println!("{} {}", "Pool:".cyan().bold(), pool.name.cyan().bold());
            println!("{} {}", "Pool ID:".cyan().bold(), pool.id);

            // Lookup projects from the precomputed map and apply special cases
            match pool_projects.get(&pool.id) {
                None => {
                    println!("{}", "Projects: None".dimmed());
                }
                Some(set) if set.is_empty() => {
                    println!("{}", "Projects: None".dimmed());
                }
                Some(set) => {
                    let total = project_ids.len();
                    let count = set.len();

                    if count == total {
                        // Present in all projects
                        println!("{} {}", "Projects:".cyan().bold(), "ALL".green().bold());
                    } else if total - count <= 5 {
                        // In all-but-5-or-fewer projects: list the excluded projects
                        let mut excluded: Vec<(String, String)> = project_ids
                            .iter()
                            .filter(|pid| !set.contains(pid))
                            .map(|pid| {
                                let name = project_map
                                    .get(pid)
                                    .cloned()
                                    .unwrap_or_else(|| pid.to_string());
                                (name, pid.to_string())
                            })
                            .collect();
                        excluded.sort_by(|a, b| a.0.cmp(&b.0));

                        println!(
                            "{}",
                            format!("Excluded Projects ({}):", excluded.len())
                                .yellow()
                                .bold()
                        );
                        for (name, id) in excluded {
                            println!("  - {} ({})", name.green().bold(), id.dimmed());
                        }
                    } else {
                        // Default: list included projects
                        let mut included: Vec<(String, String)> = set
                            .iter()
                            .map(|pid| {
                                let name = project_map
                                    .get(pid)
                                    .cloned()
                                    .unwrap_or_else(|| pid.to_string());
                                (name, pid.to_string())
                            })
                            .collect();
                        included.sort_by(|a, b| a.0.cmp(&b.0));

                        println!(
                            "{}",
                            format!("Projects ({}):", included.len()).yellow().bold()
                        );
                        for (name, id) in included {
                            println!("  - {} ({})", name.green().bold(), id.dimmed());
                        }
                    }
                }
            }

            println!();
        }

        Ok(())
    }
}
