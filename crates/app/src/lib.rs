//! Shared application startup without a prescribed command tree.
//!
//! Define a public root CLI, flatten [`GlobalArgs`] and [`figue::FigueBuiltins`],
//! implement [`ApplicationCli`], then pass your handler to [`App::run`]. Neither
//! `Debug`, `Arbitrary`, nor registry registration is required on your CLI.
//!
//! `run` owns the process-global error, tracing and signal hooks and a Tokio
//! runtime: call it once, from synchronous `main`. Libraries and tests that only
//! need argument parsing should use [`App::parse_from`] instead. That method does
//! not install startup hooks or resolve authentication; Figue's explicitly
//! requested help/schema export facilities retain their usual behavior.
//! Creating an `eyre::Report` before startup may itself install the default error
//! hook; finish shared startup before creating application error reports.

mod global_args;
#[cfg(feature = "terminal")]
mod terminal;
#[cfg(windows)]
mod windows_support;

pub use eyre::Result;
pub use facet;
pub use figue;
pub use global_args::GlobalArgs;
pub use teamy_cancellation::CancellationToken;
pub use tracing;

#[cfg(feature = "auth")]
pub use cloud_terrastodon_credentials::{AuthContext, AuthSource};

use std::ffi::OsString;
use std::future::Future;
use std::str::FromStr;
use tracing::Instrument;
use tracing_subscriber::filter::Directive;

/// The consumer owns its root schema and command dispatch.
///
/// Return the same shared global-arguments field that your Figue schema flattens.
/// Keep builtins explicit in the schema so callers can inspect and expose the
/// complete CLI type, including their own subcommands.
pub trait ApplicationCli: facet::Facet<'static> {
    fn global_args(&self) -> &GlobalArgs;
}

/// Facilities owned by this invocation, not process-global application state.
pub struct AppContext {
    /// Cooperative cancellation, including the installed Ctrl-C handler.
    pub cancellation: CancellationToken,
    /// Resolved authentication preferences; does not itself request a token.
    #[cfg(feature = "auth")]
    pub auth: AuthContext,
}

/// Caller-supplied application identity and startup policy.
#[derive(Debug, Clone)]
pub struct App {
    name: String,
    version: String,
    implementation: Option<(String, String)>,
    implementation_source_file: bool,
    egui_collector: bool,
}

