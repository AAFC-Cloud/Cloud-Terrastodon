# Explicit execution and shared typed cache migration

**Plan status:** Active migration; phases 1–2 and workspace error-handling
refinement U47 complete. Next migration task: 3.1.

**Primary implementation root:** Repository root (`.`); no branch change prescribed.

**Last updated:** 2026-10-30

**Source baseline:** `26e14bcc`; tracked files were initially clean. Pre-existing
untracked notes are preserved.

**Intent audit:** Passed 2026-10-30 against the original design discussion,
subsequent corrections, plan request, three planning-question replies, and the
latest review corrections about production data, validation, and key collisions,
and authorization to proceed with implementation.

**Planning method:** User-invoked `resumable-implementation-plans` skill.

**Current authorization:** The user authorized proceeding with the migration on
2026-10-30 and subsequently requested committing the completed foundation and
proceeding. The reviewed foundation was committed as `3ebfe69b`, and the completed
artifact-storage slice as `821acce9`. After that checkpoint and the next-phase
description, the user authorized proceeding with task 2.3. The latest request
authorizes committing that slice and its fingerprint refinement, then describing
task 2.4. The coordinated-cache checkpoint is `49352932`; the subsequent request
to proceed authorizes task 2.4. The subsequent review requires fallible directory
construction and prohibits production `unwrap`/`expect`; the user explicitly
selected cleanup of existing violations across the entire workspace now. U48
authorizes committing the completed changes and pushing the current `main`
branch to `origin`, including its preceding local checkpoints. Package releases,
production-data access, and implementation of the next phase remain outside this
checkpoint's authorization.

## How to update this plan

- `[ ]` Not started
- `[~]` In progress
- `[x]` Complete
- `[!]` Blocked: record the exact blocker, last evidence, and unblocking condition.

Put status in each work-item heading. Update its completion notes at the same time.
A phase is complete only when all its items are complete. Record decisions,
commands and outcomes, affected paths, and follow-ups beside the relevant item;
do not append a detached chronological log. Keep one implementation focus at a
time unless independent tracks and owners are explicitly recorded. Phase 8 is a
deferred follow-up and is excluded from initial-migration completion.

Use repository-relative paths and synthetic examples throughout. Do not copy
usernames, home directories, real account or tenant details, credentials, or other
personal information into this plan or its evidence. Preserve user edits and
staging. A future implementation session must reread `AGENTS.md` and reconcile
changes since the recorded baseline before starting.

## Intent audit evidence

- **Pass 1 — extraction:** Reread the original user messages from the independent
  audit/cache-race discussion through executor/request shaping, disk typing,
  authentication corrections, Arc collections, invalidation, the plan request,
  all three planning replies, the latest review corrections, and authorization to
  proceed and commit the completed foundation, the artifact timestamp type
  correction, the retention/metadata/digest clarity corrections, and the helper
  ownership, storage-type, and cache-path boundary reviews, followed by the
  request to commit this slice, describe the next phase, and proceed, then the
  fingerprint validation-order and newtype correction, and the request to commit
  the coordinated-cache slice, describe the next phase, proceed, and propagate
  directory-construction errors instead of panicking, then the workspace-wide
  prohibition and cleanup of production `unwrap`/`expect`, and committing and
  pushing the completed changes. U01–U48 preserve
  the distinct instructions; optional source-location ownership, optional
  Vec<Arc<T>>, and a possible cache
  crate remain optional. The explicit-tenant correction supersedes the earlier
  assistant compatibility suggestion. U29 preserves the prohibition on production
  data access or ingestion into agent context; U30 adds workspace Clippy; U31
  requires collision prevention/detection while keeping a REST prefix a candidate.
- **Pass 2 — traceability:** Verified all 47 ledger IDs have matching traceability
  rows and concrete task/gate/acceptance coverage. Reviewed the inverse mapping:
  crate names, aggregate persistence, output bounds, and locking details are
  implementation decisions rather than attributed user decisions. Source review
  added disk variant/input isolation, both UI consumers, staged compilation,
  dependency restart freshness, cross-task cycle handling, effectful-child policy,
  and fail-closed invalidation to the appropriate work items. Review corrections
  now map to G3, tasks 1.2/2.2–2.4, local startup validation in 6.2, workspace
  linting in 7.2, and the acceptance matrix.
  U32 now supersedes U01's initial implementation restriction; phase 1 decisions,
  caller batches, and local verification have concrete linked evidence.
- **Pass 3 — adversarial omission:** Reread the user directions after those repairs
  and repeated extraction/traceability checks. Independent final reviews found no
  remaining material omission. Rechecked explicit Azure CLI tenant and DevOps
  scope, diagnostic-only disk types, both execute spellings, no World object,
  initializer cancellation/waiter retry, shared-handle initial scope with later
  cross-process work, automatic child dependencies, and the later implementation
  authorization, with U33 adding the foundation commit while leaving publication
  and production-data access outside scope.
  Rechecked that auth-enabled tests use disabled authentication and isolated
  storage, namespace separation does not weaken collision checks or scope
  invalidation, and no production read is implied by a test command. Privacy
  review retained relative paths and no real account examples.
  The phase 1 adversarial review also tightened provider-bound CLI execution,
  sensitive credential-output retention, failed-child dependency handling, and
  publication quarantine on future drop; the revised contracts were rechecked.
- **Foundation checkpoint audit:** After tasks 2.0–2.1, all 32 ledger/traceability
  pairs and 22 work items remain present. All 118 relative links across the four
  migration documents resolve; fences and whitespace checks pass. Privacy checks
  cover those documents and both sketch files. No identifying local paths or
  personal-information patterns were found. Existing user notes remain untouched.
- **Artifact-store checkpoint audit:** After task 2.2, all 33 ledger/traceability
  pairs and 122 relative links across the four migration documents pass checks.
  Fences, tracked/new-source whitespace, and identifying-path/name checks pass.
  The storage slice retains U13's diagnostic-only disk types, U22's explicit
  ownership, U27's shared-handle boundary, U29's production-data prohibition, and
  U31's checked identity ownership. Typed reuse, hierarchical invalidation, and adapter
  migration remain explicitly pending; no storage-only evidence claims them.
- **Timestamp type correction:** U34 adds the request for strongly typed expiry.
  Task 2.2 now retains `DateTime<Utc>` in both wire-record timestamp fields;
  a field proxy owns lossless string conversion and rejects malformed persisted
  timestamps. The existing precision and freshness rules remain in force.
  All seven focused freshness tests and 13 artifact-store tests passed after
  this correction, along with cache Clippy and formatting checks; independent
  source review found no remaining issue in the change. User staging is preserved.
- **Storage-type checkpoint audit:** U39 is covered by root-bound directory/file
  handles and the worker-owned I/O guard. All 39 ledger/traceability pairs and
  129 relative links across the four migration documents pass checks. Fences,
  whitespace, and identifying-path/name checks pass. The refinement passed 47
  executed cache tests, cache Clippy with warnings denied, and nightly formatting;
  the symlink test reports a Windows privilege skip. Independent review found no
  remaining invariant regression. User notes and staging remain untouched.
- **Cache-path boundary audit:** U40 preserves the dependency question and the
  preference for a cache-local type. Caller and G1 review found no requirement to
  retain cache-specific encoding in pathing. The implementation retains general
  sanitization there and unchanged physical names. All 40 ledger/traceability
  pairs and 129 relative links pass; fences, whitespace, and privacy checks pass.
- **Typed-coordination checkpoint audit:** U41's storage commit is `821acce9`;
  U42 authorizes task 2.3. The implemented memory-coordination layer retains the
  user-selected ownership, cancellation/retry, and shared-handle boundaries.
  All 42 ledger/traceability pairs and 134 relative links pass, as do fences,
  whitespace, and identifying-path/name checks. Task 2.3 records 71 executed local
  cache tests, the existing symlink privilege skip, Clippy, formatting, and
  independent review. Generation/dependency freshness and production adapter
  cutover remain pending; user notes and staging were preserved.
- **Fingerprint invariant audit:** U43 records the validation-order correction
  and requested `NonEmptyString` type. Extraction, traceability, and adversarial
  checks preserve both typed-work and artifact admission, exact string wire
  compatibility, and transport ownership of fingerprint semantics. All 43
  ledger/traceability pairs and 135 relative links pass, as do fences, whitespace,
  and privacy checks. Constructor and proxy validation tests, the cache suite,
  cache Clippy with warnings denied, and nightly formatting pass. Independent
  review found no invariant bypass; user notes and staging remain untouched.
- **Coordinated-cache commit review:** U44 authorizes a local commit of task 2.3
  and its fingerprint refinement. Extraction maps the request to the checkpoint;
  traceability maps its next-phase description to task 2.4; adversarial review
  retains unrelated notes and keeps next-phase implementation and publication
  outside this checkpoint. Existing focused validation remains applicable because
  this checkpoint adds only plan bookkeeping after the tested source revision.
- **Generation/dependency checkpoint audit:** U45 authorizes task 2.4 after
  `49352932`. Extraction retains the accepted invalidation and dependency
  requirements; traceability maps them to the completed core and local tests;
  adversarial review rechecks nonreuse admission, failure/drop poisoning, leaf
  publication quarantine, finished-token closure, and bound disk proof lifetime.
  All 45 ledger/traceability pairs and 141 relative links pass, together with
  fences, whitespace, and privacy checks. Task 2.4 records 106 executed tests,
  the existing symlink privilege skip, Clippy, formatting, and independent review.
  Auth/executor and transport integration remain subsequent slices; production
  data was not accessed, and unrelated notes and staging were preserved.
- **Workspace error-handling audit:** Extraction adds U46's fallible directory
  boundary and U47's explicit whole-workspace cleanup decision. Traceability
  connects both to the completed task 2.4 refinement and workspace checks in 7.2.
  Adversarial review confirms test exemptions, production constant/example
  enforcement, preserved source errors, and no panic substitutes. All 47 guidance
  pairs and this plan's 60 relative links pass, together with formatting, whitespace, and
  privacy checks. The task notes record the reviewed offline test scope and live
  picker exclusion; production data access remains prohibited. User staging,
  dependency pins, and unrelated notes are preserved.
- **Commit/push checkpoint audit:** Extraction records U48's explicit commit and
  push request. Traceability maps it to this task 2.4 checkpoint; adversarial
  review keeps unrelated notes, package releases, and next-phase implementation
  outside the commit. The reviewed staged changes cover task 2.4 and U46/U47;
  dependency pins are unchanged. Existing validation applies to the unchanged
  source revision; this checkpoint updates plan bookkeeping only. Whitespace
  checking permits existing CRLF endings without normalizing staged files.
- **Known source limitation:** None. The design discussion, corrections, repository
  instructions, plan request, and planning replies are available in this session.

## Authoritative user guidance ledger

The statements below preserve user direction. Recommendations elsewhere are not
automatically confirmed decisions. Add new IDs for changes and mark supersession;
do not silently rewrite earlier guidance.

