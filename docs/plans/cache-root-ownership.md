# Exclusive cache-root ownership

**Plan status:** Complete; all Phase 8 tasks passed Windows offline acceptance.
The user authorized review and commit on 2026-11-13. Linux runtime checks were
not run and Linux execution evidence is not claimed.
**Primary implementation root:** Repository root (`.`).
**Last updated:** 2026-11-13
**Intent audit:** Passed for the available Phase 8 discussion; closed design
gates and validation evidence are recorded below.

This is the completed Phase 8 work contract for the
[executor/cache migration](executor-cache-migration.md). Phases 1–7 and the
[account/CLI ownership refinement](account-authentication-resolution-cutover.md)
are established foundation, not work to repeat. The completed changes are
authorized for a local commit; publication and production-data access are outside
this request.

## How to update this plan

- `[ ]` Not started
- `[~]` In progress
- `[x]` Complete
- `[!]` Blocked, with the precise blocker and condition for resuming

Update task headings and their completion notes together. A task is complete
only with the evidence required below. Parallel work is intentional: the root
agent owns implementation/integration, delegated review checks lifecycle and
crash recovery, and the documentation track maintains this contract. No task is
complete merely because another track has started.

## Authoritative guidance and traceability

| ID | Active guidance | Coverage and required evidence |
| --- | --- | --- |
| C1 | Parallel CT commands must not corrupt their shared cache. A waiting command must check the cache after gaining ownership so a valid first result avoids repeating the request. | 8.2/8.4/8.5: subprocess contention and second-owner disk reuse. Existing identity, freshness, and mutation rules still apply. |
| C2 | Normally only one CT instance works at a time; a coarse top-level lock is acceptable. | 8.1/8.2: exclusive root ownership, not a new per-entry process-lock scheme. No strict first-come-first-served scheduling promise. |
| C3 | High parallelism within one instance remains important; reference-counted ownership should let the same cache owner retain the lock. | 8.2/8.5: existing `Arc<WorkCacheState>` lifetime, concurrent clones and unrelated initializers, no process-wide PID bypass for independently constructed handles. |
| C4 | Acquire the lock for operations requiring the cache; lightweight construction and commands that never use the cache should not acquire it. | 8.2/8.4: lazy acquisition before cache execution, lookup, publication, inspection, or invalidation; metadata-only construction stays free of filesystem I/O. Explicit temporary-root creation remains intentional I/O. |
| C5 | Preserve existing same-owner cancellation, invalidation, and artifact-publication coherence. | 8.2–8.5: retain per-entry initialization, generation/dependency rules, required completion, and artifact-worker ownership; cancellable lock waiting. |
| C6 | Retain the lock through all cache clones and outstanding artifact workers, then release it with the final owner. | 8.2/8.5: ownership survives dropped awaiters and blocking work; no detached blocking waiter can acquire a lock after cancellation. |
| C7 | Keep the lock between operations while decoded memory values remain owned, rather than silently allowing another process to invalidate their backing data. | 8.1/8.2/8.5: no idle-operation unlock/relock; owner handoff occurs only after all prior ownership ends. |
| C8 | Update the plan, then complete the full implementation cutover. | 8.1 precedes implementation; 8.4 audits every cache I/O entry point; 8.5 records completion evidence. No optional legacy bypass path. |
| C9 | Earlier migration decisions require Windows/Linux behavior and process-crash recovery to be considered before claiming independent-owner coordination. | 8.1/8.3/8.5: explicit recovery protocol and subprocess termination tests; Windows evidence recorded, Linux evidence never inferred. |
| C10 | Preserve repository conventions and review constraints: scoped changes, no production data, no personal information in plans, commit only when requested. | All tasks: synthetic temporary storage, focused checks, documentation privacy review. The user subsequently authorized review and commit on 2026-11-13. |

## Intent audit evidence

- **Pass 1 — extraction:** Reread the available discussion from the deferred
  Phase 8 status through directory locking, single-instance expectations,
  reference-counted lifetime, lazy acquisition, and the authorization for full
  implementation. C1–C10 retain each distinct requirement and relevant existing
  migration constraint.
- **Pass 2 — traceability:** Mapped each C ID to the contract, implementation,
  recovery, integration, or validation tasks above. Coarse exclusive ownership
  follows the approved direction. Recovery format and acquisition mechanics are
  implementation decisions that must be recorded in 8.1 rather than presented
  as prior user choices.
