# Cloud Terrastodon Examples

## Single-file resource group listing

[resource_group_list.rs](resource_group_list.rs) uses the shared application runner
with a root CLI containing only `GlobalArgs` and Figue's builtins. There are no
subcommands or required positional arguments. The runner supplies logging, error
reporting, a Tokio runtime, authentication preferences and cooperative cancellation.

This is a Cargo example target: its dependencies, features, patches and lockfile
come from the repository root, so it needs no separate `Cargo.toml`. The default
`full` feature includes `app-auth` (which includes `app`) and `azure`.

Show help without resolving authentication or contacting Azure:

```powershell
cargo +1.96.0 run --example resource_group_list -- --help
```

To actually list resource groups (uses Azure CLI's default tenant and may contact
Azure):

```powershell
cargo +1.96.0 run --example resource_group_list -- --auth-source azure-cli --log-filter warn
```

`--auth-source` controls authentication for the resource-group request; tenant
selection still uses Azure CLI. Running with no arguments also performs the list,
using the shared authentication defaults. Cancellation is checked between requests
and output rows; it does not interrupt an in-flight request in these helpers.

To avoid enabling the full Cloud Terrastodon command tree and UIs, select only
the example's required facade features:

```powershell
cargo +1.96.0 run --no-default-features --features app-auth,azure --example resource_group_list -- --help
cargo +1.96.0 test --no-default-features --features app-auth,azure --example resource_group_list --offline --locked
```

The tests call Figue's arbitrary-based consistency and round-trip helpers, requiring
500 consistent serializations and 128 successful round trips. They do not run the
application or its Azure handler. CLI generation is test-only; builtins stay at
their defaults and equality compares the shared global arguments. Shared help
rendering is covered in the app crate's tests rather than repeated here.
The root manifest provides Facet, Arbitrary and the app's `arbitrary` feature as
dev-dependencies for these checks; the CLI still does not need `Debug`.

The Figue integration includes [first-party PathBuf serialization](https://github.com/bearcove/figue/pull/126),
so generated `GlobalArgs::log_file: Option<PathBuf>` values can be emitted and
parsed again without a wrapper. Both helpers pass at their original coverage
counts with paths still enabled. UTF-8 paths retain their exact text; non-UTF-8
native paths return a serialization error instead of being converted lossily.

## Independent consumer

[standalone-cli](standalone-cli) intentionally has its own `Cargo.toml`, lockfile
and repeated patches. It tests an independent project using a local Cloud
Terrastodon dependency, with its own subcommands in a single `src/main.rs`.
It enables only `app`, so authentication is not included in that consumer.

See [reusable applications](../docs/REUSABLE_APPLICATIONS.md) for the shared API
and the pinned Figue driver's missing-required-argument exit-status caveat.

Most Azure functionality also has examples in its adjacent tests, including
[ResourceGroupListRequest](../crates/azure/src/resource_group_list_request.rs).
