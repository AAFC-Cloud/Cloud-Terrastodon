# Updating Facet and Figue

Cloud Terrastodon declares compatible registry requirements and selects
reviewed, unmerged changes through `[patch.crates-io]` at the workspace root.
The development graph temporarily uses two forks. This is not a promise of
patch-free crates.io consumption.

## Source policy

| Source | Role |
| --- | --- |
| `facet-rs/facet` | Core monorepo: Facet, macros, reflection, solver, path, errors and pretty printing |
| `bearcove/figue` | Figue and `figue-attrs`, separate from the core Facet repository |
| Official registry Format/JSON/Value | Split-out packages; no bundled Teamy Format snapshot |
| `TeamDman/facet` | Current upstream plus the reviewed Cow PR until an official compatible release includes it |
| `TeamDman/figue` | Current upstream plus our selected CLI PRs |

The root manifest records immutable revisions. The initial integration
branch `teamy/upstream-pr-stack-2026-09-08` in each fork contains:

- Facet `a6101f92fa88ada6dedd80899140577e554bd5d3`: official main
  `65bae5c31a7ce401bc44630fb96250ea884cfd3e` plus [Cow PR #2657](https://github.com/facet-rs/facet/pull/2657).
- Figue `1ebad28e18d3e778c4e82cb255b26f0e23d4fed7`: official main
  `47801613b720a7d5a05a9c8222d90104331823e2` plus
  [documentation #119](https://github.com/bearcove/figue/pull/119),
  [no-Debug test helpers #121](https://github.com/bearcove/figue/pull/121), and
  [inherited help #122](https://github.com/bearcove/figue/pull/122).

Compatible baseline versions are Facet-family `0.50.0-rc.7` and Figue
`5.0.0-rc.6`. Enable `facet-json/net` explicitly for unproxied IP addresses
in outage artifacts; core Facet's `net` does not enable Format serialization.

## Root patches and consumers

All interacting Facet traits, shapes and values must share one compatible
core identity. Patch the relevant core family, not just `facet`, so Figue and
`facet-pretty` cannot pull a second implementation. Inspect actual resolution;
an unused optional-family override may produce a harmless Cargo warning.

Cargo ignores dependency-level patches. Rust consumers of this development
version must repeat necessary root overrides in their own workspace. See
the [Cargo patch reference](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section).
Test a clean external consumer without inherited overrides before claiming
patch-free publication. Remove patches only after required APIs and behavior
exist in the selected official releases.

Cloud Terrastodon currently uses Facet-free `teamy-cancellation 0.2.0`.
The cancellation repository's newer optional Facet/Figue features must be
tested in its own root. `teamy-mft` is no longer a dependency and does not
require synchronization.

## Update procedure

1. Record old SHAs and dirty state; preserve existing branches and worktrees.
2. Fetch actual official upstreams and select immutable commits. Recheck
   package topology, versions and whether our PRs have already merged.
3. Use new integration branches containing only selected upstream PRs.
   Keep proposals independently reviewable. Do not blindly copy old in-Facet
   Figue, tuple or Format adapters without consumer-level evidence.
4. Test the fork stacks before publishing their new refs. Update compatible
   registry requirements and root patch revisions together, then inspect the
   lockfile diff; avoid unrelated wholesale dependency updates.
5. Run application tests, source-identity checks and freshly built CLI help.
   Synthetic probes supplement this evidence; they do not prove a current
   application command requires every proposed library capability.

The integration is tested with installed Rust 1.96.0 on Windows; this alone
does not establish a new minimum supported Rust version.

```powershell
cargo +1.96.0 check --workspace --all-targets --locked
$env:CLOUD_TERRASTODON_REAUTH = 'DENY'
cargo +1.96.0 test --workspace --locked --no-run
cargo +1.96.0 tree --locked --invert facet
cargo +1.96.0 tree --locked --invert facet-core
cargo +1.96.0 tree --locked --invert figue
cargo +1.96.0 tree --locked --duplicates
cargo +1.96.0 run --quiet --locked --bin cloud_terrastodon -- az tenant login --help
git diff --check
```

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

At the 2026-09-08 checkpoint, this selection has 729 passing tests and eight
failing entrypoint tests (12 ignored, seven filtered). The failures concern
fixed Arbitrary data exhaustion, a missing tracked test tenant, an assertion
checking only an error's outer context, and absent application registry
producers. Source review found these conditions already in the application;
the full old-dependency test baseline was not rerun. Preserve the failures
for follow-up rather than treating them as evidence for more fork patches.
All 179 Ratatui tests, the registry tests, new help/parse/JSON regressions,
workspace compilation and fresh root/intermediate/leaf help checks pass.

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
