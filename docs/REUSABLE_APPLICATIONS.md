# Consumer-owned command-line applications

Cloud Terrastodon can supply application startup without supplying your command
tree. Define your own public CLI, reuse `GlobalArgs`, and call the shared runner.
Cloud Terrastodon's main executable uses the same runner.

## Command crate boundaries

The main command tree composes `cloud_terrastodon_azure_cli`,
`cloud_terrastodon_azure_devops_cli`, and `cloud_terrastodon_source_cli` from
`cloud_terrastodon_entrypoint`. Each CLI crate owns its command schemas, dispatch,
and presentation and depends on its corresponding SDK crate. SDK and domain
types crates do not depend on the CLI crates. `cloud_terrastodon_cli_types` holds
shared validated CLI inputs; `cloud_terrastodon_app` supplies reusable startup
and output facilities.

These boundaries give Cargo separate compilation units for the command groups.
An entrypoint-only edit can reuse unchanged CLI crates, while SDK edits still
rebuild their affected dependents. Build-time improvements depend on the change
and have not been benchmarked.

Check a command group without checking the full entrypoint:

```powershell
cargo check --offline -p cloud_terrastodon_azure_cli
cargo check --offline -p cloud_terrastodon_azure_devops_cli
cargo check --offline -p cloud_terrastodon_source_cli
```

## Start a consumer

The [standalone example](../examples/standalone-cli) is a root workspace member
with its own manifest and the repository's shared `Cargo.lock`. It uses the local
Cloud Terrastodon facade through `../..` and inherits the workspace's `facet`
dependency. Its complete CLI implementation is
[one main.rs](../examples/standalone-cli/src/main.rs).
It does not initialize or copy the Teamy Rust CLI template.

The [group-management example](../examples/group-management) demonstrates a larger
consumer in its own workspace package. Its library owns domain validation and
reconciliation planning, its command files follow the main CLI's structure, and
its small `main.rs` selects startup policy and emits one output. Local validation
uses no directory credentials; read-only planning binds explicit tenant identities
and reports incomplete inventories as blockers. See its README for the current
milestone and planned apply workflow.

This guide includes workspace changes for the next Cloud Terrastodon release.
Published 0.37.0 still requires consumers to declare `FigueBuiltins` themselves.
For a separate project using the current checked-out runner, begin with an
ordinary Cargo package and adjust the Cloud Terrastodon path:

```toml
[dependencies]
cloud_terrastodon = { path = "../cloud-terrastodon", default-features = false, features = ["app"] }
```

Also add a direct dependency named `facet`, using the `teamy-facet` package and
matching the facade's version and source in the [workspace manifest](../Cargo.toml).
Figue's attribute expansion references `::facet` even when Figue is imported
through our facade, so both must share the same Facet package identity. The
workspace examples inherit this declaration with `facet.workspace = true` and
share the root lockfile. See [the dependency release guide](UPDATING_FACET.md)
when testing local fork changes or preparing a release.

Keep `default-features = false`: the facade's existing default is `full`, which
includes the actual Cloud Terrastodon command tree and UIs.

## Own the root CLI

The consumer supplies its application schema and decides how to dispatch it.
`App::parse_from` and `App::run` supply Figue's standard builtin flags through
an internal wrapper, while parsed values and handlers use the consumer's own
CLI type:

```rust,no_run
use cloud_terrastodon::app::{App, ApplicationCli, GlobalArgs, Result, figue, tracing};
use facet::Facet;

#[derive(Facet)]
pub struct Cli {
    #[facet(flatten)]
    pub globals: GlobalArgs,
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
        .run(|cli: Cli, _context| async move {
            match cli.command {
                Command::Hello => tracing::info!("Hello!"),
            }
            Ok(())
        })
}
```

No `Debug`, `Arbitrary`, registry registration, extra command directories,
`#[tokio::main]`, or application `build.rs` is required. The standalone example
has a single `audit` command, using the authenticated Azure SDK and typed reports
with shared application startup and cooperative cancellation.

For a tool with just one operation, omit the `command` field and enum entirely.
The [resource-group listing example](../examples/resource_group_list.rs) does this:
its root contains globals and a typed `--tenant` selector, and its
handler lists resource groups. `--auth-source` selects authentication;
`--tenant` accepts a tenant ID, tracked alias, or `default`. When omitted or set
to `default`, binding through `context.auth` retains a known authentication tenant
and otherwise falls back to Azure CLI's active tenant.

```powershell
cargo +1.96.0 run --example resource_group_list -- --auth-source azure-cli --tenant YOUR_TENANT_ID_OR_ALIAS
```

