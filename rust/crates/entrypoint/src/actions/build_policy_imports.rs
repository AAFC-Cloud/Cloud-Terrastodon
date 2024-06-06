use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_management_groups;
use azure::prelude::fetch_policy_assignments;
use azure::prelude::fetch_policy_definitions;
use azure::prelude::fetch_policy_set_definitions;
use azure::prelude::ManagementGroup;
use azure::prelude::PolicyAssignment;
use azure::prelude::PolicyDefinition;
use azure::prelude::PolicySetDefinition;
use azure::prelude::ScopeImpl;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use itertools::Itertools;
use pathing_types::IgnoreDir;
use std::sync::Arc;
use std::sync::Mutex;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuWriter;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

#[allow(clippy::enum_variant_names)]
enum WorkResult {
    PolicyDefinitions {
        management_group: ManagementGroup,
        policy_definitions: Result<Vec<PolicyDefinition>>,
    },
    PolicyAssignments {
        management_group: ManagementGroup,
        policy_assignments: Result<Vec<PolicyAssignment>>,
    },
    PolicySetDefinitons {
        management_group: ManagementGroup,
        policy_set_definitions: Result<Vec<PolicySetDefinition>>,
    },
}

fn fetch_policy_definitions_for_management_group(
    management_group: ManagementGroup,
    work_pool: &mut JoinSet<WorkResult>,
    pb: Arc<Mutex<ProgressBar>>,
) {
    work_pool.spawn(async move {
        // Fetch policy definitions
        let policy_definitions = fetch_policy_definitions(
            Some(ScopeImpl::ManagementGroup(management_group.id.clone())),
            None,
        )
        .await;

        // Update progress indicator
        let _ = pb.lock().map(|pb| pb.inc(1));

        // Return results
        WorkResult::PolicyDefinitions {
            policy_definitions,
            management_group,
        }
    });
}

fn fetch_policy_set_definitions_for_management_group(
    management_group: ManagementGroup,
    work_pool: &mut JoinSet<WorkResult>,
    pb: Arc<Mutex<ProgressBar>>,
) {
    work_pool.spawn(async move {
        let policy_set_definitions = fetch_policy_set_definitions(
            Some(ScopeImpl::ManagementGroup(management_group.id.clone())),
            None,
        )
        .await;

        let _ = pb.lock().map(|pb| pb.inc(1));

        WorkResult::PolicySetDefinitons {
            management_group,
            policy_set_definitions,
        }
    });
}
fn fetch_policy_assignments_for_management_group(
    management_group: ManagementGroup,
    work_pool: &mut JoinSet<WorkResult>,
    pb: Arc<Mutex<ProgressBar>>,
) {
    work_pool.spawn(async move {
        let policy_assignments = fetch_policy_assignments(
            Some(ScopeImpl::ManagementGroup(management_group.id.clone())),
            None,
        )
        .await;

        let _ = pb.lock().map(|pb| pb.inc(1));

        WorkResult::PolicyAssignments {
            management_group,
            policy_assignments,
        }
    });
}

pub async fn build_policy_imports() -> Result<()> {
    info!("Fetching management groups...");
    let management_groups = fetch_management_groups().await?;

    debug!("Preparing progress indicators");
    let pb = ProgressBar::new(management_groups.len() as u64 * 3);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:30.cyan/blue} {pos:>7}/{len:7} {msg}")?,
    );
    let pb = Arc::new(Mutex::new(pb));

    debug!("Preparing work pool");
    let mut work_pool: JoinSet<WorkResult> = JoinSet::new();

    info!("Launching fetch workers");
    for management_group in management_groups.iter() {
        fetch_policy_definitions_for_management_group(
            management_group.clone(),
            &mut work_pool,
            pb.clone(),
        );
        fetch_policy_set_definitions_for_management_group(
            management_group.clone(),
            &mut work_pool,
            pb.clone(),
        );
        fetch_policy_assignments_for_management_group(
            management_group.clone(),
            &mut work_pool,
            pb.clone(),
        );
    }

    info!("Collecting results");
    let mut imports = Vec::<TofuImportBlock>::new();
    while let Some(res) = work_pool.join_next().await {
        let work = res?;
        let (management_group, mut results) = match work {
            WorkResult::PolicyDefinitions {
                management_group,
                policy_definitions,
            } => (
                management_group,
                policy_definitions?
                    .into_iter()
                    .filter(|def| def.policy_type == "Custom")
                    .map(|x| x.into())
                    .collect_vec(),
            ),
            WorkResult::PolicyAssignments {
                management_group,
                policy_assignments,
            } => {
                let mg_name = management_group.display_name.sanitize();
                (
                    management_group,
                    policy_assignments?
                        .into_iter()
                        .map(|x| x.into())
                        .map(|x: TofuImportBlock| {
                            let provider = x.provider;
                            let id = x.id;
                            let mut to = x.to;
                            // update to include management group name as suffix
                            to.use_name(|name| format!("{}_{}", name, mg_name));
                            TofuImportBlock { provider, id, to }
                        })
                        .collect_vec(),
                )
            }
            WorkResult::PolicySetDefinitons {
                management_group,
                policy_set_definitions,
            } => (
                management_group,
                policy_set_definitions?
                    .into_iter()
                    .filter(|def| def.policy_type == "Custom")
                    .map(|x| x.into())
                    .collect_vec(),
            ),
        };

        // Update progress indicator
        let _ = pb.lock().map(|pb| {
            pb.set_message(format!(
                "Found {} things to import from {}",
                results.len(),
                management_group.display_name
            ))
        });

        // Add to list
        imports.append(&mut results);
    }

    let _ = pb.lock().map(|pb| {
        pb.finish_with_message(format!(
            "Obtained {} import blocks from {} management groups.",
            imports.len(),
            management_groups.len()
        ))
    });

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    TofuWriter::new(IgnoreDir::Imports.join("policy_imports.tf"))
        .overwrite(imports)
        .await?;

    Ok(())
}
