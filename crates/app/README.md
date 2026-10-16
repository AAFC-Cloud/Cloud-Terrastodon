# Cloud Terrastodon application runner

Reusable startup for a CLI whose root struct and subcommands belong to its consumer.
Use the facade's `app` feature with `default-features = false`, or depend directly
on this crate. See [the standalone consumer](../../examples/standalone-cli) and
[the application guide](../../docs/REUSABLE_APPLICATIONS.md).

Flatten `GlobalArgs` in your public CLI, implement
`ApplicationCli::global_args`, and call `App::new(name, version).run(handler)`
from synchronous `main`. `App::parse_from` and `App::run` supply Figue's standard
builtin flags through an internal wrapper; parsed values and handlers receive
your own CLI type. `App::run_parsed` accepts an already parsed consumer CLI.
Your types do not need `Debug`, `Arbitrary`, or registry registration.
`parse_from` returns a non-exiting `figue::DriverOutcome<Cli>` without installing
startup hooks or resolving authentication. A successful `into_result()` contains
your parsed CLI in its `.value` field.

`App::run` prepares Windows console output before parsing, so ANSI colors work
for help and parse errors too. A custom process entrypoint using `parse_from`
should call `prepare_process()` before printing or unwrapping its parse outcome.

`GlobalArgs::output_format` exposes `--output-format text|json|facet-pretty|auto`.
`facet-pretty` (also accepted as `facet`) displays the underlying data using
Facet's pretty printer, while `text` allows commands to provide their own
presentation. Commands can
return `CliOutput::facet(value)` for generic Facet output, or use
`CliOutput::facet_with_text(value, |value| { ... })` with a
callback returning `Result<String>` for custom text. JSON and `facet-pretty`
continue to render the same underlying value through the shared Facet renderer.
Implement `CliOutputValue` and return `CliOutput::new(value)` when all formats
need custom rendering. Call
`output.emit(cli.global_args().output_format)` in the handler: `auto` or omitted
formats use text in interactive terminals and JSON in redirected stdout or
PowerShell pipelines.
`CliOutput::none()` represents commands with no result to emit.

`output.with_result(result)` defers a command error until its output has been
written and flushed, allowing an unsuccessful REST response to remain visible.
The deferred result does not change serialized output. `render` previews output;
`emit` writes to stdout, while `write_to(format, stdout_is_terminal, writer)`
supports a caller-owned writer and returns any deferred command error.
The supplied terminal status controls automatic format selection only; renderers
detect color support themselves using the stdout stream and color preferences.

Optional `auth` enables `--auth-source` and `AppContext::auth`. Optional `terminal`
enables coordinated pickers/logs. Neither enables Cloud Terrastodon's full command
tree or egui. Without them, the runner does not depend on the registry or cloud
credentials. Features are additive across a Cargo dependency graph.

The runner owns process-global error/tracing/Ctrl-C hooks and its Tokio runtime;
use once per process before another component installs those hooks.
`run` and `run_parsed` automatically wrap the handler in a cancellable execution
scope. Pass `&context.cancellation` directly to SDK operations; no extra handler
wrapper is needed. Checks occur at poll boundaries, rather than every syntactic
`.await`: already-ready awaits and synchronous loops can continue within one poll.
Use a scope checkpoint or `tokio::task::yield_now().await` in long synchronous
loops. Deeply nested deferrals retain the handler while protected work finishes;
independent sibling scopes remain cancellable.

Use `App::new(name, version).automatic_cancellation(false)` when the handler must
own asynchronous cleanup or drain background jobs after cancellation. This keeps
the handler polling while it observes cancellation through its scope or linked
cancellable child operations. Cloud Terrastodon's own CLI selects this mode so
interactive UIs can restore the terminal and drain their pending work before the
handler returns. Neither mode preempts blocking or CPU work. The raw signal is
also requested when the handler finishes or unwinds; callers still own joining
their background work. `--debug` explicitly captures
panic backtraces without mutating `RUST_BACKTRACE`; ordinary error backtraces still
follow the environment. Explicit Figue HTML-help/schema-export requests can write
files even through the non-exiting parser.

The workspace uses the independent `teamy-facet-* 0.50.0-rc.7` and
`teamy-figue 6.0.0-rc.2` packages with canonical Rust aliases. Workspace builds
resolve these exact versions from crates.io; consumers need no root patch
block. Cancellation uses the exact registry dependency
`teamy-cancellation =0.5.0` with `async`, `deferral`, `facet`, and `arbitrary`
enabled, supplying the linked `CancellationScope` API. No sibling cancellation
checkout is required. See
[`docs/UPDATING_FACET.md`](../../docs/UPDATING_FACET.md).