That command reads Azure data when executed. `App` automatically wraps the
handler in a cancellable scope. Tenant binding and the SDK fetch receive
`&context.cancellation` directly. The printing loop awaits
`tokio::task::yield_now()` before each `println!`, giving the handler's wrapper
a poll boundary at which to observe cancellation. A ready no-op await and a
zero-duration sleep do not reliably suspend, and a blocking print already in
progress cannot be interrupted.
Tokio can immediately poll the same task again; yielding does not promise another
task will run first. See the pinned
[Tokio yield documentation](https://docs.rs/tokio/1.52.3/tokio/task/fn.yield_now.html).

It remains a single Cargo example source file using the repository's manifest;
the standalone workspace package has its own manifest and command tree.

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

The standalone audit example opts into `app-auth` and `azure`, keeping default
features disabled. To add the audit's SDK facilities to the minimal external
project recipe above, use `features = ["app-auth", "azure"]` on the facade.
Its `--tenant` accepts a tenant ID or tracked alias through `AzureTenantArgument`.
Binding that argument to `context.auth` retains a known authentication tenant
when omitted and otherwise uses Azure CLI's default tenant. The handler calls
`fetch_all_resource_groups` for accessible subscriptions in that tenant; existing
SDK request caches may supply the inventory.

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
parent dispatchers. The top-level handler calls
`emit(globals.output_format, &context.cancellation)` once; the runner's handler
still returns `Result<()>`.

`CliOutput::facet(value)` uses Facet's pretty printer for `text` and
`facet-pretty`, and its serializer for JSON. For a command with its own text
presentation, derive `Facet` on the report and supply a text renderer:

```rust,no_run
use cloud_terrastodon::app::{CancellationScope, CliOutput, GlobalArgs, Result};
use facet::Facet;
use std::io::Write;

#[derive(Facet)]
struct Report {
    count: usize,
}

impl Report {
    fn write_text(
        &self,
        writer: &mut dyn Write,
        _cancellation: &CancellationScope,
    ) -> Result<()> {
        write!(writer, "{} items", self.count)?;
        Ok(())
    }
}

async fn invoke_list() -> Result<CliOutput> {
    let report = Report { count: 3 };
    Ok(CliOutput::facet_with_text(report, Report::write_text))
}

async fn invoke_cli(globals: &GlobalArgs, cancellation: &CancellationScope) -> Result<()> {
    invoke_list().await?.emit(globals.output_format, cancellation)
}
```

`facet_with_text` overrides only `text`: JSON serialization and `facet-pretty`
rendering remain shared and use the same report. Its callback receives the value,
writer, and emission's cancellation scope, and returns `Result<()>`. Use `write!`
or `writeln!` to emit text incrementally instead of collecting a `String`.
The writer checks cancellation between bounded writes; check the supplied scope
during long computation that does not write. `emit` and `write_to` append a final
newline if needed and flush after the callback succeeds, so callbacks leave that
completion work to the output owner. Errors or cancellation can leave a partial
text prefix. Apply text styles using `if_supports_color(Stream::Stdout, |text| ...)`
from `color_eyre::owo_colors::OwoColorize`, and detect stdout locally when
emitting terminal hyperlinks.

For complete control over every format, implement the re-exported
`CliOutputValue` trait and pass the value to `CliOutput::new(value)`. Its
`write_to` method receives the selected `OutputFormat`, a writer, and the
`CancellationScope`. Write records incrementally and check the scope during
computation that does not write; this supports large or unbounded streams without
collecting a complete `String`. Keep JSON free of terminal colors and hyperlinks. Use
`CliOutput::none()` for commands that perform their own interaction and have no
final value to emit.

`render(format, stdout_is_terminal, cancellation)` explicitly collects a buffered
preview without adding a final newline or flushing.
`write_to(format, stdout_is_terminal, writer, cancellation)` and `emit` check
cancellation between bounded writes and honor nested deferrals. They cannot
interrupt an operating-system write already in progress. JSON and Facet pretty
output stream through the same cancellation-aware writer, honoring the supplied
scope's deferrals. Computation that does not write, including allocation and
user-defined proxy conversions or formatters, continues until its next checkpoint.
Use `facet_json::to_writer_std_with_options(writer, value, &options)` for another
streaming JSON artifact. Callers choose the writer's cancellation policy;
wrap a writer with `teamy_cancellation::CancellableWriter` using the operation's
scope (`io` feature) when needed. CLI output renderers already receive a checked
writer. Put that adapter outside a borrowed `BufWriter` to check writes before
the buffer accepts them. Callers own explicit flushing and discard unfinished
buffered output after an error; a failure can retain an emitted prefix. Pass
`&mut Vec<u8>` to collect a complete result in memory, with cancellation policy
chosen explicitly for that writer too. Custom text callbacks stream through the same
cancellation-aware writer; they need explicit scope checkpoints for long
computation that does not emit output.
Convert I/O failures into Eyre reports and use `CancellationReportExt` for
cancellation recognition. At error boundaries, `normalize_cancellation()` exposes
typed cancellation to native Eyre downcasts while preserving the original error
chain; `CliOutput` already performs this normalization for its renderers.

The standalone `audit` command partitions resource groups by name: exactly five
occurrences of ASCII `a`, `e`, `i`, `o`, or `u`, ignoring case, belong in
`with_five_vowels`; every other name belongs in `without_five_vowels`. Repeated
vowels count separately. Both lists contain typed `ResourceGroup` values, and
the report is emitted through `CliOutput` using the shared `--output-format`.

## Process and ownership contract

- Call `App::run` once from synchronous `main`, before another component installs
  error, tracing or signal hooks. It is not an embeddable/restartable runtime and
  rejects execution inside an existing Tokio runtime. `run_parsed` has the same
  ownership contract, accepts an already parsed consumer CLI, and permits startup
  policy based on the parsed command.
  Even constructing an `eyre::Report` first can install its default error hook;
  let the runner own startup before creating application error reports.
  `App::run` prepares console output before parsing. Custom process entrypoints
  that parse first should call `app::prepare_process()` before printing or
  unwrapping parse outcomes, so Windows help and diagnostics render ANSI colors.
- `App::parse_from::<Cli>(arguments)` accepts arguments without the executable
  name and returns a non-exiting `figue::DriverOutcome<Cli>`. Builtin flags are
  supplied internally, and a successful `.value` is your own `Cli`. Use
  `into_result()` to inspect the parsed value, help, version or errors. It does not
  install startup hooks or resolve auth. Explicit HTML-help/schema-export requests
  can still write files.
- The pinned Figue driver treats a missing required CLI positional as help with
  a corrected-command diagnostic and exits **0**; unknown flags/commands fail.
  A command with an omitted required positional does not invoke its handler but
  still reports success to the shell. Automation checking only the exit status
  can therefore mistake an incomplete invocation for completed work. Explicit
  `--help` requests with valid command paths also succeed, while malformed supplied
  values and handler errors fail. Teamy Figue also rejects malformed command paths
  followed by `--help`, rather than displaying help for only the valid prefix
  ([fix](https://github.com/TeamDman/figue/pull/5)).
  The runner deliberately retains that upstream distinction, including automatic
  help for omitted subcommands. Do not use an omitted required positional as a
  reliable nonzero-status check. Changing that policy is a separate Figue follow-up,
  not a hidden runner override.
- The handler receives owned arguments and context. Its closure is invoked, and
  its returned future is polled, under the runtime and, when enabled, terminal task locals.
  Closure capture initializers still execute in the caller before `run`.
  Neither a `Send` nor a `'static` handler-future bound is imposed.
- `App::run` and `run_parsed` enable automatic cooperative cancellation by
  default. The handler's `cancellable_scope` runner supplies
  `context.cancellation`; pass that scope directly to SDK operations without
  another top-level wrapper. It checks at poll boundaries, not after every
  syntactic `.await`. Long synchronous loops need a scope checkpoint or a yield.
  Deeply nested deferrals keep their owning wrappers polling until the protected
  unit finishes; independent sibling scopes remain cancellable.
  Use `App::new(name, version).automatic_cancellation(false)` when the handler
  owns asynchronous cleanup or must drain background jobs after cancellation.
  This selects a `with_async_scope` runner: the body observes cancellation through
  checkpoints and cancellable children and controls its early return and cleanup.
  Cloud Terrastodon's own CLI selects this mode so interactive UIs can restore the
  terminal and drain pending work before the handler returns.
  The raw signal is also requested quietly when the handler finishes or unwinds;
  an existing cancellation reason is preserved. Join your background work before
  return. Blocking tasks can still delay runtime shutdown indefinitely.
- `--debug` explicitly captures panic backtraces without changing process
  environment variables. Ordinary error backtrace capture follows
  `RUST_LIB_BACKTRACE`/`RUST_BACKTRACE`. Rapid repeated Ctrl-C retains the cancellation
  library's immediate process-exit behavior, which does not run destructors.

## Cancellation in SDK operations

Pass `&context.cancellation` as the final argument to SDK fetch helpers and
request constructors. This mandatory `CancellationScope` belongs to a bounded
callback or future; requests retain a clone and pass it through serial operations,
transport execution, and pagination. A raw `CancellationToken` holds cancellation
intent and notifications, without an execution policy or a `bail_if_cancelled`
method. The caller owns deciding whether to begin another operation.

The `async` feature supplies runtime-independent `cancelled().await` notification
and two execution helpers. `App` supplies the handler's cancellable root by
default, so these helpers are for nested operations or explicit coordinators.
`with_async_scope(|scope| async move { ... })`
owns a scope while leaving cancellation checkpoints and cleanup to its body.
`cancellable_scope(|scope| async move { ... })` additionally stops polling the
body when its scope may acknowledge cancellation. Both admit their factories
on the first poll. Use the callback's supplied scope for that operation's SDK
requests; capturing an ancestor skips the new ownership relationship.

Start independent concurrent jobs with linked `scope.cancellable_scope(...)`
children. A deeply nested protected unit then keeps its owning wrappers polling,
while ordinary sibling jobs remain cancellable. For a separately cancellable UI
or invocation, `scope.with_async_child_scope(...)` or
`scope.cancellable_child_scope(...)` also returns a raw child signal for its stop
button. Signal inheritance remains separate from execution-scope ownership.

Automatic checks occur at wrapper poll boundaries, rather than every internal
`.await`. Already-ready awaits and synchronous loops may continue within one
poll. A scope checkpoint or a direct yield, as in the printing loop above,
gives cancellation an opportunity to be observed. Keep explicit checkpoints at
meaningful synchronous transitions when needed. A completed body's original
success or error wins over concurrent cancellation.

The independent `deferral` feature provides `with_deferral(reason, |protected| { ... })`
for synchronous units and, with `async`,
`deferred(reason, |protected| async move { ... }).await` for protected futures.
Enter before the irreversible step and pass the supplied protected scope through
the mutation and its required cache or journal completion. Protection ends when
the body completes or its owner drops; cloned handles do not extend it. With
tracing enabled, cancellation logs the explanation and source location once per
active protected unit. `is_cancelled()` and `cancelled().await` still expose raw
intent throughout protection.

For a single HTTP exchange, `RestRequest` supplies this policy: GET, HEAD, and
OPTIONS are cancellable by default; other methods defer cancellation through
response processing, decoding, and execution artifacts. Mark documented read-only
POST operations with `.cancellation_safe()`. SDK code can rely on that protection
for a single request. An explicit SDK deferral covers additional completion work,
such as a mutation followed by affected-cache invalidation, a durable journal
update, or related writes.

`CommandBuilder` conservatively defers cancellation by default for the complete
run operation, including subprocess execution, output decoding, and execution
artifacts. Mark read-only or otherwise safely interruptible commands with
`.cancellation_safe()`. This opt-in honors an enclosing protected scope. Keep
explicit deferrals for a command followed by required cache invalidation, durable
journal updates, or related writes; a single command needs no extra wrapper.
Cancellation requested before admission prevents either policy from starting new
work. Once admitted, the default also protects configured retries and polite
delays; timeouts, ordinary errors, and explicit future drops still apply.

New optional protected work rejects cancellation already requested before
admission. Use `finish_required(...)` or `finish_required_sync(...)` for cleanup
already owed after a request; these cannot reopen a closed scope. Protected bodies
retain their own results, with cancellation observed at the next scoped checkpoint
or child admission. Ordinary errors still follow Rust control flow: a `?` inside
the unit can return before later cleanup statements.

Keep owning and polling protected work. Explicit future drops, task aborts,
fail-fast `select!` or `try_join!` branches, and process termination can interrupt
it. Parallel required work needs a coordinator that retains and drains it on
early exits. Repeated Ctrl-C retains the force-exit behavior.

## Validation and publication boundary

```powershell
cargo +1.96.0 check -p cloud-terrastodon-standalone-cli --offline --locked
cargo +1.96.0 test -p cloud-terrastodon-standalone-cli --bin field-notes vowels --offline --locked
```

The filtered unit test in `main.rs` checks the vowel policy using synthetic names.
These validation commands do not invoke the CLI, resolve tenant defaults, obtain
credentials or read Azure resources.

The example has `publish = false`: it is a neutral application example, not
an implementation of a particular business task. No release is performed by this
workflow. See [Updating Facet](UPDATING_FACET.md) for dependency maintenance and
release checks. Before crates.io releases, verify clean
packaged consumers and publish the exact dependency versions first.
`publish.ps1 -DryRun` previews production dependency order and supports both
new packages and new versions of existing packages.

Use only `cargo clean` if build-space cleanup is necessary.