| ID | Active guidance | Required consequence | Superseded by |
| --- | --- | --- | --- |
| U01 | Review the design and curate a resumable migration plan; the earlier discussion was brainstorming. | Initial delivery was planning-only; maintain the artifact during authorized execution. | U32 supersedes the initial implementation restriction |
| U02 | Independent audit activities should live in separate files and fetch their own dependencies without a shared data-filled `World` parameter. | Separate audit logic and share execution resources, not a universal audit input bag. | — |
| U03 | Duplicate requests should reuse memory results without repeating network work or deserialization. | Prove concurrent single initialization and reuse of decoded results. | — |
| U04 | A short registry/map lock followed by a mutex for the corresponding key makes sense. | Stable per-entry coordination; never hold the entry-map guard through execution. | — |
| U05 | Command and REST should share their cache/execution lifecycle in one place. | One reusable lifecycle with execution-specific adapters. | — |
| U06 | Replace free-form executor kind strings with a stronger enum; include in-process work. | Model Command, Rest, InProcess, and extensible Other metadata. | — |
| U07 | Awaitable cached work should use `IntoFuture` or an equivalent bound execution object. | Bind runtime resources before awaiting; preserve serializable request data. | — |
| U08 | Remove auth runtime state and CancellationToken/CancellationScope from request objects while retaining source and tenant information. | Requests describe work; executor supplies bounded runtime state. | — |
| U09 | A cheaply cloneable Cloud Terrastodon executor should own authentication resources, cancellation access, and the work cache explicitly. | Invocation-owned resources, shared handles, correctly bound child scopes. | — |
| U10 | Support both `executor.execute(request)` and `request.execute(executor)`. | Both spellings dispatch through one implementation. | — |
| U11 | Operation implementations should receive `self` and their executor; nested operations use it too. | Replace request-owned runtime access throughout the call chain. | — |
| U12 | Fetch constructors take tenant ID/auth source; Graph helper construction takes URL/key/source/tenant and fetching receives executor. | Explicit operation data through constructor and pagination boundaries. | — |
| U13 | Type ownership is an in-memory concern; disk type information may remain diagnostic rather than controlling reuse. | Decode raw disk artifacts with the current canonical decoder; no persisted TypeId requirement. | — |
| U14 | Detect conflicting execution/decoding owners for one memory key. File/line/column was suggested as an option, not settled as the mechanism. | Close operation-ownership gate; retain source location diagnostics without claiming it proves type compatibility. | — |
| U15 | Share decoded results with Arc to avoid repeated owned copies and cache-bound borrow lifetimes. | Shared owned snapshots with no deserialization on warm memory hits. | — |
| U16 | `Vec<Arc<T>>` can make sense for Azure DevOps collections and page assembly. | Evaluate shared-record boundaries; do not mandate blanket Arc replacement in every model. | — |
| U17 | Azure DevOps auth selection should be separate from runtime context; bearer selection carries source/tenant, PAT resolves through executor. | Typed serializable selection distinct from credential/session/token ownership. | — |
| U18 | Azure CLI credentials must have an explicit tenant in the request; do not depend on CLI default auth/tenant/DevOps setup. | Remove active-tenant compatibility from executable paths; preserve explicit organization/project scope. | — |
| U19 | Invalidation must affect memory and disk, including descendants of hierarchical keys and work in flight. | Prefix-aware freshness and coordinated publication/invalidation. | — |
| U20 | A generation or similar mechanism should allow invalidation without deadlocking active work. | Older fills cannot republish reusable results after invalidation. | — |
| U21 | A cache crate may be introduced if necessary to avoid dependency cycles. | Evaluate a cohesive low-level cache boundary; verify the full dependency graph. | — |
| U22 | Avoid implicit global caches and hidden dependencies. | Pass shared resources explicitly; no implicit executor, task-local default, or fallback singleton. | — |
| U23 | Follow the supplied repository conventions, including cancellation, type ownership, tests, cache scopes, CLI boundaries, and preserving edits. | `AGENTS.md` remains authoritative throughout the migration. | — |
| U24 | Use the named resumable implementation plans skill and save the plan in `docs/plans`. | Maintain this living Markdown artifact and its intent audit. | — |
| U25 | The plan must not contain the user's username or other PII. | Relative paths, synthetic examples, and privacy checks before handoff and updates. | — |
| U26 | If the initiating caller of a shared read is cancelled, cancel that attempt; a waiter may retry. | Caller-owned initialization; no detached executor-owned fetch task. | — |
| U27 | Initially coordinate executors sharing one cache handle; cross-process coordination is a later phase. | State the guarantee precisely and retain a deferred phase for other processes. | — |
| U28 | Record child-request dependencies during execution so invalidating underlying data invalidates composed results. | Track dependency generations across separate key branches, including warm child hits. | — |
| U29 | An `--auth-source none` run may validate startup; do not access production data or bring it into agent context. | Use local-only execution, synthetic fixtures, and isolated storage; neither services nor existing production caches are validation inputs. | — |
| U30 | Add workspace coverage to the final Clippy check. | Include `--workspace` so member targets are linted. | — |
| U31 | Prevent cache-key collisions or detect and bail; do not rely on callers never reusing a key. A REST prefix may be the simplest approach. | Enforce namespace/owner contracts centrally, test collisions, and preserve logical-scope invalidation across namespaces; prefix spelling remains a design choice. | — |
| U32 | Proceed after confirming readiness to begin with phase 1. | Execute the migration in dependency order, starting with concrete contract closure and caller inventory; retain the production-data prohibition. | — |
| U33 | Commit the completed work and proceed. | Commit the reviewed foundation slice without unrelated user notes, then implement task 2.2 with local synthetic validation. | — |
| U34 | Use a strong type for artifact expiry instead of an interchangeable string. | Keep timestamps typed and isolate exact wire formatting and validation at the serialization boundary. | — |
| U35 | Clarify the behavior represented by `ArtifactRetention::Sensitive`. | Name and document the actual storage policy, including its scope and effects on existing artifacts. | — |
| U36 | Make operation/owner naming consistent and construct type-name metadata through `new<T>`. | Identify the canonical operation owner explicitly and centralize diagnostic type-name capture in the metadata type. | — |
| U37 | Give `ArtifactFileDigest` a constructor that owns its digest definition and include a `blake3:` prefix. | Centralize digest construction and verification; persist an explicit algorithm tag. | — |
| U38 | Inline `entry_directory` into its owning entry method and review whether other free functions should move. | Keep type-specific behavior with its existing owner; retain shared functions when they centralize meaningful common behavior. | — |
| U39 | Review whether remaining free functions reveal missing domain types and names, with associated functions on newtypes as a possible solution. | Introduce types where they own path confinement, file persistence, or lock lifetime; avoid empty utility namespaces or a blanket ban on free functions. | — |
| U40 | Review why the cache path encoder lives in pathing; prefer a cache-local `CachePathComponent` owning its guarantees unless planned dependencies require otherwise. | Verify callers and dependency direction; keep general sanitization in pathing and cache-specific physical encoding with the cache storage types. | — |
| U41 | Commit the current changes and describe the next phase. | Commit the completed artifact-storage slice and its refinements, preserve unrelated notes, and explain task 2.3 with task 2.4 as the following dependency. | — |
| U42 | Proceed after the artifact-storage commit and description of coordinated initialization. | Implement task 2.3 and its local concurrency/cancellation proofs; keep generation/dependency invalidation and adapter migration in their subsequent tasks. | — |
| U43 | Validate the fingerprint before claiming cache ownership; use a `NonEmptyString` newtype to enforce the nonempty invariant. | Require validated fingerprints for both typed work and raw artifacts, including deserialization, so invalid input cannot reserve an operation/output claim. | — |
| U44 | Commit the coordinated-cache changes and describe the next phase. | Commit task 2.3 with its fingerprint refinement and local validation, preserve unrelated notes, and explain task 2.4's generations and composed dependencies. | — |
| U45 | Proceed after the coordinated-cache commit and next-phase description. | Implement task 2.4 with synthetic invalidation, dependency, cancellation, and disk-publication race proofs before the later auth/executor and transport slices. | — |
| U46 | Remove the `expect` in `ArtifactEntry::directory`; return `eyre::Result` if necessary. | Return directory-construction failures and propagate them through callers without a panic. | — |
| U47 | Forbid `expect` and `unwrap` outside tests; clean up existing violations across the entire workspace now. | Add workspace Clippy enforcement with test-only exemptions, propagate genuine failures, and remove production violations without substituting panics or silently discarding errors. | — |
| U48 | Commit changes and push. | Commit the completed task 2.4 and workspace error-handling refinement, then push the current branch and its preceding local checkpoints to the existing remote; preserve unrelated notes. | — |

U18 supersedes the earlier assistant suggestion to preserve an Azure CLI
active-tenant selection. That suggestion was not user approval. U13 rejects the
earlier recommendation to require a persisted type/schema identity as part of
cache reuse. U14 and U16 deliberately retain their original optional qualifiers.

## Guidance traceability

| Guidance | Plan coverage | Required evidence |
| --- | --- | --- |
| U01 | 0.1; scope | Documentation-only diff; no implementation or live calls |
| U02 | 5.1, 5.2, 6.1 | Independent audit modules invoke shared executor; no World bag |
| U03 | 2.3, 2.4, 5.1–5.3 | Execution/decode counters and shared pointer identity |
| U04 | 2.3, 2.4 | Concurrent same-key/different-key and invalidation tests |
| U05 | 2.1, 4.1, 4.2, 7.1 | Both adapters use one lifecycle; old paths removed |
| U06 | 2.1 | Typed provenance metadata with compatible artifact diagnostics |
| U07 | G1, G2, 3.2 | Bound awaitable execution; no runtime fields in request wire data |
| U08 | 3.1, 3.2, 5.1–6.3, 7.1 | Inventoried request schemas and execution signatures migrated |
| U09 | 2.1, 3.2, 6.2 | Shared resource handles and linked child cancellation behavior |
| U10 | 3.2, 3.3 | Equivalent direct and reflected invocation semantics |
| U11 | 3.2, 5.1–6.3 | Nested requests receive scoped executor |
| U12 | 3.1, 5.1 | Explicit constructor data and pagination execution boundary |
| U13 | G3, 2.2, 2.3 | Disk decode is authoritative; type metadata diagnostic only |
| U14 | G3, 2.3, 3.3 | Owner/type collision rejected before wrong-value reuse |
| U15 | G4, 2.3, 5.1–5.3 | Warm results share allocation and outlive eviction safely |
| U16 | G4, 5.3 | Shared-record assembly and unchanged external output representation |
| U17 | 3.1, 5.3 | Source/tenant selection serialized without secrets/runtime context |
| U18 | 3.1, 5.3, 6.2, 7.1 | Explicit tenant command construction and missing-tenant rejection |
| U19 | 2.2, 2.4, 4.1, 4.2 | Memory/disk prefix invalidation and publication race proofs |
| U20 | 2.4, 5.1 | Nested/self-invalidation completes without stale publication |
| U21 | G1, 2.1, 7.2 | Acyclic normal/build dependency graph |
| U22 | 2.1, 3.2, 6.2, 7.1 | No hidden execution cache/auth fallback remains in migrated paths |
| U23 | Constraints; all tasks | Scoped review, approved test inventory, preserved pins/staging |
| U24 | 0.1; update protocol | Skill requirements, task contracts, and three-pass audit |
| U25 | 0.1, 7.2 | PII scan and manual artifact review |
| U26 | 2.3, 3.2 | Initializer cancellation, surviving waiter retry, root cancellation tests |
| U27 | G5, acceptance matrix, phase 8 | Shared-handle guarantee; no unproven cross-process claim |
| U28 | 2.4, 5.1, 5.2 | Sibling/transitive dependency invalidation and race tests |
| U29 | Validation policy; 6.2; acceptance matrix | Auth-feature startup with `none`, isolated synthetic storage, no production data access or ingestion |
| U30 | 7.2 | Workspace-wide Clippy command and recorded result when run |
| U31 | G3, 1.2, 2.2–2.4, 4.2 | Wrapper/transport separation, same-namespace owner/type rejection, and cross-namespace invalidation |
| U32 | Current authorization; 1.1, 1.2; execution order | Concrete contracts, caller batches, and local verification evidence before downstream migration |
| U33 | Current authorization; 2.0–2.2 | Reviewed foundation commit, preserved user notes, and artifact-store implementation evidence |
| U34 | 2.2 | Typed timestamp fields, exact nanosecond roundtrips, and malformed timestamp rejection before record construction |
| U35 | 2.2 | Explicit disabled-storage policy; synthetic tests for no staging/reuse/publication and preservation of existing artifacts |
| U36 | 2.2 | Metadata constructor and consistent operation-owner names; both persisted type names remain diagnostic-only |
| U37 | 2.2 | Digest constructors and tagged wire validation; altered bytes and missing/unknown algorithms rejected |
| U38 | 2.2 | Entry/path/selection behavior beside its owning types; shared filesystem and gate contracts preserved and verified |
| U39 | 2.2 | Root-bound directory/file handles and worker-owned I/O guard; per-operation ancestor checks and dropped-awaiter lock regression |
| U40 | G1, 2.2 | Cache-local validated encoding with unchanged physical names; existing cache-to-pathing edge retained for general sanitization |
| U41 | Artifact checkpoint; 2.2, 2.3, 2.4 | Artifact-storage commit `821acce9` and next-step description; later progress is tracked under U42 |
| U42 | 2.3 | Stable initializer ownership, shared decoded values, explicit ancestry, cancellation/retry and cycle tests |
| U43 | 2.2, 2.3 | Both entry points require `NonEmptyString`; constructor/proxy rejection and exact string preservation tests |
| U44 | Coordinated-cache checkpoint; 2.3, 2.4 | Local commit `49352932` and the description of generation/dependency invalidation |
| U45 | 2.4 | Complete generation/dependency admission and synthetic race evidence; no production adapter cutover |
| U46 | 2.4 | Fallible directory API and error propagation through staging, selection, and existing tests |
| U47 | 2.4 refinement; 7.2 | Repository rule, all member lint configurations, workspace Clippy, and focused offline regression checks |
| U48 | Current authorization; task 2.4 checkpoint | Reviewed staged scope, local commit, and confirmed remote branch update |

## Purpose and scope

Allow independently implemented audits to request the same inventory through an
explicit executor and share one decoded result. Request values remain useful for
text inspection and offline construction. Commands, HTTP requests, and in-process
work use one execution/cache lifecycle while preserving their distinct effects,
failure artifacts, cancellation requirements, and authentication rules.

The initial implementation includes the cache/execution foundation, explicit auth
selection, registry integration, Graph principals, ARM RBAC, Azure DevOps projects,
audit modularization, and the workspace callers required to complete the API
migration. Remaining request families are inventoried and migrated in bounded
batches, rather than leaving permanent hidden-runtime compatibility paths.

