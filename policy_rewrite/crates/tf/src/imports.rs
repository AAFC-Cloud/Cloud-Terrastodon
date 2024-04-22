use anyhow::anyhow;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use indoc::formatdoc;
use indoc::indoc;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct ResourceType {
    pub provider: String,
    pub kind: String,
}
impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.provider)?;
        f.write_str("_")?;
        f.write_str(&self.kind)
    }
}

#[derive(Debug)]
pub struct ResourceIdentifier {
    pub kind: ResourceType,
    pub name: String,
}
impl std::fmt::Display for ResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.kind.to_string())?;
        f.write_str(".")?;
        f.write_str(&self.name)
    }
}

#[derive(Debug)]
pub struct ImportBlock {
    pub id: String,
    pub to: ResourceIdentifier,
}

pub trait Sanitizable {
    fn sanitize(&self) -> String;
}

impl<T: AsRef<str>> Sanitizable for T {
    fn sanitize(&self) -> String {
        self.as_ref()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .skip_while(|c| *c == '_')
            .collect()
    }
}

pub trait AsTF {
    fn as_tf(&self) -> String;
}

impl AsTF for Vec<ImportBlock> {
    fn as_tf(&self) -> String {
        let mut rtn = String::new();
        let mut seen = HashSet::new();
        for import in self.iter() {
            if seen.contains(&import.id) {
                continue;
            } else {
                seen.insert(&import.id);
            }
            rtn.push_str(
                formatdoc! {
                    r#"
                        import {{
                            id = "{}"
                            to = {}
                        }}
                    "#,
                    import.id,
                    import.to
                }
                .as_str(),
            );
            rtn.push('\n');
        }

        rtn
    }
}

#[derive(Default)]
pub struct TFImporter {
    imports_dir: Option<PathBuf>,
}
impl TFImporter {
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
        let mut init_cmd = CommandBuilder::new(CommandKind::TF);
        init_cmd.should_announce(true);
        init_cmd.use_run_dir(imports_dir.clone());
        // init_cmd.use_output_behaviour(OutputBehaviour::Display);
        init_cmd.args(["init"]);
        init_cmd.run_raw().await?;
        println!("tf init successful!");

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
        let mut plan_cmd = CommandBuilder::new(CommandKind::TF);
        plan_cmd.should_announce(true);
        plan_cmd.use_run_dir(imports_dir.clone());
        // plan_cmd.use_output_behaviour(OutputBehaviour::Display);
        plan_cmd.args(["plan", "-generate-config-out", "generated.tf"]);
        plan_cmd.run_raw().await?;
        println!("tf plan successful!");

        // Success!
        println!("🚀 Successfully generated TF files from imports!");
        Ok(())
    }
}
