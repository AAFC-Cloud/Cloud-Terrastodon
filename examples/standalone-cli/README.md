# Standalone consumer CLI

`src/main.rs` owns the public `Cli`, shared global arguments, `audit` command,
and handler. Cloud Terrastodon supplies startup, logging, cooperative
cancellation, and Figue's standard builtin flags internally. The handler receives
the consumer's own `Cli`. The consumer derives neither `Debug` nor `Arbitrary`
and imports the re-exported Facet/Figue APIs. One direct dependency named `facet`, inherited
from the workspace's exact `teamy-facet 0.50.0-rc.7` dependency, is also needed:
Figue's attribute expansion currently requires `::facet` in the
consumer's extern prelude, even with a Facet crate-path annotation on the derive.
That dependency uses the same Teamy package identity as the facade. No
separate Figue dependency, root patch, or macro shim is needed.

From the repository root:

```powershell
cargo +1.96.0 check --offline --locked -p cloud-terrastodon-standalone-cli
cargo +1.96.0 test --offline --locked -p cloud-terrastodon-standalone-cli --bin field-notes vowel_audit_counts_exactly_five_ascii_vowels_ignoring_case
```

This package belongs to the Cloud Terrastodon workspace and shares its root
`Cargo.lock`. It uses the local Cloud Terrastodon crate and the workspace's
exact published Teamy Facet/Figue dependencies from crates.io. The workspace also
uses the sibling `../teamy-cancellation` development checkout with `async`,
`deferral`, `facet`, and `arbitrary` enabled. Its new execution-scope API is not in
published `teamy-cancellation 0.4.0`. The `app-auth` and `azure` features provide shared
startup, authentication, and Azure SDK operations. `default-features = false` opts out
of the full Cloud Terrastodon command tree. Publication is deliberately disabled.

## Audit resource groups

When you run it, `audit` reads Azure resource groups using
`fetch_all_resource_groups`, then partitions the typed records into
`with_five_vowels` and `without_five_vowels`. It counts exactly five occurrences
of ASCII `a`, `e`, `i`, `o`, or `u`, ignoring case; repeated vowels count, while
`y` and accented letters do not. A name with six vowels belongs in the second
partition.

These commands read Azure data when you execute them:

```powershell
cargo +1.96.0 run --offline --locked -p cloud-terrastodon-standalone-cli -- audit
cargo +1.96.0 run --offline --locked -p cloud-terrastodon-standalone-cli -- audit --tenant YOUR_TENANT_ID --output-format json
```

`--tenant` accepts an ID, tracked tenant alias, or `default`. The default uses the
authentication source's tenant when available, otherwise the active Azure CLI
tenant. The SDK fetches groups across accessible subscriptions in that tenant
and uses its existing cache. `--auth-source` selects the authentication policy.
`App` automatically wraps the handler in a cancellable scope. The handler passes
`&context.cancellation` directly to tenant resolution and the SDK fetch, without
an extra handler wrapper or repeated post-await checks. The SDK observes
cancellation during reads and between pages. The synchronous partitioning loop
checks the scope between records, with another checkpoint before emitting output.
Automatic checks occur at poll boundaries, rather than every syntactic `.await`;
already-ready awaits can complete within one poll. Explicit checkpoints make
cancellation responsive during this synchronous work without adding a runtime
dependency to the example. Blocking work is not preempted. Deeply nested deferrals
retain the handler while required work finishes.

The report retains full resource group records. Shared `--output-format` selects
text, JSON, or Facet Pretty; the default selects text in a terminal and JSON for
redirected stdout. The vowel-policy test above uses synthetic names and does
not initialize authentication or execute SDK requests.

`audit` is the only command. Add `--log-file events.ndjson --log-file-filter
debug --log-filter off` to retain detailed JSON logs without console logging.
Shared flags also work after `audit`.

Call `App::run` once from synchronous process entry: it owns a runtime and
process-global hooks. For parsing without startup or process exit, use
`App::parse_from::<Cli>(args).into_result()`; arguments exclude the program name,
and a successful outcome contains the consumer's own CLI in its `.value` field.
Both `parse_from` and `run` add builtin flags internally.
`App::run_parsed(cli, handler)` accepts an already parsed consumer CLI and has the
same automatic cancellation default. A handler that owns asynchronous cleanup or
must drain background jobs can opt into
`App::new(name, version).automatic_cancellation(false)`, then handle cancellation
through its supplied scope and linked cancellable child operations.

## Exporting the CLI as a Rust library

A `pub Cli` inside a binary is not importable by another crate. If you need a
library API, move the argument declarations and `ApplicationCli` implementation
into `src/lib.rs` (or `src/cli.rs` with `pub mod cli; pub use cli::Cli;` in
`lib.rs`). Import them from the package's library in the small `main.rs` handler.
This is optional; the executable example keeps its implementation in one file.
