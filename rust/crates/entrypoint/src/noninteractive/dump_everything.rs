use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_repos_batch;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use tracing::info;

use crate::interactive::prelude::clean_imports;
use crate::interactive::prelude::clean_processed;
use crate::interactive::prelude::init_processed;
use crate::interactive::prelude::plan_processed;
use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;

pub async fn dump_everything() -> eyre::Result<()> {
    info!("Clean up previous runs");
    _ = clean_imports().await;
    _ = clean_processed().await;

    info!("List the projects");
    let projects = fetch_all_azure_devops_projects().await?;
    for project in &projects {
        info!("Found: {}", project.name);
    }

    info!("Create the repo import blocks");
    let mut repo_import_blocks = Vec::new();
    let repos =
        fetch_azure_devops_repos_batch(projects.iter().map(|project| project.id.clone()).collect())
            .await?
            .into_iter()
            .flat_map(|(_project_id, repos)| repos.into_iter());

    for repo in repos {
        let import_block: TofuImportBlock = repo.into();
        repo_import_blocks.push(import_block);
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
    info!(
        "The import blocks were written to {}",
        project_imports_path.display()
    );

    info!("Perform the import");
    perform_import().await?;

    info!("Make it pretty");
    process_generated().await?;

    info!("Done!");
    info!(
        "The output is available at {}",
        AppDir::Processed.as_path_buf().display()
    );

    info!("Make sure there is no drift");
    init_processed().await?;
    plan_processed().await?;

    return Ok(());
}
