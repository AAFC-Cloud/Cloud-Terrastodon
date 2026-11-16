# Build time and binary size investigation

Measurements on 2026-11-16 started from `f7f04c2b423024ac437009c36ddaa7580aa3f79b`,
using Rust/Cargo 1.99.1 on Windows x64 MSVC, the normal release profile,
default features, and the existing dependency lockfile. No production requests,
credentials, or response data were used. No compiler-profile or dependency-version
changes are needed for the picker refactor.

The workspace manifest consumes the coordinated published
Facet/cancellation releases without development overrides. See
[the release record](UPDATING_FACET.md#coordinated-performance-release-2026-11-16)
for package versions and qualification status, and
[local Facet development](UPDATING_FACET.md#local-facet-development) to enable
local sources for another iteration. The measurements below describe controlled
prepublication comparisons; their staged package copies remain local evidence.

## Controlled results

The implementation is in the Facet fork checkout, in
`facet-macros-impl/src/process_enum.rs`, with its regression in
`facet/tests/enum_metadata.rs`. Both are included in the Facet rc.9 release at
[`9b8f11e12a0a4a0c377b3d394c6e4187248e416c`](https://github.com/TeamDman/facet/commit/9b8f11e12a0a4a0c377b3d394c6e4187248e416c).
Cargo's registry source cache was not edited.

For measurement, the published macro implementation package was copied into
`target/perf-investigation/facet-macro-patch`, then its implementation file was
replaced with the identical modified file from the fork. Using this separate
copy preserves the published package's normalized manifest while testing the
fork change. At measurement time, the fork file and benchmark copy had matching
SHA-256 hashes; the registry source retained its original control hash.

The controlled Facet experiment used a development override of **only**
`teamy-facet-macros-impl`. Its unmodified control and patched version used the same
local package path and normalized published manifest. Runtime Core, macro parsing,
and macro types retained their published registry identities; there was one Facet
core package in the resolved graph. The temporary lockfile change removed only
the overridden macro implementation's registry source/checksum and was restored
after each build.

| Implementation | EXE bytes | EXE MiB | Simulated UI edit rebuild |
| --- | ---: | ---: | ---: |
| Unmodified macro control | 222,547,968 | 212.24 | 127.24 s |
| Canonical nongeneric enum metadata | 180,374,016 | 172.02 | 111.82 s |
| Canonical metadata and shared picker UI | 169,362,432 | 161.52 | 91.85 s |

Each edit measurement changes only the modification time of
`crates/ui_egui/src/draw_app.rs`, then runs the normal release build. Source
contents and the original timestamp are restored afterward. Dependencies are
already built, and no other Cargo jobs run during these measurements. These are
single observations, not repeated benchmark distributions or a promise for all
edits. The combined observed change is 50.72 MiB less executable storage and
35.39 seconds less rebuild time (27.8%).

The registry build at that checkpoint also succeeded with the picker refactor
and no Facet override: its EXE is 216,476,672 bytes (206.45 MiB), compared with the
earlier published-graph baseline of 222,530,560 bytes (212.22 MiB). That is the
picker improvement available without the Facet fix. Its UI-edit rebuild takes
92.83 seconds versus the earlier registry baseline's 115.36 seconds; this is also
a single observation. The old derive's duplication is sensitive to consumer
code layout: this build retains
38 resource-type tables instead of the baseline's 36. Therefore the 10.50 MiB
picker saving measured with canonical metadata must not be assumed to be its
standalone saving with the old derive.

Changing a proc macro or the picker library invalidates many consumers. Those
setup builds are deliberately excluded from the edit-time comparison. An earlier
unchanged registry build took 1.35 seconds; a registry UI-edit baseline took
115.36 seconds, illustrating timing variation and the importance of comparing
the same dependency graph. The user's complete `update.ps1` workflow is not
identical to this simulated edit.

## Facet metadata

The generated `ResourceType` enum has 5,468 reflected variants. The unmodified
control contains 36 physical variant arrays of 612,416 bytes each. Their names
also occupy separate physical storage. Tables and names together consume
33,894,612 bytes (32.32 MiB).

The fork change gives nongeneric enums an addressable static `Shape` in their
defining crate instead of letting the entire promoted constant graph be copied
into consumers. Generic and lifetime-bearing enums retain their existing path.
It preserves the full enum, variants, fields, proxies, and wire representation.

The patched executable contains **one** complete `ResourceType` table and name
set: 941,517 bytes. The scanner reads PE sections and pointers, then checks every
ordinal name against the generated source; it does not infer duplication from
text matches or rely on PDB symbol names. Its intentional layout assumption is
the current x64 Facet `Variant` record size of 112 bytes. The whole-EXE delta also
includes other enum metadata, generated code, and relocation tables; it must not
be attributed entirely to the resource-type arrays and names.

The source change and regression test live in the Facet fork's
`facet-macros-impl/src/process_enum.rs` and `facet/tests/enum_metadata.rs`.
They are published with the rc.9 derive. Cloud Terrastodon's coordinated
dependency update removes the temporary development overrides. These physical
measurements describe the captured comparison executables, not a newly measured
registry-release executable.

## Picker code sharing

The baseline entrypoint's pre-optimization LLVM IR contains 580,801 lines in
`PickerTui` implementations, including 66 instances of its large event-loop
closure. This identifies repeated compilation; IR line counts are not final
linked bytes or a direct measure of build duration.

The refactor keeps the public `PickerTui<'a, T>`, typed choices, sinks, events,
and results. A private `PickerCandidateStore<T>` owns values and handlers behind
`PickerCandidates`; the nongeneric `PickerUi` owns the terminal lifecycle and
event loop. It works with display keys rather than erasing payload values.
There are no `Any` downcasts, additional per-candidate allocations, `Clone`
requirements, or `'static` bounds. Candidate values and handler captures now
remain alive through terminal cleanup instead of dropping immediately at the
end of the event loop.

Holding the Facet patch constant, the picker change removes another 11,011,584
EXE bytes (10.50 MiB). This is a linked binary measurement rather than an
estimate from IR. The `.text` section falls from 96.03 MiB to 88.51 MiB in that
comparison, with additional reductions in read-only data and other sections.

## Cancellation lifecycle sharing (2026-11-16)

The next experiment extracts the common admission, cancellation checks, wakeup
registration, and completion work from `CancellableScopeFuture<C, F>` into a
private `CancellableScopeLifecycle`. The typed wrapper still invokes the callback,
polls its concrete future, and returns its original result/error type. It retains
the same allocations, borrowed/non-Send support, and user-destructor ordering.
Substantial lifecycle methods stay out of line so consumers share their code.

The comparison used the actual `teamy-cancellation` checkout on its then-current
0.8.1 line, with the same change backported into the 0.7.2 maintenance worktree.
CT temporarily patched that maintenance source to preserve its API during the
experiment. The coordinated release publishes the fix as `0.8.2` for both CT
and the template, migrating CT to the `0.8` error-helper API without a backport
publication or consumer source overrides. The captured `0.7.2` comparison
remains historical evidence rather than a measurement of that API migration. See the
[release record](UPDATING_FACET.md#coordinated-performance-release-2026-11-16)
for publication and final registry-verification status.

The control and refactored builds hold the local dependency paths, all 851 package
identities, resolved feature sets, and normal release settings constant. Only
the cancellation implementation changes. One-time dependency rebuilds are
excluded from the warm UI-edit timings. Each timed run changes only the GUI
source timestamp and restores it afterward; no other build/test jobs run during
those measurements on the agent's behalf. The user subsequently reported
additional builds around the second refactored measurement; that observation is
retained below but excluded from conclusions about build speed.

| Implementation | UI edit, run 1 | UI edit, run 2 |
| --- | ---: | ---: |
| Current control | 98.32 s | 95.29 s |
| Shared cancellation lifecycle | 91.10 s | 126.29 s (possible concurrent builds) |

A repeat after the other builds finished took 99.44 s. These observations do
**not** establish a reliable speed improvement from lifecycle sharing alone.
Entrypoint compilation remains dominant. Runtime throughput was not benchmarked.

The control EXE is 169,326,080 bytes (161.48 MiB); the refactored EXE is
165,409,280 bytes (157.75 MiB), a reduction of 3,916,800 bytes (3.74 MiB).
The `.text` section shrinks by 2,954,752 bytes and `.rdata` by 769,536 bytes.
ResourceType still has one canonical metadata table.

PE unwind ranges and matching EXE/PDB public symbols confirm six retained
lifecycle methods with one implementation each. The 1,231 distinct retained
generic poll functions remain typed; their combined ranges shrink from
5,616,938 to 5,513,208 bytes. These bodies also include inner-future polling,
while other wrappers inline into their callers. This partial symbol attribution
does not explain the entire executable delta or measure pure cancellation cost.

Prepublication validation passed on the main library line: 33 async-only tests, 60 with
deferral/inheritance, and 90 with all features. Strict all-target/all-feature
Clippy passed on Rust 1.96.0. The maintenance line passed 84 all-feature tests;
CT passed all 28 scoped-join, concurrent-map, and execution-contract tests with
the override. These tests exercise cancellation and destructor ordering with
synthetic values. An initial main all-feature run hit an existing synchronous
tracing test flake; both its isolated rerun and the complete retry passed.

Raw evidence and analysis helpers are under
`target/perf-investigation/cancellation-sharing`, including captured EXE/PDB pairs,
source snapshots, metadata, timings, and matched physical function ranges.

## Incremental release compilation for the entrypoint

The root manifest now enables incremental compilation only for
`cloud_terrastodon_entrypoint`, with 16 codegen units. Optimization remains at
release level 3, and dependencies retain their release settings. This lets the
existing `update.ps1` command reuse entrypoint compilation after small edits.

Lowering only entrypoint optimization to level 1 was tried first: its setup
build took 86.36 s and produced a slightly larger 165,863,936-byte EXE. That
setting was not retained. Incremental compilation at level 3 proved more useful:

These observations use Windows/MSVC and Rust 1.99.1 with the same locked
dependency graph. The rows distinguish different edit workloads and cache states;
they are observations of this checkout, rather than a general build-time guarantee.

| Incremental entrypoint measurement | Time |
| --- | ---: |
| First build to populate the cache | 88.07 s |
| Warm timestamp-only GUI edit | 33.59 s |
| Actual GUI function-body edit | 12.35 s |
| Restore and rebuild the original function body | 12.92 s |

The body-edit experiment temporarily changed the Logs button's literal from
`Logs (on)` to `Logs (ON)`. The changed label appeared in the captured EXE;
the original label returned after restoration. Original source bytes and
timestamp are restored, and the restored body was rebuilt. No GUI was launched.
This tests a small implementation edit, not changes to public types or a
guarantee for every edit. The two body-edit observations follow earlier cache
warmup; the 33.59-second observation shows that cache state still matters.

The resulting EXE is 162,608,128 bytes (155.08 MiB), versus this phase's
169,326,080-byte control (161.48 MiB). The combined lifecycle/profile difference
is 6.41 MiB; do not attribute all of it to the lifecycle extraction. Binary
layout and code sharing can change with incremental partitioning.

The entrypoint cache occupies about 1.43 GiB in this checkout. Clean builds,
`cargo clean`, profile/toolchain changes, and broad dependency changes can lose
the reuse benefit; the initial build remains expensive. The override applies to
this workspace's entrypoint build rather than downstream SDK consumers.

An isolated `cargo install --path . --locked --offline --root <evidence-dir>`
with the normal manifest settings succeeded in 5.28 s after warmup. Top-level
help matches the control apart from its embedded build timestamp. The globally
installed executable was not replaced.

To compare a nonincremental release while preserving the source graph:

```powershell
cargo build --release --locked --offline --bin cloud_terrastodon --timings --config 'profile.release.package.cloud_terrastodon_entrypoint.incremental=false'
```

## Published dependency verification (2026-11-16)

The fresh registry-only release build succeeds on Rust 1.99.1 with the normal
release flags and entrypoint incremental settings. Its EXE is 162,647,552 bytes
(155.11 MiB). PE inspection still finds exactly one complete ResourceType
variant/name set, occupying 941,517 bytes, with all 5,468 ordinal names checked
against the source. The size benefit survives publication and the migration to
Cancellation 0.8.2 without any Cargo patches.

This is a verification point after changing dependency versions and the
cancellation error-helper API, rather than another controlled timing comparison.
The source switch rebuilds dependencies; no new warm-edit timing is claimed.
The freshly built top-level help, storage list/audit help and version checks
pass. Top-level help matches the control after normalizing its build timestamp.
No installed executable was replaced.

The [release record](UPDATING_FACET.md#coordinated-performance-release-2026-11-16)
documents the consumer tests, strict Clippy, registry identity checks, and the
initial test-only Azure debug-symbol workaround. The persistent development
profile fix below supersedes that command-line workaround.
Captured EXE/PDB, PE measurements, logs, graph audits and publication receipts
remain under `target/perf-investigation/coordinated-release` until Cargo cleanup.

## Windows debug archive linking (2026-11-16)

Normal `cargo run -- egui` failed during linking on Rust 1.99.1 with
`truncated or malformed archive (string table at long name offset 6177 not
terminated)`. The failure reproduced with `cargo build --offline --locked
--bin cloud_terrastodon`; it occurs before application startup.

Archive sizes before and after the targeted profile change:

| Crate | Full debug info (bytes) | Limited debug info (bytes) |
| --- | ---: | ---: |
| `cloud_terrastodon_entrypoint` | 4,453,784,302 | 305,431,348 |
| `cloud_terrastodon_azure` | 6,159,338,904 | 985,631,990 |

A bounded inspection of the original entrypoint's 260 member headers
found a complete archive ending exactly at a member boundary. Its `/SYM64/`
header identifies GNU64 format, but its filename table has 256 COFF-style NUL
terminators and no GNU-style line terminators. Offset 6177 points to an existing
filename. This explains LLD's diagnostic without assuming disk corruption.

Rust 1.99.1 uses `ar_archive_writer` 0.5.2. Its
[archive writer](https://github.com/rust-lang/ar_archive_writer/blob/v0.5.2/src/archive_writer.rs#L878-L889)
switches to GNU64 above the 32-bit member-offset limit after computing the COFF
filename table, without regenerating it. The
[LLVM reader](https://github.com/llvm/llvm-project/blob/llvmorg-23.1.1/llvm/lib/Object/Archive.cpp)
expects GNU filename terminators for that format. The mitigation here avoids
that compiler defect; it does not repair the upstream writer.

The root manifest sets `debug = 1` for only `cloud_terrastodon_azure` and
`cloud_terrastodon_entrypoint` in `profile.dev.package`. The
[test profile inherits dev](https://doc.rust-lang.org/cargo/reference/profiles.html#test),
so normal builds, runs, and tests share this setting without command-line
overrides. Limited debug information retains source-line/module information but
omits type and local-variable debugging in those two crates. All other packages
retain their existing debug settings; debug assertions, overflow checks,
optimization and release settings are unchanged.

Cargo detects the profile change and rebuilds affected packages. A full
`cargo clean` is unnecessary. Do not edit or repack generated archives or Cargo
registry sources to work around this error. Reconsider the overrides when a
toolchain with a corrected archive writer is verified, or if these archives
approach the offset limit again.

The normal debug build links successfully with this setting. Both rebuilt
archives use the regular COFF index, with no member offsets past the 32-bit
limit. Cargo's artifact records confirm debug level 1 and active debug assertions
for both selected crates; the egui crate still uses full debug level 2.
The normal test profile also links, and 31 selected entrypoint CLI tests pass
without any profile override on the command line. `cargo run --offline --locked
-- egui --help` succeeds. No GUI session or live service request was started.

Raw diagnostics and a bounded archive inspector are kept under ignored
`target/perf-investigation/debug-archive-fix` until Cargo cleanup.

## Resource-type lookup and PHF

[`phf`](https://docs.rs/phf/latest/phf/) supplies generated immutable lookup
tables and could replace the resource-type string match. It does not remove
Facet's reflected variant arrays, reverse mappings, or derived implementations.
The existing entrypoint IR does not contain the resource-type parser body, so
that IR cannot establish its lookup cost. PDB retained-object attribution gives
the resource-type crate approximately 442.5 KiB of code, subject to inlining and
linker-folding limitations.

No PHF dependency or lookup change was added. If lookup becomes the next measured
target, compare the existing match with both a generated sorted table using
binary search and PHF, preserving case handling and unknown-value behavior.
Generate data in the existing offline resource-type tool rather than adding work
to every application build. The current lockfile already has PHF 0.11 packages
transitively; selecting latest 0.14 blindly would introduce another version family.

## Validation and reproduction

- Facet: 214 existing integration tests and one metadata-sharing regression pass;
  the regression also denies warnings for generated uppercase variant names.
- Facet macro implementation: all-target Clippy passes with warnings denied.
- Picker: 65 default-feature and 67 all-feature library tests pass, including
  borrowed non-Clone values, partial errors, reload generations, large lists,
  nested terminal ownership, and required-work draining.
- Picker: all-target, all-feature Clippy passes with warnings denied.
- Full release builds pass. Top-level help, storage-account list/audit help,
  and version output match the control apart from the expected build timestamp.

The release build command is:

```powershell
cargo build --release --locked --offline --bin cloud_terrastodon --timings
```

Useful focused checks are:

```powershell
cargo test --locked --offline -p cloud_terrastodon_user_input --lib --all-features
cargo clippy --locked --offline -p cloud_terrastodon_user_input --all-targets --all-features -- -D warnings
```

Local raw evidence remains under ignored `target/perf-investigation`: copied
EXEs/PDBs, Cargo timing HTML/JSON, build/test logs, the PE scanner, and initial
LLVM IR. PDB files are separate from the measured EXEs. No installed executable
was replaced. These generated files are not required to build the project and
can be removed by `cargo clean` after review.
