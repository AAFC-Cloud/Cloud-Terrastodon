# Shared tracing and egui cutover

Status: implemented. Cloud Terrastodon uses the published `teamy-tracing 0.3.0` release.

## Outcome

Reusable report-aware logging moved from CT into `teamy-tracing`. CT used an
adjacent checkout through a relative `[patch.crates-io]` entry during development
and switched to crates.io after publication. CT's duplicate implementations were
removed.

The coordinated GUI stack is egui/eframe/egui_extras/emath 0.36.2 and
egui-toast 0.22.0. Explicitly retain glow instead of adopting a different renderer
through changed defaults. Bundle the original Allo variable font and its license;
select actual text weights through egui's variation support.
This set matches the published toast dependency on egui 0.36; using egui 0.37
would require a different toast strategy or an unpublished dependency. The Allo
variable font increases embedded font data by about 1 MiB while replacing both
static faces with one shared font.

## Ownership and boundaries

- `teamy-tracing`: live log records, bounded capture, retained `DiagnosticReport`,
  weak subscriber-to-buffer ownership, cursor/acknowledgement mechanics, plain
  and terminal formatting, and optional egui log and diagnostic views.
- CT: subscriber installation, independent console/file/debug filters, terminal
  coordination, exit replay policy, application capture lifetime, toast policy,
  repaint scheduling, portal layout, and service-specific errors.
- Existing telemetry/OTLP records retain their separate wire contract. Do not
  flatten live reports into strings or route local logs through an OTLP loop.
- Viewers receive application-owned buffers. They do not install collectors,
  subscribers, windows, listeners, or process stream writers.

## Required guarantees

1. Retain the original shared report and handler until rendering, including
   downcastable causes, sections, and captured span traces.
2. A capture layer holds a weak buffer reference. Reports are retained only in
   event fields; span extensions and messages render to plain text to prevent
   ownership cycles.
3. Snapshot under locks; format, call sinks, and drop replaced/evicted reports
   outside locks. Supplied sinks must not strongly capture their own buffer.
4. Keep terminal replay and bounded debug history independent. Acknowledging
   printed terminal events must not remove GUI history or replay it on exit.
5. Bound live history by record count; arbitrary reports prevent a strict byte
   bound. Copies for the clipboard contain complete plain text. Only supported
   color/style escapes reach styled rendering; other controls remain visible.
6. Keep capture/report/GUI dependencies optional upstream. Applications using
   JSON formatting alone do not inherit a GUI or eyre dependency.

## Implementation sequence

- [x] Extract capture, report, retention, and ownership regressions upstream.
- [x] Add reusable report-aware egui diagnostic and log views upstream.
- [x] Update both GUI stacks together and adopt the variable font in CT.
- [x] Cut CT consumers directly over; remove the old collector placeholder and
  duplicate implementation. Pass the capture handle explicitly from application
  startup to GUI invocation and add a Logs destination.
- [x] Verify offline behavior, optional feature boundaries, dependency identity,
  formatting, Clippy, and the actual CT executable build.

## Verification and review

Use synthetic reports and headless GUI frames. Do not launch authenticated GUI
flows, acquire credentials, or read production responses. Verify retained report
identity, source context, ownership-cycle release, reentrant formatting/drop,
bounded eviction and stale cursors, independent filters, plain clipboard text,
styling, and font rendering. Compile the upstream synthetic viewer without
launching it. Check that the CT graph has one egui version and resolves
`teamy-tracing` from the intended source at each stage.

Completed on Windows:

- Upstream default, capture-only, eyre, egui, and all-feature tests passed on
  Rust 1.96.0. Default/all-feature strict Clippy, rustdoc, formatting, synthetic
  console examples, and upstream-comparison tests/Clippy passed. The GUI example
  compiled without opening a window. It is now checked by its standalone manifest.
- Upstream default and capture/eyre normal/build graphs contain no Serde.
- CT's 35 terminal-enabled application unit tests, two real-hook report tests,
  six background-worker tests, two tracing policy tests, and two GUI unit tests
  passed. App also compiled without default/terminal features.
- After switching to crates.io, 52 CT tests passed across the application,
  command, tracing, and egui crates, including the error-formatting integration tests.
- CT all-target Clippy passed for application, command, tracing, GUI, and
  entrypoint crates with warnings denied. Rust formatting and both diff checks
  passed.
- The Windows dependency graph contains one egui/eframe/emath 0.36.2 identity,
  with egui-toast 0.22.0. The active GUI uses glow.
- With the crates.io dependency, `cargo build --offline --locked --bin cloud_terrastodon`,
  `--version`, and `egui --help` passed. Validation used synthetic data and headless UI tests.

## Source provenance

The extracted source retains CT's LiLiQ-R-1.1 license. The
[teamy-tracing 0.3.0 archive](https://crates.io/crates/teamy-tracing/0.3.0) includes
the English and French license texts, file attribution, and the library's MIT
and MPL-2.0 notices. It was built from clean commit `82cfd59`. The registry
checksum matches the verified archive and is recorded in CT's lockfile.
