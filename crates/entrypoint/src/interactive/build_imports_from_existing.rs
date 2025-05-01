use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_hcl::prelude::get_imports_from_existing;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;
use eyre::eyre;

pub async fn build_imports_from_existing() -> Result<()> {
    let tf_dir = prompt_line("Enter the path to the existing workspace: ").await?;

    // println!("Enter the name for the output file [existing.tf]:");
    // let mut name = read_line().await.unwrap_or_default();
    // if name.is_empty() {
    //     name = "existing.tf".to_string();
    // }
    let name = "existing.tf";

    let imports = get_imports_from_existing(tf_dir).await?;
    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    HCLWriter::new(AppDir::Imports.join(name))
        .overwrite(imports)
        .await?;

    Ok(())
}
