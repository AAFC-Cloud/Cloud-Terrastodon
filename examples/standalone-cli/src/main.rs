use cloud_terrastodon::app::{App, ApplicationCli, GlobalArgs, Result, facet, figue, tracing};

/// Local text utilities with shared application startup.
/// The commands and their data belong to this consumer, not Cloud Terrastodon.
#[derive(facet::Facet)]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,
    #[facet(flatten)]
    pub builtins: figue::FigueBuiltins,
    #[facet(figue::subcommand)]
    pub command: Command,
}

impl ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

#[derive(facet::Facet)]
#[repr(u8)]
pub enum Command {
    /// Print a message to standard output.
    Echo(EchoArgs),
    /// Inspect local text without contacting external services.
    Inspect(InspectArgs),
}

#[derive(facet::Facet)]
pub struct EchoArgs {
    /// The text to print, including any spaces.
    /// Quote this argument in your shell to keep the message together.
    #[facet(figue::positional)]
    pub message: String,
}

#[derive(facet::Facet)]
pub struct InspectArgs {
    #[facet(figue::subcommand)]
    pub command: InspectCommand,
}

#[derive(facet::Facet)]
#[repr(u8)]
pub enum InspectCommand {
    /// Count whitespace-separated words in local text.
    Words(WordsArgs),
}

#[derive(facet::Facet)]
pub struct WordsArgs {
    /// Text whose words should be counted.
    /// Tabs and newlines separate words just like spaces.
    #[facet(figue::positional)]
    pub text: String,
    /// Fail if the final word count differs from this value.
    #[facet(figue::named)]
    pub expect: Option<usize>,
    /// Cooperatively cancel before processing this many words.
    /// This local work limit does not require a timer or a signal.
    #[facet(figue::named)]
    pub stop_after: Option<usize>,
}

fn main() -> Result<()> {
    App::new("field-notes", env!("CARGO_PKG_VERSION")).run(|cli: Cli, context| async move {
        match cli.command {
            Command::Echo(args) => {
                tracing::debug!(text = %args.message, "echo diagnostic");
                tracing::info!("echo dispatched");
                println!("{}", args.message);
            }
            Command::Inspect(InspectArgs {
                command: InspectCommand::Words(args),
            }) => {
                let mut count = 0;
                for word in args.text.split_whitespace() {
                    if args.stop_after == Some(count) {
                        context
                            .cancellation
                            .request_cancel("local word limit reached");
                    }
                    context.cancellation.bail_if_cancelled()?;
                    tracing::debug!(word, "inspecting word");
                    count += 1;
                }
                if args.expect.is_some_and(|expected| count != expected) {
                    return Err(std::io::Error::other("word count did not match --expect").into());
                }
                println!("{count}");
            }
        }
        Ok(())
    })
}
