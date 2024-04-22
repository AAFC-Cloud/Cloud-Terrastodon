use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use azure::prelude::fetch_management_groups;
use azure::prelude::fetch_policy_assignments;
use azure::prelude::fetch_policy_definitions;
use azure::prelude::fetch_policy_initiatives;
use azure::prelude::PolicyAssignment;
use azure::prelude::PolicyDefinition;
use azure::prelude::PolicyInitiative;
use fzf::pick;
use fzf::FzfArgs;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use itertools::Itertools;
use std::path::PathBuf;
use tf::prelude::*;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;

pub async fn import_policy() -> Result<()> {
    // Fetch management groups
    println!("Fetching management groups...");
    let management_groups = fetch_management_groups().await?;

    // Pick management groups to import from
    let management_groups = pick(FzfArgs {
        choices: management_groups,
        many: true,
        prompt: Some("policy import > ".to_string()),
        header: Some("Management Groups".to_string()),
    })?;

    // Prepare progress indicators
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("💤😴🥱😪🎂");
    let m = MultiProgress::new();

    // Prepare work pool
    #[allow(clippy::enum_variant_names)]
    enum WorkResult {
        PolicyDefinitions {
            policy_definitions: Vec<PolicyDefinition>,
        },
        PolicyAssignments {
            policy_assignments: Vec<PolicyAssignment>,
        },
        PolicyInitiatives {
            policy_initiatives: Vec<PolicyInitiative>,
        },
    }
    let mut work_pool = JoinSet::new();

    // Fetch info from each management group
    println!("Fetching info from management groups...");
    for management_group in management_groups.iter() {
        // Prepare progress indicator
        let mg = management_group.clone();
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());
        pb.set_message(format!("{} - {}", "policy definitions", mg.display_name));

        // Launch background worker
        work_pool.spawn(async move {
            // Fetch policy definitions
            let result = fetch_policy_definitions(Some(mg.name.clone()), None).await;

            // Update progress indicator
            pb.inc(1);

            // Return results
            (
                mg,
                pb,
                result
                    .map(|policy_definitions| WorkResult::PolicyDefinitions { policy_definitions }),
            )
        });

        // Prepare progress indicator
        let mg = management_group.clone();
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());
        pb.set_message(format!("{} - {}", "policy assignments", mg.display_name));

        // Launch background worker
        work_pool.spawn(async move {
            // Fetch policy definitions
            let result = fetch_policy_assignments(Some(&mg), None).await;

            // Update progress indicator
            pb.inc(1);

            // Return results
            (
                mg,
                pb,
                result
                    .map(|policy_assignments| WorkResult::PolicyAssignments { policy_assignments }),
            )
        });

        // Prepare progress indicator
        let mg = management_group.clone();
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());
        pb.set_message(format!("{} - {}", "policy initiatives", mg.display_name));

        // Launch background worker
        work_pool.spawn(async move {
            // Fetch policy definitions
            let result = fetch_policy_initiatives(Some(mg.name.clone()), None).await;

            // Update progress indicator
            pb.inc(1);

            // Return results
            (
                mg,
                pb,
                result
                    .map(|policy_initiatives| WorkResult::PolicyInitiatives { policy_initiatives }),
            )
        });
    }

    // Collect worker results
    let mut imports = Vec::<ImportBlock>::new();
    while let Some(res) = work_pool.join_next().await {
        // Get result if worker success
        let (mg, pb, result) = res?;
        let result = result?;
        let mut results: Vec<ImportBlock> = match result {
            WorkResult::PolicyDefinitions { policy_definitions } => policy_definitions
                .into_iter()
                .filter(|def| def.policy_type == "Custom")
                .map(|x| x.into())
                .collect_vec(),
            WorkResult::PolicyAssignments { policy_assignments } => policy_assignments
                .into_iter()
                .map(|x| x.into())
                .map(|x: ImportBlock| ImportBlock {
                    id: x.id,
                    to: ResourceIdentifier {
                        kind: x.to.kind,
                        name: format!("{}_{}", x.to.name, mg.display_name.sanitize()),
                    },
                })
                .collect_vec(),
            WorkResult::PolicyInitiatives { policy_initiatives } => policy_initiatives
                .into_iter()
                .filter(|def| def.policy_type == "Custom")
                .map(|x| x.into())
                .collect_vec(),
        };

        // Update progress indicator
        pb.finish_with_message(format!(
            "Found {} things to import from {}",
            results.len(),
            mg.display_name
        ));

        // Add to list
        imports.append(&mut results);
    }

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    // Prepare imports dir
    let imports_dir = PathBuf::from("ignore").join("imports");
    if !imports_dir.exists() {
        println!("Creating {:?}", imports_dir);
        create_dir_all(&imports_dir).await?;
    } else if !imports_dir.is_dir() {
        return Err(anyhow!("Path exists but isn't a dir!"))
            .context(imports_dir.to_string_lossy().into_owned());
    }

    // Write imports.tf
    let imports_path = imports_dir.join("imports.tf");
    let mut imports_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&imports_path)
        .await?;
    println!("Writing {:?}", imports_path);
    imports_file.write_all(imports.as_tf().as_bytes()).await?;

    // not necessary if capturing terraform output
    // // Double check that we are logged in before running tf command
    // // Previous commands may have used cached results
    // // Capturing tf output while also sending to console to detect
    // // login failures for auto-retry is not yet implemented
    // if !is_logged_in().await {
    //     println!("You aren't logged in! Running login command...");
    //     login().await?;
    // }

    // Run tf import
    println!("Beginning tf import...");
    TFImporter::default().using_dir(imports_dir).run().await?;

    Ok(())
}
