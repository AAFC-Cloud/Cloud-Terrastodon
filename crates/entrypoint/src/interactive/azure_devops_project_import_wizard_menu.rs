use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use cloud_terrastodon_user_input::pick_many;
use eyre::Context;
use eyre::Result;
use itertools::Itertools;
use tokio::fs::remove_dir_all;
use tracing::info;

pub async fn azure_devops_project_import_wizard_menu() -> Result<()> {
    info!("Confirming remove existing imports");
    let start_from_scratch = "start from scratch";
    let keep_existing_imports = "keep existing imports";
    match pick(FzfArgs {
        choices: vec![start_from_scratch, keep_existing_imports],
        header: Some("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?".to_string()),
        ..Default::default()
    })? {
        x if x == start_from_scratch => {
            info!("Removing existing imports");
            let _ = remove_dir_all(AppDir::Imports.as_path_buf()).await;
            let _ = remove_dir_all(AppDir::Processed.as_path_buf()).await;
        }
        x if x == keep_existing_imports => {
            info!("Keeping existing imports");
        }
        _ => unreachable!(),
    }

    let projects = fetch_all_azure_devops_projects().await?;
    let projects = pick_many(FzfArgs {
        choices: projects
            .into_iter()
            .map(|project| Choice {
                key: project.name.to_string(),
                value: project,
            })
            .collect_vec(),
        header: Some("Choose the projects to import".to_string()),
        ..Default::default()
    })?;

    let mut project_import_blocks = Vec::new();
    for project in projects {
        let import_block: HCLImportBlock = project.value.into();
        project_import_blocks.push(import_block);
    }

    let tf_file_path = AppDir::Imports.join("azure_devops_project_imports.tf");
    HCLWriter::new(tf_file_path.clone())
        .overwrite(project_import_blocks)
        .await?
        .format_file()
        .await?;

    info!("Opening written imports in VSCode");
    CommandBuilder::new(CommandKind::VSCode)
        .args([tf_file_path.as_os_str()])
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await
        .wrap_err("running vscode command")?;

    Ok(())
}
