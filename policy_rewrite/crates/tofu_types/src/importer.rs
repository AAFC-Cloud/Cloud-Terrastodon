use anyhow::anyhow;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indoc::indoc;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct TofuImporter {
    imports_dir: Option<PathBuf>,
}
impl TofuImporter {
    pub fn using_dir(&mut self, imports_dir: impl AsRef<Path>) -> &mut Self {
        self.imports_dir = Some(imports_dir.as_ref().to_path_buf());
        self
    }
    pub async fn run(&mut self) -> Result<()> {
        // Check preconditions
        let Some(ref imports_dir) = self.imports_dir else {
            return Err(anyhow!("Dir must be set with using_dir"));
        };

        // Open boilerplate file
        let boilerplate_path = imports_dir.join("boilerplate.tf");
        let mut boilerplate_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&boilerplate_path)
            .await?;

        // Write boilerplate
        boilerplate_file
            .write_all(
                indoc! {r#"
                    provider "azurerm" {
                        features {}
                        skip_provider_registration = true
                    }
                "#}
                .as_bytes(),
            )
            .await?;

        // tf init
        let mut init_cmd = CommandBuilder::new(CommandKind::Tofu);
        init_cmd.should_announce(true);
        init_cmd.use_run_dir(imports_dir.clone());
        // init_cmd.use_output_behaviour(OutputBehaviour::Display);
        init_cmd.args(["init"]);
        init_cmd.run_raw().await?;
        println!("tofu init successful!");

        // remove old plan outputs
        let generated_path = imports_dir.join("generated.tf");
        if generated_path.exists() {
            if !generated_path.is_file() {
                return Err(anyhow!("generated output path exists but is not a file")
                    .context(generated_path.to_string_lossy().into_owned()));
            }
            fs::remove_file(generated_path).await?;
        }

        // tf plan
        let mut plan_cmd = CommandBuilder::new(CommandKind::Tofu);
        plan_cmd.should_announce(true);
        plan_cmd.use_run_dir(imports_dir.clone());
        // plan_cmd.use_output_behaviour(OutputBehaviour::Display);
        plan_cmd.args(["plan", "-generate-config-out", "generated.tf"]);
        plan_cmd.run_raw().await?;
        println!("tofu plan successful!");

        // Success!
        println!("🚀 Successfully generated tofu files from imports!");
        Ok(())
    }
}
