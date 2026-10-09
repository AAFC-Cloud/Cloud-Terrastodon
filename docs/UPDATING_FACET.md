# Updating Facet and Figue

Cloud Terrastodon uses TeamDman's independent Facet and Figue forks. The
prepared registry graph uses `teamy-facet-* 0.50.0-rc.7` and
`teamy-figue` / `teamy-figue-attrs 6.0.0-rc.1`, with canonical Rust dependency
aliases such as `facet` and `figue`. These packages are not the official
upstream crates. They must be published before Cloud Terrastodon 0.37.0 can
be installed from crates.io.

Local development uses exact-version dependencies with sibling paths to
`../facet`, `../facet-format`, and `../figue`. Cargo removes those paths when
packaging, leaving registry dependencies on the Teamy packages. Neither the
workspace nor the standalone example requires `[patch.crates-io]` overrides.

## 0.37.0 release preparation

The coordinated release contains 43 packages:

| Family | Packages | Version |
| --- | --- | --- |
| Core Facet fork | 11 runtime crates, plus Default, Testhelpers and Testhelpers Macros | `0.50.0-rc.7` |
| Format fork | Format, Dessert, JSON and Value | `0.50.0-rc.7` |
| Figue fork | Figue and its attribute macros | `6.0.0-rc.1` |
| Cloud Terrastodon | Facade and 22 workspace crates | `0.37.0` |

