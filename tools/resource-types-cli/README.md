# Resource-type maintenance

This unpublished workspace tool owns generation of
`crates/azure_resource_types/src/resource_types.rs`. The generated crate has no
dependency on the tool, process execution, credentials, or HTTP clients.

The tool reads a local JSON array of full provider/resource-type names. All three
commands default `--catalogue` to `tools/resource-types-cli/catalogue.json` in the
checkout used to build this unpublished tool. The path is anchored to the tool's
compile-time manifest directory, so changing the process working directory does
not change the default. Override it with `--catalogue path/to/catalogue.json`;
an explicitly supplied relative path remains relative to the working directory.
Selecting the default does not create a catalogue, and `generate`/`check` still
require an existing input file. Preserve the upstream spelling, for example:

```json
[
  "Example.Compute/servers",
  "Example.Compute/servers/extensions"
]
```

Refresh the input only when live access to the selected tenant is intended:

```powershell
cargo run --offline --locked -p cloud_terrastodon_resource_types_cli -- --account admin --auth-source azure-cli refresh --tenant example-lab
```

Omitting `--tenant`, or passing `--tenant default`, uses the explicitly configured
Cloud Terrastodon default tenant. Select another tracked tenant with
`--tenant example-lab` or a tenant UUID. `--account` selects a registered account
within that tenant; omission requires a configured account default. The global
`--auth-source` optionally constrains the account's source; `auto` uses its stored
source. Register and log in through `ct account` first. No ambient Azure CLI
login, subscription, or tenant is selected. `--auth-source none` rejects refresh
before account lookup.

`--offline` controls Cargo dependency downloads; **refresh itself contacts Azure**.

The command uses `azure::AzureResourceProviderListRequest`, backed by
[Providers - List At Tenant Scope, API 2021-04-01](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01).
The SDK owns authentication, cached pages, and pagination; `refresh` invalidates
that selected account/tenant request before fetching. The SDK confines continuation
URLs to the HTTPS public-cloud ARM `/providers` collection. Only validated full
resource-type names are saved, with no tokens, credentials, tenant identifiers,
or other response metadata. Existing input survives a failed or cancelled refresh
before publication. Refresh does not modify generated Rust source: review the
saved catalogue, then run `generate` separately.

Reusable provider records, nested metadata, and registration state live in
`azure_types`; the public list request lives in `azure`. The tool owns only
selection, catalogue validation/rendering, and file publication. Subscription
registration queries and mutations can use these models in future SDK operations;
the current list request is tenant-scoped.

Provider metadata carries namespace-relative resource-type names. The domain
types can qualify these into the existing `ResourceType` enum, whose `Other(String)`
preserves newly discovered full names. Catalogue generation uses the original
namespace and relative-name spelling so an older generated enum cannot overwrite
the service's current spelling. No bootstrap feature or alternate wire schema is
needed.

From the repository root:

```powershell
cargo run --offline --locked -p cloud_terrastodon_resource_types_cli -- generate
cargo run --offline --locked -p cloud_terrastodon_resource_types_cli -- check
```

Both commands default `--output` to
`crates/azure_resource_types/src/resource_types.rs` in the checkout used to build
the tool. Like the catalogue default, this path is anchored to the compile-time
tool manifest directory and does not depend on the process working directory.
These examples use the default catalogue and generated source. Add
`--catalogue path/to/catalogue.json` or `--output path/to/resource_types.rs` to
override either path; explicit relative overrides use the working directory.
`generate` and `check` read only the selected local input. They do not inspect Azure
configuration, acquire credentials, read execution caches, or query Azure.
`generate` writes only the selected output, whose parent must already exist,
using a temporary file in that directory and atomic replacement. Identical output
is left untouched. `check` reports missing or stale output and never writes it.
The shared application host supplies cancellation, logging, and output formatting.
Startup records authentication preferences without resolving accounts or tokens.
Only `refresh` binds those preferences to its tenant and account; `generate` and
`check` remain local operations.

Exact duplicates are removed, and the order is deterministic. Case-insensitive
spelling conflicts, generated-identifier collisions, invalid Rust identifiers,
empty path components, and empty catalogues are errors. These are generator
constraints; they do not claim to describe every Azure resource naming rule.
Known values retain the supplied spelling, parsing is case-insensitive, and
unknown values continue to use `Other(String)` without changing their input.

The tool uses the workspace-pinned Rust syntax formatter (`prettyplease`), so
rendering needs no installed command-line formatter or network connection.
Generated enum and implementation blocks carry `#[rustfmt::skip]`: their layout
belongs to this generator, and ordinary workspace formatting leaves the exact
bytes stable for subsequent `check` runs. Output paths must be representable as
UTF-8 for structured command reports; this is validated before publishing files.
The existing checked-in inventory is not refreshed merely by building or testing the
workspace. Supply a reviewed complete catalogue before intentionally regenerating
the production source; the example above is only a synthetic fixture.

Offline verification:

```powershell
cargo test --offline --locked -p cloud_terrastodon_resource_types_cli
cargo clippy --offline --locked -p cloud_terrastodon_resource_types_cli --all-targets -- -D warnings
```

The tool's tests use only synthetic names and temporary directories.
