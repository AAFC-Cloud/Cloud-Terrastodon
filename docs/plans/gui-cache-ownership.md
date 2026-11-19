# Share the work cache between an idle GUI and the CLI

**Plan status:** Implemented with Windows validation

**Primary implementation root:** Cloud Terrastodon workspace

**Last updated:** 2026-11-19

**Intent audit:** Passed against the GUI lock diagnosis, the commit-and-plan request, the instruction to implement, and the prior cache ownership requirements recorded below.

## How to update this plan

- `[ ]` Pending
- `[~]` In progress
- `[x]` Complete
- `[!]` Blocked

Update each work item's heading and completion notes together. Record decisions,
validation commands, evidence, and limitations beside the work they affect.

## Outcome and scope

After egui finishes a cache-backed action and its completion work, leave the
window open and run `ct az resource-group list`. The CLI can open the same
cache root as soon as the GUI is idle. Concurrent activities can still wait for
the shared root. Once a waiter acquires it, it validates the current disk
entries and reuses a valid result when the full cache identity matches,
including the authentication partition.

Start with the standalone `ct egui` launcher and its default persistent cache.
The CLI and embedded apps continue using their existing cache ownership
contracts. The work covers cache ownership for GUI requests, account actions
that invalidate the cache, and shutdown of admitted background work.

## User guidance and traceability

