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
`CliOutput::facet_with_text(value, |value, writer, cancellation| { ... })` with a
callback that writes custom text to the supplied `std::io::Write` and returns
`Result<()>`. Use `write!` or `writeln!` to emit records incrementally. The writer
checks cancellation between bounded writes; check the supplied scope during
long computation that does not write. JSON and `facet-pretty`
continue to render the same underlying value through the shared Facet renderer.
Implement `CliOutputValue` and return `CliOutput::new(value)` when all formats
need custom rendering. Call
`output.emit(cli.global_args().output_format, &context.cancellation)` in the handler: `auto` or omitted
formats use text in interactive terminals and JSON in redirected stdout or
PowerShell pipelines.
`CliOutput::none()` represents commands with no result to emit.

`output.with_result(result)` defers a command error until its output has been
written and flushed, allowing an unsuccessful REST response to remain visible.
The deferred result does not change serialized output. All three output methods
accept a `&CancellationScope`: `render(format, stdout_is_terminal, cancellation)`
collects an explicit buffered preview without adding a final newline or flushing;
`emit(format, cancellation)` writes to stdout; and
`write_to(format, stdout_is_terminal, writer, cancellation)` writes
to a caller-owned writer and returns any deferred command error. `emit` and
`write_to` add a final newline if needed and flush after successful rendering;
callbacks leave this completion work to the output owner. Output checks
cancellation between bounded writes, honoring protected completion scopes. It
cannot interrupt a blocking operating-system write already in progress.
The supplied terminal status controls automatic format selection only; renderers
detect color support themselves using the stdout stream and color preferences.

`CliOutputValue::write_to` receives the selected format, writer, and scope, so a
custom renderer can emit large or unbounded output incrementally without first
building a `String`. The writer supplies cancellation checkpoints during output;
check the scope during computation that does not write. The built-in JSON and
Facet pretty renderers write incrementally through the same cancellation-aware
writer, honoring linked deferrals. Computation that does not write, including
allocation and user-defined proxy conversions or formatters, continues until the
next checkpoint. An operating-system call already in progress cannot be
interrupted. Custom text
callbacks also write incrementally through the supplied writer. An error or
cancellation can leave the completed prefix in the destination.

For another JSON artifact, call
`facet_json::to_writer_std_with_options(writer, value, &options)` to stream through
a caller-owned writer. The writer owns cancellation policy: wrap it
with `teamy_cancellation::CancellableWriter` and the supplied scope when needed.
`CliOutput` already supplies a checked writer to its renderers. When buffering,
put `CancellableWriter` outside a borrowed `BufWriter`, so writes are checked
before the buffer accepts them. Facet installs no buffer and does not flush;
callers handle flushing and discard unfinished buffered output on errors.
Cancellation or another error can leave a partial prefix. Pass `&mut Vec<u8>`
when a complete in-memory result is needed; a plain vector supplies no
cancellation checkpoints itself.

`CancellationReportExt`, re-exported by `app` and `command`, recognizes typed
cancellation through Eyre contexts, exposed error sources, and I/O payloads.
Use `is_cancellation()` for control flow or `as_cancellation()` to read the reason.
`CliOutput` uses `normalize_cancellation()` to expose recovered cancellation to
native Eyre downcasts while retaining the original report and its source chain.
Other artifact writers can convert I/O errors into Eyre reports and normalize
cancellation at their error boundary in the same way.

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

Consumers must use the same Teamy Facet package identity as the app crate.
Figue's attribute macros also require a direct dependency with the canonical
`facet` alias. The [root manifest](../../Cargo.toml) defines dependency versions
and sources. See
[`docs/UPDATING_FACET.md`](../../docs/UPDATING_FACET.md).
