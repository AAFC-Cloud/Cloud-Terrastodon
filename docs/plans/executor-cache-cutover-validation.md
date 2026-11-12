# Comprehensive executor cutover evidence

Updated: 2026-11-10. Authority: U135 in
[the migration plan](executor-cache-migration.md). All changes remain uncommitted.
Phases 6 and 7 are complete with Windows acceptance evidence below. No live
authentication or service calls were used for development; tests use synthetic
data and isolated storage.

## Implementation by task

| Task | Delivered behavior |
| --- | --- |
| 6.0a | Closed A1–A4/V1 with stable CT account IDs, scoped aliases/defaults, serialized account/source/tenant intent, account leases, and distinct runtime-test registration. |
| 6.0b | Registry-driven `test list/run <pattern>`, explicit typed inputs/effect admission, optional normal clean invalidation, typed outcomes/collection coverage/cache observations, cancellation-aware partial reports. The inventory classifies 132 removed live/obsolete Cargo checks and 127 production runtime cases. |
| 6.0c | Account catalogue/CLI, isolated CLI/browser/workload/PAT/Gitea credentials, explicit legacy import, provider account map, renewal/replacement distinction, retirement cancellation/draining and partition/dependency invalidation. |
| 6.0d | Pure account selections in Azure/DevOps/Gitea requests, direct request-first callers, domain-qualified Azure/Entra and resource-group/storage-account names/files. Cow fields, Arc results and typed errors retained. |
| 6.0e | Synthetic same-tenant selection, immutable serialized replay after alias/default changes, mixed-account leases, cross-account dependency retirement, disabled-auth admission, and runtime-runner policy regressions. |
| 6.1 | Four independent serializable Azure audit requests return typed findings and share memoized inventories; five focused policy/shared-fill regressions. |
| 6.2 | App records preferences without account discovery; callers can supply a shared or temporary WorkCache. Process fixtures isolate storage and exercise parsed `--auth-source none`. |
| 6.3 | CLI/UI/registry/examples/tools use explicit request execution. UI export/discovery and SourceHunt request data no longer contain runtime scopes. Local cancellation-only work remains independent. |
| 7.1 | Removed forwarding factories, ambient tenant/subscription/DevOps defaults, obsolete PIM persistence, and reflective IntoFuture/cancellation bridges. Retained helpers perform real workflow or local-scope work. |
| 7.2 | Current authentication/runtime-verification/application guides, examples and changelog updated. Publication DAG inspected without publishing. Final workspace, feature, formatting and privacy checks are recorded below. |

## Review corrections

- Gitea CLI and runtime-test admission reject `--auth-source none` and incompatible
  source overrides before reading account/instance configuration.
- Direct authorization and registry token exchange retain account leases and
  observe retirement, alongside the SDK/REST execution boundary.
- All four direct cached/uncached work APIs acquire account leases on first poll.
  Retirement cancels optional work and drains required response/artifact/cache
  completion. Unpolled bindings neither hold logout open nor return a retired
  account's warm result; synchronous canonical ownership claims remain intact.
- DevOps project-member invalidation uses the same resolved project identity as
  execution and refreshes only its reachable child membership requests.
- ACR and PIM refresh include endpoint/current-user discovery. Entra membership
  ID discovery owns its cache entry; parent refresh invalidates the deterministic
  resolved group batch. The previous membership path lacked a required cache key.
- Runtime input manifests reject misspelled case names; cancelled cases close
  later admission even when only their account scope was cancelled.
- Verification observes collection counts through shared ownership without
  cloning or retaining inventory payloads. Other output shapes are Unmeasured.
- UI tests now model account-field expansion and put deferred completion inside
  the operation's own bounded scope, preserving the intended graceful-drain test.
- A configuration test still used the default application directory; the final
  workspace run exposed it through a permission failure. All three configuration
  fixtures now override their storage path with independent temporary directories.
- The generic Azure role-management policy-assignment request now acquires its
  account lease before invalidation, matching the other authenticated requests.
- Final registry checks now follow explicit collection-registration provenance
  and the shared picker output. Missing concrete output generators are registered;
  generated membership maps preserve descriptor/key agreement. Discovery graphs,
  cost row/column tables, and linked role inventories remain explicitly outside
  arbitrary response simulation because their relationships require construction
  beyond independent field generation.
- Workspace doctests exposed an existing standalone `#[track_caller]` attribute
  with no item. The example now includes the function it annotates; runtime code
  is unchanged.

## Validation

All Cargo commands use `CARGO_INCREMENTAL=0` and `--offline --locked`.
Offline constrains dependency acquisition; test isolation was separately reviewed.
The final rebuild uses `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0` to reduce generated artifacts. Debug assertions and
Facet's debug response checks remain enabled. No repository profile was changed.

Completed before the clean rebuild:

- Auth/auth-types synthetic policy tests: 72 passed before removal of the obsolete
  PIM-config test. Latest direct-authorization retirement regression: 1 passed;
  selected app-context tests: 2 passed.
- Cache/registry/Azure/AzureCLI batch: 279 tests and documentation tests passed.
  Four runtime verification regressions passed after collection coverage landed.
- Command/DevOps/Gitea/Excalidraw batch: 154 passed. UI initially passed 266/268;
  two fixture corrections are included in the final workspace rerun.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `publish.ps1 -DryRun` with Cargo network access disabled produced an acyclic
  34-package order. The relevant sequence is registry/cache/types, Azure types,
  DevOps types, auth_types, execution, command/registry_execution, auth, REST/SDKs,
  CLI/UI/application facade. Lockfile changes are internal dependency changes;
  no third-party versions were upgraded.

Final checks:

- `cargo test --workspace --all-features --lib --tests --no-fail-fast` passed
  1,668 tests with zero failures. Five existing offline previews/unfinished pure
  policy tests remain ignored. This includes all 72 root-entrypoint checks and
  all 268 Ratatui tests after their final corrections.
- `cargo test --workspace --all-features --doc` passed all seven runnable/compile-fail
  examples; one existing UUID macro example remains ignored.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed
  again after the clean rebuild and final review corrections.
- `cargo test -p cloud_terrastodon_app --no-default-features --lib --test process`
  passed 47 tests (32 library, 15 isolated process tests).
- Isolated configuration integration tests passed 3/3. Direct-work retirement
  regressions passed 3/3, each covering all four public work entry points.
- Formatting and whitespace checks pass; the changed plan documents contain no
  personal identity or user-directory paths. No changes are staged.

An earlier final-test attempt exhausted disk before tests; the user ran
`cargo clean`, and validation resumed from a clean target directory.

## Limits

Windows is the available validation platform. Linux compilation/runtime evidence
is unavailable in this checkout and is not claimed. Live provider/service
compatibility was not exercised. A cached verification pass does not establish
current API compatibility. Separate-process cache coordination remains Phase 8;
the initial guarantee covers a shared cache handle and session owner.

See [account contracts](accounts-runtime-tests-and-requests.md),
[runtime inventory](runtime-test-cutover-inventory.md),
[authentication](../authentication.md), and
[runtime verification](../runtime-verification.md).