- **Pass 3 — adversarial omission:** Checked that “normally one instance” does
  not erase interprocess contention, that shared ownership does not mean all
  handles with the same PID are coherent, and that avoiding duplicate reads
  does not turn writes or explicit refreshes into cache hits. Preserved lazy
  construction, high internal concurrency, worker lifetime, and the required
  second lookup after ownership transfer.
- **Known source limitation:** Earlier phases include compacted conversation.
  Their established requirements come from the owning plan and completed
  companion plans; this audit does not claim to have reread unavailable original
  messages. The current Phase 8 direction is available directly.

## Verified foundation and scope

`WorkCache` already owns `Arc<WorkCacheState>`. Clones share a short registry
mutex, per-entry initializer locks, and an asynchronous artifact I/O gate.
`ArtifactIoGuard` retains its gate inside blocking workers when the async awaiter
is dropped. Cache and artifact handles retain temporary-root ownership.

The phase began with immutable attempt directories and validated manifests;
`ArtifactSelection` selects a complete attempt. Mutable selection writes then
truncated files, and invalidation quarantine lived only in memory. The new root
lock alone cannot make interrupted invalidation safe for a new owner, so recovery
and atomic metadata publication are part of this phase rather than an unmentioned
later requirement.

The scope is cooperative exclusive ownership of one physical cache root, including
cache inspection and invalidation. Authentication selection, credential stores,
service endpoints, request schemas, registry design, and CLI command shape stay
outside this change. The root lock is not a credential-store transaction and does
not instantly cancel a different process's already admitted account work.

## Design gates

### [x] 8.1 Close root ownership and interruption-recovery contracts

**Completion notes:** The user selected coarse, lazy ownership retained for the
shared cache owner's lifetime. The implementation uses standard-library
`File::try_lock` with asynchronous cancellable retry and an active-owner marker.
`same-file` 1.0.6, already present transitively in the lockfile, becomes a direct
cache dependency solely to verify physical lock-file identity; OS locking remains
in the standard library and no dependency version is upgraded.
A marker left by an interrupted or untrusted owner
causes the next owner to discard **all reusable selectors** before admitting any
work, preserving immutable diagnostic attempts. This conservative recovery avoids
persisted per-entry generation machinery under exclusive lifetime ownership.
The last clean owner removes the marker only when state is unpoisoned and no
publication/invalidation quarantine remains. The lock file remains in place.
The selected files are `.owner.lock` (stable lock identity) and `.owner-active`
(interrupted-owner indication). A malformed entry identity or unrecoverable
filesystem error fails admission rather than allowing a partial recovery.
Before recovery and healthy completion, the held lock handle must identify the
current lock file. A waiter holding a file removed by temporary-root cleanup
cannot acquire that obsolete file and then access a recreated root as its owner.

Metadata creation and replacement stage complete JSON in same-directory
`NamedTempFile`s before publication. Canonical parent paths preserve Windows
extended-length path handling. The raw artifact format remains `artifacts-v2`;
there is no speculative upgrade of older untrusted payloads.

The lock protects process interruption on cooperating local-filesystem clients;
it does not claim power-loss durability, network-filesystem correctness, or
coordination with older binaries that ignore the lock. The CLI clean path was
identified as deleting a parent directory of the cache: task 8.4 must replace
that route with coordinated clearing that preserves the lock-file identity.

| Question | Required decision | Acceptance consequence |
| --- | --- | --- |
| Ownership identity | One OS lock file in the cache root; independently constructed owners contend even in the same process. Keep the lock file's identity stable rather than unlinking it on release; verify the held handle still identifies the current file before recovery/completion. | Independent-handle and subprocess contention; temporary-root cleanup cannot let an old lock handle coordinate access to a recreated root. |
| Async acquisition | Use nonblocking OS acquisition with asynchronous retry/cancellation; no detached blocking lock acquisition. | Cancelled/dropped waiters leave no later lock owner or permanently running worker. |
| Retained lifetime | Store acquired ownership in shared cache state; clones, attempts, and blocking filesystem work keep that owner alive. | Owner/worker/drop tests and temporary-directory cleanup order. |
| Recovery record | Active-owner marker established before cache use; stale marker clears all reusable selections under the exclusive lock. Clean final drop removes it only if no quarantine or poison remains. | Terminated-process and failed-invalidation handoff tests prove incomplete work cannot reactivate old selections. |
| Filesystem support | Establish local-filesystem Windows/Linux behavior and error handling; never silently proceed when locking is unsupported. | Native Windows tests; Linux compilation/runtime checks when available; do not claim network-filesystem or power-loss durability. |
| Existing readers | Older binaries or unrelated tools that ignore the lock are outside the coordination guarantee. | Documentation states that simultaneous use requires cooperating versions; no compatibility bypass is introduced. |

