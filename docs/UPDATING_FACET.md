# Updating Facet and Figue

Cloud Terrastodon uses TeamDman's independent Facet and Figue forks, with
canonical Rust dependency aliases such as `facet` and `figue`. These packages
are separate from the official upstream crates. The [workspace manifest](../Cargo.toml)
defines dependency aliases, versions, features, and any development overrides;
[`Cargo.lock`](../Cargo.lock) records the resolved packages and sources. Workspace
examples inherit those dependencies and the shared lockfile.

Local overrides can exercise fork changes during development. Before publication,
verify that normalized packages and their consumers resolve against the published
registry dependencies. Both graphs must keep interacting Facet traits and shapes
on one compatible core package identity. Run checks with `--offline --locked`
after resolving the graph being tested.

## Borrowed path proxy release (2026-11-17)

Cloud Terrastodon adopted the registry family below without root Cargo patches.
The release preserves one identity per participating Core/reflection package;
unchanged derive packages do not need a version bump merely to match the facade.
The temporary sibling overrides used to develop and test this fix are removed.
Cargo's downloaded source cache was not edited.

| Package group | Release version |
| --- | --- |
| Core, Path, Reflect, Solver, Pretty | `0.50.0-rc.8` |
| Facet facade, Error, Default | `0.50.0-rc.10` |
| Dessert, DOM | `0.50.0-rc.8` |
| Format, Value | `0.50.0-rc.11` |
| JSON, CSV | `0.50.0-rc.12` |
| XML, XML Node | `0.50.0-rc.9` |
| Figue | `6.0.0-rc.8` |
| Figue Attrs | `6.0.0-rc.6` |
| Cancellation | `0.8.3` |
| Tracing | `0.2.3` |

The later tracing capture release, `teamy-tracing 0.3.0`, uses the same Facet pins.

Macros/Macros Impl retain `0.50.0-rc.9`; macro parsing/types and test helpers retain
`0.50.0-rc.7`. CT does not use every format above, but the matching XML and template
consumers are included so they can adopt the changed reflection API coherently.

`PathBuf` now declares a type-level `Cow<str>` proxy in Facet Core. Serialization
borrows UTF-8 path text instead of allocating a separate string; deserialization
constructs an owned path. The generic proxy-holder allocation remains. Invalid
UTF-8 paths fail with field context rather than being replaced lossily. The
shared serializer has no path-specific dispatch branches. Map keys still use
the serializer's existing owned key representation, which copies their text;
the proxy change removes the additional path-to-string copy.

Borrowed proxies also require safe reflection access. `OwnedPeek::with_peek`
replaces `as_peek` with a higher-ranked callback whose result cannot export the
proxy's borrows. Construction rejects invariant and contravariant proxy shapes
before conversion, since their lifetimes cannot safely shrink. CT's three proxy
inspection sites now return owned text from this callback; the Format/DOM and
reflection consumers are updated in their forks.

For another Core/Format development iteration, use the local-development recipe
below and add this entry when the shared serializer also needs local changes:

```toml
teamy-facet-format = { path = "../facet-format/facet-format" }
```

Patching Format alone is insufficient: its sibling Core dependencies and CT's
registry dependencies would have distinct trait/shape identities. The resolved
graph must contain one identity per participating package. JSON, Value, Figue,
and Cancellation can remain registry packages while resolving their Core/Format
dependencies through the root patches. The published graph needs no overrides.

The 20 normalized candidate archives were assembled into an isolated local
registry and built together by a synthetic acceptance consumer. Each final
archive is checked for registry dependencies, licensing, clean Git provenance,
and matching tested Rust sources before Cargo builds it again for publication.
Do not reuse prepublication lockfile checksums after rebuilding committed
archives: VCS metadata changes their archive hashes.

Release source checkpoints:

