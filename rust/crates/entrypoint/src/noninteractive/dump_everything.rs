use cloud_terrastodon_core_azure_devops::prelude::{fetch_all_azure_devops_projects, fetch_all_azure_devops_repos_for_project};
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::{TofuImportBlock, TofuWriter};
use tracing::info;

use crate::{interactive::prelude::{clean_imports, clean_processed, init_processed, plan_processed}, noninteractive::prelude::{perform_import, process_generated}};

pub async fn dump_everything() -> eyre::Result<()> {
    info!("Clean up previous runs");
    clean_imports().await?;
    clean_processed().await?;

    info!("List the projects");
    let projects = fetch_all_azure_devops_projects().await?;
    for project in &projects {
        info!("Found: {}", project.name);
    }

    info!("Create the repo import blocks");
    let mut repo_import_blocks = Vec::new();
    for project in &projects {
        info!("Fetching repos for project {:?}", project.name);
        let repos = fetch_all_azure_devops_repos_for_project(&project.id).await?;
        for repo in repos {
            let import_block: TofuImportBlock = repo.into();
            repo_import_blocks.push(import_block);
        }
    }

    info!("Create the project import blocks");
    let mut project_import_blocks = Vec::new();
    for project in projects {
        let import_block: TofuImportBlock = project.into();
        project_import_blocks.push(import_block);
    }

    info!("Write the import blocks to disk");
    let project_imports_path = AppDir::Imports.join("azure_devops_project_imports.tf");
    TofuWriter::new(project_imports_path.clone())
        .overwrite(project_import_blocks)
        .await?
        .format()
        .await?;


    let repos_imports_path = AppDir::Imports.join("azure_devops_repos_imports.tf");
    TofuWriter::new(repos_imports_path.clone())
        .overwrite(repo_import_blocks)
        .await?
        .format()
        .await?;


    info!("Print the path");
    info!("The import blocks were written to {}", project_imports_path.display());

    info!("Perform the import");
    perform_import().await?;

    info!("Make it pretty");
    process_generated().await?;

    info!("Done!");
    info!("The output is available at {}", AppDir::Processed.as_path_buf().display());

    info!("Make sure there is no drift");
    init_processed().await?;
    plan_processed().await?;

    return Ok(());
}