# Consumer-owned command-line applications

Cloud Terrastodon can supply application startup without supplying your command
tree. Define your own public CLI, reuse `GlobalArgs`, and call the shared runner.
Cloud Terrastodon's main executable uses the same runner.

## Local development

The [standalone example](../examples/standalone-cli) is a separate Cargo workspace
with local dependencies and its own lockfile. Its
complete CLI implementation is [one main.rs](../examples/standalone-cli/src/main.rs).
It does not initialize or copy the Teamy Rust CLI template.

For a project beside another task's notes, begin with an ordinary Cargo package
and this dependency shape (adjust the path):

```toml
[dependencies]
cloud_terrastodon = { path = "../cloud-terrastodon", default-features = false, features = ["app"] }
facet = { package = "teamy-facet", version = "=0.50.0-rc.7", path = "../facet/facet" }
```

The sibling clones `facet`, `facet-format`, and `figue` must use TeamDman's
`teamy-main` branches for local development. The explicit `facet` dependency
is needed because Figue's attribute expansion references `::facet` even when
Figue is imported through our facade. Its package is `teamy-facet`, matching
the facade's fork identity. No separate Figue dependency or root patch block
is needed. Once the prepared Teamy versions and Cloud Terrastodon 0.37.0 are
published, registry consumers can remove these development paths. See
[the dependency release guide](UPDATING_FACET.md).

Keep `default-features = false`: the facade's existing default is `full`, which
includes the actual Cloud Terrastodon command tree and UIs.

## Own the root CLI

The consumer supplies its entire schema and decides how to dispatch it:

```rust,no_run
use cloud_terrastodon::app::{App, ApplicationCli, GlobalArgs, Result, figue, tracing};
use facet::Facet;

#[derive(Facet)]
pub struct Cli {
    #[facet(flatten)]
    pub globals: GlobalArgs,
    #[facet(flatten)]
    pub builtins: figue::FigueBuiltins,
    #[facet(figue::subcommand)]
    pub command: Command,
}

#[derive(Facet)]
#[repr(u8)]
pub enum Command {
    /// Print a greeting.
    Hello,
}

impl ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs { &self.globals }
}

fn main() -> Result<()> {
    App::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .run(|cli: Cli, context| async move {
            context.cancellation.bail_if_cancelled()?;
            match cli.command {
                Command::Hello => tracing::info!("Hello!"),
            }
            Ok(())
        })
}
```

No `Debug`, `Arbitrary`, registry registration, extra command directories,
`#[tokio::main]`, or application `build.rs` is required. The example adds nested
subcommands and demonstrates shared flags after a leaf, complete multiline help,
independent stderr/NDJSON filters, errors and cooperative cancellation.

For a tool with just one operation, omit the `command` field and enum entirely.
The [resource-group listing example](../examples/resource_group_list.rs) does this:
its root contains only globals and builtins, and its handler lists resource groups.
It remains a single Cargo example source file using the repository's manifest;
the independent consumer uses a separate manifest to test downstream integration.

If you want other Rust crates to import `your_package::Cli`, put the declarations
and trait implementation in `src/lib.rs` and import them in `main.rs`. A `pub`
declaration in a binary alone is not a library export. This is ordinary optional
Cargo library structure, not a requirement to split every subcommand into files.
The shared runner remains independent of whether your schema is private or public.

## Included and optional facilities

| Facade feature | Facilities |
| --- | --- |
| `app` | Shared debug/logging/output arguments; text/JSON output rendering; Figue help/version/completions; caller-owned schema; error reporting; Tokio runtime; cooperative Ctrl-C cancellation |
| `app-auth` | `GlobalArgs::auth_source`, `--auth-source`, and `AppContext::auth` using Cloud Terrastodon's authentication preferences |
| `app-terminal` | Coordinated terminal/picker task locals and buffered human logging; no full command tree or egui |

Authentication resolves preferences at startup, not during help; individual
Cloud Terrastodon API calls still determine whether credentials/network access
are needed. Enable the facade's relevant SDK features separately when using those
APIs. The auth feature currently brings in the registry through credentials.
Terminal support brings in the picker/Ratatui dependencies. Features are additive:
another dependency enabling `auth` adds its globals to the shared type too. Prefer
`GlobalArgs::default()` over exhaustive literals when constructing it manually.

The runner accepts one tracing directive for each `--log-filter` and
`--log-file-filter`, including a module directive such as `field_notes=debug`.
Existing `RUST_LOG` handling and independent file filtering remain in the tracing
crate. `--debug` selects debug as the console default; `RUST_LOG` may still override
it. Explicit file filters remain independent of both.

Use `App::implementation_github(owner_repo, revision)` to opt into source hints
for your own repository. The runner does not guess Cloud Terrastodon metadata for
your executable. The facade's build script skips binary resources/metadata when
the `entrypoint` feature is disabled.

## Command output

`GlobalArgs` includes `--output-format text|json|facet-pretty|auto`. `CliOutput` selects text when
stdout is an interactive terminal and JSON when stdout is redirected, including
PowerShell pipelines. Explicit formats override detection; `auto` uses it.
Text renderers use stream-aware color support independently of format selection,
honoring `NO_COLOR` and `FORCE_COLOR` as well as stdout detection.
`facet-pretty` (alias `facet`) displays the underlying data with Facet's pretty
printer, including for commands with a custom `text` presentation.

