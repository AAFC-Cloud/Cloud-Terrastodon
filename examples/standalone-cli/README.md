# Standalone consumer CLI

`src/main.rs` owns the public `Cli`, shared global arguments, custom subcommands,
and handler. Cloud Terrastodon supplies startup, logging, and cooperative
cancellation. The consumer derives neither `Debug` nor `Arbitrary` and imports
the re-exported Facet/Figue APIs. One direct `facet = "=0.50.0-rc.7"` dependency is
also needed: Figue's attribute expansion currently requires `::facet` in the
consumer's extern prelude, even with a Facet crate-path annotation on the derive.
That dependency participates in the same root patch; it does not introduce a
second Facet version. No separate Figue dependency or macro shim is needed.

From the repository root:

```powershell
cargo +1.96.0 run --offline --manifest-path examples/standalone-cli/Cargo.toml -- echo "hello"
cargo +1.96.0 run --offline --manifest-path examples/standalone-cli/Cargo.toml -- inspect words --help
cargo +1.96.0 test --offline --manifest-path examples/standalone-cli/Cargo.toml
```

This is an independent workspace with a local path dependency and its own
explicit patches. Keep all 14 immutable patch entries aligned with the parent
manifest: dependency patches are not inherited by consumers. The `app` feature
with `default-features = false` opts out of the full Cloud Terrastodon CLI,
authentication, and terminal interfaces. Publication is deliberately disabled.

`echo` writes its result to stdout and diagnostics to stderr. Add `--log-file
events.ndjson --log-file-filter debug --log-filter off` to retain detailed JSON
logs without console logging. Shared flags also work after a leaf command.
`inspect words "one two" --expect 3` demonstrates an ordinary handler error;
`inspect words "one two" --stop-after 1` demonstrates cooperative cancellation
between work items, without signals, sleeps, or cloud access.

The pinned Figue driver treats a missing required positional (for example,
`field-notes echo`) as a help outcome: it prints help and the missing-argument
diagnostic to stdout, exits **0**, and does not dispatch the handler. Unknown
commands/options and malformed values do fail. This fixture explicitly tests
that existing distinction; the runner does not reinterpret Figue outcomes.

Call `App::run` once from synchronous process entry: it owns a runtime and
process-global hooks. For parsing without startup or process exit, use
`App::parse_from::<Cli>(args).into_result()`; arguments exclude the program name,
and a successful outcome contains the parsed CLI in its `.value` field.

## Exporting the CLI as a Rust library

A `pub Cli` inside a binary is not importable by another crate. If you need a
library API, move the argument declarations and `ApplicationCli` implementation
into `src/lib.rs` (or `src/cli.rs` with `pub mod cli; pub use cli::Cli;` in
`lib.rs`). Import them from the package's library in the small `main.rs` handler.
This is optional; the executable fixture intentionally keeps its implementation
in one file. Its separate tests import that file only to verify the public
parsing contract, not as a recommendation to export `main.rs` as a library.