**Work:** Inspect public cache entry points, temporary cleanup, artifact workers,
and direct filesystem consumers. Keep the exact root-lock and clearing APIs in
the implementation task completion notes once the source settles.

**Validation:** Review the source paths below and check the final design against
C1–C10 before closing this task. No live requests or production cache inspection.

**Completion criteria:** Lock identity, acquisition, lifecycle, recovery, and
support boundaries are concrete enough to implement and test without changing
the user-selected coarse ownership model.

### [x] 8.2 Implement lazy shared exclusive ownership

**Completion notes:** `WorkCache::acquire(&CancellationScope)` provides lazy
once-per-owner acquisition through `WorkCacheState`'s `OnceCell<CacheRootLease>`.
`CacheRootLease` uses `File::try_lock`, asynchronous contention retries, and a
recovery worker that owns the lease. `WorkCacheState::drop` destroys cached values
before clearing a healthy active marker; field drop order retains the OS lock
through temporary-root cleanup. The full cache command recorded in 8.5 passed
127 tests, including lazy claims, artifact-handle retention, unrelated initializer
concurrency, dropped acquisition retry, subprocess cancellation, and a blocking
artifact worker retaining the root after its awaiter is abandoned.

**Work:** Add the semantic lock/ownership type beside `WorkCacheState`. Acquire
once per shared owner before operations that depend on the cache. Keep constructors
and logical claims free of lock I/O. Preserve per-key concurrency within the owner
and hold ownership until its final retained user or worker ends. Waits honor
cancellation without abandoning deferred completion already owed.

**Validation:** Focused cache tests prove lazy acquisition, same-root independent
handle exclusion, clone reuse, unrelated-root independence, cancellation/drop
while waiting, internal parallelism, and final-owner/worker release. Use explicit
barriers/notifications with bounded test timeouts rather than inferred scheduling.

**Completion criteria:** A single shared owner holds one OS lock without placing
an exclusive root mutex around each initializer; independent owners cannot touch
cache state before ownership/recovery completes.

### [x] 8.3 Make owner handoff recover interrupted disk transitions

**Completion notes:** `CacheRootLease::recover` discards all reusable selectors
when `.owner-active` already exists. The marker remains on failure, and malformed
entry identities prevent admission. `ArtifactFile::create_json` and
`replace_json` publish complete metadata from same-directory temporary files;
Windows destinations use canonical parent paths. The 127-test cache suite in 8.5
passed the killed-owner recovery scenario, failed-completion recovery and
fail-closed malformed-entry handling, physical lock replacement rejection, and
all-scope invalidation racing an in-flight initializer. Existing integrity and
partial-artifact scenarios passed in the same run. These tests preserve attempts
while proving that untrusted reusable selections cannot survive owner handoff.

**Work:** Implement the 8.1 recovery protocol around reusable-selection publication
and invalidation. Recovery happens while exclusively owning the root and before
cache admission. Retain diagnostic attempts; do not accept incomplete payloads
or resurrect selections invalidated by an interrupted previous owner.

**Validation:** Subprocess tests terminate an owner with controlled partial
artifact state, acquire from a new owner, and prove lock release plus safe cache reuse or
fail-closed misses. Existing artifact integrity, invalidation failure, and
worker-drop tests remain applicable. This proves process interruption, not
filesystem correctness under power loss or malicious concurrent writes.

**Completion criteria:** Process termination cannot leave reusable entries that
the recovery protocol mistakes for a successfully completed protected transition.

### [x] 8.4 Cut over every cache boundary

**Completion notes:** The public `WorkCache::invalidate_all(&CancellationScope)`
invalidates all reusable generations/selections and retains artifacts/lock
ownership. `ct clean` invalidates only the configured owner. It preserves both
that root and the default `Commands/work` subtree from recursive removal,
including overlapping ancestor/descendant generated directories; an unused
default cache remains untouched when an application injects a custom owner.
Acquiring that second root while retaining the first could deadlock with another
process acquiring roots in the opposite order. Other generated directories retain
normal removal. Direct source review covers typed work, artifacts, invalidation,
inventory, and execution admission. Independent lifecycle/recovery review found
no material defects after the physical identity and cleanup lock-order fixes.
Adapter acceptance passed 42 tests, including cancellation while waiting for
root ownership before Command/REST effects begin. The three targeted cleanup
tests in 8.5 also passed, covering managed-root preservation, configured-cache
invalidation without acquiring the protected default root, and existing size
formatting. The existing 22 execution/SDK regressions passed as recorded below.

