use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Context;
use eyre::Result;
use tokio::fs::remove_dir_all;
use tracing::info;

pub async fn azure_devops_project_import_wizard_menu() -> Result<()> {
    info!("Confirming remove existing imports");
    let start_from_scratch = "start from scratch";
    let keep_existing_imports = "keep existing imports";
    match PickerTui::new(vec![start_from_scratch, keep_existing_imports])
        .set_header("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?")
        .pick_one()? {
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

    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let projects: Vec<cloud_terrastodon_azure_devops::prelude::AzureDevOpsProject> =
        PickerTui::new(projects.into_iter().map(|project| Choice {
            key: project.name.to_string(),
            value: project,
        }))
        .set_header("Choose the projects to import")
        .pick_many()?;

    let mut project_import_blocks = Vec::new();
    for project in projects {
        let import_block: HCLImportBlock = project.into();
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
