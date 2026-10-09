# Cloud Terrastodon application runner

Reusable startup for a CLI whose root struct and subcommands belong to its consumer.
Use the facade's `app` feature with `default-features = false`, or depend directly
on this crate. See [the standalone consumer](../../examples/standalone-cli) and
[the application guide](../../docs/REUSABLE_APPLICATIONS.md).

Flatten `GlobalArgs` and `figue::FigueBuiltins` in your public CLI, implement
`ApplicationCli::global_args`, and call `App::new(name, version).run(handler)`
from synchronous `main`. Your types do not need `Debug`, `Arbitrary`, or registry
registration. `parse_from` returns a non-exiting Figue outcome without installing
startup hooks or resolving authentication.

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
use once per process before another component installs those hooks. Cancellation
is cooperative. The token is also cancelled when the handler finishes or unwinds,
but callers must join their own background work. `--debug` explicitly captures
panic backtraces without mutating `RUST_BACKTRACE`; ordinary error backtraces still
follow the environment. Explicit Figue HTML-help/schema-export requests can write
files even through the non-exiting parser.

The prepared release uses the independent `teamy-facet-*` and `teamy-figue`
packages with canonical Rust aliases. Local development requires the sibling
fork clones; consumers need no root patch block. Registry installation requires
publication of the exact Teamy versions first. See
[`docs/UPDATING_FACET.md`](../../docs/UPDATING_FACET.md).
