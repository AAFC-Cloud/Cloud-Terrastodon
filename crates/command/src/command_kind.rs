use crate::CommandArgument;
use crate::CommandBuilder;
pub use bstr;
use bstr::ByteSlice;
use cloud_terrastodon_config::CommandsConfig;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use std::collections::HashMap;
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
    CloudTerrastodon,
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
            CommandKind::CloudTerrastodon => {
                let current_exe = env::current_exe().unwrap_or_else(|_| "cloud_terrastodon".into());
                let is_test = current_exe
                    .parent()
                    .map(|parent| {
                        parent.ends_with(PathBuf::from_iter(["target", "debug", "deps"]))
                    })
                    .unwrap_or(false);
                if is_test {
                    // return target/debug/cloud_terrastodon.exe instead of target/debug/deps/cloud_terrastodon_123.exe used by cargo test
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    let mut path = PathBuf::from(manifest_dir);
                    path.pop();
                    path.pop();
                    path.push("target");
                    path.push("debug");
                    path.push("cloud_terrastodon");
                    #[cfg(windows)]
                    {
                        path.set_extension("exe");
                    }
                    path.to_string_lossy().to_string()
                } else {
                    // return current exe
                    env::current_exe()
                        .unwrap_or_else(|_| "cloud_terrastodon".into())
                        .to_string_lossy()
                        .to_string()
                }
            }
            CommandKind::Other(x) => x.to_owned(),
        }
    }
    pub async fn apply_args_and_envs(
        &self,
        this: &CommandBuilder,
        cmd: &mut Command,
    ) -> Result<Vec<TempPath>> {
        // Prepare list of temp paths to return
        let mut rtn = Vec::new();

        // Prepare args using a clone for idempotency
        let mut args = this.args.clone();

        // Special handling per CommandKind
        match self {
            CommandKind::Echo => {
                let mut new_args: Vec<OsString> = Vec::with_capacity(3);
                new_args.push("-NoProfile".into());
                new_args.push("-Command".into());
                // new_args.push("echo".into());
                let mut guh = OsString::new();
                guh.push("[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new();'");
                let space: OsString = " ".into();
                guh.push(
                    args.into_iter()
                        .map(OsString::from)
                        .collect::<Vec<_>>()
                        .join(&space)
                        .as_encoded_bytes()
                        .replace(b"'", b"''")
                        .to_os_str()?,
                );
                guh.push("'");
                new_args.push(guh);
                args = new_args.into_iter().map(CommandArgument::Literal).collect();
            }
            CommandKind::AzureCLI | CommandKind::CloudTerrastodon => {
                // Always add --debug if not present
                let has_debug = args
                    .iter()
                    .any(|a| matches!(a, CommandArgument::Literal(lit) if lit == "--debug"));
                if !has_debug {
                    args.push(CommandArgument::Literal("--debug".into()));
                }
            }
            _ => {}
        }

        // Write adjacent files
        let mut canonical_path_lookup: HashMap<PathBuf, PathBuf> = HashMap::new();
        for (adj_path, adj_content) in this.adjacent_files.iter() {
            let file_path = match &this.cache_key {
                Some(cache_key) => {
                    // Cache dir has been provided
                    let cache_dir = cache_key.path_on_disk();
                    cache_dir.ensure_dir_exists().await?;
                    cache_dir.join(adj_path)
                }
                None => {
                    // No cache dir has been provided
                    // We will write adjacent files to temp files
                    let temp_dir = AppDir::Temp.as_path_buf();
                    temp_dir.ensure_dir_exists().await?;
                    let path = tempfile::Builder::new()
                        .suffix(&adj_path)
                        .tempfile_in(temp_dir)
                        .context(format!("creating temp file {}", adj_path.to_string_lossy()))?
                        .into_temp_path();

                    let path_buf = path.to_path_buf();
                    rtn.push(path); // add to rtn list so its not dropped+cleaned immediately
                    path_buf
                }
            };

            debug!("Writing arg {}", file_path.display());
            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&file_path)
                .await
                .context(format!("Opening adjacent file {}", file_path.display()))?;
            file.write_all(adj_content.as_bytes())
                .await
                .context(format!("Writing adjacent file {}", file_path.display()))?;

            // canonicalize the path for transformation when passing as argument to command
            let file_path = file_path
                .canonicalize()
                .wrap_err_with(|| eyre!("Failed to canonicalize path {file_path:?}"))?;
            canonical_path_lookup.insert(adj_path.clone(), file_path.clone());
        }

        // Patch arguments to point to canonical paths
        for arg in args.iter_mut() {
            if let CommandArgument::DeferredAdjacentFilePath { key, mapper } = arg {
                let path_to_map = canonical_path_lookup
                    .get(key)
                    .ok_or_eyre("Adjacent file path not found in lookup")?;
                let mapped_path = mapper.map_path(path_to_map.as_path());
                *arg = CommandArgument::Literal(mapped_path.as_os_str().to_owned());
            }
        }

        // Apply args to tokio Command
        for arg in args {
            match arg {
                CommandArgument::Literal(lit) => {
                    cmd.arg(lit);
                }
                CommandArgument::DeferredAdjacentFilePath { .. } => {
                    // Should not happen, all deferred args should have been resolved above
                    bail!("DeferredAdjacentFilePath found during command execution");
                }
            }
        }

        // Apply envs to tokio Command
        cmd.envs(&this.env);

        // All done :)
        Ok(rtn)
    }
}