The vendored textarea has been replaced with the published
[`ratatui-textarea 0.9.3`](https://github.com/ratatui/ratatui-textarea), maintained
by Ratatui in a separate repository. It supports the selected Ratatui 0.30.1
and Crossterm 0.29 graph, so no Teamy textarea package is needed.
The 22 existing picker tests and Azure DevOps TUI/Ratatui compilation pass.
The lockfile adds this package and removes 32 obsolete vendored dependency
entries; retained third-party versions are unchanged.

Source-tree verification passes the core library check and five reflection
alias regressions, 142 Format-family tests, 760 Figue tests and 30 Figue
Rustdoc examples (eight existing examples remain ignored). The historical
converted-default regression also passes without porting old production code.
Format's process-global cache tests require serial execution for this suite.
Cloud Terrastodon's all-target workspace check, 20 app tests and eight exact
CLI schema/help/registry tests pass. The initial CLI substring filter also
matched 61 nested command tests; the corrected selection runs the intended
eight exactly. The additional tests passed and were audited: one checked
existence of a fresh UUID-based cache path, without reading cache contents;
none reached credential loading, Azure CLI, HTTP or DNS.

All 43 normalized archives have registry dependencies without Git overrides
or local paths. An independent consumer using the archived facade and Teamy
packages passes all 14 standalone integration tests through a temporary Cargo
directory source, without a patch block. The same consumer also compiles the
facade's `full` feature against the normalized registry graph, including both
UIs and the SDK/CLI crates. Locked packaging with generated lock
files also succeeds for Core and Format. These are prepublication checks;
they do not claim the unpublished versions are already available on crates.io.

The ten named scratch/build directories have been removed. Recovery notes and
uncommitted patches are retained in `../fork-recovery/2026-10-09`, and the Format
clone is now at `../facet-format`. Figue's older local edits remain in its
named stash as well as the recovery patch. Old branches retain their history.

## Source policy

| Source | Role |
| --- | --- |
| `facet-rs/facet` | Core monorepo: Facet, macros, reflection, solver, path, errors and pretty printing |
| `bearcove/figue` | Figue and `figue-attrs`, separate from the core Facet repository |
| `facet-rs/facet-format` | Official split-out Format, JSON, Value and Dessert repository |
| Teamy registry packages | Publish the compatible core, Format/JSON/Value/Dessert, and Figue families together; keep their Rust aliases |
| `TeamDman/facet` | Current upstream plus the reviewed Cow and enum-alias PRs until compatible official releases include them |
| `TeamDman/facet-format` | Current upstream plus the reviewed scalar-enum error-span PR; Format and Dessert use the same immutable revision |
| `TeamDman/figue` | Current upstream plus our selected CLI PRs |

`teamy-main` is the primary branch in all three GitHub forks and the local
clones. Facet and Figue's old divergent `main` histories are preserved under
`legacy/published-main-2026-10-09`; they are not release branches. Package
READMEs and Rustdoc identify the forks and retain upstream attribution.

The release preparation builds on this recorded 2026-10-06 integration:

- Facet `345ebb7ba42f7c6644c1d89ef53288fe81204fa5`: official main
  `65bae5c31a7ce401bc44630fb96250ea884cfd3e` plus
  [Cow PR #2657](https://github.com/facet-rs/facet/pull/2657) and
  [enum-alias PR #2664](https://github.com/facet-rs/facet/pull/2664), addressing
  [issue #2663](https://github.com/facet-rs/facet/issues/2663).
- Format and Dessert `a1ca5f97253eb49fb080a39e32377dcebb0499f5`: official main
  `4279debff780ae1cd5b028201f446c26594b1120` plus
  [scalar-enum span PR #70](https://github.com/facet-rs/facet-format/pull/70),
  addressing [issue #69](https://github.com/facet-rs/facet-format/issues/69).
- Figue `d0df4a9b76ba05a41a5e984d99e979ef917f6a25`: official main
  `47801613b720a7d5a05a9c8222d90104331823e2` plus
  [documentation #119](https://github.com/bearcove/figue/pull/119),
  [no-Debug test helpers #121](https://github.com/bearcove/figue/pull/121),
  [inherited help #122](https://github.com/bearcove/figue/pull/122),
  [transparent scalars #124](https://github.com/bearcove/figue/pull/124),
  [PathBuf serialization #126](https://github.com/bearcove/figue/pull/126), and
  the reviewed nested-subcommand help correction.

The Figue integration is a normal merge of the previous published `teamy-main`
`04f6516d2d95ab6299457652c9c1f67ad2fb09db` and the reviewed application stack
`1ad7d9e1b4a97124a93780610861b55e478d13cc`. It preserves both histories and
the older public implementation-URL helper spelling, executable normalization,
environment-prefix hints, positional Vec support and duplicate-flag diagnostics.

PathBuf support is Figue-local: ordinary UTF-8 paths serialize without a
consumer newtype or another Facet change. Non-UTF-8 paths return an explicit
error. It also restores PathBuf defaults inside config roots (issue #105).
The independent PR has recorded upstream red/green regressions; Windows native
encoding rejection was executed, while Unix runtime validation remains pending
because the existing offline environment lacks required cached dependencies.

Transparent-scalar support was added after the application already built and
its eight acceptance failures were repaired. It is a separate typed-CLI
capability, not the explanation for those application failures.

The upstream baseline versions were Facet-family `0.50.0-rc.7` and Figue
`5.0.0-rc.6`. The Teamy Figue major version changes to 6 for its updated Facet
graph. Enable `facet-json/net` explicitly for unproxied IP addresses
in outage artifacts; core Facet's `net` does not enable Format serialization.

At the 2026-10-06 library checkpoint, the integrated Facet reflection suite has
95 passing tests, Format has six, and Figue has 500 passing library tests and
243 passing integration tests, with three library tests still ignored. Figue's
integration tests run serially because some manipulate synthetic environment
variables.

A temporary consumer using Cloud Terrastodon's actual `OutputFormat` source
accepts both `facet` and `facet-pretty`, retains `facet-pretty` serialization,
and highlights the rejected output-format value rather than a preceding
argument. The fixes were reproduced separately against the recorded official
revisions and have focused red/green library regressions. These checks use
synthetic values; no production services, credentials or application response
caches were accessed. This checkpoint supplements the historical application
acceptance results below rather than replacing them.

With the published Git pins, the workspace check passes for all targets. The
audited application selection passes 20 app tests, eight entrypoint
schema/help/registry tests, and all 14 standalone consumer integration tests.
Locked Windows dependency metadata has one source identity for each interacting
Facet/Figue package in both workspaces. Root lockfile changes update only the
selected sources; the standalone lockfile also records the existing tracing
crate's `tracing-error` dependency.
Freshly built build-folder help succeeds, and a synthetic invalid format on
the actual nested CLI reports the correct value before application startup.

## Package identities and consumers

All interacting Facet traits, shapes and values must share one compatible
core identity. Every normal/build dependency in the release graph names a
Teamy package, including JSON and Value. Mixing official `facet-*` packages
with `teamy-facet-*` creates distinct Rust types even when their versions match.

Keep the canonical dependency keys required by generated macro paths:

```toml
facet = { package = "teamy-facet", version = "=0.50.0-rc.7" }
figue = { package = "teamy-figue", version = "=6.0.0-rc.1" }
```

See [Cargo's dependency renaming reference](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml).
Normal/build dependencies have both a version and a development path. Fork
internal dev-dependencies use paths only so Cargo omits them from archives,
avoiding bootstrap cycles; run their complete test suites in the source trees.
Archive validation checks library builds and an independent consumer using
the normalized registry graph. Do not claim registry installation works until
the exact versions have actually been published.

Cloud Terrastodon currently uses Facet-free `teamy-cancellation 0.2.0`.
The cancellation repository's newer optional Facet/Figue features must be
tested in its own root. `teamy-mft` is no longer a dependency and does not
require synchronization.

## Update procedure

Use only `cargo clean` to free disk space during builds. Do not manually delete
build caches or artifacts with `rm`, `Remove-Item`, or other filesystem tools.
Choose an appropriately scoped Cargo target/package cleanup when needed.

1. Record old SHAs and dirty state; preserve existing branches and worktrees.
2. Fetch actual official upstreams and select immutable commits. Recheck
   package topology, versions and whether our PRs have already merged.
3. Start each proposed fix on a new branch from the exact latest fetched
   official upstream revision. Keep the change minimal and independently
   reviewable, with its own issue, PR and focused regression. Merge the reviewed
   fixes into the existing fork's `teamy-main` with normal merge commits,
   preserving its published history and selected behavior. Do not rewrite or
   replace old branches, or blindly copy old in-Facet Figue, tuple or Format
   adapters.
4. Test the fork stacks before publishing their new refs. Record the tested
   `teamy-main` SHAs without waiting for upstream PR merges. Update exact Teamy
   package versions and development paths together, including the standalone
   CLI example, then inspect each lockfile diff; avoid unrelated dependency
   updates. Verify normalized archives and a consumer without Git overrides.
5. Run application tests, source-identity checks and freshly built CLI help.
   Synthetic probes supplement this evidence; they do not prove a current
   application command requires every proposed library capability.

If `teamy-main` is missing, inspect the fork's existing branches and the
application's current pin before creating it from the reviewed compatible
stack. If the branch is older than the current pin, reconcile both histories
and verify retained behavior before advancing it. Starting over from official
main must not discard the application's selected, unmerged contributions.

The integration is tested with installed Rust 1.96.0 on Windows; this alone
does not establish a new minimum supported Rust version.

```powershell
cargo +1.96.0 check --workspace --all-targets --locked
$env:CLOUD_TERRASTODON_REAUTH = 'DENY'
cargo +1.96.0 test --workspace --locked --no-run
cargo +1.96.0 tree --locked --invert teamy-facet
cargo +1.96.0 tree --locked --invert teamy-facet-core
cargo +1.96.0 tree --locked --invert teamy-figue
cargo +1.96.0 tree --locked --duplicates
cargo +1.96.0 run --quiet --locked --bin cloud_terrastodon -- az tenant login --help
git diff --check
```

Preview the coordinated publication order, including first-time crates:

```powershell
./publish.ps1 -DryRun -ManifestPath ../facet/Cargo.toml,../facet-format/Cargo.toml,../figue/Cargo.toml,./Cargo.toml
```

The script skips only an already published exact version, fails on registry
errors other than a missing version, and orders packages by production/build
dependencies. `-DryRun` reads metadata without publishing. Actual publication
is a separate step after the prepared changes and validation are reviewed.

Do not use an installed, potentially stale `ct` executable as proof.
Entrypoint regressions check root/leaf global options, complete authentication
docs and parse-only global flags after a leaf. Outage artifact tests check
actual IPv4/IPv6 JSON without network access. Registry/Object Explorer tests
retain Cow borrow/promote and runtime lease behavior. Record credential or
environment limitations separately from compatibility failures.

Do not run the entire workspace suite blindly: several unignored tests use
live Azure/Gitea services or the user's credential/config stores.
`CLOUD_TERRASTODON_REAUTH=DENY` does not prevent use of cached credentials.
The audited local-library selection for this checkpoint is:

```powershell
cargo +1.96.0 test --offline --locked --no-fail-fast --lib `
  -p cloud_terrastodon_registry -p cloud_terrastodon_azure_types `
  -p cloud_terrastodon_azure_devops_types -p cloud_terrastodon_azure_resource_types `
  -p cloud_terrastodon_rest -p cloud_terrastodon_ui_ratatui `
  -p cloud_terrastodon_entrypoint -p cloud_terrastodon_credentials `
  -p cloud_terrastodon_gitea -p cloud_terrastodon_command `
  -p cloud_terrastodon_tracing -p cloud_terrastodon_user_input -- `
  --skip azure_devops_rest_command_cli::test::it_works `
  --skip project_list_authenticates_before_reading_organization_or_cache `
  --skip browser_project_list_requires_a_tenant_without_a_stored_session `
  --skip azure_devops_rest_client::test::it_works `
  --skip jwt::test::it_works --skip azure_devops_pat::test::non_empty `
  --skip it_fetches_repositories_without_empty_ids
```

Keep ignored tests ignored. This selection excludes live service/credential
tests and two tests whose missing host-configuration assumptions require
separate isolation; `--lib` also excludes command integration tests that run
Azure CLI. It is a dependency acceptance check, not full release validation.

At the 2026-09-08 application-repair checkpoint, this selection has **766
passing tests, zero failures, 12 ignored and seven filtered**. The previous
eight entrypoint failures are repaired: matching fixtures are deterministic,
the headless tenant-login dispatch test avoids the user's tracked-tenant
store, the CLI-auth test checks the error chain with a fresh organization
cache namespace, and missing registry producers have value-level coverage.
OAuth generators use validating constructors; generated JSON export requests
use local basenames and are never executed by their generator tests. Container
fixtures prove typed/JSON behavior, not realistic Azure resource combinations.
No additional Facet/Figue customization was needed for these failures.

Mutation deferral during export must run even when debug assertions are
disabled. The queue operation now occurs outside `debug_assert!`. The Ratatui
suite also passed with its debug assertions disabled using:

```powershell
cargo +1.96.0 test --offline --locked --lib -p cloud_terrastodon_ui_ratatui `
  --config 'profile.test.package.cloud_terrastodon_ui_ratatui.debug-assertions=false'
```

This tests the relevant release-mode control flow without claiming a complete
optimized release build. Keep the isolated license-response generator tests
separate from live Azure DevOps tests:

```powershell
cargo +1.96.0 test --offline --locked --lib -p cloud_terrastodon_azure_devops `
  azure_devops_user_license_entitlement_update::tests
```

The Cow adapter's unsafe contract remains a caller obligation, not a
whole-application soundness certification. Object Explorer cancellation now
destroys futures and unclaimed results synchronously under the same lock used
by background Tokio polling. Returned values conservatively inherit source
leases until deletion; even an apparently owned result can contain a nested
borrow. Shutdown destroys jobs, incomplete builders, then ready borrowers
before their sources. If value destruction panics or a dependency cycle prevents
that order, remaining owners are deliberately retained instead of freed.

Registered operations receiving runtime-borrowed input must not let borrowed
data escape into detached tasks, blocking jobs, globals, or other independent
state. Awaiting a spawned task does not establish cancellation safety. Such
work must receive genuinely owned data. The lifecycle regressions do not
constitute an exhaustive audit of every registered operation's call graph;
these application obligations do not require another Facet fork customization.

## Upstream submissions

Use `gh`; create local Markdown submission text for user review before
publishing. Keep real Cloud Terrastodon motivation and immutable public
source links. Include an accurate LLM-assistance disclaimer with the model
version when known; never invent an unavailable version. Redact local
usernames from public diagnostics and use the full project name.

Commit as `TeamDman <TeamDman9201@gmail.com>`. Obsolete repository hooks may
need a documented per-invocation override after equivalent validation, not
global disabling. Never rewrite old branches as an update shortcut.