**Work:** Audit `CachedWorkSpec`, raw artifact begin/load/finish/publish,
invalidation, rejection, inventory, and cache-cleaning consumers. All operations
that inspect or modify the shared cache participate in the same ownership. Remove
test patterns that keep two independent owners alive when intentionally testing
serial reuse; preserve meaningful same-process contention scenarios explicitly.

**Validation:** Focused cache, execution, Command, REST, and relevant CLI tests
use temporary synthetic inputs only. Compile all affected consumers. Search for
direct cache filesystem access and classify any intentional bypass in task notes.

**Completion criteria:** No public cache execution/I/O route can bypass root
ownership, and request/authentication/cancellation behavior remains intact.

### [x] 8.5 Record acceptance and reconcile documentation

**Completion notes:** Current migration, coordination, and account contract docs
link this ownership contract. `REUSABLE_APPLICATIONS.md` documents lazy ownership,
independent-handle waiting, recovery, upgrade limitations, and
`invalidate_all`; authentication docs distinguish cache ownership from account
retirement. Unreleased notes record the changed concurrency and cleanup behavior.
All C1–C10 requirements have implementation and validation evidence below.
Native Windows acceptance passed; Linux and live-provider execution were not run.
No production data, credentials, or production cache contents were accessed.

The final native Windows cache suite passed **127 tests**: 52 library, 4 input
integrity, 10 artifact invalidation, 13 artifact storage, 14 cache invalidation,
2 inventory, 6 root ownership, 4 temporary ownership, 3 work-artifact invalidation,
and 19 typed-work tests. One ignored subprocess helper is deliberately invoked
by the parent ownership tests. This includes the replacement-lock regression,
the artifact-gate cancellation fixture, and all-scope in-flight invalidation.

```powershell
cargo test --offline --locked -p cloud_terrastodon_cache --lib --tests --no-fail-fast
```

The final adapter/artifact command passed **42 tests**: Command cancellation 9,
Command executor cache 9, artifact binding 3, artifact policy 5, uncached artifacts
5, and REST lifecycle 11. This includes the new cancellation checks that ensure
waiting for root ownership does not admit a Command/REST effect first; deferred
execution and artifact completion remain protected after admission.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution -p cloud_terrastodon_command -p cloud_terrastodon_rest --test artifact_binding_tests --test artifact_policy_tests --test uncached_artifact_tests --test executor_cache_tests --test command_cancellation_tests --test rest_lifecycle_tests --no-fail-fast
```

The final request/SDK regression command passed **22 tests**: execution contracts
14, execution observations 4, Azure inventory 1, and DevOps project inventory 3.
These use synthetic temporary inputs and prove existing execution/discovery
contracts against the new owner lifetime; they do not prove live compatibility.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests --test observation_tests -p cloud_terrastodon_azure --test executor_inventory_tests -p cloud_terrastodon_azure_devops --test project_inventory_cache --no-fail-fast
```

The targeted cleanup command passed **3 tests**: two cleanup behavior regressions
and the existing size-formatting check. The recorded focused total is **194
passing tests**. All service-facing validation used synthetic inputs only.

```powershell
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib noninteractive::clean::tests --no-fail-fast
```

The root-generation proof makes `WorkFreshness` larger. A narrow documented
`large_enum_variant` exception keeps its short-lived proof inline rather than
adding an allocation to every disk decode. The final full-workspace Clippy check
passed with warnings denied after that lint-only refinement.

The following final commands passed against the implementation tree:

