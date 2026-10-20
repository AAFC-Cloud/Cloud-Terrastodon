# Cloud Terrastodon Examples

## Group management application

[group-management](group-management) is a larger consumer-owned CLI with a small
entrypoint, separate command files, validated domain types, and a pure planner.
Each desired group has an explicit `tenantId`, `groupId`, and expected display
name, with independent exact/additive policies for members and owners.

Its first milestone provides local `validate` and read-only `plan` commands.
Tenant preflight rejects missing workload-identity coverage before directory
reads. Plans preserve resolved identities, observations, warnings, and blockers;
Graph v1.0's incomplete service-principal inventories prevent exact live
comparisons from being treated as actionable. There is no apply command yet.

Validate the synthetic example without directory access:

```powershell
cargo +1.96.0 run --offline --locked -p cloud-terrastodon-group-management -- validate --desired examples/group-management/desired.example.json
cargo +1.96.0 test --offline --locked -p cloud-terrastodon-group-management
```

See its [README](group-management/README.md) for the input contract, source layout,
planning commands, and subsequent apply/pipeline milestones.

## Single-file resource group listing

[resource_group_list.rs](resource_group_list.rs) uses the shared application runner
with a root CLI containing `GlobalArgs` and a typed `--tenant` selector.
There are no subcommands or required positional arguments. The runner supplies
Figue's standard builtin flags internally, logging, error reporting, a Tokio
runtime, authentication preferences, and cooperative cancellation.

This is a Cargo example target: its dependencies, features and lockfile
come from the repository root, so it needs no separate `Cargo.toml`. The default
`full` feature includes `app-auth` (which includes `app`) and `azure`.

Show help without resolving authentication or contacting Azure:

```powershell
cargo +1.96.0 run --example resource_group_list -- --help
```

These commands list resource groups and may contact Azure:

```powershell
cargo +1.96.0 run --example resource_group_list -- --auth-source azure-cli --tenant default --log-filter warn
cargo +1.96.0 run --example resource_group_list -- --auth-source azure-cli --tenant YOUR_TENANT_ID_OR_ALIAS --log-filter warn
```

`--auth-source` selects authentication. `--tenant` accepts a tenant ID, tracked
alias, or `default`. When omitted or set to `default`, it uses the authentication
context's known tenant, falling back to Azure CLI's active tenant when none is
known. The selector binds through `context.auth` before the resource-group fetch.
Running with no arguments also performs the list using these shared defaults.
`App` automatically wraps the handler in a cancellable scope. The handler passes
`&context.cancellation` directly to tenant binding and the SDK fetch, without an
extra wrapper. Automatic checks occur at poll boundaries; they do not intercept
every internal `.await`. The SDK also uses cancellable read boundaries and page
checkpoints.

Before each `println!`, the loop directly awaits `tokio::task::yield_now()`.
That suspension gives the handler's wrapper a chance to observe cancellation
before printing the row. An already-ready no-op await, such as `async {}.await`, does
not suspend; a zero-duration sleep can also be ready immediately. Neither is a
reliable replacement. Tokio may poll the same task again immediately, so the
yield does not promise that another task runs first. See the pinned
[Tokio yield documentation](https://docs.rs/tokio/1.52.3/tokio/task/fn.yield_now.html).
A blocking `println!` already in progress cannot be interrupted by this helper.

To avoid enabling the full Cloud Terrastodon command tree and UIs, select only
the example's required facade features:

```powershell
cargo +1.96.0 run --no-default-features --features app-auth,azure --example resource_group_list -- --help
cargo +1.96.0 test --no-default-features --features app-auth,azure --example resource_group_list --offline --locked
```

The tests call Figue's arbitrary-based consistency and round-trip helpers, requiring
500 consistent serializations and 128 successful round trips. They do not run the
application or its Azure handler. CLI generation is test-only. Shared help
rendering is covered in the app crate's tests rather than repeated here.
The root manifest provides Facet, Arbitrary and the app's `arbitrary` feature as
dev-dependencies for these checks; the CLI still does not need `Debug`.

The Figue integration includes [first-party PathBuf serialization](https://github.com/bearcove/figue/pull/126),
so generated `GlobalArgs::log_file: Option<PathBuf>` values can be emitted and
parsed again without a wrapper. Both helpers pass at their original coverage
counts with paths still enabled. UTF-8 paths retain their exact text; non-UTF-8
native paths return a serialization error instead of being converted lossily.

## Standalone application

[standalone-cli](standalone-cli) is the root workspace member
`cloud-terrastodon-standalone-cli`. It has its own `Cargo.toml` and shares the root
`Cargo.lock`. It inherits the workspace dependencies defined in the
[root manifest](../Cargo.toml). Its direct `facet` dependency has the same Teamy
package identity as the facade and supplies the canonical alias required by
Figue's attribute macros.
Its single `audit` command lives in `src/main.rs`.
It enables `app-auth` and `azure` with default features disabled, reusing the
shared runner's authentication context and the existing Azure SDK helpers.

Its `audit` command lists resource groups in accessible subscriptions of the
selected tenant. `--tenant` accepts a tenant ID or tracked alias; when omitted,
the existing tenant argument binds to the known authentication tenant or falls
back to Azure CLI's default. SDK request caches may supply the inventory.

Names with exactly five ASCII vowel occurrences (`a`, `e`, `i`, `o`, `u`, ignoring
case) go in `with_five_vowels`; all other names go in `without_five_vowels`.
Repeated vowels count separately. The typed report uses `CliOutput`, so the
shared `--output-format` selects text, JSON or Facet Pretty.

To read Azure resource groups yourself, run:

```powershell
cargo +1.96.0 run -p cloud-terrastodon-standalone-cli -- --auth-source azure-cli --output-format json audit
```

For local validation, check the package and run its pure vowel-policy test:

```powershell
cargo +1.96.0 check -p cloud-terrastodon-standalone-cli --offline --locked
cargo +1.96.0 test -p cloud-terrastodon-standalone-cli --bin field-notes vowels --offline --locked
```

These validation commands compile the CLI and test synthetic names without
executing the audit, resolving defaults, obtaining credentials or reading Azure.

See [reusable applications](../docs/REUSABLE_APPLICATIONS.md) for the shared API.

Most Azure functionality also has examples in its adjacent tests, including
[ResourceGroupListRequest](../crates/azure/src/resource_group_list_request.rs).