Let leaf `invoke` methods return `Result<CliOutput>` and pass that value through
parent dispatchers. The top-level handler calls `emit(globals.output_format)`
once; the runner's handler still returns `Result<()>`.

`CliOutput::facet(value)` uses Facet's pretty printer for `text` and
`facet-pretty`, and its serializer for JSON. For a command with its own text
presentation, derive `Facet` on the report and supply a text renderer:

```rust,no_run
use cloud_terrastodon::app::{CliOutput, GlobalArgs, Result};
use facet::Facet;

#[derive(Facet)]
struct Report {
    count: usize,
}

impl Report {
    fn render_text(&self) -> Result<String> {
        Ok(format!("{} items", self.count))
    }
}

async fn invoke_list() -> Result<CliOutput> {
    let report = Report { count: 3 };
    Ok(CliOutput::facet_with_text(report, Report::render_text))
}

async fn invoke_cli(globals: &GlobalArgs) -> Result<()> {
    invoke_list().await?.emit(globals.output_format)
}
```

`facet_with_text` overrides only `text`: JSON serialization and `facet-pretty`
rendering remain shared and use the same report. Its callback receives only the
value. Apply text styles using `if_supports_color(Stream::Stdout, |text| ...)`
from `color_eyre::owo_colors::OwoColorize`, and detect stdout locally when
emitting terminal hyperlinks.

For complete control over every format, implement the re-exported
`CliOutputValue` trait and pass the value to `CliOutput::new(value)`. Its `render`
method receives the selected `OutputFormat`. Keep JSON free
of terminal colors and hyperlinks. Use `CliOutput::none()` for commands that
perform their own interaction and have no final value to emit.

## Process and ownership contract

- Call `App::run` once from synchronous `main`, before another component installs
  error, tracing or signal hooks. It is not an embeddable/restartable runtime and
  rejects execution inside an existing Tokio runtime. `run_parsed` has the same
  ownership contract and permits startup policy based on the parsed command.
  Even constructing an `eyre::Report` first can install its default error hook;
  let the runner own startup before creating application error reports.
  `App::run` prepares console output before parsing. Custom process entrypoints
  that parse first should call `app::prepare_process()` before printing or
  unwrapping parse outcomes, so Windows help and diagnostics render ANSI colors.
- `App::parse_from::<Cli>(arguments)` accepts arguments without the executable
  name and returns a non-exiting `figue::DriverOutcome<Cli>`. Use `into_result()`
  to inspect `.value`, help, version or errors. It does not install startup hooks
  or resolve auth. Explicit HTML-help/schema-export requests can still write files.
- The pinned Figue driver treats a missing required CLI positional as help with
  a corrected-command diagnostic and exits **0**; unknown flags/commands fail.
  For example, `field-notes echo` (without its required `MESSAGE`) prints help
  and a missing-argument diagnostic to stdout, does not invoke the handler, and
  still reports success to the shell. Automation checking only the exit status
  can therefore mistake an incomplete invocation for completed work. Explicit
  `--help` also succeeds, while malformed supplied values and handler errors fail.
  The runner deliberately retains that upstream distinction, including automatic
  help for omitted subcommands. Do not use an omitted required positional as a
  reliable nonzero-status check. The standalone suite characterizes this behavior;
  changing that policy is a separate Figue follow-up, not a hidden runner override.
- The handler receives owned arguments and context. Its closure is invoked, and
  its returned future is polled, under the runtime and, when enabled, terminal task locals.
  Closure capture initializers still execute in the caller before `run`.
  Neither a `Send` nor a `'static` handler-future bound is imposed.
- Cancellation is cooperative. Check/pass `context.cancellation` during work.
  It is also signalled quietly when the handler finishes or unwinds; an existing
  cancellation reason is preserved. Join your own background work before return.
  Arbitrary blocking tasks can still delay runtime shutdown indefinitely.
- `--debug` explicitly captures panic backtraces without changing process
  environment variables. Ordinary error backtrace capture follows
  `RUST_LIB_BACKTRACE`/`RUST_BACKTRACE`. Rapid repeated Ctrl-C retains the cancellation
  library's immediate process-exit behavior, which does not run destructors.

## Validation and publication boundary

```powershell
cargo +1.96.0 test --manifest-path examples/standalone-cli/Cargo.toml --offline --locked
cargo +1.96.0 test -p cloud_terrastodon_app --offline --locked
cargo +1.96.0 test -p cloud_terrastodon_app --features auth,terminal,arbitrary,terminal_coordinator_debug --offline --locked
```

The example has `publish = false`: it is a neutral local acceptance fixture, not
an implementation of a particular business task. No release is performed by this
workflow. The prepared Teamy dependency graph is described in
[Updating Facet](UPDATING_FACET.md). Before a crates.io release, verify clean
packaged consumers and publish the exact dependency versions first.
`publish.ps1 -DryRun` previews production dependency order and supports both
new packages and new versions of existing packages.

Use only `cargo clean` if build-space cleanup is necessary.
