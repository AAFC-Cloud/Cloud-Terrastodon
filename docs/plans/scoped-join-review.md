# Scoped concurrency implementation review

**Status:** U114 authorizes the full fixed-join and finite-batch rollout after the
U113 three-caller preview. U115 moves mapping onto `ConcurrentIteratorExt` while
retaining executor joins. Iterator migration and callback refinement are
implemented; focused checks pass. U117 authorizes the local checkpoint. The
interactive shutdown follow-up below remains open.

**Audit date:** 2026-11-09.

This supplements the [executor/cache migration plan](executor-cache-migration.md).

## Completion contract

Fail-fast joins drop unfinished siblings on error. Cancellation deferrals cannot
prevent an explicit future drop. Even reads can owe response decoding and artifact
retention after receiving a response. Dropping a `JoinSet` also aborts its tasks.

The shared helpers establish a cooperative owner and a separately cancellable
group signal. Each factory receives its own linked child scope. On the first
observed error, the owner cancels the group, stops new admission, and drains active
children before returning that error. Optional work stops; admitted required work
finishes. Parent cancellation reaches the group, while inherited deferrals remain
effective. The entire batch is not automatically deferred.

The executor adapters preserve authentication restrictions, cache ownership,
observation, and execution ancestry while changing only the cancellation scope.
Scope-only work does not need an executor. Explicit drops, task aborts, and panics
remain outside this cooperative completion contract.

## Interface

[`execution::join2` through `join5`](../../crates/execution/src/scoped_join.rs)
handle fixed heterogeneous operations. Factories take child scopes; executor
methods take child executors:

```rust
let (definitions, assignments) = executor
    .join2(
        |child| async move {
            fetch_all_role_definitions(tenant_id, auth_source, &child).await
        },
        |child| async move {
            fetch_all_role_assignments(tenant_id, auth_source, &child).await
        },
    )
    .await?;
```

[`execution::ConcurrentIteratorExt`](../../crates/execution/src/concurrent_iterator_ext.rs)
handles finite homogeneous batches with a nonzero concurrency limit:

```rust
use cloud_terrastodon_execution::ConcurrentIteratorExt;

let inventories = projects
    .iter()
    .map_concurrent(executor, 8.try_into()?, |child, project| async move {
        fetch_inventory(&project.id, &child).await
    })
    .await?;
```

The scope-only spelling is
`items.into_iter().map_concurrent_scoped(&scope, limit, factory)`. Both methods are
async terminal operations returning a future, not another iterator or stream.
The iterator extension is blanket-implemented for `Iterator + Sized`; owned and
borrowed inputs use `.into_iter()` and `.iter()` explicitly. The executor mapping
method and public free-function export are removed without compatibility aliases.
Both methods share the existing private coordinator; scope rebinding remains
internal to the execution crate.
Factories are invoked lazily after child admission. The mapper may be `FnMut`;
only its synchronous invocation is serialized, not the returned operation.
Outputs preserve input order. Futures run inline, without spawned-task lifetime
requirements, and may borrow inputs or return non-Clone/non-Send values. This is
I/O concurrency; synchronous CPU work still occupies the polling thread.

The result values and their original indices use separate vectors, reserved from
the iterator's size-hint lower bound (the full length for ordinary collection
iterators). After successful completion, paired swaps restore input order in
linear time and return the existing values buffer without a final collection.
Failure paths drain and return before attempting to reorder partial results.

Two error policies are deliberate:

- Return `Err(error)` to stop admission, cancel siblings, and drain required work.
- Return `Ok(operation.await)` to collect ordinary operation failures as data and
  attempt every item. Inspect the inner results after the whole batch drains.

Complete-all batches now choose an ordinary error in input order rather than
completion order. Parent cancellation fails the batch instead of returning its
partial data, including any earlier collected operation errors. This differs
from a queue that always returns its first completed operation error.

## Rollout

All 14 previously audited production `try_join!` invocations use the scoped joins.
Test joins also migrate: fresh work uses child scopes; already-polled fixture
futures use `tokio::join!` and propagate errors only after both futures complete.

Finite batches now use `map_concurrent`, including:

- Azure DevOps repositories, service endpoints, project/group memberships, test
  plans, pool entitlements, and audit inventories.
- Cross-tenant subscriptions and Entra group details.
- Terraform initialization/validation/generation, generated-file discovery and
  cleanup, workspace reflow/writes, and DevOps dump inventories.

The unused interactive cache-population workflow and menu action were removed
after the rollout, along with their helper and unused progress-bar dependency.