| ID | Guidance | Coverage |
| --- | --- | --- |
| G1 | Commit the GUI authentication work before planning the lock change. | 1.1 |
| G2 | Plan how CLI cache-backed commands can run while the GUI remains open. | 1.2, 2.1, 3.1 |
| G3 | Protect the shared cache against competing processes and let a waiting reader reuse completed work. | 2.1, 3.1 |
| G4 | Keep high parallelism inside an instance and preserve cancellation, generation checks, and artifact publication. | 2.1, 2.2, 3.1 |
| G5 | Protect decoded values from silent reuse after another process invalidates their backing data. Retain exclusive ownership while those values remain reusable under the current owner. | 1.2, 2.1, 3.1. This preserves [C7](cache-root-ownership.md#authoritative-guidance-and-traceability) by bounding each GUI cache owner's lifetime. |
| G6 | Use synthetic inputs and isolated roots for development validation. Keep production data and credentials out of validation and personal information out of planning docs. | 3.1, 3.2 |
| G7 | Implement the reviewed ownership plan. | 1.2, 2.1, 2.2, 3.1, 3.2 |

## Intent audit evidence

- **Extraction.** The commit-and-plan request set the order of those
  deliverables. The preceding report identified a cache-backed CLI read
  blocked by an open window. The latest request authorized implementation.
  The root-ownership plan supplies the prior requirements for reuse,
  parallelism, and invalidation.
- **Traceability.** Each guidance item maps to a work item and a verification
  outcome. Completion notes record the implementation and validation evidence.
- **Adversarial check.** The recommendation preserves the same disk cache,
  bounded credential leases, and shutdown completion. It explicitly trades
  memory reuse across idle periods for safe handoff and calls out the separate
  question of simultaneous activity.
- **Source coverage.** The immediate request and diagnosis were reviewed
  directly. Earlier ownership decisions are also recorded in the linked plan.

## Verified foundation

The following describes the window-lifetime ownership at the start of the
follow-up.

- [Application startup](../../crates/app/src/app.rs) creates the default
  `WorkCache` under `AppDir::Commands.join("work")`. The GUI retains a clone
  through its executor, and [Home](../../crates/ui_egui/src/portal_home.rs)
  renders Resources, which starts its first load when an account is ready.
- [WorkCache](../../crates/cache/src/work_cache.rs) acquires the OS lock lazily
  and [WorkCacheState](../../crates/cache/src/work_cache_state.rs) retains it
  until the last shared owner drops. Completed request futures can all be gone
  while the window's executor still retains an acquired owner.
- The [root ownership contract](cache-root-ownership.md) defines the permanent
  `.owner.lock`, the `.owner-active` interruption marker, crash recovery, and
  cancellable waiting. [Existing process tests](../../crates/cache/tests/cache_root_ownership_tests.rs)
  prove handoff and disk reuse after an owner exits, cancellation while waiting,
  and recovery after a killed owner.
- Credential admission already has its own lease lifetime.
  [GUI account reload](../../crates/ui_egui/src/azure_account_operation.rs)
  drains the old authentication provider, and its replacement shares the
  cache. That path does not release a window-owned cache lease.

## Candidate designs

| Design | Consequence |
| --- | --- |
| **Implemented. Share one cache owner across active GUI jobs.** The window keeps the root location and a weak reference to the active owner. The last job or artifact worker can drop the owner while the window stays open. | Preserves the current disk format and lock protocol. The next group of jobs reads current disk entries and may decode them again. Concurrent jobs in one group retain existing typed memory reuse and per-key coalescing. |
| Use a separate GUI root. | CLI and GUI can use their respective roots simultaneously. Each has its own results, invalidation, retention, and cleanup. Requests may execute twice and a CLI refresh cannot update the GUI cache. |
| Add activity-scoped leases and a durable freshness protocol to every shared WorkCache. | Could preserve more in-memory hits across idle periods. It requires freshness checks before memory lookup and a compatibility policy for binaries that take the existing lock without recording an ownership epoch. |

A shorter lock around each filesystem call would expose gaps between cache
lookup, execution, publication, and invalidation. Preserving duplicate
suppression and freshness across those gaps would require a different
cross-process protocol. The recommended design keeps the existing protection
through an entire admitted job. Zero wait during simultaneous cache operations
requires independent roots or a different protocol.

## 1. Confirm the ownership boundary

### [x] 1.1 Commit the GUI account and tenant changes

**Completion notes.** Commit `bc85061f` contains the staged GUI, authentication,
and documentation changes. The staged diff passed the whitespace check with
`cr-at-eol` for existing CRLF files.

### [x] 1.2 Specify the GUI cache-owner contract

**Completion notes.** The standalone launcher creates a `WorkCacheActivity`
from the persistent root already selected by `App::run_parsed`. It passes
that activity explicitly into egui. The base executor starts unacquired and
resolves metadata. Jobs retain the GUI's credential provider and linked
cancellation scope when binding their own cache handles with
`with_work_cache`. Idle GUI state retains the activity's metadata and weak
owner reference, plus rendered
snapshots. Embedded `egui_main` continues to use the caller's existing owner,
including any temporary-directory guard.

**Work.** Record the lifetime of the root, active owner, job executor, and
rendered result. Choose the smallest cache API that can hand out strong
references to a common active owner without letting the idle window retain one.
Confirm how the standalone launcher supplies the default persistent root while
keeping the `App::run` cache unacquired. Keep the embedded `egui_main` path
explicit about caller-owned caches. Forking an already acquired injected owner
on the same path would make the app wait for itself. Preserve temporary cache
directory ownership for any explicitly supported injected path.

**Validation.** Trace all GUI executor creation and cache call sites. Include
request execution and invalidation wrappers, account retirement, artifact
workers, and shutdown. Check `App::work_cache` and temporary-owner behavior
when deciding the API.

**Completion criteria.** The contract identifies the single owner for concurrent
GUI work on the default root. The idle window has no strong reference to that
owner, and embedding follows its documented cache policy.

## 2. Implement scoped GUI ownership

### [x] 2.1 Share one cache owner for overlapping jobs

**Completion notes.** `WorkCacheActivity::work_cache` upgrades or creates the
owner under a short mutex. Canonical claims survive through a separate
shared registry. Each new owner starts with fresh reusable state and uses
the existing disk lease and recovery. `CloudTerrastodonExecutor::with_work_cache`
is limited to ordinary execution outside a cache initializer.
`cargo test --offline --locked -p cloud_terrastodon_cache` and the three
execution suites listed in 3.1 passed on Windows.

**Work.** Atomically upgrade or create the active owner under a short
in-process lock. Keep filesystem acquisition lazy. The coordinator must never
hold its registry lock while waiting for the OS lock. Bind each job executor
to that owner at the outer job boundary and preserve it for nested calls,
pagination, refresh with invalidation, and required completion. Keep the
authentication provider, cancellation links, and execution restrictions
through the binding. Disallow changing cache owners inside existing cache
ancestry.

Keep the owner alive through worker I/O and recovery even if an awaiter drops.
When its final strong handle drops, use existing cache finalization, including
the recovery marker and release ordering. Create a fresh in-process cache state
for the next group. Existing disk selectors and manifests are checked by the
current cache code. Preserve the operation and output type claims across GUI
jobs by retaining the claim registry independently of active ownership. Claims
contain metadata and can outlive the lease without preserving cached values.
Already displayed data are snapshots and never repopulate reusable memory.

**Validation.**

```pwsh
cargo test --locked -p cloud_terrastodon_cache --test cache_root_ownership_tests
cargo test --locked -p cloud_terrastodon_execution --test request_invalidation_tests
```

**Completion criteria.** Overlapping GUI jobs share one owner and unrelated keys
can run in parallel. A completed or dropped job leaves no idle ownership once
its workers finish. A new owner builds its memory from current disk state.
Conflicting operation or output type claims still fail across two GUI jobs
separated by an idle interval.

### [x] 2.2 Route GUI cache use through the active owner

**Completion notes.** Resources, subscriptions, resource groups, Azure DevOps
projects, directory details, and account retirement bind an owner inside
tracked jobs. The retirement executor drops before interactive sign-in begins.
Account catalogue operations, tenant settings, and autosave use their local
state and cancellation paths. The GUI's provider and work tracker retain their
account generation and shutdown behavior. The six headless GUI tests and the
four focused authentication tests passed on Windows. The standalone path
compiled with `cargo check --offline --locked -p cloud_terrastodon_entrypoint`.

**Work.** Apply the job boundary to Resources, subscriptions, resource groups,
Azure DevOps projects, and tenant directory Details. Route the cache effects of
account logout, remove, and reauthentication through a job owner as well.
Keep credential provisioning, account catalogue reads, local tenant settings,
and autosave independent of cache ownership. Bound the cache-bearing
retirement step of `CloudTerrastodonAccountStore::reauthenticate` so the
interactive sign-in can proceed after its retire-and-invalidate owner drops.
Carry forward the same-principal check and durable generation check.
Preserve GUI account generation checks and the authentication provider drain.
Keep the shutdown path responsible for draining the work tracker.

Audit loaded values, UI errors, and pending state changes for retained cache
handles. A cached pointer displayed by the GUI may outlive the job as a
self-contained snapshot. A worker or value that still needs cache files for
completion must retain the owner until that work finishes.

**Validation.**

```pwsh
cargo test --locked -p cloud_terrastodon_ui_egui --lib
cargo check --locked -p cloud_terrastodon_entrypoint
```

**Completion criteria.** Every standalone GUI cache operation uses a tracked
job owner. Local operations and the idle window leave the lock available.

## 3. Prove handoff and document it

### [x] 3.1 Verify independent processes while the window owner stays alive

**Completion notes.** `cargo test --offline --locked -p cloud_terrastodon_cache`
passed 129 tests on Windows. The subprocess harness proved idle handoff and
disk reuse for matching cache identities, external invalidation, cancellation
of waiters, and recovery after a killed owner. The worker test proved that a
dropped awaiter leaves the lock held until its artifact worker finishes.
`cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests --test request_invalidation_tests --test artifact_binding_tests`
passed 23 tests. The GUI tests ran with the GUI library test executable built
by the combined auth and GUI Cargo test invocation. All six passed, including
idle handoff with a retained snapshot, synthetic prompt wait, and caller-owned
embedding. The synthetic auth test also exercised the real retirement and
invalidation path.

Named Azure providers can use different authentication partitions between
invocations. The process fixture uses a matching identity so it isolates
ownership and result reuse from provider policy. Validation used temporary
roots, local synthetic requests, and no live sign-in. Unix runtime validation
was unavailable. The installed Rust target is Windows and WSL enumeration
returned access denied in this environment.

**Work.** Extend the existing process harness with isolated temporary roots and
barriers. Keep the simulated GUI coordinator and a rendered result alive after
its first job. Start a CLI-like process against the same root. Verify it reads
the prior result without another synthetic transport call. Have that process
invalidate or replace an entry. Verify a later GUI load sees the new disk state
even while the earlier snapshot is retained. Cover same-key coalescing and
different-key concurrency with overlapping GUI jobs.

Test an active job, a cancelled waiter, a dropped awaiter with outstanding
artifact work, and process interruption. Pause synthetic reauthentication at
the sign-in prompt after retirement and prove a CLI reader can acquire the
root. Check that new owners gain access after clean completion, that cancelled
waiters stay cancelled, and that crash recovery discards reusable selections
while keeping diagnostics. Use barriers for phase ordering and timeouts only
as deadlock guards.

**Validation.**

```pwsh
cargo test --locked -p cloud_terrastodon_cache
cargo test --locked -p cloud_terrastodon_execution --test artifact_binding_tests
cargo test --locked -p cloud_terrastodon_ui_egui --lib
```

Use the GUI job path in a headless test with a synthetic request and an
injected temporary root to prove that a still-live window owner hands off.
Run the process tests on Windows and on an available Unix target before making
a cross-platform claim. Keep both paths on local filesystems.

**Completion criteria.** The CLI-like reader completes while the GUI
coordinator is idle and alive. It waits during active GUI work and reuses the
published result after handoff. External changes are visible to later GUI
loads. Cancellation and interrupted-owner recovery behave as before.

### [x] 3.2 Document the behavior at the user boundary

**Completion notes.** Updated the reusable application guide, cache ownership
plan, and changelog. They describe idle handoff, active contention, snapshot
behavior, identity-dependent reuse, and explicit standalone versus embedded
ownership. The modified Rust files passed scoped rustfmt and the tracked diff
passed `git diff --check`.

**Work.** Update the cache ownership and reusable application docs and the
changelog. Explain idle handoff, waiting during active work, disk reuse, and
when already displayed GUI results are refreshed. Record the scope for
embedded and custom cache owners.

**Validation.** Compare the documentation with the tested behavior. Run
`git diff --check` and record the commands and targets used for the handoff
tests.

**Completion criteria.** Documentation describes the verified idle behavior and
the conditions that can still cause contention.

## Risks to verify

| Risk | Control |
| --- | --- |
| A long-lived executor, value, or error secretly retains the acquired owner. | Audit GUI state and prove handoff while both the simulated window owner and a rendered result remain alive. |
| Nested requests, cancellation, or shutdown release ownership before writes finish. | Keep job and artifact-worker guards and exercise required completion and a dropped awaiter. |
| An external invalidation leaves decoded GUI memory reusable. | Discard reusable memory at owner handoff and test invalidation between two GUI loads. |
| Reauthentication waits for user input while retaining the cache lock. | Scope cache ownership to the retirement and invalidation work and verify release before the prompt waits. |
| An injected owner already holds the same root or owns temporary cleanup. | Keep standalone opt-in explicit and test the existing embedded ownership contract. |
