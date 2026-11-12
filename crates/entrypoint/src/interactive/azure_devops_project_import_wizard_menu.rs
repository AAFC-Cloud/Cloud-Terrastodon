use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Context;
use eyre::Result;
use std::borrow::Cow;
use tokio::fs::remove_dir_all;
use tracing::info;

pub async fn azure_devops_project_import_wizard_menu(
    org_url: &AzureDevOpsOrganizationUrl,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    info!("Confirming remove existing imports");
    let start_from_scratch = "start from scratch";
    let keep_existing_imports = "keep existing imports";
    match PickerTui::<_>::new()
        .set_header("This will wipe any existing imports from the Cloud Terrastodon work directory. Proceed?")
        .pick_one(vec![start_from_scratch, keep_existing_imports], executor.cancellation()).await? {
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

    let projects = AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_selection,
    }
    .execute(executor)
    .await?;
    let projects = PickerTui::<_>::new()
        .set_header("Choose the projects to import")
        .pick_many(
            projects.iter().cloned().map(|project| Choice {
                key: project.name.to_string(),
                value: project,
            }),
            executor.cancellation(),
        )
        .await?;

    let mut project_import_blocks = Vec::new();
    for project in projects.iter() {
        let import_block: HclImportBlock = project.as_ref().into();
        project_import_blocks.push(import_block);
    }

    let tf_file_path = AppDir::Imports.join("azure_devops_project_imports.tf");
    HclWriter::new(tf_file_path.clone())
        .overwrite(project_import_blocks, cancellation)
        .await?
        .format_file(cancellation)
        .await?;

    info!("Opening written imports in VSCode");
    CommandBuilder::new(CommandKind::VSCode)
        .cancellation_safe()
        .args([tf_file_path.as_os_str()])
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw(cancellation)
        .await
        .wrap_err("running vscode command")?;

    Ok(())
}
