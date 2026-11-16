# Build time and binary size investigation

Measurements on 2026-11-16 started from `f7f04c2b423024ac437009c36ddaa7580aa3f79b`,
using Rust/Cargo 1.99.1 on Windows x64 MSVC, the normal release profile,
default features, and the existing dependency lockfile. No production requests,
credentials, or response data were used. No compiler-profile or dependency-version
changes are needed for the picker refactor.

The workspace now uses the sibling Facet checkout directly through the root
manifest's development path overrides. Ordinary Cargo commands consume new fork
edits without publishing. See [local Facet development](UPDATING_FACET.md#local-facet-development-2026-11-16)
for setup and the eventual return to registry dependencies. The measurements
below describe the earlier controlled comparison, whose staged package copy
is retained only as local evidence.

## Controlled results

The implementation is in the Facet fork checkout, in
`facet-macros-impl/src/process_enum.rs`, with its regression in
`facet/tests/enum_metadata.rs`. Both changes are uncommitted. Cargo's registry
source cache was not edited.

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
They are not published. Cloud Terrastodon's development path overrides now read
the actual fork checkout, including this fix. Publication and verification of
the resulting registry graph remain separate release work; the development
overrides are temporary.

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
