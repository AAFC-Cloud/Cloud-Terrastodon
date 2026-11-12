use chrono::Local;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::HclImportable;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::GenerateConfigOutHelper;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
use cloud_terrastodon_hcl::reflow::ReflowHclRequest;
use cloud_terrastodon_pathing::Existy;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tempfile::Builder;
use tracing::info;

/// Create Terraform import definitions for selected resources.
#[derive(Facet, Debug, Clone)]
pub struct TerraformSourceGenerateArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    #[facet(figue::named, default = PathBuf::from("."))]
    pub work_dir: PathBuf,
}

impl TerraformSourceGenerateArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let cancellation = executor.cancellation();
        let authentication = self.authentication.resolve(executor).await?;
        let work_dir = self.work_dir;

        let kind_to_import = HclImportable::pick(cancellation).await?;
        let imports = kind_to_import
            .pick_into_body(&authentication, executor)
            .await?;

        work_dir.ensure_dir_exists().await?;
        let temp_dir = Builder::new()
            .prefix(&format!(
                "generated_{}",
                Local::now().format("%Y%m%d_%H%M%S_")
            ))
            .suffix(".tf")
            .tempdir_in(&work_dir)?;
        let import_dir = temp_dir.keep();

        HclWriter::new(import_dir.join("imports.tf"))
            .format_on_write()
            .overwrite(imports, cancellation)
            .await?;

        GenerateConfigOutHelper::new()
            .with_run_dir(&import_dir)
            .run(cancellation)
            .await?;

        info!("Reflowing content");
        let hcl = discover_hcl(import_dir, DiscoveryDepth::Shallow).await?;
        let hcl = ReflowHclRequest::new(hcl, Some(authentication))
            .include_principal_id_comments(true)
            .execute(executor)
            .await?;
        for (path, contents) in hcl {
            HclWriter::new(path)
                .format_on_write()
                .overwrite(contents, cancellation)
                .await?;
        }

        Ok(())
    }
}
