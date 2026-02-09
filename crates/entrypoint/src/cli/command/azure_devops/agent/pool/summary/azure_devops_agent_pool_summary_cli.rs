use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsAgentPoolEntitlement;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsAgentPoolId;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pools;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use std::collections::HashMap;
use std::collections::HashSet;

/// Print a summary of agent pools and projects that belong to each pool.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolSummaryArgs {}

impl AzureDevOpsAgentPoolSummaryArgs {
    pub async fn invoke(self) -> Result<()> {
        // Organization and caches
        let org_url = get_default_organization_url().await?;

        // Fetch pools and projects once
        let pools = fetch_azure_devops_agent_pools(&org_url).await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;

        // Map project id -> project name and record project ids
        let mut project_map: HashMap<_, String> = HashMap::with_capacity(projects.len());
        let mut project_ids: Vec<AzureDevOpsProjectId> = Vec::with_capacity(projects.len());
        for p in projects {
            project_ids.push(p.id.clone());
            project_map.insert(p.id, p.name.to_string());
        }

        // Build pool -> project ids map by fetching entitlements once per project in parallel
        let mut pool_projects: HashMap<AzureDevOpsAgentPoolId, HashSet<AzureDevOpsProjectId>> =
            HashMap::new();

        // Enqueue project entitlement fetches in parallel
        let mut work: ParallelFallibleWorkQueue<(
            AzureDevOpsProjectId,
            Vec<AzureDevOpsAgentPoolEntitlement>,
        )> = ParallelFallibleWorkQueue::new("fetching project agent pool entitlements", 8);
        for pid in project_ids.iter() {
            let pid = pid.clone();
            let org_clone = org_url.clone();
            work.enqueue(async move {
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_project(&org_clone, pid.clone())
                        .await?;
                Ok((pid, entitlements))
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

        for pool in pools {
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