Out of scope: new audit policies, replacing every CLI-backed service operation
with REST, changing remote API endpoints, rebuilding authentication protocols,
unrelated HCL reflow behavior, dependency/toolchain upgrades, production data
access, and publishing a release. Cross-process cache coordination is deferred to
phase 8. This plan does not authorize implementation or modification of adjacent
repositories.

## Verified foundation and source references

These are source-inspection findings at the recorded baseline, not test results.

| Area | Existing evidence | Migration implication |
| --- | --- | --- |
| Shared work | [cacheable_work.rs](../../crates/command/src/cacheable_work.rs) owns CachedWorkSpec, raw execution, decoding, artifact retention, and post-decode publication. | Reuse its responsibility while splitting production types into their own files. |
| Memory/disk cache | [artifact_cache.rs](../../crates/command/src/artifact_cache.rs) has a global map of serialized CommandOutput; memory keys include fingerprint, disk directories do not. | Decoded sharing, explicit ownership, and physical write isolation are all needed. |
| Validity | Memory hits precede timestamp checks. Writes append timestamps; validity reads the first line while discovery reads the last. | Give memory and disk consistent expiry/refill semantics; cover with owned-policy tests. |
| Invalidation | [cache_key.rs](../../crates/command/src/cache_key.rs) scans existing descendant context files and clears global memory. | Add generations, in-flight protection, and an explicit cache owner. |
| Commands | [command.rs](../../crates/command/src/command.rs), [process_fingerprint.rs](../../crates/command/src/process_fingerprint.rs), [command_kind.rs](../../crates/command/src/command_kind.rs). | Retries reenter the cache boundary; raw success is published before typed decode; adjacent files are written into the cache directory before execution. |
| REST | [rest_request.rs](../../crates/rest/src/rest_request.rs) uses shared cached work but has a separate uncached branch; arbitrary validators/mappers run for callers. | Preserve status/header/body diagnostics, tracked caller/span, and per-caller callbacks. |
| SDK trait | [cachable_command.rs](../../crates/command/src/cachable_command.rs) supplies key/run and IntoFuture macros; it does not cache assembled SDK outputs. | Aggregate memoization is new behavior, not merely moving the current cache. |
| Auth runtime | [auth_context.rs](../../crates/credentials/src/auth_context.rs) already owns invocation-local reference-counted token caches. | Preserve source policy, headless behavior, and cache isolation while moving ownership. |
| Tenant binding | [azure_tenant_auth_context.rs](../../crates/credentials/src/azure_tenant_auth_context.rs) validates resolved enabled sources and workload-identity tenant agreement. | Keep these invariants in the new auth-selection/execution boundary. |
| CLI fallback | [azure_devops_auth_context.rs](../../crates/credentials/src/azure_devops_auth_context.rs), [azure_access_token.rs](../../crates/credentials/src/azure_access_token.rs), [tracked_tenants.rs](../../crates/azure/src/tracked_tenants.rs). | Active-tenant compatibility and optional CLI token tenant must be removed from authenticated request execution. |
| Graph | [microsoft_graph.rs](../../crates/azure/src/microsoft_graph.rs), [entra_user_list_request.rs](../../crates/azure/src/entra_user_list_request.rs), [principals.rs](../../crates/azure/src/principals.rs). | Runtime fields/progress hooks move out; continuation recovery invalidates its own subtree; records currently move into aggregates. |
| RBAC | [resource_graph.rs](../../crates/azure/src/resource_graph.rs), role_definitions.rs, role_assignments.rs, role_definitions_and_assignments.rs in the Azure SDK. | Paginated read-only POST, combined validation, and separate child keys exercise shared results and dependencies. |
| Shared domain values | [entra_principal.rs](../../crates/azure_types/src/entra_principal.rs), [entra_principal_collection.rs](../../crates/azure_types/src/entra_principal_collection.rs), [role_definitions_and_assignments.rs](../../crates/azure_types/src/role_definitions_and_assignments.rs). | Box/owned conversions, mutable collection access, and cross-record invariants constrain Arc placement. |
| Azure DevOps slice | [azure_devops_projects.rs](../../crates/azure_devops/src/azure_devops_projects.rs) and [project CLI](../../crates/azure_devops_cli/src/project/list/azure_devops_project_list_cli.rs). | Explicit org, optional tenant compatibility, page metadata, owned results, and late credential acquisition all need deliberate migration. |
| Registry | [registry lib](../../crates/registry/src/lib.rs), bind_request_cancellation.rs, function_trait_contract.rs, default_production.rs, discovery_reflection.rs. | Invocation currently awaits request IntoFuture and reflects a scope field; discovery must stay offline and effects-aware. |
| Consumers | [app](../../crates/app/src/lib.rs), [audit_azure.rs](../../crates/azure_cli/src/audit_azure.rs), [object explorer invocation](../../crates/ui_ratatui/src/object_explorer/invocation_controller.rs), [egui app](../../crates/ui_egui/src/app.rs), entrypoint dump planning, examples. | Explicit resources must reach CLI, both UI implementations and their workers, discovery execution, and reusable applications. |

Repository policy: [AGENTS.md](../../AGENTS.md). Application boundaries and
cancellation behavior: [REUSABLE_APPLICATIONS.md](../REUSABLE_APPLICATIONS.md).
The [secretless authentication plan](secretless-authentication.md) records earlier
work and pending live validation, but current source is authoritative for current
behavior. Its old active-tenant compatibility is superseded by U18; its browser,
workload-identity, PAT, and headless policies remain foundation. Do not reopen its
external validation work or copy its historical machine-specific paths.

The workspace pins dependencies in `Cargo.toml`/`Cargo.lock`; no rust-toolchain
file was found. `rustfmt.toml` uses nightly import-formatting options.
`check-all.ps1` performs mutating formatting, and `test.ps1` runs unsafe broad
service tests. No `.github/workflows` directory was found. `publish.ps1` computes
normal/build dependency order and rejects cycles; `set_version.ps1`,
`CHANGELOG.md`, and [UPDATING_FACET.md](../UPDATING_FACET.md) govern release preparation.

## Confirmed design and constraints

- Serializable requests carry domain scope, resolved authentication selection,
  filters, and limits. They carry no token cache, execution scope, or callback.
  Ordinary headers/body may still require redaction; serializable does not mean
  safe to log secrets. Preserve existing diagnostic redaction responsibilities.
- The executor is a resource coordinator, not a data-filled World or a feature
  client forwarding organization/project fields. Clones share resources; a child
  execution uses its callback's child CancellationScope. Clones never extend a
  scope or deferral beyond its owner. Libraries do not manufacture root scopes.
- Both execute spellings reach the same admission/cache/dispatch procedure.
  Request implementations retain their visible URL construction, pagination,
  interpretation, and affected-read invalidation procedures.
- Azure CLI bearer acquisition always receives the request's explicit tenant.
  PAT remains tenant-independent. Resolve permitted CLI selectors to request
  values before execution; never obtain missing scope from Azure CLI defaults.
  An unresolved/disabled auth source cannot execute an authenticated request.
- Initializer cancellation follows U26. Cancelling one waiter affects its wait;
  another admitted waiter may retry after a cancelled initializer releases
  ownership. Root cancellation does not authorize a waiter to start new work.
  Existing deferrals and required artifact completion still apply after an
  outcome exists; cancellation is not permission to discard required cleanup.
- Only successful canonical read results are reusable. Arbitrary caller
  validation/projection stays outside memoization, even when callbacks return
  the same Rust type. Preserve borrowed/non-Send command validator support or
  explicitly close its public compatibility gate before changing it.
- Mutation/zero-duration work executes every time, retains inspectable artifacts,
  and invalidates affected read requests through the executor after success.
  Effect metadata and cancellation policy are distinct from ExecutorKind.
- Memory values are shared owned snapshots. Previously returned Arcs remain
  valid after invalidation. In-flight old-generation work may finish for an
  already admitted caller, but cannot republish reusable memory or disk output.
- Cache namespaces must prevent SDK/transport aliasing; conflicting owners or
  output contracts within a namespace and logical operation key must fail before
  reuse or lock waiting. A namespace alone does not replace these checks.
  Logical-scope invalidation covers all executor namespaces and variants beneath
  that scope.
- Dependency recording includes child hits and misses, transitive composed
  reads, and non-descendant keys. A parent result cannot appear fresh when its
  inputs were invalidated during assembly. No global generation bump that
  unnecessarily invalidates unrelated resource scopes substitutes for this.
- Preserve filesystem-safe readable scope keys, scoped invalidation, strict
  domain validation, debug unknown-field checks, nonempty registry effects,
  cancellation contracts, CLI output/boxing policy, and one production type per
  file. Required local manifest/lockfile changes for new workspace crates must
  not upgrade unrelated dependency versions.

## Contract decisions and implementation gates

G1–G6 are closed for initial implementation by
[the public contracts](executor-cache-contracts.md) and
[the coordination algorithm](executor-cache-coordination.md). The table retains
the questions and original starting points for traceability; the linked decision
documents now supply their outcomes. Closed design gates do not mean the
corresponding runtime behavior has been implemented or tested.

Record any revision to a closed gate and its evidence in task 1.1 before dependent
implementation. Routine implementation choices can be resolved there;
ask the user only when changing confirmed intent or settling material public
behavior not already authorized. Do not interpret this gate table as new approval
requirements for reversible research or preparation.

| Gate | Decision to close | Recommended starting point; acceptance consequence |
| --- | --- | --- |
| G1 | Exact acyclic crate/trait boundary for command, credentials, registry, and the concrete executor. | A low-level cache/execution contract without credential implementations; concrete auth coordinator above it. Both command→executor→credentials→command and registry→executor→credentials→registry cycles must be absent. Cache extraction alone does not prove this. |
| G2 | Public method/trait names, lifetimes, cacheable versus noncacheable output bounds, and temporary API compatibility. | Data requests bind to an awaitable execution. Sharing Arc<T> across tasks adds Sync to today's Send + 'static output contract; decide affected bounds without imposing them on unrelated noncacheable outputs or borrowed callbacks. Explicit adapters may ease rollout; no default executor or indefinitely retained runtime-bearing request path. Record breaking Rust API changes and removal milestones. |
| G3 | Concrete entry namespace, logical operation owner, typed result contract, auth partition, and physical artifact identity. | Enforce U31 centrally: recommend ExecutorKind namespaces plus logical keys, with checked operation/output ownership before request/auth variant lookup. Separate SDK results from transport envelopes even when their request keys match; retain logical-scope invalidation across namespaces. Choose the exact address/layout and test the LocationListRequest case. Partition credentials/accounts without tokens in keys or live token acquisition solely for a cache hit. Reject unsafe legacy reuse if an auth partition cannot be established. TypeId is memory-only; source locations and disk type names are diagnostic. |
| G4 | Shared-record boundary and observer behavior. | Evaluate Vec<Arc<T>> plus an outer Arc for reusable aggregates; preserve Facet/Arbitrary/registry/CLI representations. Decide consuming conversions, mutable access, and progress reporting to hits/joiners without silently skipping caller behavior. |
| G5 | Disk migration/publication strategy within the shared-handle guarantee. | Isolate fingerprint variants and command input staging; publish complete artifacts under cache coordination. Decide legacy entry read/miss behavior, write-failure memory policy, retention, expiry, and entry reclamation. Document independent-handle/process limitations. |
| G6 | Concrete hierarchical/dependency generation algorithm, aggregate persistence, and local error behavior. | Stable initializer mutexes; short cache I/O/publication barrier; dependency stamps recorded through execution. Define invalidation linearization/failure, transitive stamps, self-invalidation retry, cross-task cycle rejection, and lock order. Prefer memory-only aggregate reuse initially; durable aggregate reuse would require authoritative dependency provenance on restart, separately from diagnostic type metadata. Define semantic dependencies separately from credential infrastructure, and prevent parent hits from bypassing effectful/noncacheable child work. Scope invalidation precisely. |

A call-site owner rule is not selected: two audits legitimately call the same
operation from different locations, while one generic location can produce
different outputs. G3 selects a checked in-memory ownership contract.
No persisted TypeId or mandatory disk schema discriminator is part of this plan.

For U31, G3 selects an ownership claim for `(ExecutorKind, logical operation key)`,
with semantic request variants and auth partitions below it. The cache layer
supplies the namespace consistently to memory lookup, initializer locks, and disk
artifacts; callers do not manually prepend `rest/`. A physical REST prefix is one
possible layout, not the logical invalidation scope. A memoized SDK wrapper can
occupy InProcess while its HTTP envelope occupies Rest. Other(String) tags must
not alias built-in kinds or each other after filesystem encoding.

