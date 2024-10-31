use anyhow::anyhow;
use anyhow::Result;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::get_imports_from_existing;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use cloud_terrastodon_core_user_input::prelude::prompt_line;

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
        return Err(anyhow!("Imports should not be empty"));
    }

    TofuWriter::new(AppDir::Imports.join(name))
        .overwrite(imports)
        .await?;

    Ok(())
}
