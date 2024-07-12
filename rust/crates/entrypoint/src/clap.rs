use crate::actions::prelude::jump_to_block;
use crate::menu::menu_loop;
use crate::prelude::Version;
use azure::prelude::ScopeImplKind;
use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use clap::Subcommand;
use itertools::Itertools;
use pathing_types::IgnoreDir;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "cloud_terrastodon", about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive mode
    Interactive,
    /// Clean up working files
    Clean {
        #[arg(long)]
        dir: Vec<IgnoreDir>,
    },
    /// Interact with Azure policy
    #[command(subcommand)]
    Policy(PolicyCommand),
    /// Display an object as json
    Show {
        #[arg(long)]
        kind: Option<ScopeImplKind>,
    },
    /// Terraform related commands
    #[command(subcommand)]
    Tf(TfCommand),
}

#[derive(Subcommand, Debug)]
enum TfCommand {
    /// Jump to a specific block
    Jump,
}

#[derive(Subcommand, Debug)]
enum PolicyCommand {
    Compliance,
    Remediation,
}

pub async fn main(version: Version) -> anyhow::Result<()> {

    // Set the version
    let mut cmd = Cli::command();
    cmd = cmd.version(version.to_string());

    // Parse the command-line arguments
    let cli = Cli::from_arg_matches(&cmd.get_matches())?;

    match cli.command {
        None => {
            menu_loop().await?;
        }
        Some(command) => match command {
            Commands::Interactive => print_subcommands::<Cli>(),
            Commands::Show { kind } => {
                info!("You chose: {kind:?}");
            }
            Commands::Tf(tf_command) => match tf_command {
                TfCommand::Jump => jump_to_block(IgnoreDir::Processed.into()).await?,
            },
            Commands::Policy(policy_command) => match policy_command {
                PolicyCommand::Compliance => todo!(),
                PolicyCommand::Remediation => todo!(),
            },
            Commands::Clean { dir: dirs } => {
                info!("Cleaning {dirs:#?}");
                for dir in dirs {
                    info!("Cleaning {dir:?}");
                    tokio::fs::remove_dir_all(dir.as_path_buf()).await?;
                }
            }
        },
    }
    Ok(())
}

fn print_subcommands<T: CommandFactory>() {
    let cmd = T::command();
    print_subcommands_recursively(&cmd, 0);
}

fn print_subcommands_recursively(cmd: &clap::Command, indent: usize) {
    let indent_str = " ".repeat(indent);
    let cmd_name = cmd.get_name();
    let mut cmd_param_variants = Vec::new();
    let cmd_params = {
        let mut arguments = cmd.get_arguments().peekable();
        if arguments.peek().is_none() {
            String::new()
        } else {
            format!(
                "({})",
                arguments
                    .map(|arg| {
                        let arg_ident = arg.get_id().as_str();
                        let arg_parser = arg.get_value_parser();
                        if let Some(variants) = arg_parser.possible_values() {
                            for v in variants {
                                cmd_param_variants.push(format!(
                                    "{} - {}",
                                    arg_ident,
                                    v.get_name()
                                ));
                            }
                        }
                        format!("{}: {:?}", arg_ident, arg_parser)
                    })
                    .collect_vec()
                    .into_iter()
                    .join(", ")
            )
        }
    };
    println!(
        "{}{}{}\n{:#?}",
        indent_str, cmd_name, cmd_params, cmd_param_variants
    );

    // interactive mode - match against ID to prompt for arg values before invoking the normal command

    for subcmd in cmd.get_subcommands() {
        print_subcommands_recursively(subcmd, indent + 2);
    }
}
