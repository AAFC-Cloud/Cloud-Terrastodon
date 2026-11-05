use crate::CommandArgument;
use crate::CommandBuilder;
use crate::CommandInputWorkspace;
pub use bstr;
use bstr::ByteSlice;
use cloud_terrastodon_config::CommandsConfig;
use cloud_terrastodon_config::Config;
use eyre::OptionExt;
use eyre::Result;
use eyre::bail;
use std::env;
use std::ffi::OsString;
use tokio::process::Command;
use tokio::sync::OnceCell;

#[derive(Clone, Default, Debug, Eq, PartialEq, facet::Facet)]
#[repr(u8)]
pub enum CommandKind {
    #[default]
    AzureCLI,
    Terraform,
    VSCode,
    Echo,
    Pwsh,
    Git,
    Gitea,
    Other(String),
}

async fn get_config(cache: &OnceCell<CommandsConfig>) -> Result<&CommandsConfig> {
    cache.get_or_try_init(CommandsConfig::load).await
}

pub const USE_TOFU_FLAG_KEY: &str = "CLOUD_TERRASTODON_USE_TOFU";

static CONFIG: OnceCell<CommandsConfig> = OnceCell::const_new();

impl CommandKind {
    pub async fn program(&self) -> Result<String> {
        Ok(match self {
            CommandKind::AzureCLI => get_config(&CONFIG).await?.azure_cli.to_owned(),
            CommandKind::Terraform => match env::var(USE_TOFU_FLAG_KEY) {
                Err(_) => get_config(&CONFIG).await?.terraform.to_owned(),
                Ok(_) => get_config(&CONFIG).await?.tofu.to_owned(),
            },
            CommandKind::VSCode => get_config(&CONFIG).await?.vscode.to_owned(),
            CommandKind::Echo => "pwsh".to_string(),
            CommandKind::Pwsh => "pwsh".to_string(),
            CommandKind::Git => "git".to_string(),
            CommandKind::Gitea => "tea".to_string(),
            CommandKind::Other(x) => x.to_owned(),
        })
    }
    /// Apply command arguments and environment, retaining staged adjacent inputs.
    /// Keep the returned workspace alive until the subprocess has finished.
    pub async fn apply_args_and_envs(
        &self,
        this: &CommandBuilder,
        cmd: &mut Command,
    ) -> Result<Option<CommandInputWorkspace>> {
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
            CommandKind::AzureCLI => {
                // Always add --debug if not present.
                let has_debug = args
                    .iter()
                    .any(|a| matches!(a, CommandArgument::Literal(lit) if lit == "--debug"));
                if !has_debug {
                    args.push(CommandArgument::Literal("--debug".into()));
                }
            }
            _ => {}
        }

        let workspace = CommandInputWorkspace::stage(&this.adjacent_files).await?;

        // Patch arguments to point to canonical paths
        for arg in args.iter_mut() {
            if let CommandArgument::DeferredAdjacentFilePath { key, mapper } = arg {
                let path_to_map = workspace
                    .as_ref()
                    .ok_or_eyre("deferred adjacent input has no command workspace")?
                    .input_path(key)?;
                let mapped_path = mapper.map_path(path_to_map);
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

        // Remove inherited values before applying deliberate replacements.
        if !this.removed_env_prefixes.is_empty() {
            for (name, _) in env::vars_os() {
                if this.removed_env_prefixes.iter().any(|prefix| {
                    CommandBuilder::environment_name_has_prefix(&name.to_string_lossy(), prefix)
                }) {
                    cmd.env_remove(name);
                }
            }
        }
        for key in &this.removed_env {
            cmd.env_remove(key);
        }
        cmd.envs(&this.env);

        Ok(workspace)
    }
}