Before variant lookup, disk loading, execution, or waiting, claim/check the
canonical operation/decoder owner and output contract. Mismatches return a
diagnostic error, including different owners with matching Rust types or different
fingerprints; do not treat the error as a cache miss. A generic adapter name such
as RestRequest cannot alone identify the operation owner. Do not add the expected
owner, output TypeId, or call-site location to the address merely to turn a
collision into a separate successful entry. Recursive initialization of the same
concrete variant entry also fails instead of waiting on itself; nested operations
using distinct entries remain valid. Tasks 2.1–2.2 implement G3's identity and
physical layout; these checks and invalidation behavior are mandatory. Disk type diagnostics remain
non-authoritative, and distinct namespace/variant artifacts cannot share a path.

A cache-wide RwLock for short disk-read/publication versus invalidation sections
was an initial option. G5/G6 select a single async Mutex I/O gate initially;
an RwLock remains a possible measured optimization with the same contract.
Do not hold an ancestor read guard while executing HTTP/process work, decoding,
or awaiting child requests. A queued writer can otherwise block the child that
the ancestor is awaiting. See [Tokio RwLock read ordering](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html#method.read).
[Rust caller locations](https://doc.rust-lang.org/std/panic/struct.Location.html#method.caller)
describe diagnostics, not type identity.

## Execution order and validation policy

`1 contracts/inventory → 2 value registry/cache core → 3 auth/executor/registry → 4 adapters →
5 SDK slices → 6 audit/app/consumers → 7 cutover/acceptance`; phase 8 is deferred.
Phase 2 can use synthetic in-process work until the public executor exists.
Do not expose a partially initialized global runtime to bridge stages.

Keep each slice compilable: introduce the new contracts alongside explicitly
bound adapters before switching an existing caller family. Tasks 3.1–3.3 add the
new auth/registry surfaces first; tasks 4–6 move consumers in compile-checked
batches. Retire old signatures and fields only after their callers move, with
final removal in 7.1. Compatibility must not invent missing tenants, resurrect
global caches, or mix two memory-cache owners in a single migrated request tree.
Introduce the minimal application executor plumbing needed for the first SDK
slice in 3.2; task 6.2 completes the wider application/consumer rollout.

All commands below run from repository root in PowerShell unless stated otherwise.
Commands are planned validation, not evidence that they have run. Targets marked
**proposed** do not exist at the baseline and must be introduced by their task;
update commands when G1/G2 settle names. Reinspect test code before execution.
Use synchronization barriers/counters for race proofs, not timing-only sleeps.

`--offline` prevents dependency downloads, not network access from tests. Do not
run `test.ps1`, unfiltered workspace tests, ignored tests, or live SDK tests.
Do not resolve real credentials/configuration or use production inventories to
validate this migration. Do not read production artifacts from existing caches,
logs, or fixtures, or bring production data into agent context. Use isolated empty
cache/configuration locations and synthetic records for runtime validation.
`--auth-source none` disables credential discovery but is not a network sandbox
or protection against reading an existing disk cache: select a reviewed local-only
handler and isolated storage as well. Local HTTP/process fixtures can exercise
execution policy, not establish remote API compatibility. Reuse existing injection/local
test boundaries; do not add feature-specific fake transports to SDK operations.

Existing local-only targets inspected: command library tests,
`cacheable_work_tests`, `command_cancellation_tests` (requires local `pwsh`), REST
library tests, registry tests, synthetic Azure DevOps discovery plan/simulation
targets, selected credential modules, and app process test source (auth-feature
execution requires the fixture update in 6.2). Broad command
integration tests include live Azure calls. Existing Graph/principal/RBAC/project
SDK tests include live requests. A compile check is safe but is not a runtime test.

## Phase 0 — plan preparation

### [x] 0.1 Review the design and preserve its requirements

**Completion notes:** Inspected the user-invoked skill, repository instructions,
source entry points, manifests, existing plans, test code, application docs, and
check/release scripts at baseline `26e14bcc`. Independent read-only reviews covered
cache/adapter behavior and auth/registry/consumer integration. User replies settled
initializer cancellation, shared-handle scope, and dependency recording. No
production code, tests, credentials, or services were executed for validation.
Final document checks and three-pass intent audit are recorded above after review.
The 2026-10-30 plan revision records the user's production-data boundary, workspace
Clippy correction, and mandatory cache collision checks. Source inspection found
that the app process fixture currently selects AzureCli; task 6.2 must change it
to None before auth-feature execution. LocationListRequest currently passes its
SDK cache key to REST, providing a concrete migration case without running it.
That documentation-only revision found 31 ledger IDs and 31 matching traceability IDs, 21 work
items with work/validation/completion contracts, balanced code fences, and no
broken relative links or personal-information matches. Manual review and a
new-file whitespace check also passed; no Rust build or test was run for this
documentation-only change.

**Work:** Create this artifact; preserve confirmed decisions, alternatives, and
unresolved gates without claiming implementation authorization.

**Validation:** Review the final diff, requirement mappings, relative file links,
task contracts, and privacy scan; run `git diff --check`.

**Completion criteria:** A fresh agent can resume planning from this file, identify
the first implementation gate, and distinguish inspected facts from unrun checks.

## Phase 1 — contract closure and migration inventory

### [x] 1.1 Close the public contracts and dependency graph

**Completion notes:** Closed G1–G4 in
[executor-cache-contracts.md](executor-cache-contracts.md) and G5/G6 in
[executor-cache-coordination.md](executor-cache-coordination.md). Selected low
registry_values/auth_types/cache/execution boundaries, an injected typed auth
provider, bound request/registry lifetimes, central namespace/owner checks,
memory-only aggregates, and a mutex-gated publication/invalidation algorithm.
AzureTenantId moves to the lower auth vocabulary with old import paths re-exported.
CLI/browser/PAT use session partitions until identity can be established safely;
authenticated CLI subprocesses must use that same provider-bound session.

The proposed normal/build graph was checked against offline Cargo metadata with
the selected new edges: 36 packages, no cycles. This is architectural validation,
not a claim that the new packages already compile. The baseline command,
credentials, registry, and REST check passed. The standalone
[contract sketch](sketches/executor_contract.rs) compiled against Cargo's exact
locked artifacts; four checks passed using
`./docs/plans/sketches/check-executor-contract.ps1`. It proves both execute
spellings with borrowed requests/results, child scope closure and shared resource
ownership, erased invocation with a non-Sync output, borrowed non-Send caller
projection, and disabled-auth admission. It is not a production executor/cache.

The sketch exposed that ordinary linked children share cancellation intent;
the contracts now distinguish independently cancelled callers from root
cancellation. Independent review confirmed the added safeguards for authenticated
CLI processes, sensitive credential outcomes, failed semantic children, and
publication future drop. Provider provisioning, cache races, actual registry
integration, and production adapter behavior remain future implementation proofs.
No production data, services, credentials, or existing application caches were read.

**Work:** Resolve G1–G6 with small offline contract sketches. Specify which layer
owns the executable trait, registry callback ABI, runtime credential capability,
bound IntoFuture value, cache result type, and cancellation scope. Resolve the
canonical decoder versus caller projection split. Record package DAG, public API
signatures, compatibility policy, and auth partition without runtime secrets.
Confirm disabled/local-only operations and `app` without auth remain supported.

**Validation:** Inspect manifests with `cargo metadata --offline --locked --no-deps
--format-version 1`; retain only relative, nonpersonal findings. Run:

```powershell
cargo check --offline --locked -p cloud_terrastodon_command -p cloud_terrastodon_credentials -p cloud_terrastodon_registry -p cloud_terrastodon_rest
```

**Completion criteria:** Every gate has a recorded outcome and a feasible acyclic
graph; no required public behavior is left to an implicit global fallback.

### [x] 1.2 Inventory callers and establish a safe baseline

**Completion notes:** [The caller inventory](executor-cache-migration-inventory.md)
records 24 bounded migration batches and reproducible lexical candidate counts,
including both UIs, registry/default production, raw/mapped execution, custom
invalidation, same-key wrappers, local exports, and legitimate exclusions. Source
review identified tenant-free security-group keys and CommandBuilder::bust_cache
as explicit migration cases. Counts describe candidate files, not semantic
request totals.

Reviewed local baselines passed: `cargo test --offline --locked -p
cloud_terrastodon_command --test cacheable_work_tests` (7 tests) and `cargo test
--offline --locked -p cloud_terrastodon_rest --lib` (36 tests). The former uses
unique absolute package-target directories; the latter uses synthetic values
and disabled-auth or stubbed resolver paths, without service access. No baseline
failure remains. Broad/live SDK suites and app auth-feature runs were not run.

**Work:** Classify runtime-bearing request schemas, fetch constructors, adapters,
invalidation APIs, raw receive/command consumers, registry invocations, UI callers,
CLI dispatchers, and examples. Record migration batches and owners here. Identify
direct cancellation-only requests that also need executor binding without
artificial auth requirements. Preserve existing cache inspection/clean behavior.
Inventory wrappers that pass `self.cache_key()` unchanged to a child Command or
RestRequest, starting with `crates/azure/src/locations.rs`: the SDK output is
Vec<AzureLocation>, while REST decodes a Response envelope. Record the namespaces
and canonical ownership at both levels; cover same-kind nested wrappers too.

```powershell
rg -n 'CacheableCommand|CacheableWorkRequest|impl_cacheable_into_future|impl_cacheable_work_into_future|register_into_future' crates examples
rg -n 'AuthContext|CancellationScope|MEMORY_CACHE|with_invalidation|invoke_value_boxed' crates examples
cargo test --offline --locked -p cloud_terrastodon_command --test cacheable_work_tests
cargo test --offline --locked -p cloud_terrastodon_rest --lib
```

**Completion criteria:** Each relevant surface has a batch or justified exclusion;
baseline failures are recorded separately from changes introduced by migration.

## Phase 2 — explicit cache owner and coordinated typed results

### [x] 2.0 Extract the pure registry value boundary

**Completion notes:** Added `crates/registry_values` with pure reflection, owning
values, pointer registration, and arbitrary constructor descriptors. The high-level
registry re-exports existing value imports and adapts pure constructors once while
retaining descriptor provenance. Production types have individual owning files.
An independent source review confirmed the extracted RuntimeValue implementation
is equivalent and that workspace catalog traversal includes both sources.

The registry library suite passed all 53 tests, including existing pointer/handle
identity tests and the new pure-catalog integration test. The latter verifies
discovery without invoking a generator, one stable adapter per descriptor, and
generation through the existing boxed invocation API. Consumer checks passed:

```powershell
cargo check --offline --locked -p cloud_terrastodon_registry_values -p cloud_terrastodon_command -p cloud_terrastodon_credentials -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_entrypoint --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_registry_values -p cloud_terrastodon_registry --all-targets -- -D warnings
```

Targeted nightly formatting and `git diff --check` passed. This task's lockfile
changes added only the local registry_values package and the registry dependency edge; no
external dependencies changed. No provider or SDK execution procedure changed,
and no services, credentials, or production data were accessed.
The two moved RuntimeValue compile-fail ownership examples also passed with
`cargo test --offline --locked -p cloud_terrastodon_registry_values --doc` during
the following foundation slice.

The merged catalog requires a public API adjustment: `known_functions()` returns
an iterable `KnownFunctions` view rather than a contiguous `&[Function]`. Existing
associated function handles keep their original addresses; pure descriptors have
one persistent adapter each. Workspace `.iter()` and `IntoIterator` consumers
remain compatible. External callers with explicit slice requirements must adapt
to catalog iteration. This change is recorded in the Unreleased changelog.

**Work:** Implement G1's `crates/registry_values` extraction before types in lower
crates register themselves. Move value reflection, owning RuntimeValue helpers,
Thing/RegistrationSite, and value registration macros/slices; preserve high-level
re-exports. Add the pure arbitrary descriptor without moving executable Function
metadata below execution. Preserve existing associated Function handles while
adapting low descriptors into registry catalog traversal exactly once. Keep each
production type in its own file and preserve the existing unsafe pointer contract.

**Validation:** Reinspect and run the existing local registry suite and compile
consumers. Add focused catalog integration coverage only where the extraction
changes behavior; pure metadata copying does not need duplicate tests.

```powershell
cargo test --offline --locked -p cloud_terrastodon_registry --lib
cargo check --offline --locked -p cloud_terrastodon_registry_values -p cloud_terrastodon_command -p cloud_terrastodon_credentials
```

**Completion criteria:** Low value/auth types can register without an execution
dependency; registry consumers retain one authoritative registration per value
or constructor and their existing ownership behavior. No old callback ABI is
removed before its migration slice.

### [x] 2.1 Establish explicit cache ownership and logical identity

**Completion notes:** Added `crates/cache` with an explicit WorkCache owner,
validated CacheScope, CacheKey, ExecutorKind, and canonical operation identity.
The owner atomically reserves/checks logical ownership claims through shared
state. Construction requires an absolute root without unresolved parent traversal,
performs no I/O, and discovers no defaults. This pulls ownership checking forward
from 2.3. It does not expose cache lookup, execution, or invalidation APIs.
Storage/publication arrives in 2.2, decoded entries and stable initializer locks
in 2.3, and generations/dependencies in 2.4; existing production caches are unchanged.

All 13 cache tests passed using `cargo test --offline -p cloud_terrastodon_cache
--lib`: conflicting and concurrent claims, retained ownership after rejection,
same-scope different namespaces, clone/separate-handle behavior, no constructor
I/O, root validation, exact component/prefix semantics, valid arbitrary generation,
and rejection of invalid scopes through Facet's validating proxy. Review removed
unnecessary Send/Sync bounds from `claim<T: 'static>` because it retains only
TypeId; shared-result storage will impose its own bounds. A non-Sync zero-validity
output is covered. Independent source review has no remaining findings.

The locked cache/command/REST compilation check below passed. Focused cache Clippy
with warnings denied, nightly formatting, and diff whitespace checks passed.
The initial Cargo resolution added only the local cache package
using existing pinned dependencies; no external version changed. The storage
path encoder and artifact records are intentionally deferred until 2.2 needs them.

**Work:** Introduce the G1 cache boundary without importing CommandBuilder, REST,
credentials, execution, or the high-level registry. Add validated logical
CacheScope components, CacheKey, ExecutorKind with Command/Rest/InProcess/Other,
CacheOperationOwner, and an explicit WorkCache resource owner. Clones share claims;
separate owners do not. Claim `(kind, logical scope)` against the canonical owner
and output TypeId before variants exist; namespace is part of identity, not merely
diagnostic metadata. Preserve exact component text/boundaries rather than parsing
operating-system paths, trimming identities, or silently lowercasing them.

Keep the current command CacheKey explicitly legacy until phase 4: its inherent
path/invalidation methods reach AppDir and global state, so re-exporting it from
the new crate would preserve an implicit dependency or introduce a backedge.
The new validated logical key has a distinct contract. Any temporary conversion
must be explicit/fallible and receive the owner; rebuild components from domain
inputs where legacy keys already contain lossy sanitization. Do not silently
resolve a handle or maintain two participating cache owners for one invocation.

Transport-neutral artifacts and their metadata arrive with their first storage
consumer in 2.2. CommandOutput stays in command: OS exit-status interpretation and
CommandBuilder diagnostics are transport responsibilities. No production read
switches to the new cache before its required coordination is implemented.

**Validation:** Repeat the focused package checks against the current lockfile:

```powershell
cargo check --offline --locked -p cloud_terrastodon_cache -p cloud_terrastodon_command -p cloud_terrastodon_rest
cargo test --offline --locked -p cloud_terrastodon_cache --lib
```

**Completion criteria:** Two isolated handles do not share claims implicitly;
clones share the same owner. Same-kind owner/type conflicts fail, including
matching output types with different owners. Distinct executor namespaces remain
independent. Construction requires an explicit root and creates no files or
runtime defaults. Kind does not decide effects, authentication, or cancellation.

### [x] 2.2 Isolate disk variants and make publication coherent

**Completion notes:** Tasks 2.0–2.1 and the phase 1 contracts are committed as
`3ebfe69b`; the artifact-storage slice and its refinements are committed as
`821acce9`. This completed slice adds [ArtifactEntry](../../crates/cache/src/artifact_entry.rs),
[ArtifactAttempt](../../crates/cache/src/artifact_attempt.rs), transport-neutral raw
sets, and required identity/freshness records. `WorkCache::artifacts<T>` checks
operation/output ownership before admitting a variant. Physical entries live
beneath `artifacts-v2/entries`, separated by executor namespace, exact scope
components, authentication partition, and fingerprint. The
[CachePathComponent](../../crates/cache/src/cache_path_component.rs) retains a bounded
readable prefix and digest of the original component; persisted original identity
is verified as well, so hashing does not substitute for collision detection.

The cache-path boundary review found `ArtifactEntry` is the encoder's only
production caller. No planned dependency requires this cache-specific policy in
`pathing`: G1 already permits `cache -> pathing` for general filename sanitization.
`CachePathComponent` now owns the encoded string privately and exposes
`from_identity`, read-only string inspection, and `AsRef<Path>`. Construction
encodes arbitrary original text, including empty input, rather than accepting
unchecked encoded text. The physical format remains byte-for-byte unchanged:
sanitize, retain at most 160 UTF-16 units, then append the full lowercase BLAKE3
digest of the original UTF-8 bytes. The three encoder tests moved with the type;
length assertions now enforce the documented 225-unit bound. No dependency edge,
manifest, or lockfile change is required; pathing keeps its general sanitizer.
Verification passed 34 cache unit tests, 13 artifact-store tests, three executed
input-integrity tests, and two general sanitizer tests. The Windows symlink test
retains its privilege limitation. Cache/pathing Clippy with warnings denied and
nightly formatting pass. Independent source review confirmed unchanged encoding
and dependency direction.

Each invocation has a unique input workspace and immutable completed attempt.
Input, payload, and diagnostic checksums are verified before selection/reuse;
missing or corrupt sets are misses. A failed attempt cannot overwrite a selected
one. Canonical decoding remains the caller's responsibility before publication;
stored output type names are diagnostic. Legacy directories are ignored and
preserved. Zero validity retains inspectable attempts without reuse.
`ArtifactRetention::Disabled` replaces the ambiguous `Sensitive` name: it bypasses
artifact reads, staging, retention, and publication. It does not redact content,
delete existing artifacts, or skip the underlying operation. The caller selects
it before staging; provider-specific redacted diagnostics remain provider work.

[ArtifactMetadata](../../crates/cache/src/artifact_metadata.rs) owns diagnostic
type-name capture through `new<T>(operation_owner)`. Its `output_type_name` and
`operation_owner_type_name` fields identify the output and canonical operation,
respectively; the latter owns the logical cache address, not the cache resources.
Both persisted names remain nonauthoritative. `ArtifactFileDigest::new(path, bytes)`
binds a file to a typed [ArtifactDigest](../../crates/cache/src/artifact_digest.rs),
which owns BLAKE3 hashing, verification, and validated `blake3:<hex>` serialization.
Payload digests use the same type. Missing/unknown algorithm tags and malformed
hashes fail decoding; old unprefixed manifests are cache misses, with no guessed
algorithm fallback. These are refinements to the uncutover store.

The helper ownership review removed the forwarding `entry_directory` function:
`ArtifactEntry::directory` owns layout directly, and `ensure_directory` owns entry
identity initialization. `ArtifactFilePath::join_to` owns validated component
joining. [ArtifactSelection](../../crates/cache/src/artifact_selection.rs) owns
reading the selection, verifying/loading its referenced attempt, and writing the
selection. The subsequent storage-type review replaces the remaining `artifact_io`
functions with three resource-owning types:

- [ArtifactDirectory](../../crates/cache/src/artifact_directory.rs) retains the
  original explicit root, validates descendant addresses, and owns checked
  directory creation and file-inventory I/O.
- [ArtifactFile](../../crates/cache/src/artifact_file.rs) retains its root-bound
  parent and validated file address. It owns regular-file checks, exclusive
  immutable writes, and JSON persistence, including serialization before replacement.
- [ArtifactIoGuard](../../crates/cache/src/artifact_io_guard.rs) retains the
  asynchronous mutex through blocking work and returns it for state finalization,
  including ordinary errors. Dropping it never clears publication quarantine.

Directory/file construction performs lexical validation only; every I/O operation
rechecks the original root and ancestors. This does not add coordination with
unrelated filesystem writers or protection from concurrent path substitution.
`ArtifactAttempt::stage_inputs` owns allocation of the unique input workspace;
`ArtifactEntry::begin_attempt` owns admission and cancellation boundaries in the
entry's file. Public inspection methods still return paths without performing I/O.
These types centralize invariants and resource lifetimes rather than grouping
unrelated static helpers. Existing selection, identity, and completeness rules
remain in force.

An owned I/O gate stays inside each complete blocking filesystem transaction,
even if the awaiting future is explicitly dropped. Publication quarantine is
installed before selection mutation and clears only on awaited success with the
returned gate. The drop regression proves an older worker cannot release its
gate early and overwrite a newer write; the storage failure/recovery regression
proves ambiguous publication remains quarantined. These exercise the worker and
quarantine boundaries separately, not a forced drop at every publication poll.
Required outcome retention uses the supplied scope's required-completion helper.

[Freshness](../../crates/cache/src/artifact_freshness.rs) preserves one observation,
explicit bounded/unbounded expiry, and one monotonic age anchor per attempt.
Reads and decoding cannot renew age; shorter callers receive shorter deadlines.
Runtime verification exposed whole-second serialization in the pinned Chrono
Facet implementation. `ArtifactExpiry::At` and the wire record's `observed_at`
both use `DateTime<Utc>`; a private
[timestamp proxy](../../crates/cache/src/artifact_timestamp_proxy.rs) preserves
nanoseconds and rejects malformed strings at the serialization boundary.
Absent/null expiry and reversed timestamps are also rejected.
The actual manifest reader also rejects omitted input/diagnostic inventories.

All 29 cache unit tests, 13 artifact-store tests, three input-integrity tests, and
three focused path-encoding tests passed with synthetic temporary storage. The
additional symlink regression returned early because Windows symlink privilege
is unavailable; its redirect protection is source-reviewed but unverified here.
The suites cover concurrent input staging, variant/namespace isolation, incomplete
sets, input mutation, partial staging failure, publication failure/recovery,
diagnostic-only type text, legacy isolation, caller expiry, and disabled retention.
The retention/constructor refinement reran all cache unit and integration targets;
digest validation and changed-byte checks pass, as do both diagnostic-name changes
and disabled publication without deleting prior artifacts. Cache Clippy and
formatting checks pass; independent source review found no remaining issue.
The helper ownership refactor also passed the same cache tests, Clippy, formatting,
and an independent review of I/O ordering; no new tests were needed for the moves.
The subsequent storage-type refactor passed 31 cache unit tests, 13 artifact-store
tests, and three executed input-integrity tests. The additional symlink test still
requires unavailable Windows privilege. Two new directory tests cover lexical
confinement without I/O and rejecting an ancestor replaced after handle construction;
the dropped-worker regression now belongs to `ArtifactIoGuard`. Independent review
confirmed root preservation, immutable staging, selection/quarantine ordering, and
unchanged miss-versus-identity-error behavior.
Independent review findings about returned deadlines and required timestamp fields
were repaired and verified. Focused Clippy with warnings denied, nightly formatting,
cache/command/REST compilation, and whitespace checks passed. Dependency changes
reuse pinned workspace packages; no external version changed.

The new store has no Command/REST consumers yet. Initializer coordination and
decoded sharing are task 2.3; generation/dependency admission and durable
invalidation are task 2.4. Production adapters must not cut over before those
guarantees exist. No independent-handle/process coherence, crash atomicity, or
protection against concurrent external filesystem substitution is claimed.

**Work:** Implement G5. Preserve complete process fingerprints and REST identity.
Add immutable transport-neutral artifact sets containing raw payload bytes,
metadata, and safe relative diagnostic files; never interpret a process or HTTP
status in the cache crate. Reuse pathing's readable filename sanitization, but
account for Windows case aliases as well: sanitization alone preserves case and
does not establish physical uniqueness. A bounded readable prefix plus digest of
the exact original component can distinguish variants; stored original identity
must still be checked before reuse/publication so any collision is rejected.
Prevent distinct fingerprints from sharing writable invocation inputs or partially
overwriting one artifact set. Stage adjacent command files per invocation/variant.
Coordinate artifact reads, publication, and invalidation against the physical
storage identity, including the executor namespace. Logical-scope invalidation
must still reach every namespace/variant; never infer it solely by appending a
prefix to the old disk path. Preserve legacy diagnostic inspection; decode
accepted legacy entries normally or deliberately treat unsafe entries as misses. Unify expiry and
refill timestamps. Define failure and obsolete-generation artifact retention.
Install publication quarantine before changing the visible commit; clear it only
after a complete commit, so explicitly dropped futures cannot expose ambiguous
artifacts. Apply sensitive-output retention policy before any artifact staging.

**Validation:** Source-reviewed local tests and compilation only; no production
services, credentials, existing caches, or production fixtures were used:

```powershell
cargo test --offline --locked -p cloud_terrastodon_cache --lib --tests
cargo test --offline --locked -p cloud_terrastodon_pathing --lib sanitize_windows_path_component
cargo clippy --offline --locked -p cloud_terrastodon_cache -p cloud_terrastodon_pathing --all-targets -- -D warnings
cargo check --offline --locked -p cloud_terrastodon_cache -p cloud_terrastodon_command -p cloud_terrastodon_rest
rustup run nightly -- cargo fmt -p cloud_terrastodon_cache -p cloud_terrastodon_pathing --check
```

**Completion criteria:** Within one handle, a reader observes complete artifacts;
variant inputs cannot clobber each other. Disk type diagnostics never substitute
for current decoding. No cross-process guarantee is inferred from these tests.

### [x] 2.3 Coordinate initialization and reuse canonical decoded values

**Completion notes:** The implementation and focused validation are complete.
`WorkCache::cached_work` checks the canonical operation/output claim synchronously
before creating an awaitable [CachedWorkSpec](../../crates/cache/src/cached_work_spec.rs).
The bound specification owns the cache handle and concrete identity, borrows the
caller scope and optional parent attempt, and accepts a caller-owned initializer
future without a `'static` requirement. Awaiting returns `Arc<T>`; only this shared
output boundary requires `Send + Sync + 'static`.

U43 moves fingerprint validation into
[NonEmptyString](../../crates/cache/src/non_empty_string.rs), with private storage
and a validating Facet string proxy. Both `CacheEntryIdentity::fingerprint` and
`WorkCache::artifacts` require this type before `claim` can run. No failed empty
fingerprint admission can leave behind an ownership reservation. The scalar wire
representation and exact nonempty contents remain unchanged; transport-specific
fingerprint format and semantic input coverage remain the transport's responsibility.

The registry holds weak references to per-entry initializer locks. Every waiting
and running invocation retains a strong reference, so memory eviction cannot
replace a live lock. Memory is checked both before waiting and after ownership is
acquired. Successful fresh values retain their original observation and monotonic
age. Shorter caller validity narrows reuse; an expired outcome still returns to
its original caller but is not published. Rejected or failed work is not cached.
`release_memory` reclaims decoded values and dead coordination records without
changing claims, live locks, disk artifacts, or the meaning of already-returned
snapshots. User value destructors run outside the state and artifact locks.

[WorkAttempt](../../crates/cache/src/work_attempt.rs) identifies one bounded
initializer, independently of its reusable entry address. An explicit guard
closes that identity on return, error, cancellation, or drop; cloned tokens cannot
prolong ownership or attach children after completion. The
[wait graph](../../crates/cache/src/work_wait_graph.rs) counts parent-to-child
edges until whole child invocations finish. It rejects direct recursion and
cross-task cycles, including recursion through a zero-validity intermediary.
Zero-validity operations execute independently and preserve their nearest active
initializer ancestry without acquiring a new entry lock or reusing a value.

The bound spec uses the supplied scope's `cancellable_scope` and passes the
factory's child to the initializer. Cancellation stops a pending initializer or
waiter while respecting linked required completion. An independently owned waiter
can retry; root cancellation stops all affected callers. No background fill task,
global cache, task-local ancestry, or credential access was added.

[WorkOutput](../../crates/cache/src/work_output.rs) initially represented only a
fresh execution observation at this checkpoint. Task 2.4 adds bound disk import
with original freshness, generations, and monotonic correlation, preserving any
shorter existing deadline. Generation/dependency certificates, warm dependency
cycles, and effect admission are recorded under that task. Production adapter
cutover remains a later slice.

All 40 cache unit tests, 17
[coordination integration tests](../../crates/cache/tests/work_cache_tests.rs),
13 artifact-store tests, and three executed input-integrity tests passed. The
additional Windows symlink test still skips for unavailable privilege. The new
proofs cover one execution/decode with pointer-identical concurrent and warm
results; independent identities; pending/warm owner and output conflicts;
initializer error, cancellation, and drop recovery; waiter and root cancellation;
required-completion draining; recursive and cross-task cycles; uncached ancestry;
namespace-separated wrapper/transport nesting; and eviction while a fill is live.
Unit tests additionally prove parallel wait-edge reference counts, stale versus
stricter-caller freshness, original-observation expiry after decoding, and reclaiming
dead coordination without releasing claims. U43's two new scalar tests reject
empty construction and decoding while preserving whitespace, Unicode, and the
existing string wire representation. Tests use explicit polling, barriers,
and channels with deadlock timeouts; no timing sleeps or production inputs are used.
Cache Clippy with warnings denied and nightly formatting pass. Independent review
found no remaining blocker for this slice. No manifest or dependency pin changed.

**Commit checkpoint (U44):** This checkpoint includes coordinated typed reuse,
bounded attempts and wait edges, cancellation/retry, memory eviction, and the
validated fingerprint refinement. Its subject is `Coordinate typed cache work and
enforce fingerprint invariants`. Task 2.4 remains the next slice: coordinate
hierarchical invalidation with memory/disk publication and record composed child
dependencies, using deterministic local race tests before any transport cutover.

**Work:** Implement short entry-map locking, stable per-key async ownership,
post-lock recheck, operation/output ownership verification, and shared typed
results with G2's explicit sharing bounds. Keep the ownership check effective for
in-flight entries as well as hits.
Apply G3's namespace and ownership claim before variant lookup; reject incompatible
operation owners or canonical output contracts before joining or using an entry.
Include same-output-type/different-owner collisions even if fingerprints differ;
never silently create a replacement entry to evade the check. Request key text
shared by InProcess and Rest is safe because the enforced namespaces differ.
Never await under the map guard. Apply U26 to initializer/waiter cancellation;
retain required post-outcome work. Detect recursive same-key initialization
instead of waiting forever. Also detect independently initiated tasks waiting on
each other's keys: record wait edges before awaiting ownership and clean them up
on completion/cancellation. Keep zero-duration/noncacheable operations executing.
Define bounded/reclaimable idle coordination entries without removing live locks.

**Validation:** The local `work_cache_tests` target uses counters to prove one execution
and canonical decode for concurrent successful callers, warm pointer reuse,
different-key concurrency, owner mismatch, failed/cancelled initializer recovery,
cancelled waiter isolation, root cancellation, cross-task wait cycles, and
zero-duration non-reuse. Add synthetic wrapper/transport nesting under the same
logical key, same-namespace mismatches on both pending and warm entries, matching
types with different owners/fingerprints, and recursive same-entry rejection.
This task uses local typed work, including synthetic Rest/InProcess namespace
nesting. Actual HTTP adapter validation uses the existing local REST boundary in
task 4.2; these cache tests do not claim HTTP integration. Do not run the live
locations test:

```powershell
cargo test --offline --locked -p cloud_terrastodon_cache --lib --tests
cargo clippy --offline --locked -p cloud_terrastodon_cache --all-targets -- -D warnings
rustup run nightly -- cargo fmt -p cloud_terrastodon_cache --check
```

**Completion criteria:** A hit does not deserialize or deep-copy its result; failed
work never becomes reusable; cancellation semantics and ownership errors are
deterministic. Concurrent error retries are not promised a shared error result.

### [x] 2.4 Implement hierarchical generations and composed dependencies

**Completion notes:** The cache core now captures original ancestor and exact-entry
generations before initialization. Absent ancestor records are captured as zero;
counter exhaustion permanently rejects covered certificates rather than wrapping.
[WorkCache invalidation](../../crates/cache/src/work_cache_invalidation.rs) advances
generations under the artifact gate and state lock, retires direct values, and
invalidates matching disk selections across namespaces and variants, including
entries absent from memory. Immutable attempts and unrelated siblings survive.
Failed or dropped invalidation leaves coverage-aware quarantine; an exact recovery
cannot clear a failed ancestor. Current-generation memory fills remain usable
while old disk selections stay blocked. No invalidation waits for initializer locks.

[WorkCertificate](../../crates/cache/src/work_certificate.rs) retains original
observations and flattened transitive dependencies, including warm child hits.
Repeated observations cannot overwrite stricter bounds or older generations.
Every hit and publication validates all dependencies, so cross-branch invalidation,
child expiry, and self-invalidation cannot make an old aggregate fresh. Warm
transitive cycles are rejected before reuse.

[WorkReuse](../../crates/cache/src/work_reuse.rs) declares memoization eligibility
before lookup. `ExecuteEveryTime` and zero validity bypass reuse; reusable parents
reject those semantic children before execution. Child completion guards start at
binding, so claim failures, cancellation, pending/unpolled future drops, and caught
errors make active ancestors nonreusable. Outstanding children prevent publication.
Finalization closes tokens under state before guard cleanup, so late use cannot
retroactively poison an accepted result. Credential acquisition stays outside this
semantic cache API; there is no arbitrary infrastructure-role bypass. Provider
capability and cancellation integration remain task 3, without placing credential
values or token-expiry dependencies in the work cache.

`ArtifactEntry::for_work` binds raw storage to the active initializer, keeping its
original generations even when staging follows invalidation. Bound leaf publication
freezes child admission and refuses composed outputs. Callback failure or rejected
late child admission quarantines a started leaf publication; publication rechecks
the attempt after filesystem work before clearing quarantine. `LoadedArtifact::decoded`
transfers the original age and narrowed deadline to `WorkOutput`, and that proof
is usable only by the initializer that loaded it. Unbound raw inspection cannot
be promoted into a later typed fill. Fresh owners rebuild aggregates through child
requests and the current raw decoder; no durable aggregate format was introduced.

Local tests cover the memory and durable boundaries using synthetic values,
temporary directories, explicit poll/barrier ordering, and controlled observations.
The new targets are
[cache invalidation](../../crates/cache/tests/cache_invalidation_tests.rs),
[artifact invalidation](../../crates/cache/tests/artifact_invalidation_tests.rs), and
[combined work/artifact invalidation](../../crates/cache/tests/work_artifact_invalidation_tests.rs).
They cover both publication/invalidation gate orders, invalidation during a fill
and disk decoding, child creation, cross-branch and transitive dependencies,
self-invalidation/retry, exact versus prefix coverage, caught failures, child
drop/cancellation, outstanding children, warm cycles, and failed durable invalidation
followed by eviction. Unit tests cover generation exhaustion, original child expiry,
strict-then-loose child bounds, frozen-token admission, and original disk age.
The fresh-owner test proves aggregate rebuilding and current canonical decoding
without fresh execution. No production services, credentials, caches, or data were
used; dependency versions and the lockfile are unchanged.

Final validation passed: 47 unit tests, 14 dependency-invalidation tests, nine
artifact-invalidation tests, three combined work/artifact tests, 17 coordination
tests, 13 artifact-store tests, and three executed input-integrity tests: 106 in
total. The additional Windows symlink test retains its unavailable-privilege skip.
Cache Clippy with warnings denied, nightly formatting, and whitespace checks pass.
Independent review identified and closed late leaf-publication poisoning,
finished-token admission, and retained disk-proof lifetime gaps; no remaining
blocker was found for this slice. U48 authorizes the reviewed checkpoint commit
and push, including U46/U47's refinements. The next task is 3.1, separating authentication selection
from credential runtime state.

U46 refines the directory boundary: `ArtifactEntry::directory` returns
`Result<PathBuf>` and propagates `ArtifactDirectory::for_entry` failures through
staging and selection. It no longer asserts that construction cannot fail.

U47 extends that correction across the workspace. Production calls have been
removed from cache/support, service/domain, UI/terminal, entrypoint/build, and
example code. All 34 package manifests enforce the policy. The workspace
denies Clippy's `unwrap_used` and `expect_used`; test functions and `#[cfg(test)]`
code remain exempt, while production constants do not. The lint level is `deny`
because cancellation test macros emit their own test-only allowances, which
conflict with Rust's non-overridable `forbid` level. Existing package lint policies
are retained, and the shared check script explicitly selects the whole workspace.

Fallible initialization, conversion, and execution boundaries now return errors
through their callers, including authorization headers, organization URL and
cost-query constructors, command program/summary resolution, terminal startup,
and diagram generation. Revision/identifier exhaustion is checked before state
mutation, query failures remain errors on retry, and fatal explorer errors close
waiting callers during cleanup. The tracing buffer's explicit poison recovery
preserves retained logs after a replay writer panics; no service errors are
silently converted into defaults. Dependency versions and the lockfile are
unchanged. Existing user staging and unrelated notes are preserved.

Workspace Clippy with all targets/features and warnings denied passes, together
with nightly formatting. Independent final review found no missing manifest
enforcement, production allowances, remaining production calls, or new panic
substitutes. Reviewed offline suites passed: 106 executed cache tests (plus the
existing Windows symlink privilege skip), 54 diagram tests, 268 UI tests, and 103
focused credential/domain, command, application, terminal, tracing, and HCL cases.
The UI suite explicitly excludes the live manual picker. Test fixtures were
updated for bounded cancellation scopes, current rendering, and the deliberate
nonserializability of runtime request state. No production services, credentials,
configuration, caches, or data were accessed. These results cover the listed
offline targets, not the unfiltered workspace test suite.

```powershell
cargo clippy --offline --locked --workspace --all-targets --all-features --keep-going -- -D warnings
rustup run nightly -- cargo fmt --all -- --check
cargo test --offline --locked -p cloud_terrastodon_cache
cargo test --offline --locked -p cloud_terrastodon_excalidraw --lib
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib -- --skip role_assignment_picker_app::test::manual_pick
cargo test --offline --locked -p cloud_terrastodon_credentials --lib auth_bearer_ext::tests
cargo test --offline --locked -p cloud_terrastodon_credentials --lib windows_credential_manager_target_name::tests
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib arbitrary_json::tests
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib iso8601_duration::tests
cargo test --offline --locked -p cloud_terrastodon_azure_devops_types --lib azure_devops_organization_url::tests
cargo test --offline --locked -p cloud_terrastodon_tracing --lib
cargo test --offline --locked -p cloud_terrastodon_command --lib no_spaces::tests
cargo test --offline --locked -p cloud_terrastodon_command --test cacheable_work_tests
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --test process
cargo test --offline --locked -p cloud_terrastodon_user_input --lib terminal_coordinator::tests
cargo test --offline --locked -p cloud_terrastodon_hcl --test reflow_preservation_tests
```

**Work:** Implement G6. Capture generations before disk loading/execution. Record
child dependencies on hits and misses and propagate transitive validity across
separate branches. Track dependencies of an in-flight parent, not only published
parents. Coordinate generation checks with memory/disk publication and invalidation.
Apply logical-scope invalidation across all executor namespaces and variants,
including an SDK wrapper and its transport child sharing the same logical key.
Reject publication if any relevant prefix/dependency changed, including expiry.
Invalidating a parent covers descendants created during the race; siblings remain
valid. Retain live initializer locks. Exclude stale results from reusable storage
while retaining required execution artifacts separately. Record retry dependencies
correctly after Graph self-invalidation; an obsolete outer fill cannot become fresh
merely because its final child was retried. Reject dependency cycles diagnostically.

Implement the chosen restart policy: memory-only composed results are rebuilt
from their child requests; any durable composed result must recover and validate
authoritative dependency provenance before reuse. This provenance is freshness
information, not a persisted Rust type gate. Distinguish semantic child operations
from credential acquisition infrastructure. A parent hit must never suppress a
required mutation, Solicit operation, zero-duration child, or deliberately uncached
data read; reject such memoization or propagate non-reusability. Refreshing a token
alone must not invalidate cached inventory data. Declare memoization eligibility
before looking up a parent result. Install child-completion guards at admission;
failed, cancelled, or dropped semantic children make every active parent
nonreusable, including when the parent catches an error and returns a fallback.

If durable invalidation fails, retain a stale-entry rejection/quarantine state
within the handle so a later memory miss cannot reload the old disk entry under a
new generation. Preserve and return the original I/O failure. Recovery may lift
the quarantine only after freshness has been established.

**Validation:** Focused cache-core targets and checks:

```powershell
cargo test --offline --locked -p cloud_terrastodon_cache --test cache_invalidation_tests
cargo test --offline --locked -p cloud_terrastodon_cache --test artifact_invalidation_tests --test work_artifact_invalidation_tests
cargo test --offline --locked -p cloud_terrastodon_cache --lib --tests
cargo clippy --offline --locked -p cloud_terrastodon_cache --all-targets -- -D warnings
rustup run nightly -- cargo fmt -p cloud_terrastodon_cache --check
```

**Completion criteria:** Deterministic tests cover invalidation during disk read,
network placeholder, decode, publication, child creation, and aggregate assembly;
cross-branch/transitive dependency invalidation; warm dependency registration;
cross-namespace invalidation of memory and disk under one logical scope;
unrelated-sibling preservation; self-invalidation; fresh-owner aggregate reload;
effectful/noncacheable semantic children versus credential infrastructure;
caught child failures, child cancellation/drop, and pre-hit effect admission; and
invalidation I/O failure followed by memory eviction and attempted disk reload.
After invalidation completes, newly admitted calls cannot reuse obsolete results.
Already returned Arcs remain readable. No hierarchy guard spans child execution.

## Phase 3 — serializable selection and bound execution

### [ ] 3.1 Separate auth selection from credential runtime state

**Work:** Implement resolved Azure bearer selection with explicit source/tenant
and Azure DevOps bearer/PAT selection. Preserve tenant validation, source policy,
headless/fail-fast behavior, and disabled authentication. Update token command
construction and tracked-tenant binding to reject missing bearer tenants; remove
Azure CLI active-tenant request compatibility. Executor selects compatible runtime
credentials, with no silent source substitution. Keep org/project separate.
Move the existing tenant ID and auth-source definitions to `auth_types`, retaining
compatibility re-exports and owning registrations through `registry_values`.
Implement G3's provider-bound identity partitions, including a fresh provider
session partition when source/tenant alone cannot establish the principal.
Verify isolated Azure CLI session provisioning against official documentation
before implementing it: every credential acquisition and service subprocess must
use the same bound identity, or fail before execution. Copying ambient CLI
configuration is not evidence of that binding. Credential results remain in the
provider's sensitive runtime cache; raw tokens, credential command output, and
secret-bearing errors must never enter general work-cache artifacts or diagnostics.

```powershell
cargo test --offline --locked -p cloud_terrastodon_credentials --lib auth_context::tests
cargo test --offline --locked -p cloud_terrastodon_credentials --lib azure_tenant_auth_context::tests
cargo test --offline --locked -p cloud_terrastodon_credentials --lib azure_devops_auth_context::tests
cargo check --offline --locked -p cloud_terrastodon_credentials -p cloud_terrastodon_azure
```

**Validation:** Replace the old active-tenant compatibility assertion with owned
selection-policy checks. Add offline tests of explicit token command tenant,
missing tenant, source mismatch, PAT handling, disabled auth, and workload tenant
mismatch. Inspect/filter token tests individually; do not run live token tests.
Use synthetic identity/session boundaries to test switching and refresh rejection,
independent-provider partitioning, and credential retention on success and failure.

**Completion criteria:** Executable bearer requests always carry a tenant and
source, including Azure CLI. Serializable selection contains no runtime cache,
token, session, or cancellation scope. PAT needs no fabricated tenant.

### [ ] 3.2 Bind requests to a scoped executor through both execute spellings

**Work:** Implement G1/G2's resource coordinator and operation contract. Bind
request data to an awaitable execution using explicit auth/cache handles and a
bounded CancellationScope. Provide forwarding request.execute(executor) and
executor.execute(request) entry points. Propagate child scopes correctly and
keep caller-specific validators/observers outside canonical cached decoding.
Retain operation effects and protected completion policy; do not detach fetches.
Establish the minimal application binding required for SDK migration. Enforce
G2's shared-result bounds only where memoization requires them; keep noncacheable
and borrowed callback contracts separately testable.

**Validation:** Proposed package/target after G1 confirms placement:

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features
```

**Completion criteria:** Synthetic operations prove equal behavior through both
spellings, shared handles across clones, child-scope cancellation, parent deferral
rules, caller cancellation/waiter retry, and no auth access for local-only work.
Request inspection/offline construction does not initialize runtime services.

### [ ] 3.3 Add executor-aware registry invocation and discovery

**Work:** Migrate registry invocation metadata/macros, default production,
discovery inputs/admission, and cancellation binding to G1's low-level contract.
Do not make registry depend on the concrete auth-owning executor. Preserve
conversion/arbitrary pure operations, nonempty executable effects, Solicit
exclusion for unattended discovery, and explicit auth/scope admission. Replace
synthetic AuthContext request inputs with serializable selection data. Register
shared output shapes and verify pointer/list traversal without live execution.

```powershell
cargo test --offline --locked -p cloud_terrastodon_registry --lib
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_plan
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_simulation
```

**Completion criteria:** Registered synthetic requests execute with explicit
resources; offline planning never resolves credentials or invokes operations;
shared collection outputs remain discoverable. The graph stays acyclic.

## Phase 4 — one Command/REST execution lifecycle

### [ ] 4.1 Route subprocess execution through shared work

**Work:** Extract uncached process execution with retries inside its ownership
boundary. Route cached/uncached, raw/typed, polite, mapper, and validator entry
points through the shared lifecycle as appropriate. Preserve complete process
fingerprints, isolated adjacent files, timeout/output behavior, login policy,
caller diagnostics, cancellation-safe opt-in, and default deferral. Publish
reusable output only after canonical acceptance; retain rejected artifacts and
never automatically replay a rejected mutating operation. Keep per-caller
validation on hits, including supported borrowed/non-Send callbacks.
Apply the sensitive-output policy before retaining any success or failure output;
do not start persisting successful credential commands that previously had no
artifact key. Authenticated Azure CLI subprocesses use the provider's bound session.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib
cargo test --offline --locked -p cloud_terrastodon_command --test cacheable_work_tests
cargo test --offline --locked -p cloud_terrastodon_command --test command_cancellation_tests
```

**Completion criteria:** Local subprocess fixtures prove no retry-lock recursion,
one accepted canonical result per read identity, preserved cancellation/artifacts,
and independently executed mutations. Existing tests are migrated to the new
boundary rather than maintaining a second production implementation.

### [ ] 4.2 Route REST execution and decoding through shared work

**Work:** Remove request-owned runtime auth/cancellation and implicit Auto fallback.
Route both cached and uncached receive paths through the shared lifecycle while
retaining error artifacts, HTTP status handling, continuation/header mapping,
blocking-decode caller/span attribution, and failure-extra-files. Distinguish
canonical transport decoding from per-caller policy. Preserve method cancellation
defaults and read-only POST opt-in. Do not persist secrets as identity material.
Apply sensitive-output policy before staging raw bodies, headers, or errors;
credential infrastructure is not eligible for the ordinary artifact cache.
Use the centrally applied Rest namespace and explicit canonical operation owner;
do not solve collisions with ad hoc prefixes at individual SDK call sites. Add a
synthetic wrapper/envelope scenario to the existing local REST tests to exercise
the LocationListRequest pattern without service data or the live SDK test.

```powershell
cargo test --offline --locked -p cloud_terrastodon_rest --lib
cargo check --offline --locked -p cloud_terrastodon_rest -p cloud_terrastodon_azure -p cloud_terrastodon_azure_devops
```

**Completion criteria:** Local boundary tests prove typed hits avoid decode,
caller-specific validation still runs, cached reads do not acquire tokens solely
to hit the cache, and writes retain outcome artifacts and never coalesce. Decode
failures remain attributed to the owning operation and preserve the original error.

## Phase 5 — SDK vertical slices and shared collections

### [ ] 5.1 Migrate Graph users and the composed principal inventory

**Work:** Migrate Entra user/group/service-principal requests, Graph helper, and
PrincipalListRequest to explicit construction data and executor-taking execution.
Move progress callbacks to bound observation state. Implement G4's sharing across
pages and EntraPrincipal conversions/collection access. Preserve current Graph
query/pagination behavior and recovery invalidation. Use recorded child dependency
stamps so invalidating any input invalidates the principal result across prefixes.
Use G6's aggregate persistence decision; a fresh cache owner must not restore an
aggregate as reusable without authoritative dependency freshness.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure_cli
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests
```

**Validation:** The proposed executor_inventory_tests target uses typed synthetic
values and the established execution boundary; it must never invoke real fetches.
Cover shared principal assembly, child-hit dependencies, continuation recovery
coordination, and observer semantics. Do not add assumed Graph JSON response fixtures.

**Completion criteria:** Independent callers share a completed principal inventory
without request runtime fields, repeated decoding, or stale aggregate reuse. Any
assembly copying is explicit and bounded to the selected ownership design.

### [ ] 5.2 Migrate Resource Graph and composed RBAC inventory

**Work:** Migrate role-definition/assignment requests and their combined request,
plus ResourceGraphHelper. Preserve page/query identity, rate-limit behavior,
read-only POST cancellation semantics, and cross-record validation in
RoleDefinitionsAndAssignments. Memoize the composed operation under its own
identity and record both child dependencies; distinguish aggregate/page identities.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure -p cloud_terrastodon_azure_types
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests
```

**Completion criteria:** Typed local scenarios show concurrent RBAC consumers share
one valid combined result; invalidating either input invalidates the combined
cache even though keys differ. No live role or Resource Graph tests are executed.

### [ ] 5.3 Migrate Azure DevOps projects and shared page records

**Work:** Migrate project listing and auth-selection plumbing through its CLI.
Evaluate/implement Vec<Arc<AzureDevOpsProject>> and shared aggregate storage per G4.
Preserve continuation metadata, ordering, wire output, explicit org, and bearer
tenant. Remove optional-tenant active-CLI fallback. Preserve PAT selection and
scope-sensitive cache identity without token acquisition on a valid cache hit.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_cli
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_plan
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_simulation
```

**Validation:** Add focused offline shared-record/selection checks using typed
values; update command-family filters here after inspecting the selected tests.
Verify Facet output and registry use as application integration, not derive-only
fixtures. Do not run the existing live project-list test.

**Completion criteria:** Project records can be reused across pages/aggregate views
without borrowed cache lifetimes; output remains compatible; every bearer path
has an explicit tenant and every scoped CLI path keeps explicit org/project.

## Phase 6 — audits, application ownership, and interactive consumers

### [ ] 6.1 Separate audit activities without centralizing their input data

**Work:** Split existing audit policies into cohesive files under the existing
Azure CLI responsibility. Keep SDK data operations in the SDK. Each activity
accepts the executor and explicit request scope, obtains its own dependencies,
and returns typed findings for one output boundary. Preserve policies and
diagnostic meaning; do not introduce new audit checks or a generic framework
solely to organize files. Use child scopes for concurrent independent work.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure_cli
cargo test --offline --locked -p cloud_terrastodon_azure_cli --lib audit_policy_tests
```

**Validation:** Proposed audit_policy_tests use small typed inventories for owned
policy behavior. A synthetic execution integration scenario runs two activities
sharing principals/RBAC and asserts one canonical fill, without real service data.

**Completion criteria:** Activities are independently invocable and testable, share
decoded inventories through the executor, and preserve existing audit results.

### [ ] 6.2 Construct and pass the executor from application startup

**Work:** Create one explicit runtime/cache handle for an invocation inside the
App handler's bounded scope. Migrate entrypoint/global argument wiring, service
CLI dispatch, --no-cache/clean paths, and both workspace application examples.
Keep `app` usable without auth and retain handler-owned cleanup where configured.
CLI aliases/configuration may resolve before request construction only as allowed
by existing project policy; Azure CLI defaults never supply request scope.

**Validation:** Extend `crates/app/tests/process.rs` for auth-enabled startup with
disabled authentication. Its current `cli()` fixture selects AzureCli; change it
to AuthSource::None before running the auth-feature suite. Also exercise parsing
`--auth-source none` followed by a local handler, rather than stopping at help or
argument parsing. Assert executor/cache ownership, child cancellation binding,
and terminal setup where enabled. Use isolated empty cache/configuration storage
and synthetic work only; perform no credential discovery, service requests, or
reads from production caches/logs. Retain the featureless process run. These
checks prove auth-feature startup with authentication disabled, not live provider
compatibility. The commands below depend on that fixture update and reinspection.

```powershell
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features
cargo check --offline --locked -p cloud_terrastodon_app --features auth,terminal
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --test process
cargo test --offline --locked -p cloud_terrastodon_app --features auth,terminal --test process
cargo check --offline --locked -p cloud_terrastodon_entrypoint -p cloud-terrastodon-standalone-cli -p cloud-terrastodon-group-management
```

**Completion criteria:** CLI/app consumers pass resources explicitly; local-only
startup acquires no credentials; no hidden singleton bridges old/new paths;
invalidation targets the same memory/storage owner as execution. Both feature
configurations have runtime evidence without accessing production data.

### [ ] 6.3 Migrate UI, discovery execution, and remaining consumer batches

**Work:** Update object-explorer invocation/value construction, dump planning and
simulation admission, menus, source/HCL consumers, and the inventory from 1.2.
Include `crates/ui_egui/src/app.rs`, `run_app.rs`, the subscription/resource-group
workers, and `windows/resources_window.rs`; each worker must use its child scope
while sharing the invocation's cache/auth resources.
Preserve pure conversions and offline arbitrary generation without live executor
inputs. Keep terminal coordination and output responsibilities intact. Convert
request families in bounded batches and record exclusions/removal dates here.

```powershell
cargo check --offline --locked -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_ui_egui -p cloud_terrastodon_entrypoint -p cloud_terrastodon_source_cli -p cloud_terrastodon_hcl
cargo test --offline --locked -p cloud_terrastodon_registry --lib
```

**Completion criteria:** Every inventoried invocation uses the new runtime
contract or a documented temporary explicit adapter scheduled for removal in
7.1. Interactive effects and unattended admission remain distinct. Unrelated
HCL transformations and UI behavior are unchanged.

## Phase 7 — cutover, acceptance, and documentation

### [ ] 7.1 Remove transitional implicit/runtime-bearing paths

**Work:** Complete each remaining request-family batch from 1.2. Remove legacy
global memory cache, request-owned auth/scope, implicit REST auth resolution,
active-tenant CLI execution, duplicate lifecycle, and reflective scope injection
where replaced. Keep legitimate AppContext/auth runtime fields and unrelated
pure operations; search findings require classification rather than blind edits.
Preserve output formats, operation effects, mandatory cancellation via executor,
explicit affected-read invalidation, and required-completion units in mutations.

```powershell
rg -n 'MEMORY_CACHE|bind_request_cancellation|impl_cacheable_into_future|impl_cacheable_work_into_future|AuthContext::resolve' crates examples
cargo check --offline --locked --workspace --all-targets
```

**Completion criteria:** No production caller relies on hidden execution state or
the old cache lifecycle. Each search finding is removed or justified by its actual
remaining responsibility. The full workspace compiles against the new contract.

### [ ] 7.2 Verify acceptance and document public migration/release impact

**Work:** Run the reviewed local test matrix and cross-platform checks. Update
REUSABLE_APPLICATIONS.md, affected request docs/examples, facade re-exports, and
the Unreleased changelog with concrete API/behavior changes. Reconcile references
in the earlier auth plan without importing its stale statements. Review new
package publication order and exact pinned dependency resolution; do not publish
or bump unrelated versions. Update this plan's evidence and privacy audit.

```powershell
rustup run nightly -- cargo fmt --all -- --check
cargo check --offline --locked --all --tests --examples --workspace
cargo clippy --offline --locked --workspace --all-targets --all-features -- -D warnings
git diff --check
```

**Validation:** Also run the task-specific local tests after reinspection. On an
available Linux checkout, repeat relevant checks/tests with local PowerShell
available for subprocess fixtures. If unavailable, record that support evidence
as pending rather than claiming a pass. Inspect `publish.ps1 -DryRun` behavior
and package ordering only as release preparation; actual publication is separate.

**Completion criteria:** Initial acceptance criteria below have evidence; API docs
match executable behavior; no unexpected dependency upgrades, PII, or unstated
support limitations are introduced. No implementation completion is inferred
from compilation alone or from unchanged historical test results.

## Phase 8 — deferred cross-process coordination

### [ ] 8.1 Define and implement coordination beyond a shared handle later

**Work:** In a later authorized implementation phase, decide how separate cache
owners/processes coordinate artifact publication, lock recovery, persisted
invalidation generations, auth partitions, and legacy readers. Do not claim that
atomic replacement alone coordinates generations or prevents duplicate execution.
Specify Windows/Linux behavior and crash recovery before implementation.

**Validation:** Add subprocess contention/crash-recovery tests after choosing the
contract; record exact commands here then. Initial phase tests prove none of this.

**Completion criteria:** Explicit cross-process guarantees have independent
evidence and documentation. This item remains deferred when phases 1–7 complete.

## Acceptance matrix

| Surface/target | Initial support contract | Required proof | Evidence |
| --- | --- | --- | --- |
| Shared cache handle and executor clones | Coordinated within one owner | Concurrent typed sharing, per-key ownership, generation/dependency races | Tasks 2.3–2.4 cache coordination and generation/dependency races passed; executor integration pending |
| Independent handles/processes sharing disk | No single-flight/coherent invalidation guarantee yet | Document limits; phase 8 supplies stronger proof later | Deferred |
| Command, REST, in-process reads | One lifecycle, canonical memoization | Local adapter + cache tests; different validators still run | Pending |
| Raw artifact storage | Isolated complete attempts and guarded selection within one handle | Local storage/integrity, freshness, and worker-drop tests | Tasks 2.2–2.4 storage, generation, quarantine, and bound import checks passed; production adapters pending |
| SDK/transport namespaces and ownership | Separate concrete entries; shared logical invalidation scope | Synthetic wrapper succeeds; same-namespace owner/type collisions bail before execution/reuse; invalidation covers all kinds | Pending |
| Mutations, zero-duration and uncached work | Execute each invocation; retain required artifacts | Cancellation and artifact/invalidation tests | Pending |
| Graph principals, ARM RBAC, Azure DevOps projects | Explicit auth/scope; shared assembled results | Typed local composition/discovery tests and compile checks | Pending |
| Azure CLI bearer, browser, workload identity | Explicit request tenant/source | Offline selection/command construction; no live credential tests | Pending |
| PAT and local-only/disabled auth | PAT tenant-independent; local work credential-free | Selection/admission tests | Pending |
| Registry, UI, discovery, CLI output | Explicit execution; offline planning preserved | Synthetic invocation/discovery and scoped UI/output checks | Pending |
| App with auth and without auth; workspace examples | Supported feature boundaries; no production inputs | Both process-suite feature configurations, `--auth-source none` local handler with isolated storage, and consumer compile checks | Pending |
| Windows | Primary local validation platform | Existing linker/toolchain; local PowerShell fixture suites | Pending |
| Linux | Preserve documented consumer support | Repeat relevant compilation/tests when environment available | Pending |
| Live cloud and pipeline use | Outside this migration's development validation; no production data access or ingestion | Local synthetic fixtures and compile checks only; live compatibility is not established | Not run |

## Overall initial-migration completion criteria

- [ ] Every active U01–U48 requirement has task evidence; G1–G6 are closed and
  phases 1–7 complete. Phase 8 remains explicitly deferred.
- [ ] Requests contain explicit operation/auth selection data and no execution
  resources; both execute spellings and registry invocation share one contract.
- [ ] Command/REST/in-process execution use one cache lifecycle with no hidden
  global work cache or implicit authenticated-request fallback.
- [ ] Concurrent successful reads share canonical decoded results; warm hits do
  not deserialize or deep-copy results. Type/owner misuse fails diagnostically.
  Namespace separation covers SDK/transport results without hiding same-namespace
  collisions or narrowing logical-scope invalidation.
- [ ] Prefix and recorded dependency invalidation cover memory and disk without
  stale publication or nested deadlock; unrelated scopes remain reusable.
- [ ] Initializer/waiter/root cancellation, deferrals, mutations, zero-duration
  entries, caller validators, and required artifacts obey their stated contracts.
- [ ] Azure CLI bearer requests always specify tenant; all authenticated sources
  remain explicit; PAT and disabled/local-only paths keep their distinct semantics.
- [ ] Independent audit activities share inventories without a World object or
  new audit-policy behavior. Consumers and discovery support the chosen Arc shapes.
- [ ] Workspace/feature/consumer checks and reviewed offline tests have recorded
  results; unavailable platform evidence is explicit and limits completion claims.
- [ ] Documentation, Unreleased notes, package DAG, privacy checks, and temporary
  adapter removal are complete. No release, live test, or commit is implied.

## Risk register

| Risk | Guardrail and proof |
| --- | --- |
| Source/type collision or same type with different validator semantics | G3 ownership claims before variant lookup; 2.3 and 4.x canonical decode versus caller-policy tests |
| REST prefix hides same-kind collisions or breaks scoped invalidation | U31; central namespaces, operation ownership checks, 2.2–2.4/4.2 synthetic wrapper and cross-namespace tests |
| Credential/account crossover despite matching source/tenant | G3 auth partition; 3.1 isolation tests; no secret-based printable keys |
| Distinct fingerprints corrupt artifacts or command inputs | G5; 2.2 physical identity/staging and overlapping write tests |
| Invalidated work repopulates memory or disk | 2.4 generation check coordinated with publication; error-path tests |
| Composed result survives invalidation of sibling-key inputs | U28; 2.4 dependency stamps including hits, in-flight assembly, and transitive expiry |
| Restored aggregate has no trustworthy child freshness information | G6 memory-only reuse or authoritative dependency provenance; 2.4 fresh-owner tests |
| Memoized parent suppresses an effectful child or binds data expiry to token refresh | G6 semantic dependency admission; 2.4 effect/infrastructure tests |
| Failed disk invalidation permits stale reload after memory eviction | 2.4 fail-closed scope quarantine and subsequent reload test |
| Parent/child lock deadlock or self-invalidation deadlock | Short cache barriers; 2.3 cycle rejection; 2.4/5.1 recovery tests |
| Replacing/evicting a live lock permits two initializers | Stable ownership entries and safe reclamation; 2.3/2.4 tests |
| Cancelled leader damages unrelated waiters or abandons owed completion | U26; scoped executor and retained artifacts; cancellation fixtures |
| Mutation coalescing or retry repeats side effects | Zero-duration non-reuse; uncached retry boundary; no replay after rejected decode |
| Arc migration breaks consuming conversions, output bounds, discovery, or output | G2 shared Send/Sync/static bounds; G4; 3.3/5.x typed integration and compile checks |
| Runtime observer closures silently skipped or captured in cached policy | G4; 3.2 observer semantics and per-caller validation tests |
| New executor creates command/credentials or registry cycles | G1 manifest DAG; staged checks and release-order review |
| Legacy timestamps, metadata, or cache paths misrepresent freshness | G5; 2.2 legacy/expiry policy; preserve diagnostic inspection |
| Claimed thread safety mistaken for cross-process guarantees | U27; acceptance matrix and separate phase 8 |
| Tests or cache inspection expose production data to tools/agent context | U29; reviewed local targets, auth-feature startup with none, isolated synthetic storage, no test.ps1/workspace suite/ignored tests |
| Plan drifts or includes identifying local output | Stable ledger, three-pass audit, relative evidence, privacy scan on update |