```powershell
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

The focused plan privacy scan found no user-specific paths or personal identity.
The final documentation review distinguishes exclusive pre-admission recovery
from normal invalidation under the artifact gate, and keeps Linux/runtime-service
limits explicit. The subsequent pre-commit review found no actionable issues in
cache ownership/recovery or the Command/REST/execution/cleanup integration.
Publication was not requested or performed.

The final pre-commit check repeated the 127-test cache suite and full-workspace
Clippy after adding the one-time ownership-contention warning; both passed.
Command and REST now use `WorkReuse` instead of a shared-execution boolean. Their
29 focused lifecycle/cancellation/cache tests and all-target/all-feature Clippy
also passed after that refactor. Formatting and whitespace checks passed.

**Work:** Run the tests below after inspecting their inputs, update the owning
plan's Phase 8 and acceptance table, and reconcile coordination/contracts docs and
Unreleased notes. Record test counts and exact outcomes here, including any
unavailable platform validation. Preserve historical foundation evidence as
historical rather than rewriting earlier checks as cross-process proof.

**Validation commands** (run from the repository root; no credentials/services):

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
cargo test --offline --locked -p cloud_terrastodon_cache --lib --tests --no-fail-fast
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

Add exact new-test filters and downstream test commands beside their outcomes,
not as unexecuted evidence. Broaden/repeat checks only when failures or additional
changes justify doing so.

**Completion criteria:** C1–C10 have evidence, all implementation tasks are
complete, consumers compile, documentation describes actual guarantees, and the
changes have been reviewed before the explicitly requested commit.

## Acceptance matrix

| Boundary | Required proof | Evidence |
| --- | --- | --- |
| Shared owner/clones | One root acquisition; same-owner concurrency, cancellation, generation and worker ownership preserved | 127-test cache suite passes, including lazy ownership, distinct initializer concurrency, artifact-worker retention, and generation races |
| Independent owners, same process | Real OS contention, cancellation, lifetime handoff | Cache suite passes artifact-handle lifetime, dropped acquisition retry, and obsolete physical-lock rejection |
| Separate processes | Exclusion, reusable disk result after handoff, termination releases ownership | Ownership subprocess tests pass clean handoff/disk reuse, cancelled waiter isolation, and killed-owner OS lock release |
| Interrupted disk transitions | Next owner recovers before reuse; invalidation cannot resurrect older selections | Killed-owner and failed-completion recovery tests pass; malformed recovery stays closed and diagnostic attempts remain |
| Windows local filesystem | Focused behavior and consumer compile/Clippy | 194 focused tests passed: 127 cache, 42 adapter/artifact, 22 execution/SDK, and 3 cleanup; all-target/all-feature workspace Clippy, rustfmt, and diff checks passed |
| Linux local filesystem | Same native lock/recovery tests when an environment is available | Not run; do not infer a pass from Windows |
| Network filesystems, old noncooperating readers, hostile filesystem writers | Outside the tested ownership contract | No guarantee claimed |
| Power-loss durability and transactional credential-store coordination | Outside this phase's process-interruption contract | No guarantee claimed |

## Implementation references and risks

- `crates/cache/src/work_cache.rs`, `work_cache_state.rs`, `work_cache_root.rs`:
  shared ownership and temporary-root lifecycle.
- `crates/cache/src/cache_root_lease.rs`: OS lock identity, acquisition, active
  marker, and interrupted-owner recovery.
- `crates/cache/src/cached_work_spec.rs`: typed cache admission/initialization.
- `crates/cache/src/artifact_entry.rs`, `artifact_attempt.rs`,
  `artifact_io_guard.rs`, `artifact_selection.rs`, `artifact_file.rs`:
  artifact I/O, worker retention, and publication.
- `crates/cache/src/work_cache_invalidation.rs`, `artifact_directory.rs`:
  invalidation quarantine, selectors, and inventory.
- `crates/cache/tests/`: synthetic storage, integrity, invalidation, and lifetime
  tests to preserve and extend.
- `crates/entrypoint/src/noninteractive/clean.rs`: coordinated cache invalidation
  and protection against deleting a live cache root or its ancestors.
- `docs/plans/executor-cache-coordination.md`, `executor-cache-contracts.md`:
  existing inner coordination and account-lifecycle contracts.

| Risk | Guardrail |
| --- | --- |
| Released/recreated lock file creates two owners | Stable lock-file identity, no unlink-on-release, and held/current physical identity checks; inspect cleaning paths. |
| Owner is dropped while a blocking writer continues | Worker retains root ownership; explicit regression test. |
| Cancelled waiter later acquires and retains the root | Nonblocking attempts plus async cancellation; no detached blocking wait. |
| Root lock serializes internal requests | Lock once per owner, retain existing per-entry coordination; independent-key concurrency test. |
| Second owner trusts interrupted invalidation | Recovery record and admission barrier; terminated-process regression. |
| Memory values survive an ownership gap | No unlock while the shared owner is retained; independent owners have distinct state. |
| Cleanup holds two roots in reversed order | Invalidate only the explicitly supplied owner; retain other protected cache roots without acquiring them. |
| Stale documentation overclaims support | Update current contracts, keep earlier evidence historical, explicit platform matrix. |