Existing concurrency limits and complete-all policies remain, with explicit
bounds added where old sets spawned every item. Processed workspaces are bounded
to 16; a shared semaphore limits actual file replacements across nested maps to
16. Reflow/planning failures cancel and drain siblings; file-write failures remain
data until all planned workspace writes finish. Output paths are unchanged.

All callers of `ParallelFallibleWorkQueue` have migrated, so its implementation
and re-export are removed. `.clippy.toml` and workspace/root lint tables reject
Tokio and Futures fail-fast join macros; the development conventions describe the
replacement and raw-set ownership requirements. The configuration follows the
[official Clippy macro configuration](https://doc.rust-lang.org/clippy/lint_configuration.html#disallowed-macros).

## Remaining coordinators

Three production `JoinSet` owners remain intentionally:

| Owner | Reason |
| --- | --- |
| [Recursive HCL discovery](../../crates/hcl/src/discovery/discover_hcl.rs) | Work grows during traversal. Jobs only read and parse local files; abandonment owes no required completion. |
| [Required-provider identification](../../crates/hcl/src/work_dir_lifecycle.rs) | Local read/parse work only; no commands, cache publication, or required completion. |
| [Incremental import/provider writes](../../crates/entrypoint/src/noninteractive/dump_everything.rs) | Jobs are created during planning. Its existing cooperative owner records planning failures and drains all started writes before returning. |

[Source hunting](../../crates/source/src/source_hunt_request.rs) also retains a
dynamic `FuturesUnordered` traversal with progress ticks and linked read scopes.
It performs local discovery rather than a fixed batch. Its blocking reads have
checkpoints and no mutation/artifact completion to preserve.

**U118 lifecycle follow-up:** [Picker handlers](../../crates/user_input/src/picker/picker_tui.rs)
now receive independently cancellable child scopes. Reload cancels retired
generations while continuing to poll their required completion; generation-tagged
messages, counts, and errors cannot change the new selection. Acceptance,
cancellation, and terminal/input/handler failures reach a common drain. The
terminal owner has a separate lifetime deferral so optional requests can stop
while terminal restoration and nested-owner handoffs finish. Tab warnings remain
inside the picker instead of creating an uninterruptible stdin reader.

[`AppWorkTracker::finish`](../../crates/command/src/app_work/app_work_tracker.rs)
closes admission, requests each job's linked cancellation signal, and drains all
admitted work, retaining task errors and panics even when normal pruning already
observed them. The GUI tracker also observes pruned outcomes and finishes all
jobs before reporting failure. Work factories use their supplied child scope;
the executor's public scope binding preserves authentication, cache, observation,
and execution restrictions. Nested Ratatui pickers are owned and drained on
shutdown. A poisoned terminal coordinator retires nested frames without waiting
for a backend resume that cannot succeed. These changes are separate from the
finite-batch helpers; their final verification is recorded below.

The object explorer's `TokioInvocationHost` already checks deferrals during normal
cancellation, and normal browser shutdown drains pending invocations. Its hard
shutdown/Drop path synchronously destroys futures to protect borrowed arena
lifetimes. Exceptional engine/runtime teardown can therefore still abandon work;
retain that lifetime-safety cleanup when reviewing shutdown ownership.

## Interactive lifecycle verification

U118 passes 86 focused offline tests: 64 user-input tests (one manual terminal
test remains ignored), 14 execution contracts, four command task-lifecycle tests,
one GUI tracker test, two Ratatui shutdown tests, and one filesystem traversal
test. Coverage includes obsolete-generation isolation, required completion during
reload/shutdown, factory/future/task panics, nested terminal handoffs, consumed
poison and queued-resume races, closed admission, and preservation of executor
restrictions after public scope rebinding. Tests use synthetic work, temporary
storage, and fake terminal backends; native windows and live services were not
exercised.

Workspace all-target compilation, all-feature/all-target picker compilation, and
all-target Clippy with warnings denied across the 11 affected crates pass.
Formatting and whitespace checks pass. Work remains uncommitted for review.

```powershell
cargo test --offline --locked -p cloud_terrastodon_user_input --lib
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo test --offline --locked -p cloud_terrastodon_command --lib app_work_tracker_tests
cargo test --offline --locked -p cloud_terrastodon_ui_egui --lib work_tracker::tests::panic_does_not_skip_another_background_write
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_browser::value_picker_task::tests::close_retains_required_work_until_the_task_has_finished
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_browser::run::tests::failed_terminal_handoff_waits_for_required_ui_cleanup
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib visits_fs_entries_breadth_first
cargo check --offline --locked --workspace --all-targets
cargo check --offline --locked -p cloud_terrastodon_user_input --all-targets --all-features
cargo clippy --offline --locked --all-targets -p cloud_terrastodon_user_input -p cloud_terrastodon_execution -p cloud_terrastodon_command -p cloud_terrastodon_azure -p cloud_terrastodon_azure_cli -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_cli -p cloud_terrastodon_entrypoint -p cloud_terrastodon_ui_egui -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_zombies -- -D warnings
```

## Callback capture refinement

The subsequent callback cleanup uses borrowing async closures in SDK requests,
CLI/picker callbacks, HCL/dump workflows, and their local tests. Original fields
are borrowed directly. Independently retained callbacks still clone the handles
they own; mutable factory snapshots and synchronous caller capture retain their
invocation-time behavior. Child scopes, result policies, and completion ordering
are unchanged. U118 separately implements the interactive shutdown follow-up above.

Workspace all-target compilation and all-target Clippy with warnings denied pass
for execution, cache, command, app, auth, REST, Azure/DevOps SDKs and CLIs, HCL,
and entrypoint. All 180 selected local tests pass; formatting and whitespace
checks pass. No live-service or Terraform tests ran.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests --test uncached_work_tests --test uncached_artifact_tests --test artifact_policy_tests --test observation_tests --test scoped_join_tests --test map_concurrent_tests
cargo test --offline --locked -p cloud_terrastodon_cache --lib --test work_cache_tests --test cache_invalidation_tests --test work_artifact_invalidation_tests --test artifact_invalidation_tests --test artifact_store_tests
cargo test --offline --locked -p cloud_terrastodon_rest --test rest_lifecycle_tests
cargo test --offline --locked -p cloud_terrastodon_command --test command_cancellation_tests --test executor_cache_tests
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --test process cancellation
```

## Verification

The U114 rollout passed 89 focused tests plus one documentation example:

- Execution: 59 tests, including nine map tests and five scoped-join tests. The
  final rerun covers the nested/inherited-deferral tests and the expanded
  credential-restriction probe for every executor adapter.
- Principal/RBAC inventories: five tests; DevOps project inventory: three tests.
- Cross-tenant failure/empty/success collection: one test, including the limit of
  four active tenants and deterministic ordering.
- Command cache/process lifecycle: 12 tests; REST lifecycle: nine tests.

Affected-crate all-target Clippy passes with warnings denied. Formatting and
whitespace checks pass. Workspace all-target compilation passes.

**U115 iterator refinement:** Reran all nine map behavior tests, fourteen execution
contract tests, and both documentation examples through the iterator API. All
pass, as do workspace all-target compilation and all-target Clippy with warnings
denied for the seven affected crates (execution, Azure/DevOps SDKs and CLIs, HCL,
and entrypoint). Formatting and whitespace checks pass. The old executor mapping
method and free-function export have no remaining callers; staging is unchanged.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test map_concurrent_tests --test execution_contract_tests
cargo test --offline --locked -p cloud_terrastodon_execution --doc
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_execution -p cloud_terrastodon_azure -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_cli -p cloud_terrastodon_azure_devops_cli -p cloud_terrastodon_hcl -p cloud_terrastodon_entrypoint --all-targets -- -D warnings
```

Focused validation uses only synthetic inventories, temporary caches/files,
loopback HTTP fixtures, and local fixture processes. Live-service tests are
compiled but not run. No production configuration, credentials, or service data
are needed.

Reproduction commands (use the plan's existing bounded build-storage settings):

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests --test rbac_executor_inventory_tests
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test project_inventory_cache
cargo test --offline --locked -p cloud_terrastodon_azure_cli failed_and_empty_tenants_survive_collection_beside_successful_tenants
cargo test --offline --locked -p cloud_terrastodon_command --test executor_cache_tests --test command_process_tests
cargo test --offline --locked -p cloud_terrastodon_rest --test rest_lifecycle_tests
cargo clippy --offline --locked -p cloud_terrastodon_execution -p cloud_terrastodon_command -p cloud_terrastodon_rest -p cloud_terrastodon_azure -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_cli -p cloud_terrastodon_azure_devops_cli -p cloud_terrastodon_hcl -p cloud_terrastodon_entrypoint --all-targets -- -D warnings
cargo check --offline --locked --workspace --all-targets
```
