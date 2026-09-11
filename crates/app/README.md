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

This is a locally validated development API, not a patch-free publication claim.
Consumers currently repeat the Facet/Figue root patches documented in
`docs/UPDATING_FACET.md` at the repository root.