- Core: [`e575c42ea`](https://github.com/TeamDman/facet/commit/e575c42eaffe6ef4057d884b6e3258e4cd3f2f71).
- Format: [`af0858f`](https://github.com/TeamDman/facet-format/commit/af0858f39c51324f5b3df04763529586756973a5).
- Figue: [`69fe8b6`](https://github.com/TeamDman/figue/commit/69fe8b6815c19dafdf3569d77b813319e1c9a99b).
- Cancellation: [`7a06612`](https://github.com/TeamDman/teamy-cancellation/commit/7a06612270f6ea3b8b4a4b87525f80ffcf2b2ae7).
- Tracing: [`2726e9c`](https://github.com/TeamDman/teamy-tracing/commit/2726e9c3b64547ad3b4da36f4ecd77c2ed10b5c2).
- CLI template: [`51c2c26`](https://github.com/TeamDman/teamy-rust-cli/commit/51c2c26).

Validation uses synthetic values and temporary directories: four path-proxy
regressions, 33 existing JSON proxy tests, nine reflection serialization tests,
and compiler failures for both dropping a borrowed proxy owner and exporting a
borrowed Cow as `'static`. CT's four config tests and its terminal/JSON error
report regression pass. Strict all-target Clippy passes for the affected CT
application, config, registry, egui, and Ratatui crates; the fork gates also
cover the shared serializer and migrated DOM consumers. CT's workspace/all-target
check and formatting checks also pass on the published graph. Metadata confirms one
identity per participating Facet package. No live service requests or real
settings were used.

Cancellation passes its 90 all-feature tests with `--test-threads=1` and strict
all-target/all-feature Clippy. One parallel run of its existing log-capture test
missed the late-cleanup acknowledgment; that test passes alone and serially.
Other tests can register the shared tracing callsite without a subscriber while
the capturing test is running. This qualification does not change cancellation
execution code. Tracing passes its full Windows gate, including examples,
rustdoc, comparison tests, both strict Clippy checks, and its Serde-free normal
and build graph.

The template passes its Rust 1.96 all-target/all-feature check, strict Clippy,
and 29 selected offline tests. Its normal/build graph remains free of Serde.
CT and template lockfiles contain only 14 and 15 targeted family version
updates, respectively, with no unrelated package-version changes. Their
checksums match the audited clean release archives, rather than the temporary
prepublication candidates.

The CT error-formatting fix is independent of Facet's path support.
`DiagnosticReport` retains an `Arc<eyre::Report>` in typed log fields until
presentation. Pass `error = report.as_error()` to tracing so the capture layer
can retain it; `?error` still records ordinary debug text. The app retains the
normal color-eyre theme. Terminal rendering honors the stderr color policy and
allows cosmetic styling while escaping terminal commands; JSON and GUI text
receive the complete plain report, including sections and traces. Regressions
check report identity after capture, no formatting during buffering or cloning,
source and span details, and a single valid JSON event. Replay formats outside
the buffer lock so error formatting can emit further logs. Config decoding
retains its source error, and GUI shutdown returns primary failures once while
logging otherwise-lost secondary failures.

## Coordinated performance release (2026-11-16)

The coordinated release used the following published registry releases,
without root development patches. The Facet enum-metadata fix is published in
the five rc.9 core packages from
[`9b8f11e12a0a4a0c377b3d394c6e4187248e416c`](https://github.com/TeamDman/facet/commit/9b8f11e12a0a4a0c377b3d394c6e4187248e416c).
All 11 packages are published, with annotated release tags and pushed source
commits. Each normalized archive was inspected for registry dependencies, license
files and VCS provenance, then built by Cargo before publication.

| Package group | Release version |
| --- | --- |
| Facet facade, Macros, Macros Impl, Error, Default | `0.50.0-rc.9` |
| JSON, CSV | `0.50.0-rc.11` |
| Figue | `6.0.0-rc.7` |
| Figue Attrs | `6.0.0-rc.5` |
| Cancellation | `0.8.2` |
| Tracing compatibility release | `0.2.2` |

Unchanged Core, Reflect, Path, Pretty, Solver, macro parsing/types, test helpers,
and Dessert retain `0.50.0-rc.7`; Format and Value retain `0.50.0-rc.10`.
The mixed package versions preserve one identity per participating package;
unchanged packages do not need a new release merely to share a version number.
JSON, CSV, Figue, and the other facade consumers receive new exact pins so an
older Facet derive does not remain in the same consumer graph.

Cloud Terrastodon and the template both select Cancellation `0.8.2`, which
contains the shared lifecycle implementation. Cloud Terrastodon migrates to the
`0.8` error-helper API instead of publishing a `0.7.3` backport. The coordinated
release contains 11 packages. The enum and lifecycle changes preserve their
behavioral contracts; the measured benefits and qualifications are recorded in
[the performance investigation](build-performance.md).

Both consumer graphs resolve one registry identity per participating package,
without patches or upstream Facet/Figue packages. CT's lockfile changes eight
targeted package versions; the template changes ten. Other package versions and
sources remain unchanged.

CT's workspace/all-target check, strict all-target Clippy for the app, command,
picker and group-management packages, and 122 selected tests pass: 21 output tests,
four Cow/reflected-request tests, 66 picker tests, three plan-persistence tests,
and 28 concurrent execution tests. These exercise original I/O error retention,
nested cancellation, required completion, and the migrated helper API with
synthetic values and temporary files. No live service verification was run.

The selected debug test build initially produced a 6,175,788,904-byte Azure
archive that LLD rejected with a malformed long-name-table error. Rebuilding
with `--config 'profile.test.package.cloud_terrastodon_azure.debug=0'` allowed
the tests to link and pass. That was a qualification of this initial test run,
not a required release flag. The subsequent
[Windows debug archive fix](build-performance.md#windows-debug-archive-linking-2026-11-16)
sets limited debug information for the Azure and entrypoint crates in the normal
development profile, inherited by tests, replacing this command-line workaround.
Debug assertions, optimization, production code and the release profile are
unchanged.

The template passes its Rust 1.96 all-target/all-feature check, strict Clippy
and 29 offline tests. Its normal/build graph remains free of Serde. The released
cancellation package passes all 90 feature-enabled tests, strict Clippy and
example compilation; Tracing passes its Windows gate, comparison tests,
documentation, examples and runtime graph inspection.

CT's fresh registry-only release EXE is 162,647,552 bytes (155.11 MiB). PE
inspection confirms one complete ResourceType metadata set. The newly built CLI
passes help/version checks, with top-level help matching the control except its
build timestamp. The installed executable was not replaced. See the
[performance receipt](build-performance.md#published-dependency-verification-2026-11-16).

Release source checkpoints:

- Core: [`9b8f11e12`](https://github.com/TeamDman/facet/commit/9b8f11e12a0a4a0c377b3d394c6e4187248e416c).
- JSON/CSV: [`d10601e`](https://github.com/TeamDman/facet-format/commit/d10601e1734b41b0f731ab4e05fe35d26868d495).
- Figue: [`892bf12`](https://github.com/TeamDman/figue/commit/892bf121cfabe289ff30d37aa58b64f587ff04e5).
- Cancellation: [`838beb8`](https://github.com/TeamDman/teamy-cancellation/commit/838beb82d4a95f4272e2a1dc72b6d4396bbf95f5).
- Tracing: [`7f15dfe`](https://github.com/TeamDman/teamy-tracing/commit/7f15dfe7da2a84cd1b6975b5b36cbb6ad0b1c26a).
- CLI template: [`212ed2d`](https://github.com/TeamDman/teamy-rust-cli/commit/212ed2d8377ef551685e6f0b2adb53a8ad3bbcd6).

## Local Facet development

Cloud Terrastodon's manifest uses registry dependencies. For another development
iteration, enable the recipe below, extending it with the Format entry above
when needed, to make ordinary
`cargo build`, `cargo test`, and `update.ps1` consume the sibling Facet checkout
directly, without publishing each experimental change.

Cancellation can be patched independently when its implementation also needs
iteration. Select a checkout whose package version and API match the consumer:
Cloud Terrastodon and the template both use the `0.8` line. During the
performance comparison, `.codex-cancellation-0.7.2` held the maintenance source
worktree; it was source, not generated diagnostic data. No cancellation patch
is needed for the coordinated registry release.

### Switch between local and published sources

1. **Enable local development:** keep the Facet checkout at `../facet` and add
   the block below to the workspace root `Cargo.toml`. Merge with any existing
   `[patch.crates-io]` section. Local package versions must satisfy the workspace
   and transitive dependency constraints; preserve dependency aliases and features.
2. **Return to published dependencies:** publish the fixes when ready, update
   exact workspace pins as necessary, and remove these patch entries. Root
   patches are not inherited by applications consuming published CT crates.
3. **After either switch:** resolve the lockfile once, then inspect the graph:

   ```powershell
   cargo metadata --offline --format-version 1 > $null
   git diff -- Cargo.toml Cargo.lock
   cargo tree --offline --locked -i teamy-facet-core --depth 1
   ```

   The first command intentionally permits lockfile updates. If newly published
   packages are not cached, omit `--offline` for that resolution. Subsequent
   builds should use `--locked`. Do not run a broad `cargo update` or `cargo clean`
   merely to switch sources.

Keep this block in the guide even when the manifest returns to registry sources:

```toml
[patch.crates-io]
teamy-facet = { path = "../facet/facet" }
teamy-facet-core = { path = "../facet/facet-core" }
teamy-facet-error = { path = "../facet/facet-error" }
teamy-facet-macro-parse = { path = "../facet/facet-macro-parse" }
teamy-facet-macro-types = { path = "../facet/facet-macro-types" }
teamy-facet-macros = { path = "../facet/facet-macros" }
teamy-facet-macros-impl = { path = "../facet/facet-macros-impl" }
teamy-facet-path = { path = "../facet/facet-path" }
teamy-facet-pretty = { path = "../facet/facet-pretty" }
teamy-facet-reflect = { path = "../facet/facet-reflect" }
teamy-facet-solver = { path = "../facet/facet-solver" }
```

Patch the participating core family together: a facade-only path dependency can
leave transitive registry clients using a different Core/reflection identity.
Without an active Format change, Format, JSON, Value, Figue, and Cancellation
remain registry packages but resolve their core dependencies through these
patches. Extend the block when another participating package needs local changes,
as Format did during the path-persistence iteration.

With unchanged versions, enabling these 11 patches removes only their registry
`source` and `checksum` lockfile lines; disabling restores them. Investigate other
churn. `cargo metadata --offline --locked --format-version 1` gives the full check:
each participating package must have one identity, with its manifest in the
sibling checkout for local development, or a registry source after removal.
Make fixes in the fork checkout; never edit Cargo's downloaded registry sources.

### Verify without repeated rebuilds

A routine source switch needs graph inspection and a check of the affected
consumer. Broader verification is warranted when fork behavior or APIs change
and before releasing against the published graph. Changing sources can trigger
a rebuild; changing package selections or features between Cargo commands can
also rebuild shared dependencies. Avoid competing Cargo jobs in the same target
directory and keep the selected package/feature set consistent.

For the existing consumer acceptance checks, prepare all three test binaries in
one build, then run those binaries with their respective filters `output::tests::`,
`cow_`, and `reflected_project_`:

```powershell
cargo test --offline --locked --no-run --lib -p cloud_terrastodon_app -p cloud_terrastodon_registry_execution -p cloud_terrastodon_azure_devops
```

Cargo prints the test executable paths; invoke each with its filter. This keeps
the dependency feature graph fixed instead of rebuilding it for three separate
package selections. Inspect selected tests before execution: Cargo's `--offline`
prevents downloads, not service access from test code.

The 2026-11-16 local-source cutover passed the workspace/all-target check and all 25 selected
tests (21 output, two Cow, two reflected request-clone), using synthetic values.
Metadata confirmed one local identity for each of the 11 packages, without
version or unrelated source changes.

## Nongeneric enum metadata fix (2026-11-16)

The Facet rc.9 release contains the tested derive change that gives nongeneric
enums one canonical static shape. It removes repeated promoted metadata in
downstream consumers while preserving generic enum behavior and full reflection.
The [build-performance report](build-performance.md) records the controlled
Cloud Terrastodon binary and build-time comparison. Those measurements used
controlled local sources before publication; the coordinated release above
carries the fix into the registry graph. The older versions below are historical
checkpoints rather than current consumer pins.

## Cow request refinement (2026-11-06)

This section records the earlier Cow release and its validation results. Its
versions and source checkpoints are historical; the coordinated performance
release above defines the published consumer graph.

The Cow fixes were published and verified against the complete registry graph
before the local build-performance overrides described above. The release also
includes format packages not used by Cloud Terrastodon, as listed below.
`teamy-main` remains the integration branch
for the three forks; applications select exact published versions rather than
a moving branch.

| Package group | Published version |
| --- | --- |
| Facet facade, Macros, Macros Impl, Error, Default | `0.50.0-rc.8` |
| Format, JSON, Value, CSV | `0.50.0-rc.10` |
| DOM (first Teamy release) | `0.50.0-rc.7` |
| XML, XML Node | `0.50.0-rc.8` |
| Figue | `6.0.0-rc.6` |
| Figue Attrs | `6.0.0-rc.4` |
| Cancellation maintenance releases | `0.7.2` and `0.8.1` |
| Tracing compatibility release | `0.2.1` |

These are 14 core/Format/Figue packages, two Cancellation releases, and one
Tracing release. Unchanged
Core, Reflect, Path, Pretty, Solver, macro parsing/types, test helpers, and Dessert
remain at `0.50.0-rc.7`. Distinct packages need not share one version number;
interacting code must share one source/version identity for each package.
Cloud Terrastodon used Cancellation `0.7.2`, retaining its existing error-helper
API. The template's `0.8` API line had the separate `0.8.1` maintenance release.

Cow release source checkpoints:

- Core: [`2d6d06e1d9a0ab47cd68547f5a0a4dbb60d36afa`](https://github.com/TeamDman/facet/commit/2d6d06e1d9a0ab47cd68547f5a0a4dbb60d36afa).
- Format: [`55dd746345382b81a8e9eb1652cddcd4b7cb2fda`](https://github.com/TeamDman/facet-format/commit/55dd746345382b81a8e9eb1652cddcd4b7cb2fda).
- Figue: [`85b3886592767bfe66cdf4588ee12d015efaf569`](https://github.com/TeamDman/figue/commit/85b3886592767bfe66cdf4588ee12d015efaf569).

The project-list request exposed two separate defects:

- Format's generic `Cow<T>` decoder selected an inner field from an opaque pointer
  shape. It now uses smart-pointer initialization so the pointee's validating
  proxy runs and the decoded Cow owns its value. Existing borrowed string and
  byte fast paths are retained.
- Derive's field-proxy conversion functions could not refer to enclosing lifetime,
  type, or const parameters. Noncapturing closures now preserve those parameters while
  coercing to the existing erased function-pointer contract. This applies to
  ordinary and format-specific proxies, including opaque and enum fields.

Both responsibilities are fixed in their owning forks. The project-list and
project-picker requests retain ordinary `Cow` organization fields; no application
wire/proxy workaround remains. Borrowed, owned, and deserialized inputs preserve
request identity and shared aggregate reuse. The pulled Format changes also
include bounded namespace-aware XML parsing and its read model; these packages
are published without adding XML dependencies to Cloud Terrastodon.

The prepublication source suites passed: 214 core Facet, 222 JSON, 761 Figue,
312 XML/CSV, and 82/88 Cancellation tests for its two maintenance lines.
The changed implementation crates passed Clippy. Prepublication consumer checks
passed for the Cloud Terrastodon
workspace/all targets, both reflected request-clone scenarios, and 14 DevOps
inventory/discovery tests; the template passed its all-target check and 13
selected output/CLI tests. These checks used synthetic/offline inputs.

At the Cow release checkpoint, ordinary registry compilation passed for Cloud Terrastodon's
`cargo check --offline --locked --workspace --all-targets` and the template's
`cargo check --offline --locked --all-targets`. The template retains only its two
existing macro-semicolon warnings. Their lockfiles update ten and eleven targeted
package versions respectively, with no unrelated changes. Cargo metadata confirms
one registry identity per participating Facet/Figue package in both graphs,
without upstream Facet/Figue packages or source overrides. All 16 selected
Cloud Terrastodon tests pass against this registry graph: both list/picker
reflected-clone scenarios and the 14 inventory/discovery tests. Ordinary-registry
verification is complete for both consumers.

**Template dependency release:** `teamy-rust-cli` also uses `teamy-tracing`,
whose Facet dependencies needed alignment with the new graph.
`teamy-tracing 0.2.1` was published; its release source was committed and
pushed as [`d099c486804ae7fe18d2acf5d2e74a3e4c48ff10`](https://github.com/TeamDman/teamy-tracing/commit/d099c486804ae7fe18d2acf5d2e74a3e4c48ff10)
on `release/0.2.1-facet`. Its 37 tests, two comparison checks, Clippy,
documentation, and examples passed. Template registry pin/lock updates,
compilation, and all 13 selected tests (11 library and two `cli_fuzzing`) are
complete. The template is committed and pushed to `origin/main` as
[`58d94181a3fc11d99c8691254dbffe8258dcc311`](https://github.com/TeamDman/teamy-rust-cli/commit/58d94181a3fc11d99c8691254dbffe8258dcc311),
with a clean working tree and matching local/remote refs.

Subsequent author-metadata corrections changed the Format and Tracing commit
IDs without changing their source trees. The explicitly approved Format history
correction preserves source trees, messages, dates, topology, and unrelated
identities across all 15 reconstructed commit objects. Its `teamy-main` and
`feature/domains-xml-safety` refs are pushed at `55dd746345382b81a8e9eb1652cddcd4b7cb2fda`
and `fd2af32fb2eb087d73237302c882742a9d313fc5`, respectively. The corrected
annotated CSV rc.7 tag object is `6d814e24863f65463c6c94811e5a110e7aab14e1`,
targeting `da65a4fe8ddcc391063c5b726c612fad43351247`; its message and date are
preserved. Remote refs were verified and the working tree is clean. Published
package archives and consumer checksums remain unchanged.

Cancellation's maintenance branch is merged into
`main` and pushed at
[`e4d962738dce985632bdff37a196d2c3e14500ac`](https://github.com/TeamDman/teamy-cancellation/commit/e4d962738dce985632bdff37a196d2c3e14500ac),
retaining the existing `0.8.1` source tree. Tracing's new `main` starts from its
existing implementation history and merges the release branch while preserving
telemetry support. It is committed and pushed at
[`47bc18c5738d62cdb471afee66e682e6c1ea851a`](https://github.com/TeamDman/teamy-tracing/commit/47bc18c5738d62cdb471afee66e682e6c1ea851a),
with a clean working tree and matching local/remote refs. The all-target,
all-feature check, 43 tests (37 base, including the doctest, plus six telemetry
tests), and default-feature all-target Clippy with warnings denied passed.
The separately approved GitHub default-branch change is complete: Tracing's
default branch and local `origin/HEAD` both point to `main`.

The initial local review began from core
`74b1070dc1c889f429ba79ff458a38dae46209a5` and Format
`0d490e34f13552a71a1493debb7ddba73ab787aa` (the identical baseline tree after
the author-metadata correction). All three new Cow decoder scenarios
failed before the fix. A temporary [`[patch.crates-io]` configuration](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section)
and separate lockfiles validated the unpublished source changes without changing
the original registry lockfiles. That historical check passed 214 core and 222
JSON tests, 14 DevOps inventory/discovery tests, two reflection/UI tests, and
both application all-target checks. The application compiler was Rust 1.99.1;
the fork suites used their pinned Rust 1.96.0 toolchain. The initial alternate
Cloud Terrastodon lock changed only 15 Facet sources, with no version or unrelated
dependency changes. Those temporary overrides are development evidence, not a
requirement of the newly published dependency graph.

## Historical release preparation (0.37.0)

The coordinated release contained 43 packages:

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
| Teamy registry packages | Independent fork package identities; keep canonical Rust aliases and compatible dependency constraints |
| `TeamDman/facet` | TeamDman-maintained core Facet fork |
| `TeamDman/facet-format` | TeamDman-maintained serialization fork |
| `TeamDman/figue` | TeamDman-maintained CLI parsing fork |

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

At the historical Git-pin checkpoint, the workspace check passes for all
targets. The audited application selection passes 20 app tests, eight entrypoint
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

Keep the canonical dependency keys required by generated macro paths. Workspace
members inherit the declarations from the root manifest:

```toml
facet.workspace = true
figue.workspace = true
```

See [Cargo's dependency renaming reference](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml).
Within the fork source repositories, internal normal/build dependencies have
both a version and a development path. Fork internal dev-dependencies use paths
only so Cargo omits them from archives, avoiding bootstrap cycles; run their
complete test suites in the source trees. Released Cloud Terrastodon packages
consume Teamy packages from the registry without path or patch overrides.
Archive validation checks library builds and an independent consumer using
the normalized registry graph. Do not claim registry installation works until
the exact versions have actually been published.

Teamy Cancellation's `CancellableWriter` and write extension
methods check the supplied execution scope between bounded writes, preserving
deferral and cancellation reasons. CLI JSON and Pretty output share this writer
policy; computation that does not write continues until its next checkpoint.

The API separates raw `CancellationToken` signals from linked
`CancellationScope` execution ownership. Use `cancellable_scope` for automatic
poll-boundary checks, `with_async_scope` for handler-owned cleanup, and bounded
`deferred` or `with_deferral` callbacks for protected work.

When enabling Cancellation's optional Facet or Figue integrations, check that their
dependency constraints match the application's graph. Test those optional features
in the cancellation crate's own root when changing dependencies. Publish required
dependency versions before releasing their consumers, and verify the registry
graph without development overrides.

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
   `teamy-main` SHAs without waiting for upstream PR merges. Publish the reviewed
   compatible Teamy versions, then update the workspace's exact registry pins.
   The standalone CLI inherits the workspace's Teamy dependencies. Inspect each
   lockfile diff; avoid unrelated dependency updates. Verify normalized archives and a
   consumer without Git overrides.
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
cargo +1.96.0 run --quiet --locked --bin cloud_terrastodon -- account login --help
git diff --check
```

When preparing a new coordinated fork release from the fork source checkouts,
preview the publication order, including first-time crates:

```powershell
./publish.ps1 -DryRun -ManifestPath ../facet/Cargo.toml,../facet-format/Cargo.toml,../figue/Cargo.toml,./Cargo.toml
```

The script skips only an already published exact version, fails on registry
errors other than a missing version, and orders packages by production/build
dependencies. `-DryRun` reads metadata without publishing. Actual publication
is a separate step after the prepared changes and validation are reviewed.

Do not use an installed, potentially stale `ct` executable as proof.
Global output and logging options remain shared; authentication source and
account selectors belong to authenticated leaf commands. Check the relevant
leaf's help, such as `ct az resource-group list --help`, when validating that
surface. Outage artifact tests check actual IPv4/IPv6 JSON without network
access. Registry/Object Explorer tests retain Cow borrow/promote and runtime
lease behavior. Record environment limitations separately from compatibility
failures.

Ordinary Cargo tests use offline fixtures and isolated stores. Live service
verification is an explicit `ct test run` operation, described in
[runtime verification](runtime-verification.md); do not invoke it for dependency
acceptance without authorization to access the selected data. Cargo's `--offline`
controls dependency access, not network access from test code, so review new tests
for that boundary rather than relying on the flag or
`CLOUD_TERRASTODON_REAUTH=DENY` to disable credentials.

Current workspace acceptance can use:

```powershell
cargo +1.96.0 test --workspace --all-features --offline --locked
```

The following narrower selection records the historical 2026-09-08 checkpoint,
before live verification moved out of Cargo tests. Its exclusions are not the
current acceptance procedure:

```powershell
cargo +1.96.0 test --offline --locked --no-fail-fast --lib `
  -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution -p cloud_terrastodon_azure_types `
  -p cloud_terrastodon_azure_devops_types -p cloud_terrastodon_azure_resource_types `
  -p cloud_terrastodon_rest -p cloud_terrastodon_ui_ratatui `
  -p cloud_terrastodon_entrypoint -p cloud_terrastodon_auth `
  -p cloud_terrastodon_gitea -p cloud_terrastodon_command `
  -p cloud_terrastodon_tracing -p cloud_terrastodon_user_input -- `
  --skip azure_devops_rest_command_cli::test::it_works `
  --skip project_list_authenticates_before_reading_organization_or_cache `
  --skip browser_project_list_requires_a_tenant_without_a_stored_session `
  --skip azure_devops_rest_client::test::it_works `
  --skip jwt::test::it_works --skip azure_devops_pat::test::non_empty `
  --skip it_fetches_repositories_without_empty_ids
```

At that checkpoint, this selection excluded live service/credential tests and
two tests whose missing host-configuration assumptions required separate
isolation; `--lib` also excluded command integration tests that ran Azure CLI.
It was a dependency acceptance check, not full release validation.

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
optimized release build. The checkpoint also ran the isolated license-response
generator tests:

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
