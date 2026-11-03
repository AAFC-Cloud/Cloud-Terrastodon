use crate::AppContext;
use crate::ApplicationCli;
use crate::GlobalArgs;
use crate::app_cli::AppCli;
use crate::cancel_on_drop::CancelOnDrop;
use crate::prepare_process;
use crate::startup::install_panic_hook;
#[cfg(feature = "terminal")]
use crate::terminal;
#[cfg(windows)]
use crate::windows_support;
#[cfg(feature = "auth")]
use cloud_terrastodon_auth::AuthContext;
#[cfg(feature = "auth")]
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::ffi::OsString;
use std::future::Future;
use std::process::ExitCode;
use std::process::Termination;
use std::str::FromStr;
use tracing::Instrument;
use tracing_subscriber::filter::Directive;

/// Caller-supplied application identity and startup policy.
#[derive(Debug, Clone)]
pub struct App {
    name: String,
    version: String,
    implementation: Option<(String, String)>,
    implementation_source_file: bool,
    egui_collector: bool,
    automatic_cancellation: bool,
    #[cfg(feature = "auth")]
    authentication: bool,
    #[cfg(feature = "terminal")]
    terminal_debug_capture: bool,
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
            automatic_cancellation: true,
            #[cfg(feature = "auth")]
            authentication: true,
            #[cfg(feature = "terminal")]
            terminal_debug_capture: false,
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

    /// Controls whether cancellation automatically stops polling the handler.
    ///
    /// Enabled by default. Linked nested deferrals keep the handler polling until
    /// their required work finishes. Checks occur at poll boundaries; synchronous
    /// loops still need explicit checkpoints or yields.
    ///
    /// Disable this when the handler must observe cancellation and await its own
    /// shutdown, such as restoring a terminal or draining background jobs. The
    /// supplied scope can still start explicitly cancellable child operations.
    pub fn automatic_cancellation(mut self, enable: bool) -> Self {
        self.automatic_cancellation = enable;
        self
    }

    /// Disable authentication discovery during application startup.
    ///
    /// Use this for local tools or applications whose service commands own
    /// credential selection. It remains effective if Cargo feature unification
    /// enables `auth` through another workspace member. With that feature enabled,
    /// the handler receives `AuthContext::None`; global `auto`/`none` selections
    /// perform no discovery, and explicit global credential sources are rejected.
    /// Command-specific authentication remains the consumer's responsibility.
    pub fn without_authentication(self) -> Self {
        #[cfg(feature = "auth")]
        {
            Self {
                authentication: false,
                ..self
            }
        }
        #[cfg(not(feature = "auth"))]
        {
            self
        }
    }

    /// Capture a bounded Debug-through-Error stream for an interactive log surface.
    ///
    /// Independent from the user's stderr/file filters. Enable only for commands
    /// that consume `PickerLogBuffer::captured_records_since`; retained events are
    /// limited to the latest 2,048 records and never replayed to stderr on exit.
    #[cfg(feature = "terminal")]
    pub fn terminal_debug_capture(mut self, enable: bool) -> Self {
        self.terminal_debug_capture = enable;
        self
    }

    /// Parse arguments excluding argv[0], without exiting or installing hooks.
    ///
    /// Inspect `into_result()` to distinguish help/version from real failures.
    /// Do not propagate a help outcome with `?` as an application error.
    /// Builtin flags are supplied by App; successful values contain only `C`.
    pub fn parse_from<C: ApplicationCli>(
        &self,
        args: impl IntoIterator<Item = impl Into<OsString>>,
    ) -> figue::DriverOutcome<C> {
        let builder = match figue::builder::<AppCli<C>>() {
            Ok(builder) => builder,
            Err(error) => return figue::DriverOutcome::err(figue::DriverError::Builder { error }),
        };
        let config = builder
            .cli(|cli| cli.args_os(args.into_iter().map(Into::into)).strict())
            .help(|help| {
                let mut help = help
                    .program_name(self.name.clone())
                    .version(self.version.clone())
                    .root_shape(C::SHAPE)
                    .include_implementation_source_file(self.implementation_source_file);
                if let Some((repository, revision)) = &self.implementation {
                    help = help
                        .include_implementation_github_url(repository.clone(), revision.clone());
                }
                help
            })
            .build();
        figue::Driver::new(config).run().map(|cli| cli.cli)
    }

