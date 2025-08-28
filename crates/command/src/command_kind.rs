use crate::CacheBehaviour;
use crate::CommandBuilder;
pub use bstr;
use bstr::ByteSlice;
use cloud_terrastodon_config::CommandsConfig;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use tempfile::TempPath;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::OnceCell;
use tracing::debug;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub enum CommandKind {
    #[default]
    AzureCLI,
    Terraform,
    VSCode,
    Echo,
    Pwsh,
    Git,
    Other(String),
}

async fn get_config(cache: &OnceCell<CommandsConfig>) -> &CommandsConfig {
    let config: &CommandsConfig = cache
        .get_or_init(|| async {
            let config: CommandsConfig = CommandsConfig::load().await.unwrap();
            config
        })
        .await;
    config
}

pub const USE_TOFU_FLAG_KEY: &str = "CLOUD_TERRASTODON_USE_TOFU";

static CONFIG: OnceCell<CommandsConfig> = OnceCell::const_new();

impl CommandKind {
    pub async fn program(&self) -> String {
        match self {
            CommandKind::AzureCLI => get_config(&CONFIG).await.azure_cli.to_owned(),
            CommandKind::Terraform => match env::var(USE_TOFU_FLAG_KEY) {
                Err(_) => get_config(&CONFIG).await.terraform.to_owned(),
                Ok(_) => get_config(&CONFIG).await.tofu.to_owned(),
            },
            CommandKind::VSCode => get_config(&CONFIG).await.vscode.to_owned(),
            CommandKind::Echo => "pwsh".to_string(),
            CommandKind::Pwsh => "pwsh".to_string(),
            CommandKind::Git => "git".to_string(),
            CommandKind::Other(x) => x.to_owned(),
        }
    }
    pub async fn apply_args_and_envs(
        &self,
        this: &CommandBuilder,
        cmd: &mut Command,
    ) -> Result<Vec<TempPath>> {
        let mut rtn = Vec::new();
        let mut args = this.args.clone();
        // Always add --debug for AzureCLI if not present
        if let CommandKind::AzureCLI = self {
            let has_debug = args.iter().any(|a| a == "--debug");
            if !has_debug {
                args.push("--debug".into());
            }
        }
        // Write azure args to files
        match (self, this.file_args.is_empty()) {
            (CommandKind::AzureCLI, false) => {
                // todo: add tests
                for (i, arg) in this.file_args.iter() {
                    debug!("Writing arg {}", arg.path.to_string_lossy());
                    let mut patch_arg = async |i: usize, file_path: &PathBuf| -> Result<()> {
                        // Get the arg from the array
                        // We are converting @myfile.txt to @/path/to/myfile.txt
                        let arg_to_update =
                            args.get_mut(i).expect("azure arg must match an argument");

                        // Check assumption - it should already begin with an @
                        let check = arg_to_update.to_string_lossy();
                        let first_char = check.chars().next().unwrap();
                        if first_char != '@' {
                            bail!(
                                "First character in file arg for {:?} must be '@', got {}",
                                this.kind,
                                check
                            )
                        }

                        // Write the file
                        let mut file = OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(&file_path)
                            .await
                            .context(format!("Opening azure arg file {}", file_path.display()))?;
                        file.write_all(arg.content.as_bytes())
                            .await
                            .context(format!("Writing azure arg file {}", file_path.display()))?;

                        // Update the value
                        arg_to_update.clear();
                        arg_to_update.push("@");
                        arg_to_update.push(file_path.canonicalize().context(
                            "azure arg file must be written before absolute path can be determined",
                        )?);
                        Ok(())
                    };
                    let mut file = match &this.cache_behaviour {
                        CacheBehaviour::Some {
                            path: cache_dir, ..
                        } => {
                            // Cache dir has been provided
                            // we won't use temp files
                            cache_dir.ensure_dir_exists().await?;
                            let file_path = cache_dir.join(&arg.path);
                            patch_arg(*i, &file_path).await?;
                            tokio::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&file_path)
                                .await
                                .context(format!(
                                    "opening azure arg file {}",
                                    arg.path.to_string_lossy()
                                ))?
                        }
                        CacheBehaviour::None => {
                            // No cache dir
                            // We will write azure args to temp files
                            let temp_dir = AppDir::Temp.as_path_buf();
                            temp_dir.ensure_dir_exists().await?;
                            let path = tempfile::Builder::new()
                                .suffix(&arg.path)
                                .tempfile_in(temp_dir)
                                .context(format!(
                                    "creating temp file {}",
                                    arg.path.to_string_lossy()
                                ))?
                                .into_temp_path();
                            patch_arg(*i, &path.to_path_buf()).await?;
                            let file = tokio::fs::OpenOptions::new()
                                .write(true)
                                .open(&path)
                                .await
                                .context(format!(
                                    "opening azure arg file {}",
                                    arg.path.to_string_lossy()
                                ))?;
                            rtn.push(path); // add to rtn list so its not dropped+cleaned immediately
                            file
                        }
                    };
                    file.write_all(arg.content.as_bytes())
                        .await
                        .context(format!(
                            "writing azure arg file {}",
                            arg.path.to_string_lossy()
                        ))?;
                }
            }
            (_, false) => {
                bail!("Only {:?} can use Azure args", CommandKind::AzureCLI);
            }
            (CommandKind::Echo, true) => {
                let mut new_args: Vec<OsString> = Vec::with_capacity(3);
                new_args.push("-NoProfile".into());
                new_args.push("-Command".into());
                // new_args.push("echo".into());
                let mut guh = OsString::new();
                guh.push("[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new();'");
                let space: OsString = " ".into();
                guh.push(
                    args.join(&space)
                        .as_encoded_bytes()
                        .replace(b"'", b"''")
                        .to_os_str()?,
                );
                guh.push("'");
                new_args.push(guh);
                args = new_args;
            }
            (_, true) => {}
        }
        // Apply args and envs to tokio Command
        cmd.args(args);
        cmd.envs(&this.env);
        Ok(rtn)
    }
}
