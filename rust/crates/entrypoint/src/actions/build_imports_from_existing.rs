use crate::read_line::read_line;
use anyhow::anyhow;
use anyhow::Result;
use pathing::AppDir;
use tofu::prelude::get_imports_from_existing;
use tofu::prelude::TofuWriter;

pub async fn build_imports_from_existing() -> Result<()> {
    println!("Enter the path to the existing workspace:");
    let tf_dir = read_line().await?;

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