    /// Parse process arguments and run the caller's async handler.
    ///
    /// Help, version, completions and parse errors use Figue's CLI exit behavior
    /// before runtime/auth/log initialization. Handler errors return to `main`.
    /// Call once at process entry, not inside another async runtime or a host
    /// which already owns tracing/error/signal hooks. The handler is automatically
    /// cancellable by default and respects linked nested deferrals. Call
    /// [`automatic_cancellation(false)`](Self::automatic_cancellation) when the
    /// handler owns asynchronous shutdown. Synchronous work still needs
    /// cancellation checkpoints or yields.
    pub fn run<C, F, Fut>(self, invoke: F) -> Result<()>
    where
        C: ApplicationCli,
        F: FnOnce(C, AppContext) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        prepare_process()?;
        let cli = match self.parse_from(std::env::args_os().skip(1)).into_result() {
            Ok(output) => output.get(),
            Err(outcome) => {
                // Figue reports help/version successfully and diagnostics as an
                // error exit, including failures while opening HTML help.
                let status = outcome.report();
                std::process::exit(if status == ExitCode::SUCCESS { 0 } else { 1 });
            }
        };
        self.run_parsed(cli, invoke)
    }

    /// Run an already parsed CLI at process entry.
    ///
    /// This is useful when startup policy depends on the selected command. It
    /// has the same once-per-process and synchronous-main contract as [`Self::run`].
    /// The value is your consumer CLI; builtin flags have already been handled
    /// when parsing with [`Self::parse_from`]. The automatic cancellation policy
    /// configured by [`Self::automatic_cancellation`] applies to this handler too.
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
        #[cfg(windows)]
        windows_support::enable_virtual_terminal_processing();
        let globals = cli.global_args();
        let debug = globals.debug;
        install_panic_hook(debug)?;

        let (log_filter, file_filter) = logging_filters(globals)?;

        #[cfg(feature = "auth")]
        let auth = if self.authentication {
            AuthContext::resolve(globals.auth_source)?
        } else {
            eyre::ensure!(
                matches!(globals.auth_source, AuthSource::Auto | AuthSource::None),
                "this application disables startup authentication; select credentials on the service command instead of the global --auth-source"
            );
            AuthContext::None
        };

        #[cfg(feature = "terminal")]
        let terminal = terminal::TerminalSession::new().debug_capture(self.terminal_debug_capture);
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
        windows_support::warn_if_not_utf8();

        let cancellation = teamy_cancellation::CtrlCHandler::default().install()?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        // Construct the user's future while the runtime and terminal task locals
        // are active, not while calling a synchronous closure before block_on.
        let invocation = Box::pin(
            async move {
                let _cancel_on_exit = CancelOnDrop(cancellation.clone());
                let run_handler = |cancellation| async move {
                    let context = AppContext::new(
                        WorkCache::new(AppDir::Commands.join("work"))?,
                        cancellation,
                        #[cfg(feature = "auth")]
                        auth,
                    );
                    invoke(cli, context).await
                };
                let handler = if self.automatic_cancellation {
                    cancellation.cancellable_scope(run_handler)
                } else {
                    cancellation.with_async_scope(run_handler)
                };
                handler.await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_directives_and_independent_file_filter_are_supported() {
        let globals = GlobalArgs {
            log_filter: "consumer_example=trace".into(),
            log_file_filter: Some("warn".into()),
            ..GlobalArgs::default()
        };
        let (console, file) = logging_filters(&globals).unwrap();
        assert_eq!(console.to_string(), "consumer_example=trace");
        assert_eq!(file.unwrap().to_string(), "warn");
    }

    #[test]
    fn debug_selects_console_debug_but_preserves_file_filter() {
        let globals = GlobalArgs {
            debug: true,
            log_filter: "not a valid directive".into(),
            log_file_filter: Some("error".into()),
            ..GlobalArgs::default()
        };
        let (console, file) = logging_filters(&globals).unwrap();
        assert_eq!(console.to_string(), "debug");
        assert_eq!(file.unwrap().to_string(), "error");
    }
}