impl App {
    /// Supply the consuming package's metadata, usually with `env!` in `main`.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            implementation: None,
            implementation_source_file: false,
            egui_collector: false,
        }
    }

    /// Include source hints using the caller's repository and immutable revision.
    pub fn implementation_github(
        mut self,
        owner_repo: impl Into<String>,
        revision: impl Into<String>,
    ) -> Self {
        self.implementation = Some((owner_repo.into(), revision.into()));
        self.implementation_source_file = true;
        self
    }

    /// Include source paths in help without requiring GitHub metadata.
    pub fn implementation_source_file(mut self, include: bool) -> Self {
        self.implementation_source_file = include;
        self
    }

    /// Preserve the application's existing optional GUI collector selection.
    /// The underlying tracing crate currently leaves that collector disabled.
    pub fn egui_collector(mut self, enable: bool) -> Self {
        self.egui_collector = enable;
        self
    }

    /// Parse arguments excluding argv[0], without exiting or installing hooks.
    ///
    /// Inspect `into_result()` to distinguish help/version from real failures.
    /// Do not propagate a help outcome with `?` as an application error.
    pub fn parse_from<C: ApplicationCli>(
        &self,
        args: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> figue::DriverOutcome<C> {
        let builder = match figue::builder::<C>() {
            Ok(builder) => builder,
            Err(error) => return figue::DriverOutcome::err(figue::DriverError::Builder { error }),
        };
        let config = builder
            .cli(|cli| cli.args_os(args.into_iter().map(Into::into)).strict())
            .help(|help| {
                let mut help = help
                    .program_name(self.name.clone())
                    .version(self.version.clone())
                    .include_implementation_source_file(self.implementation_source_file);
                if let Some((repository, revision)) = &self.implementation {
                    help = help
                        .include_implementation_github_url(repository.clone(), revision.clone());
                }
                help
            })
            .build();
        figue::Driver::new(config).run()
    }

    /// Parse process arguments and run the caller's async handler.
    ///
    /// Help, version, completions and parse errors use Figue's CLI exit behavior
    /// before runtime/auth/log initialization. Handler errors return to `main`.
    /// Call once at process entry, not inside another async runtime or a host
    /// which already owns tracing/error/signal hooks. Cancellation is cooperative:
    /// this function does not drop arbitrary application futures on Ctrl-C.
    pub fn run<C, F, Fut>(self, invoke: F) -> Result<()>
    where
        C: ApplicationCli,
        F: FnOnce(C, AppContext) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        let cli = self.parse_from(std::env::args_os().skip(1)).unwrap();
        self.run_parsed(cli, invoke)
    }

    /// Run an already parsed CLI at process entry.
    ///
    /// This is useful when startup policy depends on the selected command. It
    /// has the same once-per-process and synchronous-main contract as [`Self::run`].
    pub fn run_parsed<C, F, Fut>(self, cli: C, invoke: F) -> Result<()>
    where
        C: ApplicationCli,
        F: FnOnce(C, AppContext) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        eyre::ensure!(
            tokio::runtime::Handle::try_current().is_err(),
            "App::run must be called from synchronous main, outside a Tokio runtime"
        );
        let globals = cli.global_args();
        let debug = globals.debug;
        install_error_reporting(debug)?;
        let (log_filter, file_filter) = logging_filters(globals)?;

        #[cfg(feature = "auth")]
        let auth = AuthContext::resolve(globals.auth_source)?;

        #[cfg(feature = "terminal")]
        let terminal = terminal::TerminalSession::new();
        #[cfg(feature = "terminal")]
        let (terminal_buffer, terminal_activity) =
            (Some(terminal.log_buffer()), Some(terminal.activity_probe()));
        #[cfg(not(feature = "terminal"))]
        let (terminal_buffer, terminal_activity) = (None, None);

        cloud_terrastodon_tracing::init_tracing_with_terminal(
            log_filter,
            file_filter,
            globals.log_file.as_ref(),
            self.egui_collector,
            terminal_buffer,
            terminal_activity,
        )?;
        #[cfg(windows)]
        windows_support::initialize();

        let cancellation = teamy_cancellation::CtrlCHandler::default().install()?;
        let context = AppContext {
            cancellation,
            #[cfg(feature = "auth")]
            auth,
        };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        // Construct the user's future while the runtime and terminal task locals
        // are active, not while calling a synchronous closure before block_on.
        let invocation = Box::pin(
            async move {
                let _cancel_on_exit = CancelOnDrop(context.cancellation.clone());
                invoke(cli, context).await
            }
            .instrument(tracing::info_span!("cli_invocation")),
        );
        #[cfg(feature = "terminal")]
        return terminal.block_on(&runtime, invocation, debug)?;
        #[cfg(not(feature = "terminal"))]
        runtime.block_on(invocation)
    }
}

fn logging_filters(globals: &GlobalArgs) -> Result<(Directive, Option<Directive>)> {
    let console = if globals.debug {
        tracing::level_filters::LevelFilter::DEBUG.into()
    } else {
        Directive::from_str(&globals.log_filter)?
    };
    let file = globals
        .log_file_filter
        .as_deref()
        .map(Directive::from_str)
        .transpose()?;
    Ok((console, file))
}

fn install_error_reporting(debug: bool) -> Result<()> {
    use std::io::Write;

    let (panic_hook, error_hook) = color_eyre::config::HookBuilder::default().try_into_hooks()?;
    error_hook.install()?;
    std::panic::set_hook(Box::new(move |info| {
        let mut stderr = std::io::stderr().lock();
        // A broken stderr must not cause another panic inside a panic hook.
        let _ = writeln!(stderr, "{}", panic_hook.panic_report(info));
        if debug {
            // Environment mutation is unsafe once any host thread exists.
            let _ = writeln!(
                stderr,
                "\nDebug backtrace:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
    }));
    Ok(())
}

struct CancelOnDrop(CancellationToken);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if !self.0.is_cancelled() {
            // Routine shutdown is not a user cancellation event. Signal clones
            // without adding the token library's INFO message to every command.
            tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
                self.0.request_cancel("application invocation finished");
            });
        }
    }
}

#[cfg(test)]
mod tests;
