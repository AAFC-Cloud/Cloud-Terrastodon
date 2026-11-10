# Explicit execution and shared typed cache migration

**Plan status:** Active migration; current focus is the U123–U132 design revision
for accounts, runtime verification, and request-first callers before Phase 6 audit
extraction. Implementation of that revision is not yet authorized. See
[the design companion](accounts-runtime-tests-and-requests.md).

Historical completion record: tasks 2.6–2.8 and their tooling refinements were
committed as `aa02c23c`. U59–U61 revise the next boundary: option B with pure
`registry`, executor-aware `registry_execution`, runtime `auth`, and vocabulary
`auth_types`; `AzureTenantId` stays in `azure_types`. Task 3.0's package cutover and
U63–U65's direct enum-handle refinement are complete. U66 records their local
commit checkpoint, titled `Separate pure registry from execution with direct function handles`.
Task 3.1 is complete under U67/U68: runtime `auth`, pure `auth_types`, and
persistent application-owned Azure CLI login generations.
U69–U72's authorization-value refinement and replacement of the
temporary CLI service-command bridge with ARM requests are complete in task 3.1a.
U73's response-field requiredness correction is complete across that slice.
U74 records the completed work in the local checkpoint titled
`Split auth runtime and migrate Azure operations to ARM REST`.
U75's task 3.2 is complete: the scoped executor, injected authentication provider,
and minimal application binding passed offline checks. U79's task 3.3 is complete:
registry invocation, offline discovery, and direct UI callers use the explicit
executor. U80/U81's provider naming and temporary-cache ownership refinements
are included in checkpoint `b00bff46`, `Bind registry and UI execution to explicit executor ownership`.
U82 begins task 4.1's production Command integration with uncached process
ownership and retry isolation (4.1a). U83 commits that slice as `c981de2b`,
`Isolate command attempts and keep retries below cache admission`, and begins
4.1b with executor-bound raw artifacts. U84 commits that prerequisite as
`811468f3`, `Bind executor artifacts to admitted cache attempts`, and adds owned
uncached execution with explicit admission. U85 commits that prerequisite as
`5f627dfc`, `Add admitted owned execution without shared result bounds`.
U86 clarifies that implementation means completing the Command API and caller
cutover, not stopping after another prerequisite. Task 4.1b is complete, including
its production callers and offline verification. U97 requests its local checkpoint;
U87 corrects excessive executor coupling in cancellation-only helpers; U88
authorizes that correction. Task 4.1c is complete with scope-only local Command
APIs and callers, explicit managed artifact/cache paths, and offline validation.
U97's checkpoint also includes these changes. U89 additionally unifies the account/MSDN
license vocabulary introduced by the REST replacements with the existing combined
license type; its refinement evidence is recorded below. U90 retains the generic
`auth_context` setter and centralizes tenant-bound REST authentication in
`azure_auth_context`. U91 includes Azure CLI program resolution in its linked
cancellation scope. U92 audits raw JSON storage, reuses `ArbitraryJson`, and adds
a Clippy reminder with explained codec exceptions. U93 inlines the Command JSON
helpers. U94–U96 restore bound-request invalidation chaining and separate key
ownership from own-key, multiple-key, or custom async invalidation policies.
REST task 4.2 is complete under U98: the full transport and caller cutover,
workspace all-target check/Clippy, and focused offline verification passed.
U99's request-specific error refinement is complete in task 4.2a; all 63 focused
offline tests and the workspace all-target check/Clippy passed.
U100/U101's cached SDK ownership refinement is complete in task 4.2b: shared
values reach consumers without cloning requirements introduced only to recover
owned copies. All 46 selected offline tests and workspace all-target check/Clippy
passed; existing unrelated Clippy warnings remain.
U102's concrete domain-response refinement is complete; workspace all-target
compilation and focused Azure types/SDK Clippy passed without warnings.
The completed REST/shared-output work and batch-entry consolidation are now
committed as `d2a5e4f2`, `Complete REST executor migration and preserve shared SDK results`.
U104's phase 5 is complete from that clean checkpoint and was left uncommitted
for review; U113 records the subsequent commit. Tasks 5.1, 5.2, and 5.3 implement
the shared principal, RBAC, and project
inventories, serializable request selections, and per-invocation observation.
All 106 selected offline tests, workspace all-target compilation, and Clippy
passed; only existing unrelated Clippy warnings remain. At that checkpoint the
next task was 6.1, separating audit activities so each obtains shared inventories;
U123–U128 now insert the account/test/request prerequisites before that task.
U105/U106 refine that review checkpoint: restore the project-list/picker URL
`Cow` contracts and fix the exposed decoding/derive defects in the Facet forks.
U107 authorizes the coordinated fork releases and consumer dependency updates.
The Facet/Figue closure and Cancellation maintenance versions are published;
both consumers now pin that registry graph and their all-target compilation
checks pass. The required Tracing package is also published. The template passes
all 13 selected registry tests and is committed/pushed as `58d94181` with a clean
tree. Cloud Terrastodon's two reflected-clone tests and 14 inventory/discovery
tests pass against the ordinary registry graph. That checkpoint preserved user
staging and remained uncommitted until U113.
U108 subsequently corrects release commit author metadata and merges the
Cancellation/Tracing release branches into `main`. Both mainline merges are
validated and pushed. The broader authorized Format history correction is
complete, with source trees and published archives unchanged. Tracing's separately
approved GitHub default-branch change to `main` is also complete.
U109–U111 refine the review checkpoint with shared instrumented test
authentication, imported derive names, and automatic collection registration.
Implementation and verification are complete: 151 selected offline tests,
workspace all-target compilation, targeted Clippy, and optional-feature checks
passed. U112 clarifies raw registry registration-point names and audits fail-fast
joins. U113 commits the completed checkpoint as `1e63a553`, then authorizes a
limited scoped-join helper and three initial Azure callers for interface review.
U114 approves bounded concurrent mapping and rollout across fixed joins and finite
batches. Implementation and offline verification are complete; U117 authorizes
the local checkpoint after callback and result-buffer refinements and removal
of the unused interactive cache-population workflow. The audit identified picker
handler reload/exit draining and background-task shutdown as a separate lifecycle
follow-up before task 6.1. See
[scoped join review](scoped-join-review.md) for the remaining coordinators.
U117's checkpoint is `8af2d188`, `Coordinate concurrent work with scoped cancellation and draining`.
U118 authorizes the interactive handler/background shutdown follow-up from that
checkpoint. Implementation and offline validation are complete: 86 focused tests,
workspace all-target compilation, picker feature compilation, and affected-crate
Clippy pass. U120 commits that checkpoint as `58f454e4`,
`Drain interactive and background work through cancellation`, then authorizes
consolidating the UI loading/background adapters and removing unused delivery
policy flags. This cleanup is complete: six focused offline tests,
workspace all-target compilation, and affected-crate Clippy pass. Audit extraction
(6.1) was the next scheduled work; U123–U128 insert the prerequisites described
below before tasks 6.1–6.3. U122 records
the cleanup in the local checkpoint titled
`Consolidate UI background work and scoped autosaves`.
U115 approves iterator-owned mapping through `ConcurrentIteratorExt`, with
explicit executor/scope methods and no redundant executor mapping method. Callers
are migrated; 23 focused tests, two documentation examples, workspace compilation,
and affected-crate Clippy pass. Existing staging is preserved.
Explicit SDK/CLI executor propagation removes the implicit REST cache path;
phase 5 completes its targeted aggregate caching and request-data extraction.
Other request families retain their explicit 6.3/7.1 migration batches.
U97's preceding checkpoint requested
the completed Command migration commit and a description of REST task 4.2;
U98 subsequently authorizes its full implementation.
U78 authorizes the local checkpoint titled
`Add scoped executor with shared cache and injected authentication`, including
U76's execution-mode names and U77's startup diagnostics.
That checkpoint is `3478e3bb`; U79 begins from its clean working tree.
The registry checkpoint is `09d8bb11`.

**Primary implementation root:** Repository root (`.`); no branch change prescribed.

**Last updated:** 2026-11-09

**Source baseline:** `26e14bcc`; tracked files were initially clean. Pre-existing
untracked notes are preserved.

**Intent audit:** Updated 2026-11-06 using the preserved ledger and available user
instructions. The original audit covered the design discussion,
subsequent corrections, plan request, three planning-question replies, and the
latest review corrections about production data, validation, and key collisions,
authorization to proceed with implementation, the full-cutover correction, and
the response-type dependency review, types-crate execution extraction, and
resource-provider API ownership correction, default-catalogue refinement, and
shared tenant-selection correction, default-output refinement for the
maintenance tool, the request to commit this work and describe the next part,
and the dependency review, diagram/candidate request, and selection of option B
with the `auth`/`auth_types` naming correction and U62's authorization to implement
the next planned slice, task 3.0. U63–U65 subsequently authorize replacing the
registry's adapter lookups and optional provenance fields with direct enum handles.
U66 authorizes committing that completed work and identifying the next task.
U67 authorizes proceeding with task 3.1, retaining offline-only validation.
U68 selects a persistent Cloud Terrastodon Azure CLI login store after discussion
of persistent versus per-executor login sessions.
The 2026-11-03 refinement audit covers U69–U72: explicit authorization variants,
the service-command allowlist explanation, the REST replacement proposal, and
authorization to implement that replacement.
U73's subsequent review rejects speculative optional response fields across the
new ARM records, retaining optionality only for concrete omission or conditionality.
U74 requests a local commit of the completed changes and a description of the next
plan phase; it does not start task 3.2 or authorize pushing.
U75 subsequently authorizes proceeding with that next described task, 3.2.
U76 refines the execution-mode names to expose the cache and authorization
restrictions directly, preserving their existing behavior.
U77 requires stderr diagnostics for hook-setup errors previously discarded by
the extracted app startup helper.
U78 requests committing the completed changes and identifying the next
implementation phase; it does not start task 3.3 or authorize pushing.
U79 subsequently authorizes implementing task 3.3 with synthetic/offline checks;
its completion evidence is recorded below. Transport tasks remained unstarted at
that checkpoint; U82 begins task 4.1 below.

**Planning method:** User-invoked `resumable-implementation-plans` skill.

**U118 intent audit — 2026-11-09:** Reconciled the latest request to proceed with
the preceding next-task description, the preserved user ledger, and the current
development conventions. The scope is interactive cancellation ownership, with
scope-only UI APIs, explicit child-executor binding where SDK work needs it,
synthetic validation, and an uncommitted review boundary. Independent review
identified terminal-poison handoff races, nested-picker lifetime retention, GUI
admission-before-callback ordering, and idle cancellation wakeup; these fixes
belong to this ownership task. Audit policies and service behavior are unchanged.
Earlier historical mappings remain the preserved ledger rather than a claimed
fresh reconstruction of unavailable assistant turns.

**U119–U120 intent audit — 2026-11-09:**
- **Extraction:** The latest discussion distinguishes UI loading/error state,
  cached result reuse, and task lifetime ownership. The user requests committing
  the existing work before implementing the recommended consolidation and flag
  removal. Memory-only caching already exists; no cache-policy redesign is needed.
- **Traceability:** U120's checkpoint is `58f454e4`. The UI background-work cleanup
  below owns the shared adapter/tracker cutover, cancellation semantics, offline
  validation, and uncommitted review boundary.
- **Adversarial review:** Preserve useful loading/error presentation and drain
  required completion; do not turn this into audit extraction or remove the task
  owner. Final settings must survive removal of the old raw-spawn autosaves, so
  GUI shutdown flushes the latest settings after earlier saves drain.
- **Source limit:** Earlier historical mappings use the preserved ledger; this
  audit covers the available latest user messages and verified source.

**U121–U122 intent audit — 2026-11-09:**
- **Extraction:** U121 asks why the direct work enqueue API requires a static state
  type; it does not request another API change. U122 requests committing the
  reviewed cleanup and identifying the next comprehensive implementation phase.
- **Traceability:** The static-bound explanation is recorded in the guidance
  ledger. The local cleanup checkpoint preserves its validation evidence. The
  Phase 6 completion boundary below covers audits, startup evidence, and remaining
  consumers, with completed foundations identified separately.
- **Adversarial review:** Preserve commit-before-next-work sequencing, scope-only
  cancellation helpers, independent audit data access, and offline-only validation.
  Do not equate already-completed UI propagation with full Phase 6 completion or
  restart its completed foundations. Earlier historical intent uses the preserved
  ledger; this audit covers the available latest messages and current source.

**U123–U128 intent audit — 2026-11-09:**
- **Pass 1 — extraction:** Reread the latest three-part design request and both
  planning replies. U123 covers accounts distinct from sources, tenants, and
  service scope, including multiple users/workloads and PAT/Gitea. U124 covers
  live verification as registered executable requests, Cargo isolation, effect
  filtering, local writes, and debug response coverage. U125 covers removal of
  forwarding fetch functions in favor of request-first execution. U126 preserves
  the brainstorming/plan-before-implementation boundary. U127/U128 confirm scoped
  aliases and deliberately configured defaults without ambient/first-account fallback.
- **Pass 2 — traceability:** The companion maps these requirements to gates
  A1–A4/V1, tasks 6.0a–6.0e, validation, and the expanded acceptance matrix.
  Existing audit/application/consumer tasks 6.1–6.3 remain pending after these
  prerequisites. Account defaults do not relax explicit service scope or bearer
  request tenant identity. Recommendations remain distinct from confirmed choices.
- **Pass 3 — adversarial omission:** Checked two accounts in one tenant, multiple
  auth mechanisms, tenant-independent PATs, service-specific identity, runtime
  account add/remove, local-file effects versus cache artifacts, empty-response
  coverage, debug-only strictness, custom request errors/Cow/Arc ownership, and
  genuine workflows that must not be deleted as forwarding helpers. Review also
  identified frozen binding revisions (subsequently refined by U131), logout
  during required completion, and cross-account aggregate dependencies as
  account-contract gates.
- **Source limit:** This audit covers the available latest messages, current
  source, and official platform documentation. Earlier historical decisions use
  the preserved ledger rather than a fresh reconstruction of unavailable turns.

**U129 intent audit — 2026-11-09:**
- **Extraction:** The user requires properly namespaced types, explicitly naming
  CT's internal account identity and citing Azure resource-group requests and
  storage accounts as existing inconsistencies.
- **Traceability:** The companion specifies `CloudTerrastodonAccountId` and the
  domain naming rule; 6.0a/6.0c apply it to new account types and 6.0d inventories
  and renames affected service families and consumers together.
- **Adversarial review:** Rust names must distinguish local profiles from remote
  principals/resources even after imports. Do not confuse type-name prefixes with
  alias lookup scopes, require needless types, or turn this planning refinement
  into an immediate workspace rename. Preserve service wire/cache contracts.

**U130–U132 intent audit — 2026-11-09:**
- **Extraction:** U130 replaces mandatory fresh test data with optional `--clean`,
  a warning when omitted, and the ability to diagnose parser changes against
  cached responses. U131 makes the per-request execution account/source/tenant
  visible for approval, using an admin read on behalf of another user as the
  example; runtime credentials stay executor-owned. U132 accepts separate test
  registration rather than `Effect::Test`.
- **Traceability:** V1/6.0b/6.0e own both clean modes, nested invalidation and
  saved-response diagnosis; A2/A3/6.0c/6.0d own account identity, credential
  renewal, explicit multi-account execution and approval data. The inventory
  records resource-group/storage requests as unfinished context migrations.
- **Adversarial review:** Omitted `--clean` allows caching, not guaranteed offline
  execution or decoder replay. Failed-decode artifacts are not reusable success
  entries, and typed hits skip parsing. Invalidation must cover children beyond
  the root key. Reviewed account identity is distinct from private login/token
  generations; defaults cannot silently replace the approved identity. These
  refinements supersede the earlier mandatory fresh-root and per-login serialized
  revision recommendations without changing cache/cancellation safety contracts.

**Current authorization:** U123–U132 authorize design review and updating this
plan before further implementation. The prior UI cleanup is committed as
`39bee039`, `Consolidate UI background work and scoped autosaves`. The next safe
work is closing the account/runtime-verification contracts in 6.0a; implementation
of tasks 6.0b–6.0e and the existing 6.1–6.3 follows a subsequent instruction to
proceed. No code changes, commit, push, live credentials, or production-data
access are authorized by this planning request.

**Phase 5 authorization history:** U104 authorized completing all three phase 5 SDK
vertical slices, their shared observation support, necessary consumers, and offline
verification. Requests retain explicit tenant/source or DevOps auth selection;
runtime providers remain executor-owned. Aggregate caches are memory-only and
record child dependencies. Preserve existing staging and user edits; do not stage,
commit, or push Cloud Terrastodon, access production data, or begin phase 6 audit
reorganization under this checkpoint.

U105 preserves borrowed organization fields. U106 directs the Facet fixes to the
existing `teamy-main` forks and provides `docs/UPDATING_FACET.md` as guidance.
Those instructions initially authorized local preparation and validation only.
U107 subsequently authorizes committing/pushing the Facet, Format, Figue, and
template repositories, publishing the fork packages, and updating both consumers
to the published graph. Include necessary dependent-package releases to preserve
one Facet identity. This release authorization does not include committing the
Cloud Terrastodon review work or accessing production services.
U108 authorizes applying the supplied Git identity, correcting release commit
author metadata, and merging the Cancellation/Tracing release branches into
`main`. Subsequent explicit approvals cover the seven affected older Format
commits, the feature branch and annotated CSV tag, and Tracing's GitHub default
branch. Do not record personal identity values in this plan.

**Prior authorization history:** The user authorized proceeding with the migration on
2026-10-30 and subsequently requested committing the completed foundation and
proceeding. The reviewed foundation was committed as `3ebfe69b`, and the completed
artifact-storage slice as `821acce9`. After that checkpoint and the next-phase
description, the user authorized proceeding with task 2.3. The following request
authorizes committing that slice and its fingerprint refinement, then describing
task 2.4. The coordinated-cache checkpoint is `49352932`; the subsequent request
to proceed authorizes task 2.4. The subsequent review requires fallible directory
construction and prohibits production `unwrap`/`expect`; the user explicitly
selected cleanup of existing violations across the entire workspace now. U48
authorizes committing the completed changes and pushing the current `main`
branch to `origin`, including its preceding local checkpoints. Package releases,
production-data access, and implementation of the next phase remain outside this
checkpoint's authorization. That checkpoint was committed and pushed as
`fd989bd3`. U49 now requires a full cutover with no compatibility re-exports;
that correction was committed as `f9329427`. U50 now targets the narrow
`azure_types` dependency on `command` through Resource Graph response decoding.
This cleanup does not start the auth/executor implementation in phase 3.
U51 authorizes moving HCL formatting to `hcl` and extracting resource-type
generation into a standalone `tools/` package. U52 selects offline generation
plus explicit live refresh; the initial tool required a tenant and authentication
source. U56 later changes only its tenant-selection policy. Only synthetic/offline
validation is authorized. No commit or live run
is implied by this extraction.
U53 adds existing Cloud Terrastodon tenant-alias support to refresh. Resolve the
explicit alias to a tenant ID before credential acquisition, retaining required
selection and rejecting the default selector at that checkpoint. U56 supersedes
that tool-specific restriction and the direct-UUID bypass of shared resolution.
U54 moves resource-provider API records into `azure_types` and execution into
`azure`, leaving the maintenance tool to consume the API and write its catalogue.
Future registration tooling motivates the reusable boundary; subscription
registration operations are not part of this extraction. The existing generated
enum and its unknown fallback remain the bootstrap vocabulary.
U55 makes `--catalogue` optional for refresh, generate, and check, using the
maintenance tool's `catalogue.json` by default. Anchor that default to the build
checkout's tool manifest directory and retain explicit path overrides. Output
selection remained required at this checkpoint; U57 supersedes that requirement.
This refinement does not create a catalogue or authorize a live refresh.
U56 allows the maintenance tool to use the existing default tenant selection and
removes its duplicate tenant argument/resolver. Use the shared
`AzureTenantArgument` and `AzureTenantArgumentExt::resolve`: omitted or `default`
selection resolves the Azure CLI default account's tenant; explicit UUIDs and
aliases follow the shared tracked-tenant checks. The credential source remains
required and the resolved UUID is explicitly bound before the SDK request.
This is a tool-selection exception, not permission for SDK requests to contain
an implicit tenant, substitute credential sources, or run against live data during
development validation.
U57 gives generation a default output destination at the existing generated enum
source. Use a shared manifest-directory default for generate and check targeting
`crates/azure_resource_types/src/resource_types.rs`, while retaining explicit
path overrides. Validation must preserve the checked-in source and default
catalogue; this change does not authorize a live refresh or inventory regeneration.
U58 authorizes committing the completed type-boundary, maintenance-tool, and
provider-API refinements, then describing task 3.1. Preserve the user's staged
generation of `resource_types.rs` and the ignore rule for the local catalogue.
The generated inventory is user-supplied work, not evidence of agent-run refresh
or live compatibility testing. Do not read the local catalogue for validation.
This checkpoint does not authorize pushing or implementing phase 3.
That work was committed as `aa02c23c`. U59–U61 now authorize refining the living
proposal: keep `AzureTenantId` in `azure_types`, select option B's registry
boundary, rename runtime `credentials` to `auth`, and move `AuthSource` to
`auth_types`. Record the changes in the plan/contracts/inventory now; no package
rename, implementation, commit, push, or live validation is implied by this
design-selection checkpoint.
U62 then authorizes proceeding with task 3.0: complete the registry ownership
cutover and all consumers, preserving offline validation and full-cutover rules.
No commit, push, or production-data access is implied. Auth renaming/extraction
remains the following task 3.1.
U63–U65 authorize the enum-handle refinement within task 3.0. Replace the newly
introduced registration scans and copied adapter records, migrate every consumer,
and record offline evidence. This does not authorize committing, pushing,
starting auth migration, or reading production data.
U66 subsequently authorizes a local commit of the completed registry package
cutover and enum refinement, then a description of task 3.1. The reviewed staged
changes belong to that slice. No push or implementation of task 3.1 is requested.
U67 subsequently authorizes task 3.1's auth runtime/type cutover and its tenant,
identity, and sensitive-output contracts. No commit, push, live credential access,
or production-data validation is implied.
U68 selects persistent application-owned CLI authentication. Explicit login
creates a new generation; acquisition remains noninteractive and cannot import
the ambient CLI login. This permits implementation, not a live login for testing.
U69 requests an authorization enum with valid typed states. U70 requests an
explanation of the CLI allowlist. U71 proposes replacing policy remediation and
container listing with REST; U72 authorizes proceeding with the described ARM
implementation and removal of the service-command bridge. Preserve user staging
and the persistent CLI credential store. No commit, push, live login, or
production-data validation is authorized by this refinement.
U73 authorizes tightening the new container, remediation, and shared provenance
response records. An absent OpenAPI `required` declaration alone does not justify
`Option`; weaken a response field only when concrete evidence warrants absence.
Review nested records too, and simplify consumers whose checks move into decoding.
U74 authorizes committing the completed staged auth/ARM changes and these plan
checkpoint notes, then describing task 3.2. Preserve the staged scope; no push,
live validation, or next-phase implementation is part of this checkpoint.
U75 subsequently authorizes task 3.2's scoped execution contract, runtime provider
adapter, and minimal application binding. Continue using synthetic/offline
validation and preserve user edits. No new commit, push, credential access, or
production-data validation is implied by proceeding with this slice.
U76 authorizes the narrow execution-mode naming refinement within task 3.2.
Preserve the user's staging and the distinction between general work caching,
provider-owned token caching, and recursive authorization.
U77 authorizes reporting the existing nonfatal error-hook installation and
panic-hook retention failures on stderr, preserving user staging and startup policy.
U78 authorizes committing the reviewed task 3.2 implementation and its U76/U77
refinements locally, then describing task 3.3. Preserve the staged scope; no push,
next-phase implementation, or live validation is implied by this checkpoint.
U79 now authorizes task 3.3's registry invocation, discovery, and direct UI caller
integration. Preserve independent job cancellation and G2's separately tagged
legacy callback during the scheduled SDK migration. No commit, push, phase 4
implementation, credential lookup, or production-data validation is implied.
U80 authorizes renaming the anonymous provider and extracting a reusable
test-only unreachable provider. U81 authorizes cache-owned temporary-directory
cleanup while preserving caller-owned path storage. U82 authorizes committing
those changes with task 3.3, identifying the next phase, and beginning task 4.1.
The existing staged work was committed as `b00bff46`; new task 4.1 changes remain
uncommitted. No push or production-data validation is authorized by this request.
U83 subsequently requests committing the completed 4.1a changes, identifying the
next phase, and beginning it. The reviewed ten-file staged scope was committed as
`c981de2b`; task 4.1b starts from that clean checkpoint. Its first implementation
focus is shared executor artifact binding. No push, live service access, or
production-data validation is implied.
U84 repeats the commit, next-phase identification, and implementation request.
The reviewed six-file staged artifact-binding scope was committed as `811468f3`;
the working tree was clean afterward. Task 4.1b continues with owned uncached
execution, preserving non-`Sync` outputs and admission before the actual Command
API/caller cutover. New changes remain uncommitted; no push or live validation
is authorized by this continuation.

U85 again requests committing existing changes, identifying the next phase, and
implementing it. The six-file owned-work prerequisite was committed as
`5f627dfc`; no push was requested. U86 explicitly requires carrying the next
implementation task through its real callers and validation. The active scope
is all of task 4.1b, including replacement of ambient authenticated service
subprocesses, rather than another bounded prerequisite. New cutover work remains
uncommitted. Production-data and credential access remain prohibited for validation.

U98 is the current implementation authorization: complete task 4.2's REST
transport and caller migration, validate offline, and stop before committing for
review. Preserve existing staging and user edits; do not stage new work. The
transport cutover includes the SDK, CLI, UI, tools, and example callers needed to
remove the old Command-owned global cache. This does not authorize production
data access or phase 5 aggregate-cache implementation.

U99 additionally authorizes preserving custom errors across direct request
execution and converting them only where registry reflection erases the result.
It does not widen authentication, cache reuse, data access, or commit authority.

U100/U101 correct cached SDK response ownership, including equivalent request
paths beyond the agent-package example. Return shared records or collections
without cloning decoded data for the caller, and remove `Clone` derives/bounds
whose only purpose was that detachment. This brings the shared-record portion
of phase 5 forward into task 4.2b; aggregate memoization, dependency freshness,
remaining request-data extraction, and observer migration remain later work.

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

- **Phase 5 extraction — 2026-11-06:** U104 requests completion of the next phase,
  preserving the earlier instruction that beginning implementation means finishing
  the authorized scope. The next unchecked phase contains 5.1–5.3, all included.
  U103 preserves the intervening approved batch-entry consolidation.
- **Phase 5 traceability:** U104 maps to all three SDK slices, G4 observation,
  G6 memory-only aggregate/dependency behavior, synthetic integration tests,
  consumer compilation, and an uncommitted review checkpoint.
- **Phase 5 adversarial omission:** Retain explicit bearer tenant, PAT selection,
  admission before hits, no duplicate decoding or cache-driven record cloning,
  independent caller notifications, continuation recovery, existing uncached
  policies, and no production-data access. This audit uses available original
  instructions and the preserved ledger; it does not claim access to unavailable
  compacted assistant discussion. No endpoint behavior changes are intended.

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
  pushing the completed changes, and the full-cutover correction prohibiting
  compatibility re-exports, then the response-type dependency review and tooling
  extraction with explicit live-refresh selection, tenant-alias support, and
  reusable provider API ownership, default-catalogue refinement, and the tool's
  shared tenant-selection correction and default-output refinement, followed by
  the request to commit the completed changes and describe the next part.
  U01–U58 preserve the distinct instructions; optional source-location ownership, optional
  Vec<Arc<T>>, and a possible cache
  crate remain optional. The explicit-tenant correction supersedes the earlier
  assistant compatibility suggestion. U29 preserves the prohibition on production
  data access or ingestion into agent context; U30 adds workspace Clippy; U31
  requires collision prevention/detection while keeping a REST prefix a candidate.
- **Pass 2 — traceability (U58 checkpoint):** Verified all 58 ledger IDs have matching traceability
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
- **Full-cutover correction audit:** Extraction records U49 without narrowing it
  to a recommendation. Traceability replaces the auth-type re-export policy in
  G1/G2, task 3.1, and the public contracts; tasks 7.1/7.2 and the caller inventory
  cover remaining imports and documentation. Adversarial review preserves the
  completed registry extraction as historical evidence while requiring removal
  of its compatibility exports at cutover. Moving a type includes its consumers
  and removal of the old export in the same slice; no new compatibility exports
  may bridge phases. Pure registry adaptation remains a distinct responsibility.
  All 49 ledger/traceability pairs and 135 relative links across the three updated
  documents pass, along with privacy, code-fence, and whitespace checks. This is a
  documentation correction; no production code or test execution changed.
- **Response dependency audit:** Extraction records U50's direct-dependency
  concern and SDK relocation as a proposed solution, not a required destination.
  Traceability assigns task 2.5: the decoder only needs `Facet`, so reusable API
  records and pure conversion remain in `azure_types` while the command marker
  and direct dependency are removed. Adversarial review distinguishes that edge
  from the remaining normal dependency paths through `hcl_types` and
  `azure_resource_types`; it does not claim the whole crate is execution-free or
  supersede the lower auth vocabulary contract. Task 2.5 records the five passing
  offline tests, focused Clippy, formatting, and dependency evidence. All 50
  guidance/traceability pairs, relative links, fences, and privacy checks pass.
- **Types-crate extraction audit:** U51 requests moving thin execution concerns
  into their owning runtime/tool packages; U52 retains explicit live acquisition
  alongside offline generation. Tasks 2.6 and 2.7 have independent implementation
  owners for HCL and maintenance tooling, with integration and validation owned
  by the primary agent. The iconography checkout is a read-only pattern reference.
  No compatibility exports, generated inventory refresh, production access, or
  phase-3 auth implementation is part of this slice.
  Offline metadata verifies all three affected types crates have no normal/build
  path to `command`; the maintenance tool is unpublished and has no consumers.
  Source review and synthetic process checks cover feature-unified startup,
  while catalogue tests and CLI smoke checks cover rendering, publication, JSON
  output, formatter stability, and pre-authentication refresh rejection. All 52
  guidance/traceability pairs, 136 relative links, fences, and privacy checks pass.
- **Tenant-alias correction audit:** U53 broadens refresh input to the existing
  tracked-alias vocabulary. A validating argument excludes `default`; direct IDs
  retain their previous behavior, while aliases reuse the shared resolver and its
  ambiguity errors. No alias matching, credential-source preference, or implicit
  tenant selection is reimplemented in the tool. Task 2.7 owns the correction.
- **Provider API ownership audit:** U54 identifies service metadata and fetching
  as reusable SDK responsibilities. Task 2.8 assigns independent type extraction
  and SDK request owners, with tool integration and verification owned by the
  primary agent. Future registration support is motivation, not authorization to
  register providers now. Relative resource-type names remain distinct from the
  full generated vocabulary; unknown names and current service spelling must
  survive catalogue generation. A feature replacing public types with strings is
  an optional fallback proposal, not a required implementation.
- **Default-catalogue refinement audit:** U55 makes the catalogue argument optional
  across the three maintenance commands. Task 2.7 owns the shared default and
  declarative argument defaults. The selected implementation anchors the default
  to the compile-time tool manifest directory; explicit relative overrides retain
  working-directory semantics. Output selection, authentication requirements,
  and offline-validation limits remain unchanged. No catalogue content is created.
- **Shared tenant-selection correction audit:** U56 authorizes the maintenance
  tool's default tenant selection and removal of duplicate selection code. Task
  2.7 uses the existing shared argument/resolver, including tracked explicit UUIDs,
  aliases, and the Azure CLI default-account tenant. This supersedes the tool's
  earlier default rejection and direct-UUID bypass, while preserving required
  credential-source selection and the SDK's explicit resolved tenant contract.
  Validation stops at synthetic output preflight before tenant discovery.
- **Default-output refinement audit:** U57 adds the existing generated enum as
  generation's default destination. Task 2.7 applies one shared default to
  generate and check, anchored to the tool manifest directory just like the
  catalogue. Explicit relative overrides keep working-directory semantics.
  This supersedes U55's retained required-output behavior without changing
  catalogue content, authentication, or the prohibition on live validation.
- **Tooling commit checkpoint audit:** U58 authorizes a local commit of the
  completed type-boundary/tooling/provider API work and a description of task
  3.1. The staged generated inventory and local-catalogue ignore rule are user
  changes to preserve; earlier inventory-unchanged evidence predates that
  generation. Final checks use offline compilation and inspected synthetic tests,
  without opening the catalogue or claiming live refresh evidence. Pushing and
  phase-3 implementation remain outside this checkpoint.
- **Boundary revision, pass 1 — extraction:** Reread the available user messages
  asking whether the dependency cleanup removes the need to move tenant/source
  types, accepting `AzureTenantId` in `azure_types`, questioning registry and auth
  naming, requesting diagrams and refined candidates, and choosing B with
  `credentials` renamed to `auth` and `AuthSource` moved to `auth_types`.
  Added U59–U61 without changing the earlier guidance or completed evidence.
- **Boundary revision, pass 2 — traceability:** All 61 ledger IDs have matching
  traceability entries. U59–U61 map to revised G1, new task 3.0, tasks 3.1/3.3,
  the caller inventory, and final cutover criteria. Current manifests establish
  the source names; the new DAG is a target, not a claim of implemented packages.
  Task 3.0 owns pure conversion registrations as well as value constructors.
- **Boundary revision, pass 3 — adversarial omission:** Rechecked source versus
  runtime separation, explicit tenant binding, disabled auth, one authoritative
  registration, no compatibility exports, no production reads, and PII-free
  documentation. B changes ownership/names while retaining A's dependency
  boundary; C's larger execution crate remains unselected. No implementation or
  release is inferred from approval of the design. Existing request/cache
  lifecycle, cancellation, and invalidation decisions are unchanged.
- **Boundary revision validation:** Checked all 61 ledger/traceability pairs,
  150 relative links across the four migration documents, code-fence balance,
  whitespace, and identifying-path/account patterns. The proposed Mermaid
  dependency diagram has 10 nodes and no cycles. This checks the documented
  graph only; it does not replace 3.0/3.1's actual Cargo dependency validation.
  No Rust code, manifests, or lockfile changed, and no application/test or
  credential/service operation ran for this documentation update.
- **Known source limitation:** The user requests and durable U01–U58 ledger are
  available; earlier assistant/tool transcripts were compacted. Prior validation
  statements above are retained historical evidence, not newly repeated checks.
- **Registry implementation, pass 1 — extraction:** Reread the available user
  instructions through U62, including the full-cutover correction, accepted
  option B, auth naming, tenant ownership, and instruction to proceed. Added U62
  to the ledger and traceability; it authorizes task 3.0 without a commit, push,
  live validation, or unrelated changes.
- **Registry implementation, pass 2 — traceability:** Checked U01–U62 against
  their task coverage. Task 3.0 records the completed owning-crate cutover,
  authoritative metadata and invocation identity, package graph, and offline
  evidence. Tasks 3.1–3.3 retain the pending auth rename, executor/provider, and
  invocation ABI changes; implementation of those contracts is not claimed.
- **Registry implementation, pass 3 — adversarial omission:** Rechecked pure
  conversions as well as constructors, explicit associated-handle migration,
  metadata-only discovery, no compatibility exports or aliases, unchanged
  dependency pins, and preserved tenant/auth behavior. Validation used only
  inspected synthetic tests and compilation; production data and the local
  generated catalogue were not read. The completed slice remains uncommitted.
- **Registry implementation documentation validation:** All 62 ledger IDs match
  their 62 traceability rows. The four migration documents have 150 valid relative
  links, balanced code fences, and no identifying-path/account patterns. Task 3.0
  is complete; 3.1 is pending. Final source review found no blocking issue, and
  formatting plus whitespace checks passed after the cutover.
- **Enum refinement, pass 1 — extraction:** Reread the registration lookup
  question, the invalid-state/enum proposal, and the instruction to perform the
  proposed changes. U63–U65 capture constant-time access, exclusive variants,
  and implementation authorization without weakening U49's full-cutover rule.
- **Enum refinement, pass 2 — traceability:** U63–U65 map to G1 and task 3.0's
  refinement. The implementation must retain the lower registration boundary,
  direct descriptor access, original identity across copied handles, and runtime
  cancellation behavior. The previously recorded adapter implementation is
  historical; task evidence describes its replacement explicitly.
- **Enum refinement, pass 3 — adversarial omission:** Rechecked that wrapping a
  known registration introduces no lookup, allocation, or linked-membership
  requirement. Pure and arbitrary variants cannot carry runtime cancellation or
  ambiguous provenance. Existing custom-catalog admission, source errors,
  lifetime guarantees, metadata wire spelling, dependency pins, and user staging
  remain in scope for preservation. Auth migration remains task 3.1; no commit,
  push, or production-data validation is inferred from this refinement.
- **Enum-refinement documentation validation:** All 65 guidance IDs have matching
  traceability rows. The four migration documents retain 150 valid relative links,
  balanced code fences, and no identifying-path/account patterns. Task 3.0's
  implementation evidence includes the corrected test-module selections;
  task 3.1 remains pending. Formatting and whitespace checks passed.
- **Registry commit checkpoint, pass 1 — extraction:** U66 records the explicit
  request to commit completed changes and identify what comes next. The existing
  auth naming, tenant ownership, full cutover, and offline-validation decisions
  remain unchanged.
- **Registry commit checkpoint, pass 2 — traceability:** U66 maps to task 3.0's
  completed checkpoint and task 3.1's next-step description. The reviewed staged
  package moves, imports, handles, manifests, and docs form one implementation
  slice; the 96 selected offline tests, workspace check, and strict scoped Clippy
  from U65 remain its functional evidence.
- **Registry commit checkpoint, pass 3 — adversarial omission:** Preserve user
  staging in the authorized commit, exclude unrelated notes/generated data, and
  retain no compatibility exports. Do not interpret committing as permission to
  push, release packages, start auth migration, or access production data.
- **Auth migration, pass 1 — extraction:** U67 records the request to proceed
  with the next described slice. U68 records the selected persistent CLI store
  after the user requested an explanation of the two session lifetimes.
  Earlier tenant ownership, no compatibility exports, explicit source/tenant,
  caller cancellation, and no-production-data requirements remain active.
- **Auth migration, pass 2 — traceability:** U67 maps to task 3.1 and G1/G3;
  U68 maps to generation storage, explicit login, local identity validation,
  and the authentication guide. Provider injection remains 3.2 and shared cache
  adapter admission remains phase 4. Legacy unbound request migrations stay in
  5/6 and are enumerated instead of being silently treated as complete.
- **Auth migration, pass 3 — adversarial omission:** Rechecked the available
  original user requests and durable U01–U66 ledger: tenant ID stays in
  `azure_types`, source vocabulary moves to `auth_types`, runtime becomes `auth`,
  PAT does not gain a fabricated tenant, and CLI token acquisition never falls
  back to an active tenant or implicit login. Scope includes the selected
  persistent store but no actual login, production test, commit, or push.
  Earlier compacted implementation details rely on the recorded task evidence;
  this audit does not claim to have reread unavailable assistant turns.

- **ARM refinement, pass 1 — extraction (2026-11-03):** Reread the available
  authorization-enum request, allowlist explanation request, REST proposal, and
  explicit instruction to proceed. U69–U72 retain each distinct request. Earlier
  constraints remain in U01–U68 and their recorded audits; unavailable compacted
  implementation detail is not treated as newly reread evidence.
- **ARM refinement, pass 2 — traceability:** U69 maps to typed authorization
  variants and their focused validation. U70–U72 map to task 3.1a: request-owned
  ARM execution and reusable API records, container-picker reuse/invalidation,
  deletion of the service bridge, and preserved CLI credential sessions. The
  chosen management-plane container inventory follows the proposal approved by
  U72; it does not claim blob-data access. Parallel owners are explicit in 3.1a.
- **ARM refinement, pass 3 — adversarial omission:** Rechecked that the change
  removes the allowlist instead of relocating it, covers both remediation
  branches and both container-list paths, retains explicit tenant/source,
  protects mutations, and leaves staged changes untouched. Synthetic tests and
  source/schema verification establish local behavior; no production validation,
  executor implementation, commit, or push is implied.

- **Response-field correction, pass 1 — extraction (2026-11-03):** U73 covers
  the selected container identity record and the other new response types, not
  only one file. It requests required fields first, weakening them when actual
  omission is justified. Earlier scope and offline-only constraints still apply.
- **Response-field correction, pass 2 — traceability:** U73 maps to task 3.1a's
  container/remediation/provenance records, nested field review, and direct typed
  container-page decoding. Remaining optional fields need specific absence or
  conditionality evidence; schema omission of `required` is insufficient.
- **Response-field correction, pass 3 — adversarial omission:** Confirmed this
  correction removes speculative `Option` and empty `Default` records without
  inventing fallback values, erasing genuinely conditional data, adding JSON
  fixture tests, or requiring live requests. Historical verification remains
  labeled separately from the refinement's checks.

- **Commit checkpoint, pass 1 — extraction (2026-11-03):** U74 preserves both
  requested outcomes: commit the completed changes and describe the next plan
  phase. U67–U73's implementation and response-field corrections remain included.
- **Commit checkpoint, pass 2 — traceability:** U74 maps to the task 3.1a
  checkpoint, reviewed staged auth/ARM scope, and task 3.2's executor/provider
  description. Existing offline verification evidence remains attached to its
  actual implementation checkpoint rather than being reported as newly rerun.
- **Commit checkpoint, pass 3 — adversarial omission:** Confirmed that the
  checkpoint includes the required-field correction and full crate cutover, keeps
  runtime provider implementations outside `execution`, and leaves task 3.2
  unstarted. A local commit does not imply pushing or accessing production data.

- **Executor implementation, pass 1 — extraction (2026-11-03):** U75 records the
  instruction to proceed after the U74 checkpoint and task 3.2 explanation.
  Earlier detailed direction remains in the existing ledger; this update does
  not claim to re-review unavailable compacted messages.
- **Executor implementation, pass 2 — traceability:** U75 maps to task 3.2,
  G1/G2/G3, provider admission, application binding, and focused offline checks.
  Task 3.3 retains registry integration and phase 4 retains production adapters.
- **Executor implementation, pass 3 — adversarial omission:** Retain both execute
  spellings, the existing bounded cancellation owner, shared cache/session
  identity, and the lower crate's independence from `auth`. Public app context
  fields remain available until their scheduled consumer migration. Task 3.2
  records focused offline evidence; no production compatibility or downstream
  transport cutover is inferred from those checks.

- **Task 3.3 refinements and phase 4, pass 1 — extraction (U80–U82):** Preserve
  the user's provider names, reusable test-provider responsibility, and temporary
  cache cleanup ownership. The latest request explicitly orders commit, next-phase
  identification, then implementation. Earlier guidance remains in the preserved
  ledger; no reread of unavailable intervening conversation is claimed.
- **Pass 2 — traceability:** U80/U81 map to task 3.3's refinement checkpoint;
  U82 maps to commit `b00bff46` and task 4.1's first bounded implementation slice.
  Retries and input ownership are prerequisites for the shared lifecycle, not
  evidence that canonical decoded caching or executor-bound callers are complete.
- **Pass 3 — adversarial omission:** Retain full cutover without compatibility
  exports, caller-owned cancellation, final failure artifacts, explicit auth,
  sensitive-output exclusion, and offline-only validation. Do not mistake
  independent temporary input directories for single-flight cache coordination.
  Keep all unfinished task 4.1 acceptance criteria visible; no push is implied.
- **Task 4.1 continuation, pass 1 — extraction (U83):** The latest request again
  orders commit, next-phase identification, then implementation. The prior
  completed slice is 4.1a; the next existing task is 4.1b. Preserve the earlier
  explicit-executor, full-cutover, typed-sharing, cancellation, and privacy
  requirements in the ledger rather than treating this as a new design request.
- **Pass 2 — traceability:** U83 maps to checkpoint `c981de2b` and the artifact
  binding prerequisite inside 4.1b. Source review establishes that adapters cannot
  recover the executor's private active `WorkAttempt`; a shared binding method
  must carry its admitted identity and effective disk-reuse policy.
- **Pass 3 — adversarial omission:** Keep `ExecuteEveryTime` independent even
  with positive requested validity; disabled raw retention is separate from
  decoded memory sharing. Preserve admission checks and original disk freshness.
  The owned non-`Sync` path, Command callers, and REST remain explicit follow-ups;
  the new boundary must not be described as a completed transport cutover.
- **Owned execution continuation, pass 1 — extraction (U84):** Commit the current
  artifact-binding changes, identify the next unfinished task, then implement it.
  The available request and preserved ledger retain explicit resources, caller
  cancellation, canonical type ownership, and non-`Sync` owned outputs.
- **Pass 2 — traceability:** U84 maps to checkpoint `811468f3` and task 4.1b's
  owned-work prerequisite. Existing Command output bounds allow `Send` without
  `Sync`; forcing uncached decoding through an `Arc<T>` cache would narrow that
  contract. Plain owned execution also preserves invocation-borrowed results.
- **Pass 3 — adversarial omission:** Uncached work still requires admission and
  authentication rechecks. Reusable parents must reject it before effects and
  cannot cache a caught-error fallback. Artifact ownership remains canonical,
  even without memory reuse. Keep the 42-file caller migration and provider-bound
  subprocess behavior visible; do not add a parallel Command adapter or claim a
  completed transport cutover. Validation remains synthetic and local.

- **Command cutover, pass 1 — extraction (U85–U86):** Commit the existing owned
  execution changes, identify the remaining Command task, and complete its
  implementation. The user's clarification expressly rejects stopping after
  another prerequisite. Preserve all earlier no-compatibility, explicit-resource,
  cancellation, type-ownership, and no-production-data requirements.
- **Pass 2 — traceability:** U85 maps to checkpoint `5f627dfc`; U86 maps to all
  remaining 4.1b acceptance criteria, production consumers, and offline checks.
  Azure/DevOps service subprocesses must move to explicit-context REST because
  restoring an ambient or generic CLI service runner violates the approved auth
  boundary. Local configuration, Terraform, and Gitea receive the real executor.
- **Pass 3 — adversarial omission:** Owned and shared Command APIs must cut over
  together. Preserve borrowed caller policy, decode-before-publication, exact
  rejection, sensitive-output suppression, and required cancellation completion.
  Do not manufacture executors from scopes or claim all REST/SDK migration is
  complete. Tea and Terraform retain externally managed process credentials;
  executor anonymous admission is not a subprocess security sandbox. Tea selects
  an explicit matching profile and its service results remain nonreusable.
  Earlier conversation requirements use the preserved ledger and available
  history; no reread of unavailable material is claimed.

- **Dependency correction, pass 1 — extraction (U87–U88):** The user identifies
  unused executor parameters and cancellation delegation in HCL audit/writer as
  excessive coupling, then authorizes the correction. Preserve executor value
  for actual auth/cache work while restoring narrow local helper dependencies.
- **Pass 2 — traceability:** Task 4.1c owns scope-only Command APIs, explicit
  artifact variants, HCL/workflow/configuration callers, tests, and documentation.
  Earlier requirements for explicit managed ownership and no hidden globals
  remain; they do not require promoting every physical operation to an executor.
- **Pass 3 — adversarial omission:** Preserve shared and independent managed
  admission, sensitive suppression, caller attribution, and required completion.
  Do not replace the executor with a parameter bag, construct hidden executors,
  or remove useful child binding for genuinely managed operations. No production
  data validation, commit, or push is authorized by this correction. Earlier
  intent is checked against the preserved ledger; this audit uses the available
  U87/U88 messages and does not claim to reread unavailable earlier material.
- **License refinement, pass 1 — extraction (U89):** Unify the newly introduced
  account-license enum with the existing combined license model, avoiding
  duplicated account/MSDN vocabularies.
- **Pass 2 — traceability:** The task 4.1b REST replacement refinement below owns
  shared family types, combined serialization, response/update consumers, and CLI
  matching. Task 4.2 remains pending.
- **Pass 3 — adversarial omission:** Preserve combined strings and aliases while
  keeping REST family fields distinct. Match group rules by licensing source;
  reject unmodeled mutation selections before authentication. Preserve future
  response values without changing their spelling. Use only synthetic validation,
  preserve prior work, and do not commit or push. This audit uses U89 and the
  preserved ledger, without claiming access to unavailable conversation material.

- **REST setter refinement, pass 1 — extraction (U90):** Preserve the user's
  `auth_context(&AuthContext)` rename; add `azure_auth_context` accepting the
  tenant-bound Azure context and transferring both source and tenant.
- **Pass 2 — traceability:** The REST setter refinement below owns the helper,
  Azure DevOps bearer delegation, and existing tenant-context callers.
- **Pass 3 — adversarial omission:** Preserve user staging and existing work;
  do not manufacture contexts for token-only or tenant-only legacy operations,
  change SDK signatures, acquire credentials, or broaden into task 4.2. This
  audit uses U90 and the preserved ledger.

- **JSON/cancellation refinement, pass 1 — extraction (U91–U92):** Include
  asynchronous Azure CLI program resolution in the supplied cancellable child;
  replace opaque domain `RawJson` storage with the existing `ArbitraryJson` and
  audit other uses before configuring a lint reminder.
- **Pass 2 — resolution:** Keep raw JSON for deliberate codecs and transport
  representations. Avoid new dependency cycles for HCL and lower-level crates;
  document narrow exceptions. Preserve custom generator relationships and object
  shapes even when the shared wrapper provides unconstrained arbitrary JSON.
- **Pass 3 — scope audit:** This refinement does not complete service schemas,
  move the JSON wrapper between crates, change optionality, or begin REST task 4.2.
  Verify with local compiler/lint probes and reviewed synthetic tests, without
  service requests or production data. Audit uses U91–U92 and the preserved ledger.

- **Request ergonomics refinement, pass 1 — extraction (U93–U95):** Inline the
  Command JSON helpers, restore `.with_invalidation(bool)` on bound requests,
  and use a shared key trait for generic invalidation instead of repeating
  inherent key methods and invalidation implementations.
- **Pass 2 — traceability:** Task 4.1c's refinements record the removed helpers,
  `cache::HasCacheKey`, the blanket `CacheInvalidatableRequest` implementation,
  migrated request callers, and offline execution-policy tests.
- **Pass 3 — adversarial omission:** Keep request-owned scopes and delegated
  invalidation for default selections, both execute spellings, caller tracing,
  cancellation, and borrowed/non-`Sync` support. Preserve staging and unrelated
  changes. The legacy Command trait uses a different key and cannot become an
  execution dependency; REST task 4.2 remains separate. The latest user messages
  and current source support this refinement; earlier completed work retains its
  recorded evidence.

- **Invalidation policy refinement, pass 1 — extraction (U96):** The RBAC
  composition illustrates invalidation of several child keys. Preserve three
  choices: the own-key default, an explicit key list, and arbitrary async logic.
  A request with `HasCacheKey` must remain eligible for all three alternatives.
- **Pass 2 — traceability:** Task 4.1c uses an explicit `InvalidatesOwnCacheKey`
  marker, a `CacheInvalidationKeys` list contract, and the async
  `CacheInvalidatableRequest` contract. Tests exercise multi-key refresh while
  retaining unrelated entries, and a keyed request with custom async behavior.
- **Pass 3 — overlap audit:** A blanket list implementation for all `HasCacheKey`
  types would merely relocate the same Rust coherence conflict. Only explicit
  own-key opt-in receives that default. The three choices are alternatives;
  key ownership alone commits to none. Legacy RBAC remains part of its scheduled
  SDK migration rather than mixing old/new cache keys in this refinement.

- **Commit checkpoint, pass 1 — extraction (U97):** Review the completed changes,
  commit them locally, and identify the next phase and its scope.
- **Pass 2 — traceability:** Task 4.1c's checkpoint records the staged migration,
  review fixes, and offline validation. Task 4.2 remains the next implementation
  step, followed by the SDK vertical slices in phase 5.
- **Pass 3 — authorization audit:** The existing staged scope belongs to the
  completed Command migration and its reviewed refinements. Preserve unrelated
  notes and dependency pins; no push, publication, production-data access, or
  next-phase implementation is requested. Review fixes remain in this checkpoint.

- **REST cutover, pass 1 — extraction (U98):** The latest instruction authorizes
  the complete next implementation phase and its callers, stopping before a
  commit for review. Preserve explicit source/tenant selection, cancellation
  ownership, full cutover without compatibility exports, and offline-only checks.
- **Pass 2 — traceability:** Task 4.2 covers owned and shared REST execution,
  canonical decoding, provider-owned Graph/ACR authorization, modern request
  invalidation, removal of the global Command cache, and all affected consumers.
  Workspace all-target check/Clippy and focused offline verification passed;
  task 4.2 is complete at the uncommitted review checkpoint.
- **Pass 3 — boundary audit:** Transport `Arc` reuse does not establish shared
  assembled SDK inventories. Phase 5 retains aggregate dependency freshness,
  remaining runtime-auth extraction, and domain record sharing. Existing live
  tests were adapted for compilation only; no live service, real credential,
  user-configuration, or production-cache access is evidence for this cutover. Historical ledger
  entries retain their checkpoint authorization rather than becoming current
  restrictions. This update uses the preserved ledger and available task
  instructions; it does not claim to reread unavailable intervening conversation.

- **Request errors, pass 1 — extraction (U99):** Preserve operation-specific
  failures through ordinary execution rather than forcing every request to
  return an erased report. Cancellation still needs a conversion into that error.
- **Pass 2 — traceability:** Task 4.2a and contract G2 own the associated error,
  generic execution future, existing-request cutover, picker handling, and the
  registry adapter's explicit report conversion. Focused regressions cover
  recoverable data, cancellation, and structured error sources at erasure.
  All 63 selected offline tests and workspace all-target check/Clippy passed.
- **Pass 3 — boundary audit:** Direct errors need only the execution lifetime,
  `Send`, and cancellation conversion. Registry boxing separately requires
  `'static` and `Into<eyre::Report>`; it must not impose `Sync` or
  `std::error::Error` on all requests. Invalidation's report conversion belongs
  only to the combinator that needs it. Cache sharing and phase 5 scope remain
  unchanged. The refinement has its own verification evidence; prior task 4.2
  results remain historical. User staging is preserved and work is uncommitted.

- **Shared SDK values, pass 1 — extraction (U100/U101):** The agent-package
  example exposes a cached response immediately cloned back to an owned value.
  Return its shared value and audit equivalent paths; eliminate response
  `Clone` requirements added solely to support that copy.
- **Pass 2 — traceability:** Task 4.2b, G4, and the caller inventory cover
  transport conversion, SDK collection shapes, consumer borrowing/sharing,
  removed cloning bounds, and offline ownership regression checks. Verification
  is pending for this refinement; earlier task results are not reused as proof.
- **Pass 3 — boundary audit:** Preserve ordinary uncached execution, request
  identity and wire JSON, failure artifacts, and intentional owned domain
  projections. Do not replace a copied record with a freshly allocated `Arc`
  on each hit, or impose `Clone` through a generic convenience path. Returning
  shared page records does not finish aggregate caching or dependency tracking.
  No production-data access, staging, or commit is authorized. Historical intent
  continues to use the preserved ledger where original messages are unavailable.

### U109–U111 refinement intent audit — 2026-11-06

- **Extraction:** The latest request asks to consolidate repeated test
  `AuthenticationProvider` implementations, add shared atomic call counting,
  import `Facet`/`Arbitrary` derive names, and automatically register collection
  forms with their underlying values. These remain three distinct requirements.
- **Traceability:** The phase 5 review refinement below maps them to the shared
  execution test provider, workspace/generator imports, and the two registry
  registration macros, with offline behavioral verification.
- **Adversarial omission:** Preserve the strict unreachable guard; keep only
  credential-lineage probes whose operation is itself under test. Do not impose
  `Clone` or `Sync` on owned registrations, recursively expand collections, lose
  explicit named constructor identities, or reintroduce qualified derives when
  generating resource types. Preserve staging and leave changes uncommitted.
  Earlier intent uses the existing ledger; no new historical audit is claimed.

### U112–U113 scoped-join intent audit — 2026-11-09

- **Extraction:** The two latest requests ask about fail-fast joins bypassing
  deferrals, scoped sibling cancellation, possible lint enforcement, clearer
  registry registration-point names, committing current work first, and a helper
  with only a few migrated callers before reviewing a broader rollout.
- **Traceability:** U112's registry names are in checkpoint `1e63a553`; the scoped
  join preview below implements U113. The linked review records the JoinSet audit
  and future scheduling/error-policy requirements.
- **Adversarial omission:** Keep the prototype uncommitted, retain custom errors
  and borrowed results, avoid unnecessary executor coupling, and do not migrate
  all joins or enable a workspace ban before interface review. Existing data-access
  restrictions continue to apply. Earlier guidance uses the preserved ledger;
  this is not a new audit of unavailable historical messages.

### U115 iterator mapping intent audit — 2026-11-09

- **Extraction:** The latest design request and approval move mapping onto
  iterators, retaining a visible executor argument and iterator composition.
- **Traceability:** The iterator refinement below adds `ConcurrentIteratorExt`,
  migrates all mapping callers/tests, and removes the executor mapping method.
- **Adversarial omission:** Keep scope-only use independent of executor resources,
  retain child-first callbacks and complete-all policies, share coordination, and
  preserve borrowed futures, custom errors, staging, and the no-commit boundary.
  Earlier guidance relies on the existing ledger rather than a new historical audit.

## Authoritative user guidance ledger

**U116 intent audit (2026-11-09):** Make all UUID newtypes consistently `Copy`
and remove redundant clones, including project IDs in concurrent audit mapping.
Audit both tuple and named-field UUID wrappers across the workspace; keep
string-backed identities, resource records, and shared `Arc` ownership unchanged.
Validate callers by compilation and Clippy without running service tests or
adding tests of standard derive behavior. Preserve staging and leave edits
uncommitted. Earlier guidance remains recorded in the existing ledger.

**U79 intent audit (2026-11-03):** Extraction preserves the latest instruction to
proceed after the task 3.2 checkpoint. Traceability maps it to task 3.3, G2's
borrowed invocation ABI, offline discovery, and direct UI consumer integration.
The adversarial pass retains separate job cancellation, shared resources, pure
registry boundaries, explicit auth selection, and the scheduled legacy callback
removal; it does not expand authorization to later adapters or live validation.
Earlier design details use the existing ledger where intervening conversation
has been compacted; no reread of unavailable material is claimed.

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
| U18 | Azure CLI credentials must have an explicit tenant in the request; do not depend on CLI default auth/tenant/DevOps setup. | Remove active-tenant compatibility from executable paths; preserve explicit organization/project scope. | U56 permits default selection in the maintenance tool before constructing an explicitly tenant-bound SDK request; request contract unchanged |
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
| U49 | Full cutover, no compatibility exports. | Move every consumer to the owning crate/API when moving a type, remove old-path exports in the same slice, and remove existing migration compatibility exports before completion. | — |
| U50 | The only apparent `azure_types` command dependency is in `query_response.rs`; moving it to the Azure SDK may remove that dependency more easily. | Verify actual use and remove the unnecessary direct dependency while retaining reusable API records in their owning types crate; distinguish remaining transitive coupling when assessing the auth crate boundary. | — |
| U51 | HCL formatting and resource-type generation are thin command dependencies; move formatting into `hcl` and generation into standalone `tools/` tooling following the iconography pattern. | Keep pure HCL conversions and generated resource types in their existing crates; extract execution and maintenance responsibilities and verify normal dependency paths. | — |
| U52 | Keep offline generation plus an explicit live refresh command. | Require explicit tenant and authentication source for refresh, save a reusable catalogue, and validate only with synthetic/offline inputs. | U56 allows the tool's shared default tenant selection; explicit source and offline-validation rules remain |
| U53 | Refresh should support existing Cloud Terrastodon tenant aliases as well as UUIDs. | Reuse shared alias parsing/resolution before credential acquisition; retain explicit tenant/source selection and existing direct-UUID behavior. | U56 uses the shared default/UUID/alias resolver, including its tracked-ID check; alias support remains |
| U54 | Provider API response types and operations should live in `azure_types`/`azure`, with a thinner maintenance tool; future tools may inspect registration or register providers. The existing generated enum and `Other(String)` allow bootstrapping. | Extract public reusable records and tenant-scoped listing, leave catalogue/file concerns in the tool, preserve unknown names and wire spelling, and keep future registration mutations separate. No string-backed feature is required if the existing fallback suffices. | — |
| U55 | Make the catalogue argument optional, defaulting to the maintenance tool's `catalogue.json`. | Use one manifest-directory default for refresh/generate/check, retain explicit path overrides and required output selection, and do not create a catalogue as part of the implementation. | U57 replaces required output selection with a shared generated-source default; catalogue behavior remains |
| U56 | Allow the maintenance tool to use default tenant selection and remove duplicate tenant-selection code. | Use the existing shared tenant argument and resolver for omitted/default, UUID, and alias selection; retain explicit auth source and bind the resolved UUID before SDK execution. | — |
| U57 | Give generation a default output path targeting the existing resource-type enum source. | Default generate/check output to the generated source relative to the tool's build manifest directory, preserve explicit overrides, and validate without replacing the checked-in inventory. | — |
| U58 | Commit the completed changes and indicate the next part of the plan. | Commit the completed tooling/type-boundary/provider API refinements while preserving user changes, then describe task 3.1; do not push or begin its implementation. | — |
| U59 | Reassess moving tenant/source types after removing command dependencies; keeping AzureTenantId in azure_types makes sense. | Keep the tenant ID in its domain crate and remove its dependency on executor-aware registrations instead of relocating it to auth vocabulary. | — |
| U60 | Explain the registry split and auth naming, include diagrams, and refine the proposal with different candidates together. | Record the current-versus-target graph, responsibility boundaries, and A/B/C tradeoffs in G1; distinguish runtime calls from package dependencies. | — |
| U61 | Select option B with AuthSource in the types crate; rename credentials to auth and credentials_types to auth_types. | Pure registry absorbs registry_values, registry_execution owns executor-dependent invocation, auth owns credential runtime, and auth_types owns AuthSource and resolved selections. Perform full owning-crate cutovers without compatibility exports. | — |
| U62 | Proceed after recording option B and the auth/auth_types naming decision. | Implement next task 3.0, including all registry consumers and offline validation, then report the following slice; no commit or live data access is implied. | — |
| U63 | Explain why a previously direct registration handle now requires a linear scan over the function registry. | Preserve direct access to known registrations; eliminate adapter lookup from handle conversion rather than accepting the introduced complexity and failure path. | — |
| U64 | Optional registration provenance fields allow invalid combinations; would Function be better represented as an enum? | Model runtime, pure, and arbitrary registrations as exclusive variants; reference original records without copying metadata or maintaining nullable provenance. | — |
| U65 | Perform the proposed enum-handle changes. | Refine task 3.0: pass copyable handles by value, remove adapter allocation/scans, preserve variant/source identity and runtime cancellation, migrate consumers, and validate offline. | — |
| U66 | Commit the completed changes and identify what comes next. | Commit the reviewed registry cutover and enum refinement locally; describe task 3.1 without starting it or pushing. | — |
| U67 | Proceed after the task 3.1 description. | Implement the auth runtime/type cutover, explicit selections and identity policy; preserve full-cutover and offline-only validation constraints. | — |
| U68 | Use a persistent Cloud Terrastodon login store after elaborating on the isolated CLI session alternatives. | Explicit login publishes a new app-owned generation, later invocations reuse its identity, existing providers retain their binding, and acquisition never prompts or uses ambient CLI credentials. | — |
| U69 | Authorization should be an enum to make invalid states unrepresentable. | Anonymous, bearer, and PAT variants use distinct validated payloads; preserve sensitive headers and redacted diagnostics without raw-header construction bypasses. | — |
| U70 | Explain the policy-remediation/storage-container command allowlist. | Record its temporary identity-binding purpose and the misplaced service-specific policy in auth; this explanation alone does not authorize a new implementation. | — |
| U71 | Could policy remediation creation and storage container listing use REST to remove the allowlist? | Verify REST equivalents and identify the ARM inventory behavior choice, reusable container request, removable bridge, and retained CLI credential lifecycle. | — |
| U72 | Proceed with the described REST conversion. | Implement typed ARM remediation and container inventory, reuse the latter in the picker, remove the service bridge, preserve explicit auth and offline-only verification; no commit/push. | — |
| U73 | Prefer required response fields and weaken them only when omission is justified; apply this to the other new types as well as StorageAccountBlobContainer. | Review all new ARM response records, including nested and shared provenance types; require ordinary response fields, document concrete conditional/omitted exceptions, and simplify consumers. A missing OpenAPI required list alone is insufficient. | — |
| U74 | Commit changes and describe the next phase of the plan. | Commit the reviewed auth/ARM work and response-field refinement locally, then explain task 3.2's scoped executor and injected provider; do not push or begin that task. | — |
| U75 | Proceed after the auth/ARM checkpoint and description of the scoped executor phase. | Implement task 3.2's executor, injected authentication provider, and minimal app binding, preserving bounded cancellation and offline-only validation; registry and transport migrations remain later tasks. | — |
| U76 | Improve the execution variants' names, suggesting normal execution and a mode disabling caching and reentrance. | Name the type `ExecutionMode` with `Normal` and `NoWorkCacheOrAuthorizationReentry`; document exactly which capabilities are restricted without changing nested execution or sensitive token caching. | — |
| U77 | Print hook-setup errors with `eprintln!` instead of silently discarding them. | Report eyre hook installation errors and describe an already-initialized panic-hook retention conflict; preserve nonfatal startup and fallible hook construction. | — |
| U78 | Commit changes and identify the next implementation phase. | Commit the completed executor/provider/app slice and its reviewed refinements locally; identify task 3.3 and the following transport phase without starting them or pushing. | — |
| U79 | Proceed with the next described implementation task. | Implement task 3.3's explicit registry invocation and offline discovery; update direct UI consumers while preserving shared resources and cancellation ownership. | — |
| U80 | Rename DisabledAuthenticationProvider to AnonymousAuthenticationProvider and extract UnusedAuthentication as a reusable UnreachableAuthenticationProvider for tests. | Make anonymous admission explicit and share the fail-on-use provider through opt-in test support rather than coupling it to discovery. | — |
| U81 | Preserve temporary-directory cleanup when WorkCache owns its root, alongside normal path storage. | Retain the TempDir guard in shared cache ownership through clones, artifacts, and workers; keep path-only roots caller-managed. | — |
| U82 | Commit changes, identify the next phase, then begin its implementation. | Commit the reviewed task 3.3/refinement scope, explain phase 4's Command-first integration, and implement/test the first bounded task 4.1 slice without claiming the full adapter cutover. | — |
| U83 | Commit changes, identify the next phase, then begin its implementation. | Commit completed task 4.1a locally, identify 4.1b, and begin the shared executor artifact boundary required by the Command cutover; retain the unfinished caller and transport acceptance criteria. | — |
| U84 | Commit changes, identify the next phase, then begin its implementation. | Commit the completed artifact-binding prerequisite, continue task 4.1b with owned uncached execution and explicit admission, and preserve the remaining Command API/caller cutover. | — |
| U85 | Commit changes, identify the next phase, then begin implementation. | Commit owned execution as 5f627dfc and carry out the remaining Command cutover. | — |
| U86 | Begin implementation means do the work, not the first part and then pause. | Complete task 4.1b across its API, production callers, validation, and plan evidence; do not stop at another prerequisite. | — |
| U87 | Keep cancellation-only helpers independent of the executor; HCL audit, formatting, and writing illustrate excessive coupling. | Restore scope-only local execution and narrow affected callers. Executor cancellation wrappers are for child operations that actually need executor resources. Retain explicit cache/auth/artifact boundaries. | — |
| U88 | Perform the dependency-boundary correction. | Complete task 4.1c, including affected callers, regression checks, and plan evidence before continuing REST. Preserve the existing uncommitted work; no commit or push requested. | — |
| U89 | Unify the new account-license type with the existing Azure DevOps license type. | Reuse shared account/MSDN family types in the combined selection, REST responses, updates, and CLI consumers. Preserve family identity and existing combined text. | — |
| U90 | Keep `auth_context` for an unbound context; make `azure_auth_context` accept the Azure tenant context and set source/tenant together. | Centralize the transfer in REST and simplify existing bound-context callers, preserving the user's rename. | — |
| U91 | Put Azure CLI program resolution inside the cancellable scope. | Keep asynchronous preparation within the linked child operation in local account discovery. | — |
| U92 | Reuse `ArbitraryJson` for opaque values, audit `RawJson`, and consider a Clippy reminder. | Replace opaque domain storage, preserve generator constraints, and configure a workspace warning with explained wire-boundary exceptions. | — |
| U93 | Inline the Command JSON helpers because their indirection is not justified. | Remove the helper module and exports, using the underlying JSON operations at callers without changing error handling. | — |
| U94 | Restore `fetch_all_gitea_organizations(tenant, executor).with_invalidation(invalidate).await`, including equivalent migrated requests. | Add lazy invalidation on the bound execution and simplify callers while preserving each request's scope. | — |
| U95 | Use `HasCacheKey` to blanket-implement `CacheInvalidatableRequest` for executable requests. | Centralize the modern key contract in `cache`; the direct blanket is replaced by U96's selectable invalidation policies. | U96 |
| U96 | Separate single-key, multiple-key, and arbitrary async invalidation so having a key does not prevent custom behavior. | Use explicit own-key opt-in, a key-list contract for compositions, and the async contract for custom procedures; avoid overlapping blanket implementations. | — |
| U97 | Review and commit the completed changes, then identify what the next phase encompasses. | Review the staged Command migration/refinements, correct findings, validate offline, commit locally, and explain REST task 4.2 without beginning it. | — |
| U98 | Proceed with the full implementation, stopping before committing for review. | Complete task 4.2, its SDK/CLI caller cutover, and offline validation; leave changes unstaged and uncommitted. | — |
| U99 | Preserve custom errors through request execution, including recoverable result data. | Add an associated request error and generic execution future; convert to reports only at erased registry invocation, with focused cancellation and source-preservation checks. | — |
| U100 | Return cached `Arc` values instead of cloning them back into owned request outputs; audit equivalent paths and leave copy decisions to consumers. | Task 4.2b updates cached SDK responses, pagination/composition, and downstream borrowing/sharing; fixed wire-to-domain conversion happens inside canonical decoding. | — |
| U101 | Remove `Clone` derives and bounds introduced only to support copying cached response values. | Audit response envelopes and domain records alongside their callers; retain cloning only for a separate meaningful contract or intentional owned transformation. | — |
| U102 | Remove unnecessary type parameters from domain-specific response wrappers, including provider operations and the other eleven identified wrappers. | Each response names its concrete shared domain records; update SDK decoders and preserve wire attributes, optionality, pagination, and reusable generic envelopes. | — |
| U103 | Consolidate the duplicate ARM batch entry and serialization entry types. | Serialize the public entry directly through a borrowed outer body; remove body-copy `Clone` requirements. Completed in `d2a5e4f2`. | — |
| U104 | Complete the next plan phase and leave changes uncommitted for review. | Complete 5.1–5.3 with explicit request data, shared aggregate caching/dependencies, observer semantics, consumer cutover, and offline verification. Do not stop at a prerequisite or stage/commit the result. | U113 supersedes the commit restriction for the completed checkpoint only |
| U105 | Preserve the existing Cow organization-field strategy and explain why it changed. | Restore borrowed project-list construction and invalidation; serialization must not force callers to clone their URLs. | — |
| U106 | Fix the field-level bug in the Facet fork; consult the update guide. | Correct generic field-proxy derivation and generic Cow decoding upstream in the owning forks; remove application workarounds and verify consumers with a single compatible core identity. | — |
| U107 | Publish the corrected Facet/Figue families and required dependents; update Cloud Terrastodon and the CLI template without patches or duplicate Facet identities. Commits/pushes are authorized for the named fork/template repositories. | Release only the required compatible dependency closure, verify registry identities and consumers, preserve Cloud Terrastodon staging and uncommitted review changes. | Supersedes U106's local-only release boundary. |
| U108 | Apply the supplied Git identity, correct release author metadata, and merge Cancellation/Tracing release branches into `main`; later explicit approvals include affected Format history/feature/tag refs and Tracing's GitHub default branch. | Preserve source contents, published archives, and unrelated history; verify the resulting trees and refs and retain Cloud Terrastodon staging. Do not include identity values in the plan. | — |
| U109 | Reuse built-in authentication test support instead of repeating test providers; add shared atomic call counts where needed. | Consolidate counting, offline authorization, revocation, and partition fixtures; retain the strict unreachable guard and specialized credential-lineage probes. | — |
| U110 | Use imported names rather than qualified `arbitrary::Arbitrary` or `facet::Facet` derives. | Normalize workspace derives and generator output while preserving conditional imports. | — |
| U111 | Register `Arc<T>` and `Vec<T>` forms automatically with `T`, rather than registering them individually. | Supply a finite collection family from both value and arbitrary-constructor registrations; preserve owned-only bounds and explicit constructor identities. | — |
| U112 | Investigate whether fail-fast joins drop required completion, consider scoped sibling cancellation and enforcement, and distinguish registry registration points from the expanded catalog. | Rename raw registry slices and audit tuple/JoinSet completion behavior; treat linting as a proposed follow-up. | — |
| U113 | Commit current changes first, then implement the helper and a few call sites for review before migrating all callers; investigate JoinSet use too. | Checkpoint `1e63a553`; scope-only joins and executor adapters at three Azure sites, offline completion tests, and a dynamic-set audit. Leave this preview uncommitted. | U114 supersedes the limited rollout boundary |
| U114 | Implement bounded concurrent mapping, apply the join/map helpers throughout suitable callers, and identify what remains. | Extend scope-only/executor helpers, migrate fixed joins and finite batches including queue wrappers, preserve complete-all policies, enforce fail-fast macro prohibition, and audit remaining coordinators. Keep changes uncommitted. | U117 authorizes the completed checkpoint |
| U115 | Make concurrent mapping an iterator extension with an explicit executor argument; approve the proposed scope-only variant and removal of the executor mapping method. | Add `ConcurrentIteratorExt::map_concurrent` and `map_concurrent_scoped`, migrate all callers, and keep one private coordination implementation. Preserve cancellation, error policies, borrowing, and staging without compatibility exports. | — |
| U116 | Implement `Copy` for UUID newtypes and remove redundant ID clones. | Audit UUID wrappers, add missing derives, and update affected callers using compiler diagnostics. Preserve non-UUID ownership and validate offline. | — |
| U117 | Commit the accumulated changes after the approved concurrency refinements and removal of the unused cache-population workflow. | Create a local checkpoint of the reviewed staged changes; retain the separate interactive shutdown follow-up. No push requested. | Supersedes the checkpoint's no-commit boundary |
| U118 | Proceed with the next planned interactive cancellation follow-up. | Complete picker generation cancellation/draining and background shutdown ownership, propagate explicit child scopes, and validate without production data. Leave changes uncommitted. | U120 authorizes its checkpoint |
| U119 | Review whether UI loadable work is superseded by the cache, whether deferrals remove required-work classification, and whether memory-only caching is supported and useful. | Distinguish loading/error presentation, task ownership, and successful-result reuse. Confirm existing memory-only aggregate caching; recommend sharing UI adapters and removing unused delivery flags. | — |
| U120 | Commit the existing work, then perform the recommended cleanup. | Commit U118 first; consolidate command/egui loading and task implementations, remove unused delivery-policy flags, preserve scoped completion and final settings, validate offline, and leave cleanup uncommitted. | — |
| U121 | Explain the introduced `State: 'static` bound on the shared work API. | Explain that direct typed channel capture exposes the bound previously supplied by the sender adapter; it constrains borrowed fields, not the state instance's allocation lifetime. No code change requested. | — |
| U122 | Commit changes and identify the next comprehensive phase to complete. | Commit the reviewed UI cleanup, then describe all remaining Phase 6 responsibilities and acceptance criteria, recognizing completed foundations. Do not start implementation or push. | U126 revises the next-phase ordering; checkpoint remains complete |
| U123 | Design account selection distinct from auth source and tenant: support two users in one tenant, consider multiple workload identities, DevOps bearer/PAT and organization independence, and Gitea/service accounts. Provide runtime account management; top-level CLI placement and exact model are proposals. | Companion account model and gates A1–A4; 6.0a/6.0c/6.0d/6.0e cover profile/storage/provider lifecycle, request intent, cache admission, and CLI. Do not assume a tenant or organization uniquely identifies an account. | — |
| U124 | Separate tests that use actual credentials/APIs from Cargo tests. Model intentional runtime checks as registered executable request types and a possible `ct test` surface; consider a test marker, read/write/interactive filtering, local-file effects, and debug strict deserialization coverage. | Companion verification design and V1; 6.0b/6.0e classify individual tests, preserve offline coverage, and implement explicit scoped verification. Test purpose versus effect is a recommendation to settle, not a confirmed new `Effect::Test` variant. | — |
| U125 | Remove functions whose only purpose is constructing/binding requests; prefer request construction followed by `.execute(executor)` at callers. Avoid duplicate helper APIs and executor-last positional plumbing. | 6.0d family cutover, existing ExecutableRequest API, preservation of borrowing/errors/invalidation/registration, and classification of real workflows/scope-only helpers. No compatibility forwarding exports. | — |
| U126 | Pause the previously described next phase for brainstorming, analysis, and updating the plan with these concerns before implementation. | Current authorization and revised Phase 6 ordering; retain completed foundations and pending audit/app work. This request changes documentation only and does not authorize live testing or committing. | — |
| U127 | Account aliases are scoped: reuse names such as admin/user in different tenants or service instances. | Gate A1 settles exact namespace/qualification; 6.0c proves same alias in separate namespaces with distinct stable identities. Alias names are not cache identities or permission guarantees. | — |
| U128 | Authenticated commands may use an explicitly configured account default; neither ambient Azure CLI selection nor the first available account is acceptable. | Gate A1 and 6.0c/6.0e cover stable default references, explicit overrides, missing/removed/incompatible defaults, and resolution before request construction. Service scope requirements remain intact. | — |
| U129 | Properly namespace types by their semantic domain: use CloudTerrastodonAccountId for CT's internal model; existing ResourceGroupListRequest and StorageAccount should expose their Azure domain in their names. | Companion naming contract; 6.0a/6.0c apply it to new account types, and 6.0d inventories affected service families for full renames with callers/registrations/files. Preserve wire/cache behavior and avoid unrelated blanket renaming. | — |
| U130 | Fresh responses should be optional for runtime tests: add --clean to invalidate all underlying requests; warn when absent and allow cached data, especially while diagnosing parser errors against a repeated response. | V1/6.0b/6.0e replace mandatory fresh-root policy with explicit clean/cached modes, complete child invalidation, provenance, and a deliberate saved-response decoder path because failed decodes are not normal cached successes. Development still uses synthetic inputs only. | — |
| U131 | Serializable requests must contain execution account/source/tenant, while the executor owns runtime auth. An approval workflow may intentionally list resource groups under an admin account rather than the requesting user's account. Resource-group/storage requests still holding contexts reveal unfinished migration. | A2/A3/6.0c/6.0d define stable approved account selection and executor-owned sessions, permit credential renewal without serializing session generations, prevent account substitution, and complete the remaining data-only request cutover. This is an approval-data requirement, not authorization to build a workflow engine. | — |
| U132 | A separate test registration, analogous to arbitrary registration, makes sense instead of a Test effect. | V1/6.0b use dedicated test metadata linked to normal executable requests; retain Read/Write/Solicit effects and subset admission. Exact registration API remains a design detail. | — |

U18 supersedes the earlier assistant suggestion to preserve an Azure CLI
active-tenant selection. That suggestion was not user approval. U13 rejects the
earlier recommendation to require a persisted type/schema identity as part of
cache reuse. U14 and U16 deliberately retain their original optional qualifiers.
U56 subsequently permits default tenant selection only at the maintenance tool's
CLI boundary. It supersedes U52/U53's tool-specific default rejection and direct
UUID bypass; U18's explicit tenant requirement for SDK request data remains.
U57 subsequently supersedes U55's retained required-output behavior for the
maintenance tool; its catalogue default and explicit overrides remain intact.
U59/U61 supersede G1's earlier agent-selected relocation of `AzureTenantId` and
the proposed registry package names. They preserve the runtime-independent
boundary; the earlier `registry_values` extraction remains completed foundation,
and its package is retired by task 3.0. `credentials_types` was a candidate name,
not an implemented package; U61 selects `auth_types` paired with runtime `auth`.

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
| U49 | G1/G2; 3.1; 7.1; 7.2; caller inventory | Owning-crate imports across consumers, no compatibility re-exports, workspace compile checks, and updated public migration documentation |
| U50 | 2.5; G1; 3.1 dependency rationale | Pure Facet decoding, direct dependency removed, checked remaining dependency paths, and focused offline conversion/consumer checks |
| U51 | 2.6, 2.7; G1; caller inventory | Owning-crate formatter trait, unpublished maintenance tool, no normal dependency path from types crates to command |
| U52 | 2.7 | Separate offline generate/check and live refresh with explicit source and resolved tenant; tool selection follows U56; synthetic tests and no live execution |
| U53 | 2.7 | Shared alias parsing/resolution and synthetic resolver tests; U56 replaces default rejection/direct-UUID bypass with the shared resolver; pre-authentication preflight remains |
| U54 | 2.8; caller inventory | Public provider models, SDK pagination/cache/invalidation request, thin refresh command, unknown-type bootstrap tests, and offline dependency proof |
| U55 | 2.7 refinement | Shared manifest-directory catalogue default, optional CLI argument with explicit overrides, output selection subsequently governed by U57, and offline validation without writing the default catalogue |
| U56 | 2.7 refinement; G3 selection boundary; caller inventory | Removed duplicate tenant type/resolver, shared default/UUID/alias selection, unchanged explicit source and bound SDK tenant, synthetic preflight before tenant discovery, focused offline checks |
| U57 | 2.7 refinement | Shared manifest-directory generated-source default, optional output argument for generate/check, working-directory-independent default selection, explicit override smoke checks, unchanged checked-in source and default catalogue |
| U58 | Current authorization; tasks 2.6–2.8 checkpoint; 3.1 next-step description | Reviewed commit scope and preserved user generation/staging, final offline checks, local commit identity, and description of authentication-selection/runtime separation; no push or phase-3 implementation |
| U59 | G1; 3.0; 3.1 | AzureTenantId remains in azure_types; all its registrations use the pure foundation; proposed normal/build graph is acyclic |
| U60 | G1 diagrams and candidate record; 1.1 revision | Current source versus target dependencies and runtime-provider dispatch explained; A/C retained as unselected alternatives |
| U61 | G1/G2; 3.0; 3.1; 3.3; 7.1/7.2; inventory | Full registry/auth package and import cutovers, no legacy package aliases/re-exports, one catalogue, workspace compilation and package-graph checks |
| U62 | Current authorization; 3.0 | Complete registry cutover, preserve pure metadata/conversion behavior, record offline checks and next task |
| U63 | G1; 3.0 refinement | Constant-time direct handle construction; no for-registration lookup or adapter table |
| U64 | G1; 3.0 refinement | Exclusive Function variants and original static metadata/callback ownership |
| U65 | Current authorization; 3.0 refinement | Workspace consumer cutover, identity/deduplication and invocation tests, inspected offline checks; no commit or live access |
| U66 | Current authorization; 3.0 checkpoint; 3.1 next-step description | Reviewed staged scope, local commit identity, clean post-commit status, and description of auth runtime/selection separation |
| U67 | Current authorization; 3.1; G1/G3 | Auth package/type ownership, tenant/source admission, sensitive credential retention, and inspected offline validation |
| U68 | 3.1; G3; authentication guide | Immutable CLI login generations, explicit tenant/account binding, serialized generation use, logout revocation, no implicit login, synthetic session tests |
| U69 | 3.1 refinement | Authorization enum and separate validating bearer/PAT payloads; six auth_types tests and focused Clippy passed on 2026-11-02 |
| U70 | 3.1a rationale; G3 | Temporary guard explained; auth/service responsibility separation drives its removal |
| U71 | 3.1a API decisions and scope | Official ARM remediation/container-list references; management metadata behavior explicit |
| U72 | Current authorization; 3.1a; inventory and public auth docs | Typed SDK requests, consumer cutover, obsolete service runner deletion, focused offline checks |
| U73 | Current authorization; 3.1a response-field refinement | Required container/remediation/provenance fields; documented residual optionality; direct typed page decoding and focused offline compilation/lint checks |
| U74 | Current authorization; 3.1a commit checkpoint; 3.2 next-step description | Reviewed staged scope, local commit title, clean post-commit status, and executor/provider scope description |
| U75 | Current authorization; G1/G2/G3; 3.2; inventory B04/B19 | Task 3.2's 12 execution, 4 provider, 32 no-auth app, 2 auth app-binding, and 1 cache-token tests; workspace checks, targeted Clippy, formatting, and dependency-boundary review |
| U76 | Current authorization; 3.2 naming refinement | Execution-mode definition and all executor guards; focused execution tests and formatting |
| U77 | Current authorization; 3.2 startup diagnostics refinement | Startup helper diagnostics, existing isolated early-error-report process test, and focused formatting |
| U78 | Current authorization; 3.2 commit checkpoint; 3.3 next-step description | Reviewed staged scope and local checkpoint title; prior focused verification and final whitespace/status checks |
| U79 | Current authorization; G2; 3.3 | 127 focused offline tests, workspace compilation, targeted Clippy, formatting, and 37-package normal/build boundary review passed; no commit or later-phase implementation |
| U80 | Current authorization; 3.3 provider refinement | Anonymous admission and reusable opt-in unreachable test provider; 55 focused registry tests and targeted checks |
| U81 | Current authorization; 3.3 temporary-root refinement | Cache-owned TempDir lifetime across owners/workers; 146 focused cache/registry/UI tests and targeted checks |
| U82 | Current authorization; 3.3 checkpoint; 4.1a | Local commit b00bff46; Command process/retry extraction, isolated input ownership, synthetic regression evidence and remaining 4.1 work recorded below |
| U83 | Current authorization; 4.1a checkpoint; 4.1b | Local commit c981de2b; executor artifact binding, synthetic disk/memory/policy checks, and explicit remaining caller cutover |
| U84 | Current authorization; artifact-binding checkpoint; 4.1b | Local commit 811468f3; owned execution, canonical artifact claims, parent admission, cancellation and authentication tests; production transport migration remains open |
| U85 | Current authorization; owned-work checkpoint; 4.1b | Local commit 5f627dfc; actual Command integration and callers |
| U86 | Current authorization; complete 4.1b acceptance criteria | Full implementation and offline validation, including service subprocess replacement, without another prerequisite-only stopping point |
| U87 | 4.1c; contracts; B02/B22 | Scope-only process, HCL, and local configuration paths; explicit managed artifact/cache execution |
| U88 | Current authorization; 4.1c | Caller correction and local regression evidence recorded beneath the task |
| U89 | 4.1b license refinement | Shared family types, source-aware rule matching, combined-wire and mutation-validation checks |
| U90 | 4.1b REST setter refinement | Typed context transfer, Azure DevOps bearer delegation, and 23 Azure caller simplifications |
| U91 | 4.1c local account cancellation refinement | Program resolution moved into the existing child scope; auth compilation passed |
| U92 | 4.1b JSON representation refinement | Shared opaque wrapper, preserved object/identity generators, audit inventory, and targeted Clippy enforcement |
| U93 | 4.1c Command JSON helper refinement | Helper module/exports removed, callers inlined, workspace compilation |
| U94 | 4.1c bound-request invalidation refinement | Lazy refresh/reuse, failure/cancellation handling, both execute spellings, updated Gitea browse callers |
| U95 | 4.1c bound-request invalidation refinement; superseded by U96 for blanket selection | Shared key trait retained; invalidation now requires a selected policy |
| U96 | 4.1c bound-request invalidation refinement | Own-key marker, custom key lists, custom async keyed requests, multi-key refresh isolation, borrowed/non-`Sync` proof |
| U97 | 4.1b/4.1c local checkpoint; 4.2 next-phase scope | Reviewed local commit, corrected error/invalidation and first-login audience paths, offline validation; U98 subsequently begins REST |
| U98 | Current authorization; 4.2; phase 5 ownership boundary | Full REST/caller cutover, workspace all-target check/Clippy, focused offline tests, and final caller review passed; task complete, changes uncommitted for review with user staging preserved |
| U99 | Current authorization; G2; 4.2a | Typed direct errors, narrowly bounded invalidation and registry conversion, picker recovery, 63 offline tests, and workspace all-target check/Clippy passed; complete and uncommitted with user staging preserved |
| U100 | Current authorization; G4; 4.2b; phase 5 ownership foundation | Shared cached SDK outputs, canonical conversion, and caller cutover verified by 46 focused offline tests and workspace all-target check/Clippy |
| U101 | G2/G4; 4.2b | Response/envelope `Clone` audit and non-`Clone` shared-value coverage passed |
| U102 | Current authorization; G4; 4.2b refinement | Twelve domain-specific response envelopes and SDK decoders are concrete; workspace all-target compilation and focused Azure types/SDK Clippy passed |
| U103 | Batch-entry refinement following 4.2b | Direct Facet entry, borrowed outer envelope, no body-copy bounds; Azure/Azure CLI all-target check and Azure Clippy passed before `d2a5e4f2` |
| U104 | Current authorization; G4/G6; 5.1–5.3 | All three SDK slices complete; 106 selected offline tests, workspace all-target check/Clippy, and independent review passed; changes uncommitted for review |
| U105 | G4; 5.3 refinement | Borrowed/owned list and picker inputs, runtime reflection, replay and cache sharing |
| U106 | 5.3 refinement; Facet update guide | Fork regressions and local consumer acceptance; U107 authorizes publication and consumer pin updates |
| U107 | 5.3 release refinement; Facet update guide | Fourteen Facet/Format/Figue packages, two Cancellation maintenance versions, and Tracing 0.2.1 published; both consumers have registry-only graphs with one identity per package and passing all-target compilation. Template's 13 registry tests pass; committed/pushed as `58d94181`. All 16 selected Cloud Terrastodon registry tests pass. |
| U108 | Release repository follow-up | Cancellation mainline merge `e4d96273` is pushed and retains 0.8.1. Tracing mainline merge `47bc18c5` is validated/pushed, retaining telemetry; GitHub default and local remote HEAD are `main`. Format history correction is pushed at `55dd7463`, with unchanged source trees and published archives. |
| U109 | Phase 5 review refinement | Shared test provider replaces nine duplicates; execution, SDK, REST, and registry tests pass with explicit partitions and call counters. |
| U110 | Phase 5 review refinement | Workspace and generator derive imports normalized; workspace all-target and feature-gated checks, generator tests, and qualified-derive audit pass. |
| U111 | Phase 5 review refinement | Finite collection registration passes shared-allocation, owned non-Sync, duplicate/alias, named-handle, and discovery tests. |
| U112 | Registry naming checkpoint; scoped join review | Explicit registration-point/catalog names committed; 14 production tuple joins and 14 JoinSets audited. |
| U113 | Scoped join preview below | Existing work committed as `1e63a553`; helper, three callers, focused completion tests, and task-set audit prepared for review. |
| U114 | Scoped concurrency rollout below | Bounded mapping and fixed joins replace finite task batches; 89 focused tests plus one documentation example, affected-crate Clippy, and workspace compilation pass. Interactive handler/shutdown ownership remains a separate lifecycle task. |
| U115 | Iterator mapping refinement below | Iterator extension and caller migration complete; 23 focused tests, two documentation examples, workspace compilation, and affected-crate Clippy pass. Existing staging preserved. |
| U116 | UUID value semantics refinement below | All 25 direct UUID wrappers implement `Copy`; eight missing derives added. Redundant caller clones removed; workspace all-target Clippy with `clone_on_copy` denied passes. |
| U117 | Scoped concurrency review and local checkpoint | Prior workspace/affected-crate checks and selected local tests passed; final mapping tests and execution/entrypoint all-target Clippy passed after preallocated paired-vector ordering and cache-population removal. |
| U118 | Interactive handler/background ownership follow-up; scoped join review | 86 focused offline tests, workspace all-target compilation, all-feature picker compilation, and warnings-denied all-target Clippy across 11 affected crates pass. Independent reviews resolved nested terminal/poison races and GUI shutdown gaps. Committed under U120 as `58f454e4`. |
| U119 | UI background-work cleanup; existing memory-only inventories | Source review confirms `cached_work` retains decoded aggregates without aggregate disk artifacts; principals/RBAC and DevOps projects already use it. |
| U120 | UI background-work cleanup below | Existing work committed as `58f454e4`. Cleanup complete; six focused offline tests, workspace all-target compilation, and warnings-denied all-target Clippy for command and both UIs pass. U122 records the cleanup checkpoint. |
| U121 | Shared `Work::enqueue` lifetime contract | Source review and an isolated compiler check confirm the typed sender needs the bound when captured in the independently owned background task; the UI state borrow stays synchronous. |
| U122 | UI cleanup checkpoint; Phase 6 completion boundary | Local checkpoint titled `Consolidate UI background work and scoped autosaves`; remaining Phase 6 scope verified against source. |
| U123 | Companion account model, A1–A4; 6.0a/6.0c/6.0d/6.0e | Source inventory confirms single-selection stores and existing session partitions; implementation and two-account/lifecycle proof pending. |
| U124 | Companion verification model, V1; 6.0b/6.0e | Live unignored tests and existing effect semantics verified; full test classification, runtime runner, and offline acceptance pending. |
| U125 | Companion helper classification; 6.0d/7.1 | Request-first execution already exists; full family cutover and caller checks pending. |
| U126 | Current authorization; revised Phase 6 order | Design-only update; completed foundations preserved and new gates explicit. No implementation or live validation performed. |
| U127 | Companion alias model, A1; 6.0c/6.0e | User reply confirms scoped aliases; precise lookup contract and synthetic resolution tests pending. |
| U128 | Companion defaults contract, A1; 6.0c/6.0e | User reply permits configured defaults; no ambient/first-account fallback. Resolution and lifecycle validation pending. |
| U129 | Companion type naming; 6.0a/6.0c/6.0d | Proposed account name corrected; existing Azure request/storage names verified in source. Production family renames and consumer/reflection checks remain pending. |
| U130 | Companion clean/replay policy, V1; 6.0b/6.0e | Existing invalidation and REST artifact behavior reviewed; mandatory fresh-root recommendation superseded. Implementation and synthetic clean/cached/redecode proof pending. |
| U131 | Companion approval model, A2/A3; 6.0c/6.0d/6.0e | Runtime auth-context fields confirmed in both cited requests; per-account serialized approval and executor mapping remain planned. Credential generations are runtime concerns, not mandatory durable request fields. |
| U132 | Companion test registration, V1; 6.0b | User accepts registration/effect separation; exact registration and runner implementation pending. |

## Purpose and scope

Allow independently implemented audits to request the same inventory through an
explicit executor and share one decoded result. Request values remain useful for
text inspection and offline construction. Managed commands, HTTP requests, and
in-process work use one execution/cache lifecycle while preserving their distinct
effects, failure artifacts, cancellation requirements, and authentication rules.
Cancellation-only helpers and plain local processes remain independent of that
context; they receive the bounded cancellation scope directly (U87/U88).

The initial implementation includes the cache/execution foundation, explicit auth
selection, registry integration, Graph principals, ARM RBAC, Azure DevOps projects,
audit modularization, and the workspace callers required to complete the API
migration. Remaining request families are inventoried and migrated in bounded
batches, rather than leaving permanent hidden-runtime compatibility paths.

U123–U128 expand the planned completion boundary with named account selection,
registered intentional runtime verification, offline Cargo test separation, and
request-first caller cutover. The [design companion](accounts-runtime-tests-and-requests.md)
distinguishes verified source, confirmed requirements, recommendations, and open
contracts. Authentication protocols remain foundation; account/session ownership
and selection are the new responsibility.

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
| Auth runtime | [auth_context.rs](../../crates/auth/src/auth_context.rs) already owns invocation-local reference-counted token caches. | Preserve source policy, headless behavior, and cache isolation while moving ownership. |
| Tenant binding | [azure_tenant_auth_context.rs](../../crates/auth/src/azure_tenant_auth_context.rs) validates resolved enabled sources and workload-identity tenant agreement. | Keep these invariants in the new auth-selection/execution boundary. |
| CLI fallback | [azure_devops_auth_context.rs](../../crates/auth/src/azure_devops_auth_context.rs), [azure_access_token.rs](../../crates/auth/src/azure_access_token.rs), [tracked_tenants.rs](../../crates/azure/src/tracked_tenants.rs). | Active-tenant compatibility and optional CLI token tenant must be removed from authenticated request execution. |
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

- Type moves are full cutovers: update consumers, dependencies, registrations,
  tests, examples, and documentation together. Import moved types from their
  owning crate; do not retain or add old-path compatibility re-exports.
  Ordinary module re-exports within the owning crate remain valid.
- Serializable requests carry domain scope, resolved authentication selection,
  filters, and limits. They carry no token cache, execution scope, or callback.
  Ordinary headers/body may still require redaction; serializable does not mean
  safe to log secrets. Preserve existing diagnostic redaction responsibilities.
- Type names identify their semantic domain: CT-owned account concepts use
  `CloudTerrastodon` (such as `CloudTerrastodonAccountId`); service concepts use
  their established domain prefixes. Include existing ambiguous request/value
  families in the 6.0d naming audit, with full consumer cutover and no accidental
  service-wire or cache-identity changes.
- Reviewed request data identifies each operation's chosen account/source/tenant,
  independently of the person proposing the workflow. Runtime credentials and
  token/session generations stay executor-owned; credential renewal must not
  itself redirect or unnecessarily invalidate the approved account choice.
- Runtime test freshness is optional: `--clean` invalidates the selected tests'
  underlying reads; omission permits eligible cache reuse with a warning. Test
  registration is separate from effects. Saved-response decoding must not promote
  failure artifacts into normal reusable successes.
- The executor is a resource coordinator, not a data-filled World or a feature
  client forwarding organization/project fields. Clones share resources; a child
  execution uses its callback's child CancellationScope. Clones never extend a
  scope or deferral beyond its owner. Libraries do not manufacture root scopes.
- Both execute spellings reach the same admission/cache/dispatch procedure.
  Request implementations retain their visible URL construction, pagination,
  interpretation, and affected-read invalidation procedures.
  U125 prefers request-first syntax at ordinary operation call sites and removes
  trivial construction/binding helpers; equivalent executor execution remains
  useful to generic infrastructure.
- Account aliases are scoped by tenant/service and may have explicit configured
  defaults. Resolve selection before constructing requests; never choose the
  first account or depend on ambient Azure CLI state. Exact account contracts
  remain gates A1–A4 in the companion. New live verification is an intentional
  runtime operation; it does not authorize live development tests.
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

U123–U128 add gates A1–A4/V1 in the
[design companion](accounts-runtime-tests-and-requests.md#gates-before-implementation).
These extend the completed foundation; they do not reopen its resolved cache or
crate boundaries. Close them in 6.0a before their dependent implementation.

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
| G1 | Exact acyclic crate/trait boundary for command, auth, registry, registry_execution, and the concrete executor. | U59/U61 select B: pure registry; executor-dependent registry_execution; runtime auth; AuthSource/resolved vocabulary in auth_types; AzureTenantId stays in azure_types. execution depends on auth_types/cache, not auth/command/registry_execution. Check normal/build dependencies including pure conversions; cache extraction alone does not prove acyclicity. |
| G2 | Public method/trait names, lifetimes, cacheable versus noncacheable output bounds, and full API cutover. | Data requests bind to an awaitable execution. Sharing Arc<T> across tasks adds Sync to today's Send + 'static output contract; decide affected bounds without imposing them on unrelated noncacheable outputs or borrowed callbacks. Move each caller family to its new contract; no compatibility re-exports, default executor, or indefinitely retained runtime-bearing request path. Record breaking Rust API changes and removal milestones. |
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

`1 contracts/inventory → 2 value registry/cache core → 3 registry cutover/auth/executor → 4 adapters →
5 SDK slices → 6.0 accounts/runtime verification/request-first cutover →
6.1–6.3 audit/app/consumers → 7 cutover/acceptance`; phase 8 is deferred.
Phase 2 can use synthetic in-process work until the public executor exists.
Do not expose a partially initialized global runtime to bridge stages.

Keep each slice compilable: introduce the new contracts alongside explicitly
bound adapters before switching an existing caller family. Task 3.0 completes
the pure `registry`/`registry_execution` package cutover before task 3.1 renames
`credentials` to `auth` and extracts `AuthSource`/resolved selections into
`auth_types`. Tasks 3.2–3.3 integrate the executor and invocation ABI; tasks 4–6
move consumers in compile-checked
batches. Each type move updates all of its consumers and removes its old-path
exports in that same slice; compatibility re-exports must not bridge phases.
Retire old signatures and fields with their callers, with the final sweep in
7.1. Transitional execution must not invent missing tenants, resurrect global
caches, or mix two memory-cache owners in a single migrated request tree.
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

**Completion notes:** Initially closed G1–G4 in
[executor-cache-contracts.md](executor-cache-contracts.md) and G5/G6 in
[executor-cache-coordination.md](executor-cache-coordination.md). Selected low
registry_values/auth_types/cache/execution boundaries, an injected typed auth
provider, bound request/registry lifetimes, central namespace/owner checks,
memory-only aggregates, and a mutex-gated publication/invalidation algorithm.
That original decision moved AzureTenantId to the lower auth vocabulary. U49 supersedes the original
re-export decision: all consumers move to the owning crate and old exports are
removed in the same slice.
CLI/browser/PAT use session partitions until identity can be established safely;
authenticated CLI subprocesses must use that same provider-bound session.

**U59–U61 revision:** G1 now selects option B. Keep `AzureTenantId` in
`azure_types`; absorb `registry_values` into pure `registry`, move
executor-dependent registrations/invocation to `registry_execution`, rename
runtime `credentials` to `auth`, and move `AuthSource` plus resolved selections
to `auth_types`. Complete the registry/import cutover in 3.0 before 3.1's auth
cutover; integrate the executor ABI in 3.3. Pure constructors/conversions and
context-free metadata/planning remain below execution. One authoritative
registration feeds the integrated catalogue. A has the same dependency boundary
with the earlier names; C folds invocation into execution and was not selected.
The updated G1 diagrams distinguish planned package dependencies from runtime
provider calls. Earlier graph/sketch evidence below applies to the earlier
contract and does not prove this revised workspace compiles; task 3.0 must
validate the revised normal/build graph and task 3.1 its auth additions.

**Phase 5 G4/G6 refinement — 2026-11-06:** Bound observation uses a per-invocation
event channel and a caller-polled wrapper, preserving borrowed/non-`Send`
callbacks and the request's typed error. Callbacks are prompt notifications, not
a synchronous execution/reentrancy boundary. A suspended producer may retain an
initializer or filesystem guard; acquiring and releasing the I/O gate cannot
prove quiescence because its next permit may already belong to that producer.
The coordination contract now distinguishes queued delivery outside state
bookkeeping from directly invoking callbacks inside a protected operation.
There is no extra cache barrier or detached task. Explicitly dropping an observed
future produces no terminal notification. This resolves the implementation detail
without changing request, cache, cancellation, or authentication ownership.

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
values, pointer registration, and arbitrary constructor descriptors. Under the
earlier policy, the high-level registry introduced value compatibility re-exports
and adapted pure constructors once while retaining descriptor provenance. U49
supersedes that export policy: task 3.0 removes those exports and migrates their
consumers. Pure constructor adaptation remains an execution responsibility.
Production types have individual owning files.
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

**Historical task contract:** This work implemented the original G1 boundary.
U61 now retires the package name in task 3.0 while preserving its pure facilities;
the commands below record the original extraction and are not current targets
after that cutover.

**Work:** Implement the original G1's `crates/registry_values` extraction before types in lower
crates register themselves. Move value reflection, owning RuntimeValue helpers,
Thing/RegistrationSite, and value registration macros/slices. The original
high-level re-export requirement was completed under the earlier policy and is
superseded by U49/U61, with removal tracked in 3.0 and audited in 7.1. Add the pure arbitrary descriptor
without moving executable Function
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

### [x] 2.5 Remove unnecessary command coupling from response types

**Completion notes:** Source review confirmed that `query_response.rs` is the
only direct command import in `azure_types`. `FromCommandOutput` is a marker for
`Facet<'static> + Send + 'static`; row decoding itself only needs `Facet<'static>`.
The response records and pure conversion can stay in their domain crate. At this
checkpoint, Cargo metadata found normal paths to `command` through `hcl_types` and
`azure_resource_types`, so this change does not make `azure_types` suitable as
the low execution layer's dependency at that checkpoint. The then-planned
registry-to-execution edge was another concern. U59/U61 now address it by
making `registry` the pure foundation in task 3.0; the tenant ID remains in
`azure_types`, and the separate auth vocabulary uses that domain identity.

The five response/data/error types now each have an owning file in `azure_types`.
`ResourceGraphQueryResponse<T>` has no declaration bound;
`ResourceGraphData::into_records` and the raw-to-typed conversion require only
`Facet<'static>`. Crate-root imports, wire fields, response tolerance, row-width
validation, and failed-row diagnostics are retained. The manifest and lockfile
remove only the direct command dependency edge; dependency versions are unchanged.

Validation passed: three synthetic table reconstruction/diagnostic tests, one
string-backed domain-field conversion test, and the SDK's failed-entry artifact
regression. The existing timing-based fixture was excluded. Clippy for both
types and SDK crates with all targets and warnings denied, nightly workspace
formatting, and whitespace checks pass. Offline Cargo metadata confirms the two
remaining normal paths to `command`. No production services, credentials, data,
or existing caches were accessed; user notes and staging are untouched.

**Work:** Remove the command bound from the response container and use Facet
only at decoding boundaries. Split the five production types into their owning
files within `azure_types`, retaining public type names, wire representations,
row validation, and error diagnostics. Remove the direct manifest dependency
and only its corresponding lockfile edge. No SDK relocation or compatibility
exports are required; existing consumers keep the same owning-crate imports.

**Validation:** Inspect and run only the synthetic response conversion tests,
excluding the existing timing-based benchmark test. Check the types crate and
Azure SDK with the pinned offline dependencies, run focused Clippy and formatting,
and use offline Cargo metadata to verify the direct and remaining indirect edges.
No cloud requests, credential discovery, or production cache reads are involved.

```powershell
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib resource_graph_data::tests -- --skip it_slow
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib resource_graph_query_response::tests
cargo test --offline --locked -p cloud_terrastodon_azure --lib resource_graph::tests::writes_only_failed_resource_graph_entry_extra_file -- --exact
cargo clippy --offline --locked -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure --all-targets -- -D warnings
rustup run nightly -- cargo fmt --all -- --check
cargo metadata --offline --locked --no-deps --format-version 1
```

**Completion criteria:** No command imports or direct command dependency remain
in `azure_types`; decoded response behavior and SDK callers still pass focused
checks. Remaining transitive dependencies are recorded rather than silently
expanded into unrelated cleanup.

### [x] 2.6 Move Terraform formatting out of HCL types

**Completion notes:** The runtime extension trait and both formatting callers
now live in `hcl`; the two pure traits remain in separate `hcl_types` files. The
five import conversion tests and four provider conversion tests pass. All-target
Clippy for `hcl`, `hcl_types`, and `azure_resource_types` passes with warnings
denied, compiling the moved formatter test without executing it. Offline Cargo
metadata confirms no normal/build path from either HCL types or Azure types to
`command`. No executable configuration or existing cache was read.

**Work:** Keep pure `AsHclString` and `TryAsHclBlocks` in `hcl_types`, each in its
own file. Move process-backed formatting into `hcl::AsFormattedHclString`, update
its consumers, and remove `command` and `async-trait` from `hcl_types`. Preserve
the supplied cancellation scope, interruptible stdin formatting, and returned
text. Do not add compatibility exports or change HCL transformations.

**Validation:** Run reviewed pure import/provider conversion tests; compile the
moved formatter test and its consumers. The formatter test resolves executable
configuration through Windows known folders, which have no isolated override in
the current API; do not run it against the user's configuration. Check the normal
dependency graph, scoped Clippy, and formatting.

```powershell
cargo test --offline --locked -p cloud_terrastodon_hcl_types --lib import_block::tests
cargo test --offline --locked -p cloud_terrastodon_hcl_types --lib provider_block::tests
cargo clippy --offline --locked -p cloud_terrastodon_hcl -p cloud_terrastodon_hcl_types --all-targets -- -D warnings
```

**Completion criteria:** Types retain their pure APIs, formatting callers import
the runtime trait, and no normal dependency path from `hcl_types` reaches
`command`. Validation does not load existing command configuration or caches.

### [x] 2.7 Extract resource-type maintenance into standalone tooling

**Completion notes:** The unpublished `resource-types` CLI now provides offline
`generate`/`check` and explicit tenant/source `refresh` into a saved catalogue.
The ignored live generator and its dependencies are removed from the reusable
crate. At the initial extraction checkpoint, its checked-in inventory and wire
behavior were unchanged; only the source banner named the new tool. The user's
later staged generation is included in the U58 commit scope and does not alter
that historical validation evidence. Newly generated fallback variants skip
arbitrary fuzz generation, preserving their existing unknown-value parsing behavior.

Eight synthetic tool tests and the three existing resource-type tests pass. The
new startup process tests pass without app auth (one) and with app auth (two),
using invalid synthetic workload inputs to detect accidental discovery. An
auth-enabled tool binary generated and checked a three-name synthetic catalogue;
nightly rustfmt left its bytes unchanged. Refresh with a synthetic tenant and a
missing output parent reached the destination error before credential resolution,
verifying leaf/global flag routing without network activity. No valid live refresh
or existing credential/config/cache inspection occurred.

Workspace all-target compilation, scoped Clippy with warnings denied, nightly
workspace formatting, and whitespace checks pass. Offline metadata confirms no
normal/build dependency path from `azure_types`, `hcl_types`, or
`azure_resource_types` to `command`, and no package depends on the unpublished
tool. The lockfile changes only local package edges and the new workspace member;
external versions are unchanged. Live API and Linux support evidence remain
unestablished. User notes and staging were preserved; the changes remained
uncommitted at that checkpoint. U58 records the later commit authorization.

**Tenant-alias refinement (U53, historical behavior before U56):** Refresh
accepted an explicit UUID or tracked Cloud Terrastodon alias. The tool's CLI
argument delegates parsing to the existing
tenant vocabulary and excludes the default selector. Aliases resolve through
`AzureTenantAliasExt` before authentication; UUIDs retain direct use without
requiring a tracked entry. The resolved ID is bound to the explicitly chosen auth
source without loading tracked auth preferences. Only the tool gains an Azure SDK
dependency; the types crates remain independent of command execution.

Refinement validation passed: nine tool tests, the three shared resolver tests
listed below (temporary directories and synthetic IDs), auth-enabled tool build
and all-target Clippy, nightly formatting, whitespace, and offline dependency
checks. CLI preflight accepts both a synthetic alias and UUID, then stops at a
missing destination parent before tenant/config lookup. `default` fails parsing;
omitting `--tenant` shows the missing-required-argument diagnostic without invoking
refresh. The pinned CLI framework renders the rejected proxy as a generic
uninitialized-value error and exits successfully after missing-argument help;
those diagnostic/exit-status limitations are not changed by this correction.
No real tenant records, credentials, or services were accessed. All 53 ledger and
traceability pairs pass the privacy/fence audit.

```powershell
cargo test --offline --locked -p cloud_terrastodon_resource_types_cli --lib
cargo test --offline --locked -p cloud_terrastodon_azure --lib tracked_tenants::tests::it_adds_lists_resolves_and_removes_aliases -- --exact
cargo test --offline --locked -p cloud_terrastodon_azure --lib tracked_tenants::tests::it_rejects_alias_conflicts_across_tenants -- --exact
cargo test --offline --locked -p cloud_terrastodon_azure --lib tracked_tenants::tests::it_rejects_ambiguous_tenant_id_substring_resolution -- --exact
cargo clippy --offline --locked -p cloud_terrastodon_resource_types_cli --all-targets --features cloud_terrastodon_app/auth -- -D warnings
```

**Work:** Replace the ignored generator test with an unpublished workspace member
at `tools/resource-types-cli`. `generate` and `check` use a selected saved JSON
catalogue without credentials. `refresh` resolves the shared tenant selector and
explicitly selected credential source,
fetches provider metadata through the reusable SDK request extracted in task 2.8,
and saves the catalogue separately. Validate collisions and deterministic rendering;
publish output atomically and make drift checks read-only. Keep the generated
enum's existing behavior and inventory, changing only its generator provenance.

**Default-catalogue refinement (U55, complete):** Added
`ResourceTypeCatalogue::default_path()` for the unpublished tool's
`catalogue.json`, anchored to `env!("CARGO_MANIFEST_DIR")`. That default is applied
declaratively to the existing `PathBuf` catalogue fields on refresh, generate,
and check. Omitting `--catalogue` selects the build checkout's
`tools/resource-types-cli/catalogue.json`, independently of the process working
directory; explicit relative overrides remain working-directory relative.
At this checkpoint `--output` stayed required for generate/check; U57 supersedes
that output policy below. Resolving the catalogue default performs no I/O,
creates no file, and did not change the then-current tenant/source rules.
The README shows the shorter default workflow and explains the build-checkout
relationship. Validation passed: auth-enabled tool build, scoped all-target
Clippy with warnings denied, focused nightly formatting, and whitespace checks.
From a different working directory, `generate` and `check` without `--catalogue`
both reported the expected missing default file; its absence was confirmed before
execution. Explicit relative overrides generated and checked a synthetic
catalogue under `target/`. These offline probes used `--auth-source none`.
Refresh help describes the default. No default catalogue was created or read,
and no live refresh or credential/configuration discovery occurred.

**Shared tenant-selection refinement (U56, complete):** Removed
`ResourceTypesRefreshTenantArgument`, its file, module, and export. Refresh uses
`AzureTenantArgument<'static>` directly with its declarative default and calls
`AzureTenantArgumentExt::resolve` before binding authentication. Omitted
`--tenant` and `--tenant default` select the Azure CLI default account's tenant;
explicit UUIDs now use the shared tracked-ID check, and aliases use the shared
resolver's matching and ambiguity handling. `--auth-source` remains required.
Bind the resulting UUID to that selected source without importing tracked auth
preferences. The reusable SDK request still receives a concrete tenant ID and
does not resolve an ambient tenant itself.

This supersedes the earlier tool-only default prohibition and UUID bypass. It
does not broaden default selection for other SDK operations or authorize reading
real tenant/configuration/cache data during development. Validation passed: nine
offline tool tests, an auth-enabled tool build, and scoped all-target Clippy with
warnings denied. CLI preflight accepted omitted tenant, `default`, a synthetic
alias, and a synthetic UUID; each stopped at a deliberately absent destination
parent before tenant resolution, configuration/credential discovery, cache
access, or a service call. No output was created. Focused nightly formatting and
whitespace checks passed. Documentation audit verified all 56 guidance pairs,
137 relative links, balanced fences, and absence of personal or local-machine paths.

**Default-output refinement (U57, complete):** Added
`ResourceTypeCatalogue::default_output_path()` using the tool's compile-time
manifest directory and `../../crates/azure_resource_types/src/resource_types.rs`.
The resulting `PathBuf` is the declarative default for generate/check output arguments.
Both commands can use the default catalogue and generated source without
path flags. Defaults remain tied to the build checkout independently of the
process working directory; explicit relative overrides remain working-directory
relative. Catalogue selection and authentication behavior are unchanged.

Validation passed: auth-enabled tool build, scoped all-target Clippy with warnings
denied, focused nightly formatting, and whitespace checks. From another working
directory, read-only `check` with a synthetic catalogue reported drift against
the intended crate source. `generate` accepted omitted output and rejected an
empty synthetic catalogue before publication. Explicit relative overrides
successfully generated and checked synthetic source under `target/`. All probes
used `--auth-source none`; no live access occurred. A before/after hash confirmed
the checked-in source was unchanged; the default catalogue was not read or written.

Cargo feature unification can enable `app/auth` even when this tool disables
default features. Add `App::without_authentication()` as an explicit startup
policy: bypass credential discovery, return `AuthContext::None` with auth enabled,
and reject explicit global credential sources instead of silently ignoring them.
The refresh leaf alone resolves its validated source. This small host policy is
required for the offline contract; it does not begin executor implementation.

**Validation:** Use synthetic names and temporary files for catalogue validation,
determinism, file replacement, and drift checks. Compile refresh without executing
it; synthetic tests cannot establish live compatibility. Verify package graph and
`publish = false`; the produced types crate must not depend on the tool. Update
only local dependency edges in the lockfile, preserving external pins.

```powershell
cargo test --offline --locked -p cloud_terrastodon_resource_types_cli
cargo test --offline --locked -p cloud_terrastodon_azure_resource_types --lib
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --test process disabled_startup_authentication
cargo test --offline --locked -p cloud_terrastodon_app --features auth --test process disabled_startup_authentication
cargo clippy --offline --locked -p cloud_terrastodon_resource_types_cli -p cloud_terrastodon_azure_resource_types --all-targets -- -D warnings
cargo clippy --offline --locked -p cloud_terrastodon_resource_types_cli -p cloud_terrastodon_app --all-targets --features cloud_terrastodon_app/auth -- -D warnings
cargo check --offline --locked --workspace --all-targets
rustup run nightly -- cargo fmt --all -- --check
```

CLI smoke evidence used a synthetic JSON array under `target/`, with no source
inventory or existing cache as input:

```powershell
cargo run --offline --locked -p cloud_terrastodon_resource_types_cli --features cloud_terrastodon_app/auth -- generate --catalogue target/resource-types-tool-smoke/catalogue.json --output target/resource-types-tool-smoke/resource_types.rs
rustup run nightly -- rustfmt --edition 2024 target/resource-types-tool-smoke/resource_types.rs
target/debug/resource-types.exe check --catalogue target/resource-types-tool-smoke/catalogue.json --output target/resource-types-tool-smoke/resource_types.rs
```

The smoke check compares hashes before/after formatting. Generated blocks use
`#[rustfmt::skip]` so pinned `prettyplease` owns their deterministic layout.

**Completion criteria:** Offline catalogue workflows are usable and tested, live
refresh requires an explicit source and a resolved tenant UUID, live acquisition
remains unrun, generation no longer lives
in the reusable crate, and neither `azure_resource_types` nor `azure_types` has a
normal dependency path to `command`. Tasks 3.0/3.1 still separate registration
from execution and credential selection from runtime acquisition, while keeping
the tenant ID in `azure_types` under U59/U61.

### [x] 2.8 Extract the reusable resource-provider API from maintenance tooling

**Completion notes:** Public provider records, nested metadata, registration
states, and validated identities now live in `azure_types`.
`azure::fetch_all_resource_providers` owns tenant-scoped pagination, typed REST
decoding, tenant/source page-cache isolation, invalidation, and a registered
`Read` effect. The tool's copied response models and transport procedure are
removed. Refresh resolves its tenant selection and explicit source, invalidates the SDK
request, converts its result to a complete validated catalogue, and publishes it.
Provider-relative names qualify into the existing generated vocabulary, including
`Other(String)`; catalogue generation preserves the returned spelling directly.

Validation passed: four domain-invariant tests, two request-policy tests, and ten
tool tests, all synthetic and offline; scoped all-target Clippy with app auth
enabled and warnings denied; workspace all-target compilation; nightly workspace
formatting; and whitespace checks. Offline metadata confirms no normal/build path
from `azure_types`, `hcl_types`, or `azure_resource_types` to `command`. The tool
remains unpublished with no dependents; lockfile changes only local dependency
edges, without external version changes. No live request, credential/config
discovery, or real cache invalidation was executed. Subscription registration
queries and mutations remain future work.

The first tool-test build exhausted disk space. Removing only the confined,
regenerable `target/debug/incremental` compiler artifacts allowed the checks to
complete. Documentation review verified all 54 guidance/traceability pairs,
relative links, balanced fences, and absence of personal/local-machine paths.
No staging or commit operation was performed.

**Commit checkpoint (U58):** This checkpoint includes completed tasks 2.6–2.8 and
their refinements, with the subject `Separate type crates from execution and
maintenance tooling`. It includes the user's staged generated resource-type
source and the `.gitignore` rule excluding the local tool catalogue, preserving
unrelated user work. Earlier statements that the checked-in inventory was unchanged describe
the agent's extraction/default-path validation before the user generated it.
No agent-run refresh or live compatibility validation is claimed, and the local
catalogue remains unread. Final workspace all-target compilation, three generated
resource-type tests, nine offline tool tests, nightly workspace formatting, and
whitespace checks passed. Independent review found no blocker in the HCL
extraction or credential-free app startup. After this checkpoint, describe task
3.1's authentication-selection/runtime separation with
the owning-crate cutover and explicit SDK tenant contract. Do not push or begin
that implementation as part of this request.

**Work:** Move provider response records, nested shapes, and meaningful identities
into `azure_types`, with official schema links, strict debug response decoding,
typed registration state, and value registrations. Add
`azure::fetch_all_resource_providers` / `ResourceProviderListRequest` with explicit
tenant-bound authentication, cancellation, visible pagination, scoped page cache,
invalidation, and registry `Read` effects. The maintenance tool resolves selection,
invalidates this request for a fresh inventory, and turns its typed result into a
validated catalogue before atomically saving the file.

Namespace-relative type names in provider metadata cannot directly use the full
`ResourceType` enum. Model those identities explicitly and offer qualification
into the existing vocabulary, retaining `Other(String)` for new full names.
Generation must use the returned component spelling rather than round-trip known
names through the older enum. No bootstrap feature or copied schema remains in
the tool. Keep the produced enum crate independent of SDK/tool execution.

The current endpoint lists at tenant scope. Subscription registration-state
queries and registration mutations remain future SDK operations; tenant metadata
does not establish readiness for a selected subscription. No live operations are
authorized for validation, and no registration operation is added here.

**Validation:** Review official API/source schema, then use synthetic domain
values for namespace/type composition, unknown vocabulary preservation, provider
ID invariants, catalogue completeness and service spelling. Test request-owned
continuation confinement and tenant/source cache isolation without executing the
request or invalidating a real cache. Compile all consumers, run focused Clippy
and formatting, and inspect the normal dependency graph and local lockfile edges.
Preserve the user's existing staged changes; leave this refinement unstaged.

```powershell
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib azure_resource_provider
cargo test --offline --locked -p cloud_terrastodon_azure --lib resource_provider_list_request::tests
cargo test --offline --locked -p cloud_terrastodon_resource_types_cli --lib
cargo clippy --offline --locked -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure -p cloud_terrastodon_resource_types_cli --all-targets --features cloud_terrastodon_app/auth -- -D warnings
cargo check --offline --locked --workspace --all-targets
rustup run nightly -- cargo fmt --all -- --check
```

**Completion criteria:** Provider metadata has one public owning API, the tool
contains no transport procedure or duplicate response models, and catalogue
generation discovers unknown resource types without losing service spelling.
Existing alias/source behavior and production-data restrictions remain intact;
types crates still have no dependency path to command execution.

## Phase 3 — serializable selection and bound execution

### [x] 3.0 Cut over to pure registry and registry_execution

**Completion notes:** Package cutover and U63–U65's enum-handle refinement
completed 2026-11-02. U66 includes this work in the local checkpoint titled
`Separate pure registry from execution with direct function handles`.
Pure `registry` now owns
reflection, erased values, arbitrary constructors, `PureFunction` conversions,
`FunctionDescriptor`, shape discovery, and descriptor-only planning.
`registry_execution` owns the `Function` enum, `RuntimeFunction` records,
invocation, cancellation binding, simulation, and the merged executable catalogue.
The existing request ABI remains
until 3.3. `registry_values` and its compatibility exports/package are removed;
workspace, SDK, CLI, UI, cache, examples, tooling, tests, and documentation use
the owning package. `AzureTenantId` stays in `azure_types`.

Lower registrations expose immutable callback/metadata relationships. Common
`Function` values reference exactly one runtime, pure, or arbitrary registration;
`descriptor(self)` returns its original static descriptor. `known_functions()`
wraps those references while traversing the original slices. The pure
`KNOWN_FUNCTION_DESCRIPTORS` slice references the same descriptors, including
runtime metadata, so shape discovery needs no upward dependency. Runtime macros
publish original records through `KNOWN_RUNTIME_FUNCTIONS` and associated enum
handles. Pure handles wrap directly, with no lookup, allocation, or new error path.
The initial adapter table and optional provenance fields were removed under
U63–U65; no compatibility lookup API remains.

Function equality and hashing use variant plus source registration address.
Simulation deduplication, reflected handles, and UI selections use this identity
while passing handles by value. Metadata-only consumers borrow descriptors
directly. Pure/arbitrary variants have no runtime cancellation storage; runtime
invocation retains its existing callback, input ownership, and cancellation
semantics. Independent core/consumer reviews found no blocking issue.

At the original package-cutover checkpoint, the registry packages passed 7 pure unit,
48 invocation unit, and 2 compile-fail documentation tests. Inspected consumer
tests passed 6 discovery-plan, 4 discovery-simulation, 3 Azure registered-value,
3 project-argument, and 1 SDK response-constructor tests (74 total). Workspace
compilation covered every target; strict Clippy covered both registry packages.
No service, real credential, local configuration, production cache, or generated
catalogue was accessed. Broad workspace tests were deliberately not run because
some existing tests use live services.

**Enum-refinement validation:** The registry suites now pass 7 pure unit,
49 invocation unit, and 2 compile-fail documentation tests. The new regression
invokes an explicit pure registration absent from every linked catalogue.
Existing identity tests preserve copied-handle deduplication, separate same-shape
registrations, and shared source descriptors. Workspace all-target compilation
and strict Clippy for both registry packages, `ui_ratatui`, and `entrypoint` pass.
The inspected consumer tests passed 10 SDK discovery, 23 UI action/invocation/
production/lifetime/export, and 5 CLI planner cases: 96 tests total with the
registry suites. Lifetime tests ran directly from the newly built UI test
executable with `object_explorer::explorer_engine::lifetime_tests`; the Cargo
command below reproduces that selection. Initial filename-based lifetime/planner
filters selected zero tests and are not counted as evidence; the corrected module
filters ran all 8 lifetime and 5 planner cases. Test code used synthetic values
and a temporary export directory only. Formatting, whitespace, and final
source scans passed: no adapter table/lookups, optional registration provenance,
or `&'static Function` storage remains. This refinement changes no manifests or
dependency pins. Existing staging was preserved during implementation; U66 now
includes the completed cutover and refinement in the authorized local commit.
The commit review corrected CRLF-only whitespace warnings in the new invocation
crate's manifest without changing its TOML values.

The normal/build Cargo graph contains 35 workspace packages and no cycles.
Neither `azure_types`, `azure_devops_types`, nor `cache` reaches `command`,
credential runtime, REST, or `registry_execution`. The obsolete test-only use of
`command` no longer requires an `azure_devops_types` normal dependency. Applying
G1's pending auth/execution edges to that graph yields 37 packages with no cycle;
3.1–3.3 must recheck the actual graph as those packages land. All 820 external
lockfile package/version/source/checksum records match the preceding commit.
Source/manifests contain no `registry_values` import or package alias; historical
plan evidence remains labeled. Nightly formatting and `git diff --check` passed.

**Work:** Implement U61's option B without changing request execution semantics
yet. Absorb `crates/registry_values` into `crates/registry`; move the higher
registration/invocation responsibilities from today's `registry` into
`crates/registry_execution`. The latter can initially retain the current request
ABI until 3.3 binds it to `execution`. Keep the foundation independent of
`execution`, `registry_execution`, `auth`, `command`, and service SDKs.

Keep value reflection, owning erased values, offline constructors, pure
`From`/`TryFrom` conversions, and context-free operation metadata/planning below
execution. Review today's `Function` metadata/callback/cancellation coupling
explicitly: an operation descriptor and its executor-bound invocation have
different responsibilities. Do not relocate every function or planning type to
the upper crate merely because it is callable. Preserve authoritative registration
identity/provenance and explicitly migrate each associated registration handle
and its consumers; preserving the old handle's Rust type is not required.
The upper catalogue wraps original registrations in `Function` enum values
without registering or invoking them again. U63–U65 require direct constant-time
construction, static descriptor access, and variant/source-address identity.
Remove adapter storage and lookup methods; pass handles by value throughout
registry, UI, and SDK test consumers. Preserve custom-catalog boundaries,
distinct same-signature registrations, pure invocation, and cancellation policy.

Complete the package and consumer cutover in this slice: workspace membership,
package/dependency names, registry macros and owning imports, cache registrations,
SDK/type/CLI/UI consumers, examples, tools, tests, CI/release scripts, and public
documentation. Remove the old `registry_values` package and compatibility exports;
do not leave Cargo dependency aliases or forwarding crates. `azure_types` must
depend only on the pure registry, including its six principal conversion
registrations. Keep `AzureTenantId` in `crates/azure_types/src/tenant_id.rs`.
Preserve unrelated dependency/toolchain pins and the earlier extraction's
historical evidence.

**Validation:** The selected tests were inspected for local-only behavior before
execution. These commands validate the implemented package names.

```powershell
cargo test --offline --locked -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution
cargo clippy --offline --locked -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_entrypoint --all-targets -- -D warnings
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_plan --test discovery_simulation
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_explorer::root_action_snapshot::tests
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_explorer::invocation_controller::tests
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_explorer::production_controller::tests
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_explorer::explorer_engine::lifetime_tests
cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib object_explorer::produce_dump_plan_excalidraw_tests
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib cli::command::dump::dump_planner::tests
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib registered_
cargo test --offline --locked -p cloud_terrastodon_azure_devops_types --lib azure_devops_project_argument::tests
cargo test --offline --locked -p cloud_terrastodon_azure_devops --lib registered_response_constructor_generates_round_trippable_json
cargo check --offline --locked --workspace --all-targets
cargo metadata --offline --locked --no-deps --format-version 1
rustup run nightly -- cargo fmt --all -- --check
git diff --check
```

Inspect the normal/build package graph and model G1's pending auth/execution
edges; do not infer acyclicity from direct manifests alone. Verify constructor
and conversion discovery still presents each registration once and catalogue
traversal executes no constructors. Use focused existing pointer/handle tests
and meaningful conversion/catalogue coverage where the boundary changes it.
Audit old owning imports, package aliases, and re-exports, including grouped/glob
uses; historical migration documentation may retain explicitly labeled names.

**Completion criteria:** The full workspace compiles against pure `registry`
and upper `registry_execution`; `registry_values` no longer exists as a package
or compatibility import. `azure_types` and cache cannot reach executor-aware
registration through normal/build dependencies. The proposed auth/execution
graph is acyclic, and offline constructors/conversions retain their contracts.

### [x] 3.1 Rename credential runtime to auth and extract auth_types

**Completion notes — 2026-11-02 checkpoint:** Implementation and offline
verification passed. The service bridge described in this checkpoint is superseded
by task 3.1a; its test counts below are historical. `credentials` has become `auth`,
without compatibility exports.
`auth_types` owns `AuthSource`, resolved bearer/DevOps/general selections,
`AzureCredentialResource`, and nonserializable redacted `Authorization`.
`AzureTenantId` remains in `azure_types`. All consumer manifests, imports,
features, CLI surfaces, examples, and tool references use the new owners.

`AuthContext::Resolved` owns immutable, reference-counted runtime state; replacing
workload identity creates fresh caches and session state. Admission checks source,
tenant, disabled/headless policy, and configured identity without acquiring tokens. Workload
identity uses authority/tenant/client partitions; browser, CLI, and PAT contexts
use fresh session partitions shared only by their clones. PAT is captured once;
browser refresh is serialized across audiences and cannot silently replace its
tenant, client, or refresh chain after failed refresh. Direct REST execution no
longer resolves an implicit Auto context. Provider/executor injection remains 3.2;
the shared cache adapter's call to admission before every hit remains phase 4.

U68 selected persistent isolated CLI sessions. Explicit tenant login creates a
new generation under the application's auth configuration directory, with an
atomic per-tenant selector. Existing contexts retain their chosen generation.
Token and service subprocesses use that generation's explicit config directory;
OS file leases coordinate its refresh and execution. Brokers, ambient Azure
environment overrides, CLI file logging, debug output, and automatic login
retries are disabled for this path. Account identity is checked around execution;
logout revokes owned generations. Credential stdout/stderr and decoder excerpts
bypass general command artifacts and diagnostics, including failures. Acquired
tokens remain in sensitive runtime caches. Credential subprocess completion is
protected so cooperative cancellation does not release the lease while a launcher
child may still be writing credentials. Service callers acquire their lease before
deferral, so waiting remains cancellable. Logout collects per-generation failures
and continues revoking remaining sessions; revoked directories stay in place so
already-bound contexts fail closed. Explicit future abort/drop remains outside
cooperative completion guarantees.

Initial browser consent uses the same supported-audience set as explicit browser
login, while token redemption remains resource-specific. Failed refresh never
silently substitutes a new account or starts another interactive handoff.

Official behavior was verified from [CLI configuration](https://learn.microsoft.com/cli/azure/azure-cli-configuration),
[interactive tenant login](https://learn.microsoft.com/cli/azure/authenticate-azure-cli-interactively),
and the official CLI [profile](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py),
[persistence](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/auth/persistence.py),
and [logging](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/azlogging.py)
implementations. `--tenant` alone does not select a principal; no ambient cache
copy is used. Windows retains DPAPI cache encryption. Unix retains CLI's default
file persistence inside owner-only directories; shared keychain persistence is
not enabled. Synthetic checks establish local policy and coordination only;
no production data, credential store, CLI login, or service was accessed.

**Remaining boundary:** Explicit tenant login and CLI token acquisition retain
the owned session. Task 3.1a moves policy remediation and backend container listing
to ARM requests. Legacy SDK requests without an auth parameter remain in the
request-migration batches in 5/6; the
inventory names them explicitly. Their old ambient CLI behavior is not evidence
that they satisfy the new provider contract, and they must not be wrapped in
shared executor caching until migrated. The legacy PIM workflow likewise remains
outside provider identity binding; this slice only sanitizes its credential
diagnostics. The shared command/REST cache lifecycle still moves in phase 4.

**Verification evidence:** Inspected test bodies before execution. All selected
tests use synthetic values, temporary directories, or held local leases; no live
login, credential/configuration discovery, service request, production cache,
or local resource-type catalogue was used. Passed:

- `auth_types --lib`: 5 tests.
- `auth --lib auth_context::tests`: 17 tests across the matching context modules.
- `auth --lib browser_`: 16 tests; `auth --lib azure_cli_`: 10 tests, including
  cancellation of typed/raw service waiters and aggregation of logout failures.
- `auth --lib azure_access_token::test`: 5 passed, 2 live tests ignored;
  `auth --lib workload_identity`: 6 passed; each PIM response module: 1 passed.
- `command --lib execution_policy_tests`: 4 tests; `rest --lib request_execution::tests`:
  5 tests, including missing-tenant rejection before credential/network access.
- Filtered tenant-login CLI tests: 5; project-list CLI tests: 4; auth-card UI tests: 3.

These are per-command counts and include overlapping filters. Commands used
`cargo test --offline --locked -p cloud_terrastodon_<package>` (the initial
auth-types run also updated only workspace lock records). Nightly workspace
formatting, `cargo check --offline --locked --workspace --all-targets`, and Clippy
for `auth_types`, `auth`, `rest`, and `command` with `--all-targets -- -D warnings`
passed. CRLF-aware `git diff --check` passed. The normal/build graph plus planned
executor edges is acyclic across 37 workspace packages; `auth_types`,
`azure_types`, `cache`, and `registry` cannot reach runtime auth, command, REST,
or executable registration. All 820 external lock records are unchanged.
Documentation review checked all 68 ledger/traceability pairs, case-sensitive
relative links, and identifying-path patterns. Existing operational instructions
remain in [authentication](../authentication.md); the public
[auth migration guide](../auth-execution-migration.md) describes the cutover.

Task 3.1a refines this checkpoint and U74 records both completed tasks before
task 3.2 creates `execution`, defines the injected provider contract, and binds
request/executor execution.

**Work:** Implement resolved Azure bearer selection with explicit source/tenant
and Azure DevOps bearer/PAT selection. Preserve tenant validation, source policy,
headless/fail-fast behavior, and disabled authentication. Update token command
construction and tracked-tenant binding to reject missing bearer tenants; remove
Azure CLI active-tenant request compatibility. Executor selects compatible runtime
credentials, with no silent source substitution. Keep org/project separate.
After 3.0, rename runtime `crates/credentials` and its package to `crates/auth`
and `cloud_terrastodon_auth`. Move `AuthSource` into `crates/auth_types` alongside
resolved authentication selections; pure registrations use `registry`.
`AuthSource::Auto` remains application preference vocabulary, resolved before
building explicitly tenant/source-bound requests. Keep acquisition, configuration
discovery, token caches, browser/OS integration, and source resolution in `auth`.
`AzureTenantId` stays in `azure_types`; `auth_types` may depend on `azure_types`
and pure `registry`, with no reverse dependency from `azure_types` to `auth_types`.
Do not duplicate or re-export the tenant ID from `auth_types`.

Update every consumer's manifest and owning imports, including workspace package
declarations, SDKs, CLI/UI crates, registry code, tests, examples, tools, feature
references, build/release automation, and public docs in this slice. Remove the
old `credentials` package/export paths; `auth` must not re-export `AuthSource`
from `auth_types`. Do not add forwarding crates, compatibility re-exports, Cargo
dependency aliases preserving old names, or duplicate type definitions.
Implement G3's provider-bound identity partitions, including a fresh provider
session partition when source/tenant alone cannot establish the principal.
Verify isolated Azure CLI session provisioning against official documentation
before implementing it: every credential acquisition and service subprocess must
use the same bound identity, or fail before execution. Copying ambient CLI
configuration is not evidence of that binding. Credential results remain in the
provider's sensitive runtime cache; raw tokens, credential command output, and
secret-bearing errors must never enter general work-cache artifacts or diagnostics.
Keep this slice compilable using the existing runtime execution boundary;
task 3.2 introduces the provider trait and executor wiring, including the target
`auth -> execution` edge. Do not make the rename/extraction depend on an
unimplemented executor crate.

```powershell
cargo test --offline --locked -p cloud_terrastodon_auth_types --lib
cargo test --offline --locked -p cloud_terrastodon_auth --lib auth_context::tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib azure_tenant_auth_context::tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib azure_devops_auth_context::tests
cargo check --offline --locked -p cloud_terrastodon_auth -p cloud_terrastodon_azure
cargo check --offline --locked --workspace --all-targets
```

**Validation:** Replace the old active-tenant compatibility assertion with owned
selection-policy checks. Add offline tests of explicit token command tenant,
missing tenant, source mismatch, PAT handling, disabled auth, and workload tenant
mismatch. Inspect/filter token tests individually; do not run live token tests.
Use synthetic identity/session boundaries to test switching and refresh rejection,
independent-provider partitioning, and credential retention on success and failure.
Inspect imports and public exports for the runtime rename and moved AuthSource,
including grouped/glob imports, and verify every consumer uses the owning crate.
Recheck the normal/build dependency graph after the auth additions, including
configuration and registry dependencies. Review documentation
and examples alongside the workspace compile check; do not add compile-fail tests
solely to prove that removed exports are absent.

**Completion criteria:** Executable bearer requests always carry a tenant and
source, including Azure CLI. Serializable selection contains no runtime cache,
token, session, or cancellation scope. PAT needs no fabricated tenant.
`AzureTenantId` consumers retain `azure_types` imports; `AuthSource` and resolved
selection consumers import `auth_types`; runtime consumers import `auth`.
The old `credentials` package and exports are absent, and the full workspace
compiles without compatibility exports or aliases for the moved auth types.

### [x] 3.1a Replace the CLI service bridge with typed ARM operations

**Completion notes — 2026-11-03:** Implementation and offline verification passed
under U72; U73's response-field refinement also passed its checks. Independent
tracks delivered the policy-remediation request and
workflow, storage-container records/request and backend picker, and auth/command
bridge cleanup. The primary agent integrated module exports, documentation,
plan evidence, and serial verification. Existing staged work is preserved without
staging or committing this refinement.

[PolicyRemediationCreateRequest](../../crates/azure/src/policy_remediation_create_request.rs)
owns the typed PUT, zero-lifetime artifact key, and explicit tenant authentication.
Both workflow branches use it. Submission does not wait for policy deployments;
there is no existing remediation read cache to invalidate, and later resource or
compliance inventory refresh remains the caller's responsibility after completion.
[StorageAccountBlobContainerNamesListRequest](../../crates/azure/src/storage_account_blob_container_names.rs)
owns cached ARM pages, account-bound continuation validation, and parent-key
invalidation. The picker reuses it and forwards reload. Reusable records live in
`azure_types`, including shared ARM system data, typed remediation scope/name/ID,
container identity, and nested response vocabularies. Container names now accept
documented special names and reject trailing hyphens. The generic CLI service
runner, route/flag allowlist, and bridge-only command hooks are removed; persistent
credential sessions remain.

**U73 response-field refinement — complete:** Separate storage and remediation
owners reviewed core and nested records; the primary agent integrated shared
provenance, container decoding, and verification. Container identity/type/etag,
properties and page values are required, as are ordinary state and nested audit
fields. Remediation identity, assignment, progress, timestamps, deployment counts,
status text, and present failure-threshold percentage are required. A present
provenance record requires all six fields. Remaining optional fields have specific
omission/conditionality comments linked to official examples or field semantics;
no field is nullable solely because the schema omits `required`.

Removed obsolete empty-record `Default` derives. Container pages now use direct
typed `RestRequest::receive`, so missing required fields fail during decoding
with response artifacts retained; there is no supplementary missing-name/list
mapper. Validation strategy selected compilation/linting and official contract
review for these type-shape changes, without new derive-mirroring fixtures or
live requests. Passed `cargo clippy --offline --locked -p cloud_terrastodon_azure_types
-p cloud_terrastodon_azure --all-targets -- -D warnings`,
`cargo check --offline --locked --workspace --all-targets`, nightly workspace
format checking, and CRLF-aware diff checks. All 73 ledger/traceability pairs and
64 relative plan links resolve; identifying-path/email checks passed. Existing
tests were compiled but not rerun for this field-shape correction. The earlier
U72 test counts below remain historical evidence for that checkpoint.

**Decisions:** Use Policy Insights create/update at resource-group and
management-group scopes (`2024-10-01`) and Storage Resource Provider container
listing (`2025-06-01`). These use the existing ARM audience. Container listing
is management metadata inventory, with no claim that the user can access state
blobs. REST records belong to `azure_types`; executable operations and pagination
belong to `azure`. Reuse the existing container-names request and its typed output.

**Work:** Replace both remediation branches' CLI invocation with one reusable
typed SDK mutation. Retain the selected assignment/reference and supported scopes,
mutation execution artifacts, cancellation completion, and affected-read
invalidation where applicable. Convert `StorageAccountBlobContainerNamesListRequest`
to authenticated ARM pagination and have the picker call it with its reload flag.
Remove `bind_command`, its allowlist, and the auth context/session service-runner
methods and obsolete bridge-only command hooks. Keep CLI login, isolated token
acquisition, identity validation, generation leases, and logout. Update current
documentation without rewriting historical test evidence.

**Validation:** Inspect new/existing tests before selecting offline filters;
the former live `blob_works` test was removed. Check owned domain invariants, selection
conversion, cache scope/pagination validation where warranted, and preserved
session lifecycle tests. Run focused package Clippy and
`cargo check --offline --locked --workspace --all-targets`; format with the pinned
nightly formatter and inspect the diff. Official schema/source review supplies
endpoint/shape evidence; fabricated responses cannot prove service compatibility.

**Verification evidence:** Inspected each selected test before execution; all
inputs are synthetic and no service, credential/configuration discovery, production
cache, or local catalogue was accessed. Passed with `cargo test --offline --locked`:

- `azure_types --lib storage_account_blob_container`: 4 passed, 1 local preview
  test ignored; `azure_types --lib policy_remediation`: 3 passed.
- `azure --lib storage_account_blob_container_names::tests`: 2 passed, covering
  continuation destination/account confinement and tenant/source/account cache scope.
- `auth --lib azure_cli_`: 6 passed, retaining tenant admission, redacted failure,
  lease cancellation, generation binding, and logout failure aggregation.
- `command --lib command::execution_policy_tests`: 3 passed. The deleted bridge's
  flag-validation and debug-suppression tests are intentionally absent.

`cargo check --offline --locked --workspace --all-targets` passed. Clippy passed
for `azure_types`, `azure`, `auth`, `command`, and `entrypoint` with
`--offline --locked --all-targets -- -D warnings`; the final mapper formatting
simplification was rechecked through `azure` and `entrypoint`. Nightly workspace
format checking and CRLF-aware diff checks passed. Independent source/schema
reviews covered the SDK workflows, response shapes, credential destinations,
cache invalidation, and removal of bridge references. Documentation review checked
all 72 ledger/traceability pairs, relative links, and identifying-path patterns.
No commit or push was performed during U72/U73 implementation. U74 authorizes
the local checkpoint below. Task 3.2 remained unstarted at this checkpoint;
U75 subsequently authorizes it.

**U74 local commit checkpoint:** `Split auth runtime and migrate Azure operations to ARM REST`.
Review covered all 563 staged files: auth runtime/type ownership and consumers,
explicit tenant selection, persistent CLI login generations, ARM remediation and
container pagination, required response fields, manifests, and supporting docs.
The lockfile changes only workspace package records; no external version churn,
local catalogue, user notes, or credential/cache artifacts are included. The
staged tree matched the source validated by the U73 workspace check and focused
Clippy; this checkpoint adds documentation only. CRLF-aware staged/working-tree
diff checks passed. The next described task was 3.2: an explicit shared executor, bound requests,
and an injected authentication provider. Registry integration remains task 3.3;
the common Command/REST cache lifecycle remains phase 4.

**Completion criteria:** Both workflows use typed ARM requests, the picker shares
the existing container request and refreshes its pages, no bound CLI service
runner or allowlist remains, credential session behavior is preserved, and
offline checks plus documentation accurately record the supported boundary.

### [x] 3.2 Bind requests to a scoped executor through both execute spellings

**Completion notes — 2026-11-03:** U75's lower execution contract, runtime
credential-provider adapter, and application startup binding are implemented and
verified offline. These independent source tracks were integrated before the
workspace checks. Implementation under U75 did not stage, commit, push, or use
live authentication; U78 separately authorizes the local checkpoint below.

The new [execution crate](../../crates/execution/src/lib.rs) contains
`CloudTerrastodonExecutor`, `ExecutableRequest`, awaitable `RequestExecution`,
`AuthenticationProvider`, and `AnonymousAuthenticationProvider`. Both execute
spellings bind requests through a linked child scope. The executor exposes
cooperative/protected child helpers and adapts the existing explicit cache owner
with admission before reuse and semantic child dependency tracking. Credential
acquisition has a separate infrastructure boundary that rejects recursive
authorization, general cached work, and direct cache access through `cache()`.
Ordinary outputs can borrow invocation data; retained shared outputs alone require
`Send + Sync + 'static`.

Authentication admission is checked at binding, before cache access, before and
after initialization, and before returning the result. A revoked waiter cannot
receive a result or retry initialization; changing the partition during an
initializer cannot publish its output under the previous identity. Semantic child
admission failures mark the active parent and its ancestors nonreusable through
[WorkAttempt::prevent_reuse](../../crates/cache/src/work_attempt.rs), even when
the parent catches the error and returns a fallback. Completed tokens cannot
poison already-accepted results.

[CredentialAuthenticationProvider](../../crates/auth/src/credential_authentication_provider.rs)
adapts an already-selected `AuthContext`; construction does not discover
configuration or acquire credentials. Admission retains source/tenant policy and
session partitions. The existing sensitive token/session paths remain the
acquisition implementation until production adapters migrate in phase 4.

[AppContext](../../crates/app/src/app_context.rs) owns one private `WorkCache` and
offers `executor()` to bind its current public `auth` and `cancellation` values.
Repeated bindings share cache ownership and the resolved credential session;
already-returned executors keep their original bindings. This avoids storing a
second mutable scope/provider snapshot while preserving existing consumers.
Startup constructs the cache handle inside the handler's bounded scope, using
`AppDir::Commands.join("work")`. Construction validates that path without cache
I/O; parsing does not construct the runtime resources. Without the `auth` feature,
the executor receives the anonymous-only provider. App's touched types now have their
own modules with the existing public exports retained.

**Verification:** The following commands passed against the completed source:

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib credential_authentication_provider::tests
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --lib
cargo test --offline --locked -p cloud_terrastodon_app --features auth --lib app_context::tests
cargo test --offline --locked -p cloud_terrastodon_cache --lib work_attempt::tests
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_cache -p cloud_terrastodon_execution -p cloud_terrastodon_auth -p cloud_terrastodon_app --all-targets --features cloud_terrastodon_app/auth -- -D warnings
cargo +nightly fmt --all -- --check
```

The execution suite passed **12 tests** covering equivalent execute spellings,
borrowed/non-`Sync` ordinary outputs, borrowed non-`Send` caller callbacks, shared
decoding, warm-child dependencies, caller cancellation/waiter retry, protected
completion and owed cleanup, admission revalidation, and credential isolation.
The provider suite passed **4 tests** for source/tenant policy, session partition
preservation, anonymous operation, and PAT audience rejection. The no-auth app
library passed **32 tests**, including existing parsing/output coverage and the
new scope/cache binding test; the auth-enabled app binding suite passed **2 tests**.
The cache's completed-token regression passed **1 test**. These tests use
synthetic values and isolated or untouched temporary cache roots, with no live
credentials, configured cache reads, or service data.

Normal/build metadata review found **37 local workspace packages** and no cycle.
`execution`, `auth_types`, `azure_types`, `cache`, and pure `registry` cannot reach
`auth`, `command`, `rest`, or `registry_execution`. The no-default-features app
dependency tree excludes those runtime packages too. Lockfile changes add only
18 lines for local package dependencies and the new execution package, with no
external dependency-version changes.

Final documentation review found all **75 guidance ledger/traceability pairs**,
resolved **162 relative file links** across the five revised plan/guide/changelog
files, and found no identifying local-path or email patterns. CRLF-aware diff
checks passed. Earlier checkpoint evidence remains explicitly historical.

Caller callbacks remain outside canonical decoding; the synthetic wrapper tests
do not establish a lifecycle-observer API. Task 3.3 is next. Current
SDK/Command/REST requests retain their existing execution until their scheduled
migration; this completion does not claim a production transport-cache cutover.

**U76 naming refinement:** Rename the private type and field to `ExecutionMode`
and `mode`, with `Normal(Option<WorkAttempt>)` and
`NoWorkCacheOrAuthorizationReentry`. The latter prevents general work/artifact
cache access and recursive authorization; ordinary nested execution, linked
scopes, and provider-owned sensitive token caches remain available. This changes
names and documentation only. All 12 execution-contract tests and the focused
nightly formatting check passed; the U76 ledger/traceability pair and CRLF-aware
diff checks passed. Existing user staging was preserved.

**U77 startup diagnostics refinement:** Hook-installation and hook-retention
conflicts now print to stderr instead of silently discarding the errors. Fallible
hook construction still propagates through `?`. The panic handler's best-effort
stderr writes retain their existing behavior to avoid panicking during a panic.
The existing isolated process test
`an_early_error_report_does_not_prevent_runner_startup` passed with
`--offline --locked --no-default-features`, exercising an already-installed eyre
hook without authentication or service access. Focused nightly formatting and
CRLF-aware diff checks passed.

**U78 local commit checkpoint:**
`Add scoped executor with shared cache and injected authentication`.
The reviewed 31-file scope contains the execution crate, runtime provider, app
binding and extracted startup modules, active-attempt reuse prevention, synthetic
tests, facade/manifests, and supporting documentation. The lockfile changes only
local package records. U76/U77's focused checks follow the broader task 3.2
verification above; this checkpoint adds plan documentation only. Task 3.3 next
binds registry invocation and discovery to the executor. Command/REST adapters
remain phase 4; this checkpoint does not authorize their implementation or a push.

**Work:** Implement G1/G2's resource coordinator and operation contract. Bind
request data to an awaitable execution using explicit auth/cache handles and a
bounded CancellationScope. Provide forwarding request.execute(executor) and
executor.execute(request) entry points. Propagate child scopes correctly and
keep caller-specific validators/observers outside canonical cached decoding.
Retain operation effects and protected completion policy; do not detach fetches.
Establish the minimal application binding required for SDK migration. Enforce
G2's shared-result bounds only where memoization requires them; keep noncacheable
and borrowed callback contracts separately testable.
Define `AuthenticationProvider` in `execution`, implement it in `auth`, and wire
the runtime provider at application startup. The lower executor must not import
`auth`; its injected provider may use the executor for credential infrastructure
without recursive authentication or general-cache retention of credential output.

**Validation:** Focused synthetic checks and feature-boundary compilation:

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib credential_authentication_provider::tests
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --lib app_context::tests
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --features auth --lib app_context::tests
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features
```

**Completion criteria:** Synthetic operations prove equal behavior through both
spellings, shared handles across clones, child-scope cancellation, parent deferral
rules, caller cancellation/waiter retry, and no auth access for local-only work.
Request inspection/offline construction does not initialize runtime services.

### [x] 3.3 Add executor-aware registry invocation and discovery

**Execution tracks:** Registry ABI and registrations; default production and
offline discovery; direct UI invocation consumers. The integration owner handles
shared manifests, executor child cancellation, validation, and this living plan.
These tracks share the explicit executor API and do not independently build or
change dependencies. Source baseline: `3478e3bb`, clean before this slice.

**Completed under U79 (2026-11-03):** `AsyncExecutionFn`, lifetime-bearing
`InvocationFuture`/`FunctionInvocation`, and `register_executable_request!` bind
data-only requests to a borrowed executor. The macro exposes a direct
`REGISTERED_EXECUTABLE_REQUEST` handle and enforces nonempty effects. Ordinary
erased outputs remain `Send` without requiring `Sync`; retained shared results
keep the cache's stronger bounds. Default production uses the same executor.

`DiscoveryPlanBuildRequest` now contains only editable planning data and runs
through `ExecutableRequest`. Planning/simulation recognizes the new registration
contract, requires explicit selection inputs, traverses shared pointer/list
outputs, and retries empty collections behind shared pointers. Five legacy SDK
metadata tests remain explicitly labeled, while new offline fixtures use
serializable selections and `Arc<Vec<Arc<T>>>` responses. Request callbacks and
planning-provider probes fail if offline discovery attempts execution/authentication.

The entrypoint binds `AppContext::executor()` through Ratatui to the invocation
host. Workers own an executor clone and create/await borrowed invocations inside
that worker, preserving borrowed-value leases and shutdown ordering. The executor
adds `cancellable_child_scope` and `with_async_child_scope`, explicitly enabling
the pinned cancellation crate's `inheritance` feature. Independent cancellation
does not cancel sibling jobs; cooperative UI shutdown retains its own signal.
Only test hosts can synthesize disabled resources; production admission is an
explicit host responsibility.

G2's separately tagged legacy IntoFuture callback remains for the request
families scheduled in phases 4–6. It receives an executor-owned child scope before
binding a legacy cancellation field. New executable requests receive no such
field injection. This is not a production Command/REST cache cutover and adds no
default executor or compatibility re-export.

**Recovery:** The first compile stopped on insufficient local disk space. The
user ran `cargo clean`; the one interrupted test-file write was restored from
its exact original source before applying the intended change. Validation then
completed from the clean build directory. No production data or credential state
was read.

**Discovery admission boundary:** Offline planning/simulation requires explicit
auth-selection and resource-scope inputs and excludes writes/solicitation. It
does not acquire credentials, call provider admission, or prove that arbitrary
tenant/organization values may be combined. Runtime discovery admission remains
task 6.3; synthetic reachability is not evidence of live authorization.

**Verification:** All 127 selected offline tests passed: execution contracts 13;
pure registry 7; registry execution 55; Azure DevOps planning 7 and simulation 4;
UI host/controller/production/lifetime/export checks 40; entrypoint small-stack
dispatch 1. The controller deferral fixture now protects work inside the actual
invoked request scope, retaining its assertions that source leases survive
cancellation until required work completes. An externally opened host deferral
would protect a sibling and does not establish that operation's completion duty.

Commands used for the focused tests and checks:

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo test --offline -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution --lib
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_plan --test discovery_simulation
$filters = @(
    'object_explorer::invocation_host::tests',
    'object_explorer::tokio_invocation_host::tests',
    'object_explorer::invocation_controller::tests',
    'object_explorer::production_controller::tests',
    'object_explorer::explorer_engine::lifetime_tests',
    'object_explorer::produce_dump_plan_excalidraw_tests',
    'object_browser::event::tests::export_plan_field_can_use_an_explicitly_configured_plan_builder'
)
foreach ($filter in $filters) {
    cargo test --offline --locked -p cloud_terrastodon_ui_ratatui --lib $filter
    if ($LASTEXITCODE -ne 0) { throw "UI verification failed: $filter" }
}
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib cli::tests::build_folder_dispatch_reports_auth_error_on_a_small_stack -- --exact
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_execution -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution -p cloud_terrastodon_azure_devops -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_entrypoint --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
```

The unlocked offline registry-test invocation updated only local package records
in `Cargo.lock`; dependency versions/pins are unchanged. Workspace compilation
includes both consumer examples. Metadata review confirms all 37 workspace
packages form an acyclic normal/build graph, `registry_execution -> execution`
exists, and `registry`, `auth_types`, `azure_types`, `cache`, and `execution`
cannot reach runtime `auth`, `command`, `rest`, or `registry_execution`.
The pure registry also cannot reach `execution`. Clippy and formatting passed;
final whitespace/privacy checks passed. Existing user staging is preserved.
No live compatibility, Linux test result, commit, push, or publication is claimed.

**Follow-up refinements and checkpoint (U80–U82):** The anonymous provider is now
`AnonymousAuthenticationProvider`. `UnreachableAuthenticationProvider` lives in
`execution` behind its opt-in `test-support` feature and fails immediately if a
test unexpectedly attempts authentication or provider admission. The provider
refinement passed 55 registry tests and targeted Clippy/formatting checks.

`WorkCache::temporary()` and `WorkCache::from_tempdir()` retain a `TempDir` guard
inside the shared cache state. Cache clones, artifact owners, and asynchronous
workers retain that state; invalidation workers also retain the cache rather than
only a copied path. Cleanup follows the last owner. `WorkCache::new(path)` retains
caller-managed storage. This refinement passed 146 focused tests: cache library
47, registry execution 55, artifact invalidation 9, artifact store 13, temporary
roots 4, UI host 17, and UI export 1, plus targeted Clippy/formatting checks.
These are the recorded refinement results, not tests newly rerun for U82.

U82 committed the reviewed 62-file staged scope as `b00bff46`,
`Bind registry and UI execution to explicit executor ownership`. The working tree
was clean after that commit; no push was requested or performed. New task 4.1
implementation follows this checkpoint and is not included in that commit.

**Work:** Bind `registry_execution` invocation metadata/macros, default production,
discovery inputs/admission, and cancellation to G1's `execution` contract.
Store the new callback in `RuntimeFunction` and delegate through the existing
`Function::Runtime` handle without reintroducing adapter records or lookup.
`registry_execution` may depend on `execution`; pure `registry` must not depend
on either. Runtime `auth` implements the injected provider interface; neither
`execution` nor its lower vocabulary depends on the implementation. Preserve
conversion/arbitrary pure operations, nonempty executable effects, Solicit
exclusion for unattended discovery, and explicit auth/scope admission. Replace
synthetic AuthContext request inputs with serializable selection data. Register
shared output shapes and verify pointer/list traversal without live execution.

**Completion criteria:** Registered synthetic requests execute with explicit
resources; offline planning never resolves credentials or invokes operations;
shared collection outputs remain discoverable. The graph stays acyclic.

## Phase 4 — one Command/REST execution lifecycle

### [x] 4.1 Route subprocess execution through shared work

**Completed task 4.1b (U85–U86):** Process ownership, shared artifact binding, and
owned execution prerequisites are committed below. The Command API and all
production callers were migrated and final offline verification passed. U87/U88
corrected the overly broad executor requirement in completed task 4.1c below. REST lifecycle
integration follows that correction in task 4.2.

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
artifact key. Credential Azure CLI subprocesses use the provider's bound session;
service operations use explicit-context REST rather than an ambient CLI runner.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib
cargo test --offline --locked -p cloud_terrastodon_command --test cacheable_work_tests
cargo test --offline --locked -p cloud_terrastodon_command --test command_cancellation_tests
```

**Completion criteria:** Local subprocess fixtures prove no retry-lock recursion,
one accepted canonical result per read identity, preserved cancellation/artifacts,
and independently executed mutations. Existing tests are migrated to the new
boundary rather than maintaining a second production implementation.

#### [x] 4.1a Own uncached process attempts and keep retries below admission

**Execution tracks:** Process/input resource ownership; local subprocess
regressions; integration of the existing Command retry and cache boundary. Only
the integration owner runs builds and updates shared manifests and this plan.

**Implementation:** `CommandProcess` owns the child and its input workspace;
stdin writing and stdout/stderr collection are awaited together under the same
timeout, with kill-on-drop retained. `CommandInputWorkspace` stages each attempt's
files in a unique temporary directory, validates portable relative paths through
`cache::ArtifactFilePath`, and keeps cleanup ownership through process completion.
The staging worker itself owns the guard if its awaiting future is dropped.
Process inputs no longer share filenames in the retained cache directory.
`CommandKind::apply_args_and_envs` returns `Result<Option<CommandInputWorkspace>>`;
manual subprocess callers must retain that public guard through child completion.

`CommandBuilder::run_uncached` keeps the existing single retry inside a loop.
It never revisits cache lookup or repeats the polite delay. Nested legacy login
commands also run below that admission boundary and cannot recursively retry.
The existing outer Command path still owns caller attribution, cancellation,
final status acceptance, and retained output. No parallel production adapter or
compatibility API was added. Checked retry-duration arithmetic returns an error
instead of overflowing. The unused command dependency on `async-recursion` is
removed; the local package record adds `cache` without changing version pins.
Explicit `Box::pin` boundaries preserve the previous heap ownership at ordinary
and nested login entry points, avoiding larger inline caller futures after the
recursion macro is removed.

**Verification (2026-11-03):** All 48 selected local tests passed: command library
26, legacy in-process cache 7, cancellation 7, process regressions 5, echo 2, and
stdin 1. Inspected synthetic PowerShell fixtures cover
concurrent same-name inputs, cache injection during retry, bounded retry/failure
artifacts, duplex pipe draining, and timeout. Direct input-admission coverage
checks that absolute/traversing paths cannot overwrite an outside sentinel.
Workspace/all-target compilation, Command/all-target Clippy with warnings denied,
and workspace nightly formatting passed after the final boxing refinement.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib --test command_process_tests --test command_cancellation_tests --test cacheable_work_tests --test command_stdin_tests --test command_echo_tests
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_command --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

No Azure, Terraform, credential, or live service tests were run. This is Windows
local-process evidence, not Linux or live compatibility evidence. Source review
confirms retry branches call only the uncached process path and the nested login
helper, not the cache boundary. These changes were uncommitted at the end of U82;
no production single-flight or typed memoization was claimed.

**Commit checkpoint (U83):** The reviewed ten-file staged scope was committed as
`c981de2b`, `Isolate command attempts and keep retries below cache admission`.
The working tree was clean afterward. Verification above is the recorded result
for that slice; no push or additional service validation was performed.

#### [x] 4.1b Bind Command artifacts and decoded results to the executor

**Execution tracks:** Shared executor binding and integration; disk/memory
lifecycle tests; admission/retention policy tests; independent source review.
Only the integration owner runs builds and updates the plan. Validation uses
temporary cache roots, synthetic payloads, and injected offline providers.

**Completed artifact binding prerequisite (2026-11-03):**
`CloudTerrastodonExecutor::cached_work_with_artifacts`
shares admission and typed coordination with `cached_work` through a private
binding procedure. Its initializer receives an `ArtifactEntry` for the exact
admitted identity and active `WorkAttempt`; adapters do not reconstruct ancestry
or resolve a different authentication partition after waiting. Memory hits skip
artifact construction and decoding. Disk hits must return
`LoadedArtifact::decoded` so the original freshness/generation proof survives.
The initializer retains observed outcomes and selects successful canonical leaf
artifacts through the existing store; aggregate publication remains forbidden.

`ExecuteEveryTime` uses zero effective artifact validity, even when the request
supplies a positive duration. This prevents disk substitution and reusable
selection while preserving inspectable outcomes. `ArtifactRetention::Disabled`
suppresses raw disk I/O independently of decoded memory sharing. Credential mode
cannot use the shared boundary. Callers retain cancellation policy and must wrap
effectful work in the executor's deferral; this binding adds no automatic root,
transport, provider, or compatibility path.

**Verification:** All 21 focused executor tests passed: 13 existing execution
contracts, 3 artifact-binding scenarios, and 5 artifact-policy scenarios. The new
checks prove active disk import and memory reuse without another decode,
invalidation during blocked decoding without stale refill, child dependency
invalidation with no durable aggregate selection, independent execute-every-time
invocations despite an existing positive-lifetime disk entry, disabled raw
retention with memory sharing, credential-mode rejection before initialization,
and quarantine after authentication changes following raw publication. A caught
nonreusable-child admission failure prevents caching its parent's fallback.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test artifact_binding_tests --test artifact_policy_tests --test execution_contract_tests
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_execution --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

Workspace/all-target compilation, targeted Clippy, formatting, whitespace,
ledger/traceability, and plan-privacy checks passed. No dependency versions or
lockfile entries changed. These changes were uncommitted at the end of U83 and
are now included in checkpoint `811468f3` below. These are synthetic local Windows
checks; no credentials, production data, or remote services were accessed. They establish the executor
boundary, not a completed Command/REST transport cutover. Source review found
no outstanding correctness issue in this prerequisite.

**Commit checkpoint (U84):** The reviewed six-file staged scope was committed as
`811468f3`, `Bind executor artifacts to admitted cache attempts`. The working
tree was clean afterward. No push or live validation was performed.

**Completed owned execution prerequisite (2026-11-03):**
`CloudTerrastodonExecutor::uncached_work` admits independent execution and returns
the caller-owned value without cache lookup, joining, retention, or a `Sync` bound.
Its output may borrow invocation data. `uncached_work_with_artifacts` additionally
claims the canonical operation/output type at binding, before polling or disk I/O,
and supplies an admitted artifact entry with zero validity. Its `T: 'static`
bound serves only that ownership claim; `T: Sync` and `Arc<T>` are unnecessary.
Existing selections cannot substitute for execution, and new outcomes remain
inspectable without becoming reusable selections.

Both APIs use linked cancellable scopes and inherit caller deferrals. The caller
still owns protecting effects, decoding, and required artifact completion together.
Credential infrastructure is excluded. Admission is checked synchronously, then
the same authentication partition is required before polling and after completion.
The cached and owned paths share these admission/result checks while retaining
the cached initializer's additional checks around publication. A reusable parent
cannot admit an uncached child; rejection prevents ancestor reuse and quarantines
already-published artifacts even if its caller catches the error.

**Verification:** All 30 focused executor tests passed: the previous 21 scenarios,
4 owned-work lifetime/cancellation scenarios, and 5 owned-artifact policy scenarios.
The new tests cover borrowed and non-`Sync` outputs, borrowed non-`Send` caller
policy, independent simultaneous executions, scope closure, pre-poll/in-flight
cancellation, inherited deferral, canonical owner/type conflicts before callbacks,
no disk substitution or selection, caught-child rejection, authentication changes,
and credential-mode exclusion. Independent source review found no correctness issue.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test uncached_work_tests --test uncached_artifact_tests --test artifact_binding_tests --test artifact_policy_tests --test execution_contract_tests
cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_execution --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

The focused tests, workspace/all-target compilation, Execution/all-target Clippy
with warnings denied, workspace nightly formatting, whitespace, U01–U84
ledger/traceability, and plan privacy checks passed.
These prerequisite changes were uncommitted at the end of U84 and are included
in checkpoint `5f627dfc` below. Validation uses synthetic temporary caches
and injected providers on Windows; no credentials, production data, or remote
services are involved. Dependency pins and the lockfile are unchanged.

**Commit checkpoint (U85):** The six-file owned-execution prerequisite was
committed as `5f627dfc`, `Add admitted owned execution without shared result bounds`.
No push was requested or performed.

**Full Command cutover (U86; complete 2026-11-03; narrowed by 4.1c):** This
checkpoint made Command receive the
caller-owned executor throughout its owned/shared, raw/typed, polite, mapper,
and validator APIs. `run` and `run_raw` always execute independently and return
owned values. `run_cached` and `run_raw_cached` require a canonical operation
owner and an explicit cache key, returning `Arc<T>`. Their mapper/validator
variants apply caller policy on hits and joiners without copying or evicting the
canonical value. Owned outputs still need no `Sync`; callbacks may borrow
non-`Send` caller state. Default deferral spans process execution, decoding,
caller policy, and retained artifacts. Read callers opt into cancellation.

The shared adapter decodes before publishing, loads disk with its original
freshness/generation proof, and rejects only the exact entry on canonical disk
failure. Independent variants and child pages survive rejection. Retries remain
inside the admitted initializer; failed canonical decoding is never replayed.
Sensitive output suppresses raw files and decoder/process payload diagnostics.
Command kind participates in the fingerprint because it changes argument
semantics; shared execution requires captured output, while owned execution can
display output on every invocation. The implicit login retry is removed.

HCL/Terraform, interactive workflows, CLI dispatch, local DevOps configuration,
Gitea, and GUI paths pass the existing application executor and linked child
executors. No scope-only adapter or implicit executor constructor remains in
production Command consumers. Local DevOps request objects are data-only and
registered through `ExecutableRequest`; configuration reads share raw output.
Gitea request objects similarly use explicit execution. Tea selects exactly one
profile matching the requested instance and receives `--login`; service reads
remain independent because an external profile name cannot supply a credential
generation. General subprocess execution does not consult the Azure credential
provider and is not a security sandbox for external programs such as tea or
Terraform.

Remaining Azure/DevOps service CLI operations now use the established explicit
context REST boundary: Graph/current-user/group membership, ARM role assignment
and name availability, public pricing, Key Vault secret metadata, ACR repository
metadata, and DevOps agent/core/git/graph/licensing operations. Key Vault and ACR
use their dedicated credential audiences. ACR obtains the actual registry login
server and performs challenge-bound token exchanges outside raw artifacts;
credential redirects are disabled. Pagination retains each operation's selected
scope. Local account/default selection and logout remain auth infrastructure
with bounded direct processes, no interactive login retry, no reusable semantic
result, and no general artifact cache.

Independent reviews corrected command-kind fingerprinting, exact artifact
rejection, DevOps Server URL preservation, documented response omissions, and
case-normalized project-name scopes. Role/group/licensing mutations invalidate
affected reads after attempted execution even when response decoding or artifact
retention fails; both failures remain reported if invalidation also fails.
Storage/Key Vault availability checks retain zero-validity artifacts and always
contact the service. Readable pricing scope components use the shared escaping
policy. Local account failures preserve process status/stderr without including
account-list stdout in diagnostics.

These REST requests still use the existing REST cache/lifecycle until task 4.2;
their source/tenant context is explicit, but the remaining data-only SDK and REST
executor migration is not claimed complete. Legacy cache helpers in `command`
remain solely for REST and its scheduled SDK consumers. Validation is synthetic
and local; remote API compatibility requires later authorized use, never
production data fetched for development.

**Verification:** All 51 reviewed Command tests passed: 23 library, 7 legacy
cacheable-work (retained for REST), 7 cancellation, 5 process, 2 echo, 1 stdin,
and 6 executor-cache tests. They cover one canonical decode across joiners/hits,
per-caller borrowed policy, non-`Sync` owned outputs, owner/type rejection,
disabled sensitive retention, independent execution, retry isolation, protected
completion, and exact disk rejection without replay or child invalidation.
The command-kind regression fixture initially omitted PowerShell's `-Command`
argument; correcting the fixture made the complete suite pass.

All 16 additional reviewed offline tests passed. These exercise ACR challenge
validation and secret suppression, no-auth rejection before network access,
continuation boundaries, domain URL/timestamp invariants, delegated-consent
policy, logout cancellation/cleanup, unambiguous tea profile selection, Server
URL preservation, registered response construction, and project cache identity.
No credentials, production cache, configured account metadata, service data, or
live cloud/tea/Terraform operations were used. These checks do not establish live
service compatibility; Linux validation remains outstanding.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib --test executor_cache_tests --test command_process_tests --test command_cancellation_tests --test cacheable_work_tests --test command_stdin_tests --test command_echo_tests

$cases = @(
    @('cloud_terrastodon_auth', 'container_registry_'),
    @('cloud_terrastodon_rest', 'container_registry_request_execution::tests::'),
    @('cloud_terrastodon_azure', 'container_registry_pagination::tests::'),
    @('cloud_terrastodon_azure_types', 'container_registry_login_server::tests::'),
    @('cloud_terrastodon_azure_types', 'utc_date_time_epoch_seconds_proxy::tests::'),
    @('cloud_terrastodon_azure_types', 'entra_directory_object_url::tests::'),
    @('cloud_terrastodon_auth', 'initial_browser_consent_covers_named_delegated_permissions'),
    @('cloud_terrastodon_auth', 'logout_all_auth_sources::tests::'),
    @('cloud_terrastodon_gitea', 'instance_selection_never_falls_back_to_an_unrelated_default_or_ambiguous_profile'),
    @('cloud_terrastodon_rest', 'explicit_service_accepts_server_hosts_without_inferring_unknown_destinations'),
    @('cloud_terrastodon_azure_devops_types', 'core_api_preserves_server_scope_but_cloud_only_areas_reject_it'),
    @('cloud_terrastodon_azure_devops', 'registered_response_constructor_generates_round_trippable_json'),
    @('cloud_terrastodon_azure_devops', 'project_cache_scopes_preserve_selector_identity_and_name_matching')
)
foreach ($case in $cases) {
    cargo test --offline --locked -p $case[0] --lib $case[1]
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

cargo check --offline --locked --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_command -p cloud_terrastodon_execution -p cloud_terrastodon_cache -p cloud_terrastodon_auth -p cloud_terrastodon_rest -p cloud_terrastodon_azure -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_types -p cloud_terrastodon_gitea -p cloud_terrastodon_hcl -p cloud_terrastodon_entrypoint -p cloud_terrastodon_ui_egui -p cloud_terrastodon_azure_cli -p cloud_terrastodon_azure_devops_cli --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

Final workspace/all-target compilation, affected-crate/all-target Clippy with
warnings denied, nightly formatting, and whitespace checks passed. Searches
confirm no CommandBuilder remains in the Azure SDK; DevOps retains it only for
local configuration. Removed Command cache methods have no compatibility API.
The lockfile changes only workspace package dependency edges; dependency pins
are unchanged. The U01–U86 ledger and traceability each retain all 86 entries;
the plan privacy audit remains free of personal names and absolute local paths.
The completed cutover is uncommitted after checkpoint `5f627dfc`; no push occurred.

#### [x] 4.1c Keep local execution and cancellation helpers independent

**Work (U87–U88):** Plain Command `run`, `run_raw`, polite, mapper, and validator
methods take only `CancellationScope`. They execute independently without cache
admission, synthesized artifact identities, or persistent artifacts. An attached
cache key is rejected instead of silently ignored. Explicit `*_with_artifacts`
methods retain executor admission and owned outcome artifacts; cached methods
retain canonical sharing. All paths share process lifetime, retry, and decoding
implementations and preserve default deferral and cancellation-safe opt-in.

HCL audit, writing, formatting, import preparation, provider handling, local
Terraform workflows, and their cancellation-only callers receive scopes rather
than executors. Fresh Azure DevOps configuration/default helpers also use scopes;
their explicitly registered cached request forms keep the executor. Gitea
semantic work and configuration mutations retain explicit managed execution.
No helper manufactures an executor, cache, or authentication provider.

Executor child-scope methods retain their real responsibility: preserving cache
ancestry and authentication restrictions for children that still need executor
resources. Lower artifact processing receives its admitted entry and scope.
Scope-only primitives cannot enforce semantic ancestry; managed requests must
admit uncached/cached work before invoking physical operations. `execute` alone
does not enforce effects. Cached initializers may invoke physical transport
without re-admitting it as a separate nonreusable semantic child.

**Completion criteria:** All affected callers compile with their narrowest
dependency. Local process tests construct no executor, cache, or provider and
verify independent outputs, cancellation, required completion, caller diagnostics,
and sensitive suppression. Managed tests preserve artifact retention and parent
admission. Run reviewed synthetic checks only, then update contracts/inventory,
ledger/privacy evidence, and this task's status together.

**Completed 2026-11-05:** HCL and egui have no executor references or direct
execution/cache dependencies. The example local command also constructs no cache
or provider. Local menu/workflow dispatch passes scopes; genuine Gitea/registry
operations retain managed execution. Gitea tenant add/list and alias resolution,
and the default-organization picker, were narrowed after a final caller audit.
The picker's response handler keeps its executor for mutation and invalidation.

The shared process/retry/decoder implementation serves both paths. Artifact
processing receives an already-admitted entry plus its scope. Mapper/validator
callbacks have a captured caller span covering the callback lifecycle, and
blocking decoding preserves the caller's tracing dispatcher. An independent
review found and corrected the callback span gap; no remaining material issues
were reported in the boundary/caller review.

**Verification:** All 57 reviewed Command tests passed: 23 library, 7 legacy
cacheable-work, 8 cancellation, 5 process, 2 echo, 1 stdin, 7 executor-cache, and
4 plain-command tests. Six new regressions prove scope-only repeated execution
with non-`Sync` owned/borrowed outputs, rejection of explicit cache intent before
process start, tracing/source-error retention, sensitive suppression, protected
local completion after cancellation, and managed-child rejection under a cached
parent even when its error is caught. All five pure HCL audit tests also passed.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib --test executor_cache_tests --test plain_command_tests --test command_process_tests --test command_cancellation_tests --test cacheable_work_tests --test command_stdin_tests --test command_echo_tests
cargo test --offline --locked -p cloud_terrastodon_hcl --lib audit::tests::
cargo check --offline --workspace --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_command -p cloud_terrastodon_execution -p cloud_terrastodon_hcl -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_cli -p cloud_terrastodon_gitea -p cloud_terrastodon_entrypoint -p cloud_terrastodon_ui_egui --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git diff --check
```

Workspace/all-target compilation and affected-crate/all-target Clippy passed.
The unlocked offline check reconciled only removed workspace dependency edges;
subsequent locked checks/tests passed and dependency versions were unchanged.
Formatting, whitespace, all 88 ledger/traceability pairs, and plan privacy checks
passed. Tests use synthetic temporary storage/local processes; no production
services, credentials, configured command paths, or production caches were read.
Broad HCL tests remain deliberately unrun because some invoke configured Terraform
or inspect profile/plugin state. No commit or push was requested or performed.

**License vocabulary refinement (U89; 2026-11-05):** The REST replacements now
share `AzureDevOpsAccountLicenseType` and `AzureDevOpsMsdnLicenseType` with
`AzureDevOpsLicenseType::Account` / `Msdn`. The combined type preserves existing
entitlement strings, aliases, display labels, and interactive choices; `None`
normalizes to `Account(None)`. The account-only REST field no longer uses a
combined type, and the duplicate rule-specific MSDN enum is removed. Unmodeled
family response values remain observable in release builds, with debug diagnostics;
combined unknown identifiers retain their spelling and family. Mutation conversion
rejects unknown values, including nested ones, before CLI authentication.

Group-rule matching now uses the active licensing source and corresponding family
field. Account and MSDN values with the same name remain distinct in comparisons
and summaries. The existing limited cost estimator is unchanged; this refinement
does not introduce prices for newly modeled historical values.

**Refinement verification:** Nine reviewed synthetic tests passed: six license
type/rule tests, two CLI summary tests, and the existing registered update-response
constructor test. They cover alias normalization, family distinctions, combined
serialization, unknown-value retention/rejection, and source-aware group matching.
The Azure DevOps CLI dependency graph and all its targets compile. All-target
Clippy for the types, SDK, and CLI crates passed with warnings denied. Formatting,
whitespace, all 89 ledger/traceability pairs, and plan privacy checks passed.
No live requests, credential resolution, or production caches were used.

```powershell
cargo test --offline --locked -p cloud_terrastodon_azure_devops_types --lib azure_devops_license_
cargo test --offline --locked -p cloud_terrastodon_azure_devops_cli --lib azure_devops_license_entitlement_user_summary_cli::tests::
cargo test --offline --locked -p cloud_terrastodon_azure_devops --lib azure_devops_user_license_entitlement_update::tests::registered_response_constructor_generates_round_trippable_json
cargo check --offline -p cloud_terrastodon_azure_devops_cli --all-targets
cargo clippy --offline --locked -p cloud_terrastodon_azure_devops_types -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_cli --all-targets -- -D warnings
```

**REST setter refinement (U90; 2026-11-05):** `RestRequest::auth_context` remains
the generic `AuthContext` setter. `azure_auth_context(&AzureTenantAuthContext)`
attaches both the shared authentication context and explicit tenant without
credential acquisition. The Azure DevOps bearer branch delegates to it. Twenty-three
Azure caller sites now use their existing bound contexts, including Graph,
Resource Graph, reversed setter chains, and Cost Management. Fourteen explicit-token
or legacy tenant-only calls remain unchanged; no missing contexts were fabricated
and no SDK operation signatures changed. The user's staged rename is preserved.

**Setter verification:** Seven existing synthetic checks passed: six REST
authentication-guard/policy tests and the DevOps browser-tenant preservation test.
All-target Clippy for REST, Azure, and Azure DevOps passed with warnings denied.
Formatting, whitespace, 90 ledger/traceability pairs, and plan privacy checks
passed. No live requests or production-data validation were performed.

```powershell
cargo test --offline --locked -p cloud_terrastodon_rest --lib request_execution::tests::
cargo test --offline --locked -p cloud_terrastodon_azure_devops --lib work_item_tests::browser_auth_retains_tenant_and_revision_uses_a_test_operation -- --exact
cargo clippy --offline --locked -p cloud_terrastodon_rest -p cloud_terrastodon_azure -p cloud_terrastodon_azure_devops --all-targets -- -D warnings
```

**Local account cancellation refinement (U91; 2026-11-05):** Azure CLI program
resolution now runs inside the existing linked cancellable scope in
`auth::azure_cli_local_accounts`. The supplied child scope still owns subprocess
execution. Auth compilation passed without invoking Azure CLI or credentials.

**JSON representation refinement (U92; 2026-11-05):** Opaque Azure/DevOps records,
Resource Graph cells, resource properties, and query consumers use the existing
`ArbitraryJson` wrapper. The license response retains bounded object collections,
user identity coherence, and timestamp ordering in its generator; the DevOps
group-member response retains its object envelope. `try_from_peek` centralizes
reflected-field encoding using the existing pinned `facet-reflect` dependency.
Known modeled service fields remain typed. This does not complete partial schemas
or change their optionality.

Clippy warns on direct `facet_json::RawJson` use across workspace members,
including the root and example with independent lint tables. Intentional codecs
and transport boundaries have explained exceptions. Facet emits sibling codec
items, so those exceptions cover the owning module when necessary; Graph raw
response types now have separate modules instead of exempting orchestration.
Lower-level HCL and document codecs retain raw JSON to avoid inappropriate Azure
dependencies. See [the audit inventory](../raw-json-usage.md).

**JSON verification:** Twenty-six reviewed synthetic tests passed, covering
resource-row conversion/error retention, string-property caching, sentinel and
discriminator codecs, generator invariants, and work-item field/patch behavior.
Workspace all-target Clippy passed with `disallowed_types` denied. Existing root
warnings about the `test-support` feature name and a bare documentation URL remain
outside this change. Direct Clippy probes also confirmed that allowing a raw
import or a neighboring codec module does not exempt another type's fields.
Formatting, whitespace checks recognizing CRLF line endings, 92 paired guidance
and traceability rows, and plan privacy checks passed. No live service or
production-data validation was performed.

```powershell
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib -- arbitrary_json::tests resource_graph_data::tests resources::test role_management_policy_assignments::tests::registered_assignment_generator_can_produce_nonempty_coherent_json all_or::test entra_directory_object::tests pim_entra_role_settings::tests::settings_json_round_trips policy_definition_policy_rule::test --skip resource_graph_data::tests::it_slow
cargo test --offline --locked -p cloud_terrastodon_azure_devops --lib -- --exact azure_devops_user_license_entitlement_update::tests::registered_response_constructor_generates_round_trippable_json work_item_tests::create_patch_preserves_typed_fields_and_parent_relations work_item_tests::patches_distinguish_null_from_remove
cargo clippy --offline --locked --workspace --all-targets -- -D clippy::disallowed_types
cargo +nightly fmt --all -- --check
git -c core.whitespace=cr-at-eol diff --check
```

**Command JSON helper refinement (U93; 2026-11-05):** Removed
`command/src/json.rs` and its exports. Command decoding and external callers now
use the underlying Facet JSON operations directly, retaining their error mapping.
Workspace all-target compilation passed with the inlined callers.

**Bound-request invalidation refinement (U94–U96; 2026-11-05):**
`RequestExecution::with_invalidation(bool)` invalidates and executes lazily in
one linked child scope with the binding's tracing span. Invalidation errors stop
execution; a post-invalidation checkpoint prevents starting the request after
protected invalidation completes during cancellation. Both execute spellings
retain borrowed and non-`Sync` request/output support.

`cache::HasCacheKey` owns the fallible modern key contract independently of
invalidation. `InvalidatesOwnCacheKey` explicitly opts into a one-key
`CacheInvalidationKeys` implementation; compositions implement the key-list
trait directly. The executor blanket implements `CacheInvalidatableRequest`
only for the key-list contract. A request can instead implement custom async
invalidation while retaining `HasCacheKey`, by omitting both key-selection
policies. Automatically implementing the list for every keyed request would
reintroduce the original overlap at that layer.

Fifteen Gitea requests and `AzureDevOpsCliConfigRequest` opt into their own keys.
Three default-selection requests select their underlying request keys. The Gitea
organization/user browse menus use chained invalidation; mutation-owned
invalidation remains explicit. Existing scopes and freshness policies are
unchanged, including Gitea's execute-every-time policy. This does not promise
recursive refresh of composed requests that only invalidate their own scope.
The full key list is constructed before any invalidation. Its scopes are
invalidated sequentially, and a failure or cancellation prevents execution;
earlier completed invalidations remain effective. This is not an atomic batch.
Legacy RBAC's two-key composition demonstrates the need for the list contract
but remains on its existing runtime/cache boundary until its planned migration.

The legacy `command::HasCacheKey` still belongs to the old path-based cache.
Reusing it here would introduce an execution-to-command dependency cycle. It
remains scheduled for removal with the remaining legacy consumers; no bridge or
compatibility export was added.

**Invalidation verification:** Workspace all-target compilation and all 18 tests
in the execution-contract and request-invalidation targets passed. Tests cover
real typed-cache refresh/reuse,
two-child refresh with unrelated cache retention, lazy invalidation, failure
and cancellation preventing execution, and borrowed/non-`Sync` requests through
the blanket implementation, and custom async behavior on a keyed request.
All runtime validation uses temporary caches and
synthetic inputs without credentials or production data.

```powershell
cargo check --offline --locked --workspace --all-targets
cargo test --offline --locked -p cloud_terrastodon_execution --test request_invalidation_tests --test execution_contract_tests
cargo +nightly fmt --all -- --check
git -c core.whitespace=cr-at-eol diff --check
```

**Commit checkpoint (U97; 2026-11-05):** The completed Command migration and its
refinements are reviewed for the local commit titled
`Complete Command executor migration and refine cache invalidation`.
Review covered the staged execution/cache lifecycle, scope-only callers, Gitea,
Azure/DevOps service replacements, authentication, typed JSON, and dependency
changes. Two findings were corrected before the checkpoint:

- A local DevOps defaults command can change configuration before its artifact
  persistence fails. Its existing completion scope now attempts cache
  invalidation even when execution reports an error, retaining both errors when
  invalidation also fails.
- A fresh browser session requesting ACR could redeem a code with only
  `offline_access`. It now acquires a consented Graph token first, retains and
  persists that session under the Graph key, then exchanges the refresh token
  for ACR. Failure or cancellation in the second exchange cannot relabel the
  bootstrap token or reopen the account handoff. Independent review found no
  further issues in the corrected lifecycle.

**Checkpoint verification:** All 57 selected local Command tests and all 10 pure
browser scope/URL/callback tests passed. The 18 execution/invalidation tests
recorded above remain current. Workspace all-target Clippy passed with the raw
JSON rule denied; its existing root feature-name and documentation-URL warnings
remain outside this change. Focused auth/DevOps Clippy, formatting, staged
whitespace, and guidance/privacy checks cover the final fixes. Validation uses
synthetic data and local temporary resources; no live service or credential
acquisition was performed. These checks do not establish live API compatibility.

```powershell
cargo test --offline --locked -p cloud_terrastodon_command --lib --test executor_cache_tests --test plain_command_tests --test command_process_tests --test command_cancellation_tests --test cacheable_work_tests --test command_stdin_tests --test command_echo_tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib browser_access_token::tests::
cargo clippy --offline --locked --workspace --all-targets -- -D clippy::disallowed_types
cargo clippy --offline --locked -p cloud_terrastodon_auth -p cloud_terrastodon_azure_devops --all-targets -- -D warnings
cargo +nightly fmt --all -- --check
git -c core.whitespace=cr-at-eol diff --cached --check
```

At the U97 checkpoint task 4.2 was unstarted; that request authorized a local
checkpoint and next-phase description. U98 subsequently authorizes its
implementation below, without a new commit or push.

### [x] 4.2 Route REST execution and decoding through shared work

**Completion notes — 2026-11-05:** Implementation and offline verification are
complete; changes remain uncommitted for user review, with user staging preserved.

- `RestRequest` contains nonsecret `AuthenticationSelection`, request data, and
  artifact policy. Constructors do not take cancellation or runtime credentials;
  execution receives the caller's executor. Context setters only extract the
  explicit source and tenant. Implicit Auto resolution and raw bearer fields
  are removed.
- Owned `receive`/`receive_raw` paths execute each invocation. Explicit
  `receive_cached`/`receive_raw_cached` paths require a canonical operation owner
  and reuse decoded `Arc` values through `ExecutorKind::Rest`. Per-caller
  validators/mappers remain outside canonical sharing. Continuation metadata,
  typed HTTP errors, failure artifacts, original decoder errors, and method-based
  cancellation remain at the REST boundary. Read-only POST operations opt in to
  cancellation; mutations retain decoding and artifacts and their SDK procedures
  retain affected-read invalidation.
- Azure and Azure DevOps requests now bind execution explicitly; callers across
  CLI, entrypoint, UI, HCL workflows, maintenance tooling, and both workspace
  examples pass the executor. At this checkpoint shared transport pages still
  fed owned SDK inventories and could clone their records. U100/U101 correct
  that boundary in task 4.2b below. Aggregate memoization and dependency freshness
  remain phase 5 responsibilities.
  Existing uncached Azure DevOps Work Item Tracking reads remain uncached;
  executor propagation does not silently opt an operation into reuse.
  `PrincipalRequest`'s unknown-ID Graph batch shares its typed response envelope
  with opaque `ArbitraryJson` success bodies. Its response ID still chooses the
  concrete user/group/service-principal decoder on each SDK invocation. Phase 5
  owns memoizing that assembled domain result and avoiding this remaining inner
  decode; task 4.2 does not claim to eliminate all SDK deserialization.
- Graph helpers retain explicit tenant/source intent and execute with the
  supplied executor. PIM's captured Graph credential lives in
  `MicrosoftGraphTokenAuthenticationProvider`, with a fresh partition and exact
  tenant/source/audience admission. Container Registry uses a validated challenge
  and provider capability through restricted credential execution; exchange
  credentials and authorization headers never become ordinary cached artifacts.
- Caller scope selection also binds the selected authentication context to the
  executor. A tracked tenant's override of global Auto must change the provider
  together with request selection, including each subscription-list tenant and
  UI job. `AuthContext::bind_executor` and the DevOps counterpart preserve the
  cache, cancellation, and execution lineage; restricted credential execution
  cannot rebind a provider to escape its restrictions. SDK and REST procedures
  retain the supplied provider, including injected test providers. PIM's captured
  Graph provider uses the same lineage-preserving rebind boundary.
- Isolated HCL testing exposed a test-provider feature that workspace feature
  unification had supplied accidentally. HCL, Azure DevOps CLI, and entrypoint
  now declare `execution/test-support` through explicit dev-dependencies.
  Production dependencies do not rely on this fixture feature.
- Removed the Command crate's global artifact cache, legacy cache key,
  cacheable-request traits/macros, and their compatibility surfaces. Cached
  request invalidation uses the explicit owner and modern key contracts.
  `WorkCache::inventory` reads addresses and selected metadata under its artifact
  gate without reading payloads or credential inputs; cache list/clean UI uses
  this owner-bound inventory and invalidation.
- Workspace all-target check and Clippy passed after the caller-auth binding
  correction and isolated feature fixes. Both examples and test targets compile.
  Only the pre-existing root `test-support` feature-name and `src/main.rs` bare
  documentation URL warnings remain. Formatting, the post-binding CLI rerun,
  and small-stack execution check passed. Final independent source review found
  no blockers in subscription overrides, PIM, raw CLI, egui jobs, or preservation
  of injected SDK providers.
  Existing live SDK tests were compiled, not executed. No production service,
  real credential, user configuration, or existing cache data was accessed.
  Local fixtures establish lifecycle behavior, not remote API compatibility.

U98 requires leaving this work uncommitted for review. Preserve existing user
staging; do not stage the new changes.

**Offline verification evidence:** Counts describe the named runs, not a sum of
distinct tests; the 14-test execution-contract rerun overlaps the earlier core
suite and includes the new provider-rebinding regression.

| Boundary | Result |
| --- | --- |
| Cache/execution/REST core before final auth rebinding | 189 passed, including six HTTP loopback lifecycle tests |
| Execution contract after auth rebinding | 14 passed; selected partitions, dependency lineage, cancellation, and restricted credential execution remain enforced |
| Auth context rebinding | 1 passed; selected context and disabled policy preserved |
| Auth-types registry challenge | 1 passed |
| Captured Graph provider and Azure pure helper/invalidation checks | 1 + 5 passed |
| DevOps work-item and scope tests | 16 passed, 1 live test ignored |
| DevOps discovery plan and simulation | 7 + 4 passed |
| Isolated HCL reflow target | 18 passed after explicit test-support dev-dependency fix |
| DevOps CLI project admission and raw REST CLI | 4 + 13 passed again after final caller-auth binding |
| Group-management consumer | 33 unit + 3 CLI tests passed using local validation or disabled authentication |
| Workspace all-target check and Clippy after final binding/feature fixes | Passed; two unrelated pre-existing warnings retained |
| Final small-stack execution check | 1 passed: build-folder dispatch reports unresolved authentication on a small stack |

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
cargo test --offline --locked -p cloud_terrastodon_rest --test rest_lifecycle_tests
cargo test --offline --locked -p cloud_terrastodon_execution --test execution_contract_tests
cargo test --offline --locked -p cloud_terrastodon_auth --lib auth_context::tests::executor_binding_uses_the_selected_context_and_preserves_disabled_policy
cargo test --offline --locked -p cloud_terrastodon_hcl --test reflow_tests
cargo test --offline --locked -p cloud-terrastodon-group-management --lib --tests
cargo test --offline --locked -p cloud_terrastodon_azure_devops_cli --lib project::list::azure_devops_project_list_cli::tests::
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib cli::command::rest::rest_cli::test::
cargo test --offline --locked -p cloud_terrastodon_entrypoint --lib build_folder_dispatch_reports_auth_error_on_a_small_stack
cargo check --offline --locked --workspace --all-targets --message-format short
cargo clippy --offline --locked --workspace --all-targets --message-format short
```

**Completion criteria:** Local boundary tests prove typed hits avoid decode,
caller-specific validation still runs, cached reads do not acquire tokens solely
to hit the cache, and writes retain outcome artifacts and never coalesce. Decode
failures remain attributed to the owning operation and preserve the original error.

### [x] 4.2a Preserve operation-specific request errors

**Completion notes — 2026-11-05:** Implementation and offline verification are
complete; changes remain uncommitted for review, with user staging preserved.
`ExecutableRequest` now declares
`Error: Send + From<CancellationError>`. `ExecutionFuture<'a, T, E = eyre::Report>`
and the bound request preserve that error and its invocation lifetime. Existing
report-returning requests declare their error explicitly. Direct execution can
retain custom recovery data without a downcast, `Sync`, `std::error::Error`, or
`'static` requirement. Cancellation remains part of the operation's typed result.

The reflection adapter in
[function_invocation.rs](../../crates/registry_execution/src/function_invocation.rs)
requires `Into<eyre::Report> + 'static` only for registered requests and converts
on invocation. The request owns that conversion, including source preservation.
The bound invalidation combinator separately requires `From<eyre::Report>` for
cache failures; ordinary execution does not inherit that requirement.

**Verification evidence:** All 63 selected offline tests passed. The registry
regression uses a custom error
containing a non-`Sync` partial-progress value and an I/O source. Direct execution
retains the typed error; registered invocation converts it while retaining the
source and diagnostic context. Execution tests cover request-specific failures
and cancellation; picker tests retain typed recovery behavior. No production
data, user configuration, real credentials, or existing caches were accessed.
These are the results for this refinement, separate from task 4.2's earlier runs.

| Boundary | Result |
| --- | --- |
| Execution library and integration tests | 39 passed, including three typed-error tests |
| Registry invocation and reflection module | 21 passed, including the custom-error conversion regression |
| Picker result handling | 3 passed |
| Workspace all-target check and Clippy | Passed; only the pre-existing redundant `test-support` feature suffix and `src/main.rs` bare documentation URL warnings remain |

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --lib --tests --message-format short
cargo test --offline --locked -p cloud_terrastodon_registry_execution --lib registry_tests::
cargo test --offline --locked -p cloud_terrastodon_user_input --lib pick_result::test::
cargo check --offline --locked --workspace --all-targets --message-format short
cargo clippy --offline --locked --workspace --all-targets --message-format short
```

**Completion criteria:** Direct calls and cancellation retain the declared error;
the erased registry boundary preserves the custom conversion's source; existing
request and picker callers compile; focused offline tests and workspace checks
pass. Leave the changes uncommitted and preserve user staging under the current
authorization.

### [x] 4.2b Preserve shared cached SDK values through consumers

**U102 refinement — 2026-11-05:** Removed ownership-only generic parameters from
the twelve domain-specific list responses for locations, provider operations,
resource providers, compute SKUs, registry repositories/tags, eligible child
resources, Key Vault secrets, retail prices, role eligibility schedules, role
management policy assignments, and VM sizes. Each contains the concrete
`Arc<Domain>` item type and preserves its existing outer collection and
optionality. SDK decoders now name the concrete envelope; public request outputs
and pagination remain unchanged. Graph, Resource Graph, PIM, and Azure DevOps
multi-domain envelopes remain generic. This refinement adds no decoding fixtures
or runtime behavior. Workspace all-target compilation and focused Clippy passed
without warnings; formatting, whitespace, and stale-generic-use checks passed.
No service requests or live SDK tests ran. Verification commands:

```powershell
cargo check --offline --locked --workspace --all-targets --message-format short
cargo clippy --offline --locked -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure --all-targets --message-format short
```

**Completed — 2026-11-05:** Ownership changes are implemented across
cached Azure and Azure DevOps requests, Graph/category collection helpers,
Resource Graph conversion, and SDK/CLI/UI/example consumers. The checks below
validate this refinement independently of prior task 4.2/4.2a results.
This task shares known-type cached records and canonical Resource Graph row
conversion. Unknown-ID principal batches still select a concrete principal type
from each response ID and decode its opaque success body after the batch cache
returns; moving that discrimination and decoding into a cached principal result
remains task 5.1.

**Work:**

- Return `Arc<T>` for cached singleton values and `Arc<Vec<T>>` for an existing
  single-page collection. Decode page entries as `Arc<T>` when pagination or
  composition needs independent records, returning `Vec<Arc<T>>` without
  duplicating the underlying records. Whole aggregate memoization is separate.
- Update selectors, pickers, output payloads, and exports to borrow or retain
  the shared handles. Keep owned ID/name/path projections and new validated
  values where that transformation is the operation's actual responsibility.
  Preserve existing uncached requests and mutation results as owned outputs.
- Put fixed wire-to-domain conversion inside canonical REST decoding. Both
  source and target type identities must participate in the memory contract;
  conflicting interpretation fails rather than silently reusing another value.
  Converted failures retain response artifacts and caller attribution. Resource
  Graph row conversion must not repeat on cache hits. The heterogeneous
  unknown-ID principal batch branch remains outside this conversion boundary.
- Remove response/envelope `Clone` derives and generic bounds that existed
  only to copy cached values out of their `Arc`. Keep request cloning for
  reflected invocation and other independent ownership contracts where needed.
- Register the actual shared output shapes beside existing domain arbitrary
  generators so offline discovery exercises production request outputs. Do not
  introduce new generators for domains that previously had none merely for
  this ownership change. The group-member dictionary's shared output has
  executable metadata but no exact arbitrary generator: independently generated
  map keys would violate their required agreement with each member's descriptor.
  A future offline constructor must derive keys from the generated members;
  registering the generic map generator does not satisfy that invariant.
- Preserve invalidation, cancellation, authentication partitioning, external
  JSON shape, and existing user staging. Do not read production data or commit.

**Validation:** Inspect the existing offline targets before running them. Verify
non-`Clone` shared values, pointer identity across repeated cached reads,
conversion count/type conflicts and failure artifacts, and affected pure
consumer grouping/discovery behavior. Compile live SDK test targets without
executing them.

```powershell
cargo test --offline --locked -p cloud_terrastodon_rest --test rest_lifecycle_tests
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib entra_principal::tests::it_works
cargo test --offline --locked -p cloud_terrastodon_hcl --lib reflow_principal_id_comments
cargo test --offline --locked -p cloud_terrastodon_registry_execution --lib registry_tests::
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_simulation
cargo test --offline --locked -p cloud_terrastodon_azure_devops --lib azure_devops_build_folder_validated_snapshot::tests::
cargo test --offline --locked -p cloud_terrastodon_azure_devops_cli --lib folder_listing_preserves_empty_folders_and_counts_direct_definitions
cargo check --offline --locked --workspace --all-targets --message-format short
cargo clippy --offline --locked --workspace --all-targets --message-format short
```

**Verification:** All 46 selected tests passed: eight REST lifecycle tests, one
principal-sharing test, six HCL principal-comment tests, 21 registry tests, four
offline discovery tests, five folder snapshot tests, and one folder-listing test.
The new REST cases prove that a non-`Clone` converted value retains pointer
identity and converts once, conflicting wire types fail before another request,
and failed conversions preserve their source error and response artifacts without
publishing a reusable result. Workspace all-target compilation and Clippy passed,
including SDK, CLI, UI, tooling, and examples. Clippy retains only the existing
generated `test-support` feature-name and application bare-URL warnings. Focused
formatting and whitespace checks passed. Only synthetic local transport and
offline domain/registry tests ran; live SDK tests were compiled but not executed.
Changes remain uncommitted and user staging is preserved.

**Completion criteria:** Known-type cached request outputs retain the cached
allocation or its record handles without deep-cloning their payloads. Repeated
Resource Graph reads reuse canonical row conversion. Consumers compile with the
new ownership types, targeted offline tests pass, and any retained clone has a
concrete ownership/transformation role. Unknown-ID principal batch discrimination
and concrete-body decoding, aggregate memoization, and dependency freshness
remain phase 5 work. Record actual commands and results here before marking the
task complete.

## Phase 5 — SDK vertical slices and shared collections

**Foundation from task 4.2:** Transport execution, SDK caller propagation, modern
key/invalidation contracts, and explicit REST authentication are implemented.
Task 4.2b brings forward shared record ownership and canonical wire-to-domain
conversion across SDK consumers. The remaining tasks refine request data,
observers, composed-result caching, and dependency freshness. Do not reintroduce
an old transport adapter or mistake shared records for a memoized aggregate.

### [x] 5.1 Migrate Graph users and the composed principal inventory

**Implementation notes — 2026-11-06:** Graph category requests cache their
assembled `Arc<Vec<Arc<T>>>`; the principal inventory caches one
`Arc<EntraPrincipalCollection>` assembled from those handles. Source/tenant are
request data, and the Graph/batch helpers receive the executor for execution.
Matching get/search/pick requests and all consumers use this explicit selection.
`EntraPrincipalLookup` preserves a known entity route when a request is serialized
and replayed, without changing the API UUID representation of `EntraPrincipalId`.
Unknown-ID discrimination and concrete decoding run only inside the canonical
principal initializer. Failed principal results are not retained; the existing
successfully decoded opaque batch leaf remains reusable until invalidation or
expiry, so retrying interpretation does not promise another HTTP request.

Bound `.observe` replaces the request/helper's progress hook. Each invocation has
its own admission/cache/result events; only the initializer reports assembly
progress. Tests cover borrowed non-`Send` observers, separate waiter/hit events,
cancellation retry, one lifecycle for nested requests, and draining a large event
burst before completion. `.with_invalidation` retains lazy binding and composes
with observation. Gitea picker consumers explicitly convert that binding into a
future where their callback contract requires `Future` rather than `IntoFuture`.

The offline principal inventory scenario seeds typed child operations through
the established cache boundary, then executes the real composed request. It
checks outer and record pointer sharing, warm-child/page dependencies, cross-key
invalidation, retained old snapshots, and the absence of durable aggregate reuse.
The unknown-principal scenario checks canonical decode failure/retry and shared
known-route request replay. These tests do not establish remote API compatibility.

**Work:** Migrate Entra user/group/service-principal requests, Graph helper, and
PrincipalListRequest to explicit construction data and executor-taking execution.
Move progress callbacks to bound observation state. Implement G4's sharing across
pages and EntraPrincipal conversions/collection access. Preserve current Graph
query/pagination behavior and recovery invalidation. Use recorded child dependency
stamps so invalidating any input invalidates the principal result across prefixes.
Use G6's aggregate persistence decision; a fresh cache owner must not restore an
aggregate as reusable without authoritative dependency freshness.
For unknown-ID principal lookups, retain the heterogeneous Graph batch envelope
contract and move ID-selected concrete domain decoding into the canonical
principal result when introducing memoization. Its current `ArbitraryJson` body
is intentional transport data, not evidence that the assembled principal is
already decoded and shared. Preserve operations explicitly left uncached unless
their own cache contract is separately defined and verified.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure -p cloud_terrastodon_azure_types -p cloud_terrastodon_azure_cli
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests
```

**Validation:** The implemented executor_inventory_tests target uses typed
synthetic values and the established execution boundary. Shared principal
assembly and child-hit dependencies pass. The existing cache self-invalidation
scenario verifies the generation contract used by continuation recovery, without
claiming to exercise a live Graph continuation. Bound observer scenarios and
canonical principal lookup/replay pass. No assumed Graph response fixture is
treated as evidence of service compatibility. Combined phase evidence is below.

**Completion criteria:** Independent callers share a completed principal inventory
without request runtime fields, repeated decoding, or stale aggregate reuse. Any
assembly copying is explicit and bounded to the selected ownership design.

### [x] 5.2 Migrate Resource Graph and composed RBAC inventory

**Implementation notes — 2026-11-06:** Both role inventories and their combined
request contain explicit tenant/source data and cache shared in-process results.
The combined initializer validates the original cross-record relationship and
records both children, including warm hits. Its refresh policy invalidates its
own key and both input inventories. Resource Graph's helper contains no runtime
auth context; callers supply its selection and execution owner separately while
retaining the existing rate-limit and read-only POST policies. Consumer code
borrows records or clones handles. Arbitrary combined inventories now preserve
definition references instead of generating invalid relationships.

Four typed offline integration scenarios pass: concurrent composition/record
sharing, invalidation from either child, failure followed by corrected retry,
and authentication-selection separation. They execute the production aggregate
against seeded canonical children and reject attempted credential acquisition.

**Work:** Migrate role-definition/assignment requests and their combined request,
plus ResourceGraphHelper. Preserve page/query identity, rate-limit behavior,
read-only POST cancellation semantics, and cross-record validation in
RoleDefinitionsAndAssignments. Memoize the composed operation under its own
identity and record both child dependencies; distinguish aggregate/page identities.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure -p cloud_terrastodon_azure_types
cargo test --offline --locked -p cloud_terrastodon_azure --test rbac_executor_inventory_tests
```

**Completion criteria:** Typed local scenarios show concurrent RBAC consumers share
one valid combined result; invalidating either input invalidates the combined
cache even though keys differ. No live role or Resource Graph tests are executed.

### [x] 5.3 Migrate Azure DevOps projects and shared page records

**Implementation notes — 2026-11-06:** Project list/pick requests carry a
serializable organization URL and `AzureDevOpsAuthSelection`. The list retains
ordered page record handles in one shared aggregate; picker solicitation remains
outside that cache. Bearer tenant/source and PAT selections stay distinct, and
all consumers retain or borrow the returned handles. U105 restores borrowed
organization fields on list and picker requests after review rejected owned storage
to work around a Facet replay failure. U106 fixes generic `Cow<T>` navigation in
Facet Format and generic field-proxy callbacks in Facet's derive implementation.
The requests have no additional wire/proxy types. U107 releases the corrected
fork dependency closure; Cloud Terrastodon now selects it through ordinary
registry pins. Source-override checks and ordinary-registry workspace all-target
compilation passed. Both reflected-clone and all 14 inventory/discovery tests
also pass against the ordinary registry graph.

Three offline inventory scenarios exercise the real pagination procedure using
typed synthetic leaf artifacts, with authorization rejected before any network
request. They verify page order, shared records, page and parent invalidation,
unrelated-organization isolation, Server scheme/authority/instance/collection
identity, selected bearer tenants/PAT variants, request replay, admission on warm
hits, and rebuilding from leaves under a new cache owner. The seven discovery
plan and four simulation tests now use the real project list/pick registrations.
Two inspected request metadata/reflection tests and two UI rendering checks
validate those production shapes without invoking services.

**Foundation:** Task 4.2b carries project page entries and their SDK/CLI consumers
as `Vec<Arc<AzureDevOpsProject>>`; its own verification establishes that ownership
change. It does not add an outer cache for the assembled inventory.

**Work:** Finish project request-data/auth-selection separation and add shared
aggregate storage per G4, preserving the shared page records already introduced.
Preserve continuation metadata, ordering, wire output, explicit org, and bearer
tenant. Remove optional-tenant active-CLI fallback. Preserve PAT selection and
scope-sensitive cache identity without token acquisition on a valid cache hit.

```powershell
cargo check --offline --locked -p cloud_terrastodon_azure_devops -p cloud_terrastodon_azure_devops_cli
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_plan
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test discovery_simulation
```

**Validation:** Typed shared-record/selection tests, real production pagination
over synthetic saved leaves, request replay, reflection/rendering, and registry
discovery pass. The existing live project-list test was not run. Exact selected
targets/filters and combined evidence follow.

**Completion criteria:** Project records can be reused across pages/aggregate views
without borrowed cache lifetimes; output remains compatible; every bearer path
has an explicit tenant and every scoped CLI path keeps explicit org/project.

### Phase 5 completion evidence — 2026-11-06

Work remains uncommitted from `d2a5e4f2`; preserve the user's staged review selections.
Independent reviews of
Graph/RBAC, DevOps, and bound observation found and resolved request replay,
consumer ownership, and completion-event delivery issues. In particular, the
observer now drains the final queued burst without losing events to Tokio's
cooperative polling budget. The large-burst regression passes.

| Inspected offline targets | Passed |
| --- | ---: |
| Cache `work_cache_tests` and `cache_invalidation_tests` | 31 |
| Execution `execution_contract_tests`, `request_invalidation_tests`, `observation_tests` | 23 |
| Azure `executor_inventory_tests`, `rbac_executor_inventory_tests` | 5 |
| DevOps `project_inventory_cache`, `discovery_plan`, `discovery_simulation` | 14 |
| Selected Azure unit tests below | 8 |
| Selected DevOps request metadata/reflection tests below | 2 |
| Registry execution `registry_tests::` | 21 |
| Selected Ratatui rendering tests below | 2 |
| **Total distinct selected tests** | **106** |

These commands record the phase 5 checkpoint before U105/U106. Cow request replay
initially required local fork overrides; U107 now supplies published fixes.
Do not treat the earlier registry graph as validation of that refinement.
Commands were run from the repository root, using the then-existing locked dependencies:

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution --test observation_tests --test request_invalidation_tests --test execution_contract_tests -p cloud_terrastodon_cache --test work_cache_tests --test cache_invalidation_tests --message-format short
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests --test rbac_executor_inventory_tests -p cloud_terrastodon_azure_devops --test project_inventory_cache --test discovery_plan --test discovery_simulation --message-format short

$phase5UnitFilters = @(
    'principal_get_request::tests::unknown_principal_decode_failures_retry_and_hits_share_the_typed_result'
    'microsoft_graph_helper::tests::consistency_level_eventual_adds_the_expected_header'
    'entra_user_get_request::tests::url_keeps_special_upns_in_the_user_key'
    'entra_user_pick_request::tests::refresh_covers_all_selected_tenant_searches_without_crossing_tenants'
    'entra_user_search_request::tests::url_escapes_search_terms_as_odata_query_values'
    'entra_user_search_request::tests::paginated_search_cache_expires'
    'resource_graph_helper::tests::reads_resource_graph_quota_headers'
    'resource_graph_helper::tests::writes_only_failed_resource_graph_entry_extra_file'
    'azure_devops_projects::tests::reflected_project_list_clone_preserves_request_data'
    'azure_devops_project_pick_request::test::registry_discovers_default_organization_dependency'
    'registry_tests::'
    'request_card_separates_fields_and_actions_and_filters_whole_sections'
    'owned_organization_url_slot_renders_its_value'
)
cargo test --offline --locked -p cloud_terrastodon_azure -p cloud_terrastodon_azure_devops -p cloud_terrastodon_ui_ratatui -p cloud_terrastodon_registry_execution --lib --message-format short -- @phase5UnitFilters
cargo test --offline --locked -p cloud_terrastodon_execution --test observation_tests --message-format short
cargo check --offline --locked --workspace --all-targets --message-format short
cargo clippy --offline --locked --workspace --all-targets --message-format short
```

The final observation rerun includes the fourth, large-burst regression. Workspace
compilation includes CLI/UI/tool/example consumers. Final Clippy succeeds with
only the pre-existing root `test-support` feature-name and `src/main.rs` bare-URL
documentation warnings. New iterator warnings were fixed by filtering before
cloning handles or borrowing records directly. Formatting uses the repository's
nightly rustfmt settings; whitespace checks pass. No lockfile change, production
data, credentials, existing application cache, or live request was used.
Generated incremental compiler files were cleared during validation to recover
build space; source and completed build outputs were retained.

**U105/U106 initial fork refinement evidence — 2026-11-06:** The project replay scenario
reproduced the opaque-Cow failure against published dependencies. After fixing
the forks, 214 Facet and 222 JSON integration tests pass, including six new
regressions; both changed implementation crates pass Clippy with warnings denied.
Cloud Terrastodon's 14 DevOps inventory/discovery tests plus its reflected-clone
and request-card tests pass with the local overrides. Borrowed, owned and
deserialized requests reuse the same aggregate; reflection retains each Cow
variant. The Cloud Terrastodon workspace all-target check passes with the same
local overrides. teamy-rust-cli's all-target check also passes, with two existing
macro-semicolon warnings. No application wire/proxy workaround remains.
At that checkpoint all changes remained uncommitted, published manifests/lockfiles
were unchanged, and user staging was preserved. Production data and credential
stores were not accessed.

**U107 release refinement evidence — 2026-11-06:** Fourteen Facet/Format/Figue
packages are published. Current release source checkpoints are core
`2d6d06e1d9a0ab47cd68547f5a0a4dbb60d36afa`, Format
`55dd746345382b81a8e9eb1652cddcd4b7cb2fda`, and Figue
`85b3886592767bfe66cdf4588ee12d015efaf569`. The facade, macro implementation/macros,
Error, and Default use rc.8; Format/JSON/Value/CSV use rc.10; DOM uses rc.7;
XML/XML-node use rc.8; Figue uses 6.0.0-rc.6 and Attrs 6.0.0-rc.4. Unchanged
Core/Reflect/Pretty and related packages remain at 0.50.0-rc.7. Cancellation
0.7.2 and 0.8.1 are published separately to preserve each consumer's existing API
line. Cloud Terrastodon uses 0.7.2, avoiding an unrelated cancellation API migration.

Source suites passed 214 core, 222 JSON, 761 Figue, 312 XML/CSV, and 82/88
Cancellation tests. Prepublication consumer checks passed the Cloud Terrastodon
workspace/all targets, two list/picker reflected-clone tests, and 14 DevOps
inventory/discovery tests. The template passed its all-target check and 13
selected output/CLI tests. Cloud Terrastodon's normal manifest/lockfile now select
the registry graph without patches; only ten targeted package versions changed,
and metadata confirms one registry identity per participating Facet/Figue package.
The ordinary-registry `cargo check --offline --locked --workspace --all-targets`
passed, along with both list/picker reflected-clone tests and all 14
inventory/discovery tests. This completes 16 selected registry tests.

The template's required Tracing 0.2.1 is published; its current release source is
committed/pushed as `d099c486804ae7fe18d2acf5d2e74a3e4c48ff10`
on `release/0.2.1-facet`. Its 37 tests, two comparison checks, Clippy, documentation,
and examples passed. Template registry pin/lock updates are complete with only
eleven targeted package version/checksum changes and no unrelated updates.
Its ordinary-registry `cargo check --offline --locked --all-targets` passed,
retaining only two existing macro-semicolon warnings. Metadata confirms one
registry identity for each of its seventeen Facet/Figue packages, with no
upstream Facet/Figue packages or source overrides. All 13 selected template
registry tests pass (11 library tests and two `cli_fuzzing` tests). The template
is committed/pushed to `origin/main` as
`58d94181a3fc11d99c8691254dbffe8258dcc311`; its working tree is clean and local/
remote refs match.
See [Updating Facet](../UPDATING_FACET.md#cow-request-refinement-2026-11-06) for the
full version map, source baselines, historical override evidence, and current
release state. Cloud Terrastodon changes remain uncommitted and user staging is
preserved. No production data or credential stores were accessed for validation.

**U108 repository follow-up — 2026-11-06:** Release author-metadata corrections
are pushed for Format and Tracing at the current hashes above. The corrected
commits have identical source trees; published archives and consumer checksums
were not replaced. The separately approved Format history correction covers
seven affected historical commits and reconstructs 15 commit objects while
preserving trees, messages, dates, topology, and unrelated identities. Format's
feature ref is `fd2af32fb2eb087d73237302c882742a9d313fc5`; its corrected annotated
CSV rc.7 tag object is `6d814e24863f65463c6c94811e5a110e7aab14e1`, targeting
`da65a4fe8ddcc391063c5b726c612fad43351247`. The tag message/date are preserved,
remote refs are verified, and the working tree is clean.

Cancellation's maintenance branch is merged into `main` at
`e4d962738dce985632bdff37a196d2c3e14500ac` and pushed, with the existing 0.8.1
source tree retained. Tracing's new `main` starts from the existing implementation
history and merges the release branch while retaining telemetry support. It is
committed/pushed at `47bc18c5738d62cdb471afee66e682e6c1ea851a`, with a clean tree
and matching local/remote refs. The all-target/all-feature check, 43 tests
(37 base including the doctest plus six telemetry tests), and default-feature
all-target Clippy with warnings denied passed. The separately approved GitHub
default-branch change is verified: both the default branch and local
`origin/HEAD` point to `main`.

### [x] Phase 5 review refinement — shared fixtures and registry collections

**Completion notes — 2026-11-06:** All three refinements below are implemented
and verified. This review checkpoint preserved staging and remained uncommitted
until U113's `1e63a553` checkpoint. The scoped concurrency rollout now precedes phase 6.

**Work:** U109 adds `execution::TestAuthenticationProvider` behind
`test-support`, with explicit synthetic session identity, source/tenant-separated
partitions, revocation/generation controls, and atomic invocation counts. Its
offline constructor rejects authorization; its other constructor accepts supplied
synthetic authorization for local transport tests. Nine duplicated providers now
reuse it. `UnreachableAuthenticationProvider` remains an immediate panic guard;
three credential-lineage probes remain specific to the restrictions they test.

U110 replaces qualified derives across 1,024 Rust files, including the
resource-type generator and generated enum. Conditional imports retain their
feature/test guards.

U111 makes `register_thing!(T)` and `register_arbitrary!(T)` supply `Vec<T>` and,
when `T: Sync`, `Arc<T>`, `Vec<Arc<T>>`, `Arc<Vec<T>>`, and
`Arc<Vec<Arc<T>>>`. The family is finite and adds no `Clone` requirement.
Generated companions deduplicate by Facet shape; explicit arbitrary
registrations retain their original static handles, including separately named
constructors of the same shape. Complete catalog APIs include companions;
distributed slices retain explicit registrations. Redundant wrapper registrations
are removed.

**Validation:** 151 selected offline tests passed: execution 45, registry 10,
registry execution 57, REST lifecycle 9, Azure inventories 5, principal lookup 1,
DevOps inventories/discovery 14, registered Azure type generators 3, and the
resource-type generator 7. Workspace all-target compilation, targeted Clippy,
and application checks with/without its optional `arbitrary` feature passed
without warnings. The qualified-derive and whitespace audits passed.

The catalog test now distinguishes exact output from list-element producers
instead of assuming only one producer exists. The REST wrapper/leaf fixture
uses the same authentication selection to continue proving executor-kind
isolation within one scope and partition. Tests use synthetic data and local
transports only; no production data or credential stores were accessed.

Commands used `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and
`CARGO_PROFILE_TEST_DEBUG=0` to bound generated build storage:

```powershell
cargo check --offline --locked --workspace --all-targets
cargo test --offline -p cloud_terrastodon_execution --tests
cargo test --offline --locked -p cloud_terrastodon_registry --tests
cargo test --offline --locked -p cloud_terrastodon_registry_execution --lib
cargo test --offline --locked -p cloud_terrastodon_rest --test rest_lifecycle_tests
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests --test rbac_executor_inventory_tests
cargo test --offline --locked -p cloud_terrastodon_azure --lib principal_get_request::tests::unknown_principal_decode_failures_retry_and_hits_share_the_typed_result -- --exact --test-threads=1
cargo test --offline --locked -p cloud_terrastodon_azure_devops --test project_inventory_cache --test discovery_plan --test discovery_simulation
cargo test --offline --locked -p cloud_terrastodon_azure_types --lib registered_
cargo test --offline --locked -p cloud_terrastodon_resource_types_cli resource_type_catalogue::tests
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features
cargo check --offline --locked -p cloud_terrastodon_app --no-default-features --features arbitrary
cargo clippy --offline --locked -p cloud_terrastodon_registry -p cloud_terrastodon_registry_execution -p cloud_terrastodon_execution --all-targets
```

The next migration task is **6.1**, after scoped concurrency rollout/review and the
picker handler ownership follow-up below.
The coordinated package release and consumer verification are complete.
Split the existing audit policies
into independent activities which request principals/RBAC through the shared
executor, preserve their findings, and verify shared fills with synthetic data.
Phase 6 was not started as part of this review checkpoint.

### [x] Scoped join preview before phase 6

**Completion notes — 2026-11-09:** The completed phase 5/refinement checkpoint is
committed as `1e63a553`, `Complete shared inventory caching and execution observation`.
At that checkpoint, the helper and three-call-site preview were implemented and
verified, and left uncommitted for interface review. Broader caller migration and
lint enforcement were deferred until U114 authorized the rollout below.

**Work:** Implement scope-only `execution::join2`/`join3` using an isolated
cooperative group and linked cancellable branch scopes. Small executor adapters
preserve authentication restrictions, cache lineage, and observation. Return the
first observed error after siblings stop or finish required work. Keep borrowed,
heterogeneous results and request-specific errors without adding Clone/Send bounds.
Migrate only principal assembly, RBAC assembly, and role-assignment choices.

**Validation:** Synthetic completion tests cover sibling errors, group isolation,
automatic ancestor cancellation, pre-admission cancellation, and borrowed results.
Existing credential-lineage and typed principal/RBAC inventory tests verify the
executor adapters. Run focused Clippy, formatting, and compilation for affected
crates. No live service calls or production fixtures are permitted.

**Evidence:** All 50 execution tests plus one documentation example pass; the five
typed principal/RBAC inventory tests pass with the new joins. The execution suite
includes five new scoped-join tests and the existing credential probe extended
through both executor adapters. Validation uses the pinned registry dependencies,
synthetic data, and the existing bounded build-storage environment variables.
The affected crates pass all-target Clippy with warnings denied. Formatting and
whitespace checks pass. This does not claim full-workspace or live-service testing
for the preview.

```powershell
cargo test --offline --locked -p cloud_terrastodon_execution
cargo test --offline --locked -p cloud_terrastodon_azure --test executor_inventory_tests --test rbac_executor_inventory_tests
cargo clippy --offline --locked -p cloud_terrastodon_execution -p cloud_terrastodon_azure --all-targets -- -D warnings
```

**Historical review boundary:** U113 initially limited migration to three callers.
U114 now authorizes the broader rollout below. The original preview evidence above
remains historical; [scoped join review](scoped-join-review.md) records the current
implementation and remaining ownership cases.

**Completion criteria:** The helper and three callers pass the focused checks;
the remaining callers and interface decision are explicit, and changes remain
uncommitted. The subsequent rollout supersedes the initial interface-review gate.

### [x] Scoped concurrency rollout before phase 6

**Work — 2026-11-09, U114:** Extend scope-only joins through five branches and add
`map_concurrent(scope, items, NonZeroUsize, factory)` with an executor adapter.
Run borrowed futures inline, limit admitted work, preserve input order, and keep
the first observed error while cancelling/draining the group's active children.
Preserve credential restrictions, cache lineage, and inherited deferrals. Do not
protect an entire batch merely to keep its required children alive.

Migrate all production `try_join!` calls and finite service/command/file batches,
including callers hidden behind `ParallelFallibleWorkQueue`. Remove that obsolete
queue after its callers migrate. Keep complete-all errors as data until all jobs
drain; document that parent cancellation returns a batch error rather than partial
results. Add a Clippy prohibition for Tokio/Futures fail-fast join macros.

**Validation:** Synthetic join/map tests exercise borrowed outputs, bounded work,
ordering, lazy admission, original errors, isolated cancellation, automatic
ancestors, required completion, and nested maps. Run the execution suite, selected
inventory/Command/REST lifecycle tests, workspace all-target compilation, and
focused Clippy. Live service and Terraform integration tests are not authorized.

**Completion evidence — 2026-11-09:** Helpers and caller migrations are implemented.
All 89 focused tests plus one documentation example pass, including nine map tests
and five join tests. The final changed-target rerun verifies nested writes,
inherited deferrals, and credential restrictions across every adapter. All
workspace targets compile; all nine affected crates pass all-target Clippy with
warnings denied. Formatting and whitespace checks pass. No live service tests
were run. See [scoped join review](scoped-join-review.md) for reproducible commands,
retained coordinators, policy differences, and the separate interactive follow-up.

**Completion criteria:** Applicable joins and finite queues use the shared helpers;
required completion and complete-all policies survive the migration; focused
offline checks pass; exceptional coordinators and follow-ups are explicit. Leave
changes uncommitted for review.

### [x] Iterator mapping refinement

**Work — 2026-11-09, U115:** Make `ConcurrentIteratorExt` the public mapping
interface. `ConcurrentIteratorExt::map_concurrent` accepts an executor;
`map_concurrent_scoped` accepts a cancellation scope. Keep bounded concurrency, input-order results,
child-first factories, lazy admission, first-error draining, complete-all error
data, and borrowed futures. Remove the executor method and free-function export;
retain one private coordinator and internal executor scope rebinding. Migrate SDK,
CLI, HCL, entrypoint, and test callers without compatibility aliases.

**Validation:** Rerun the nine map behavior tests, fourteen execution contract
tests, documentation examples, workspace all-target compilation, and affected
Clippy. Existing tests cover borrowed/non-Clone/non-Send outputs, mutable non-Sync
mappers, nested required writes, inherited deferrals, cancellation, ordering, and
executor credential restrictions. No service or Terraform tests need to run for
this interface-only refinement.

**Completion evidence — 2026-11-09:** All 23 focused tests and two documentation
examples pass. All workspace targets compile; the seven affected crates pass
all-target Clippy with warnings denied. Formatting and whitespace checks pass.
The original map coordinator is unchanged except for crate-private visibility.
All callers use the iterator extension, the executor mapping method is removed,
and its internal scope rebinding preserves credential restrictions. No live
service tests ran. Existing staged changes are preserved and the refactor remains
uncommitted. See [scoped join review](scoped-join-review.md) for commands.

### [x] UUID value semantics refinement

**Work — 2026-11-09, U116:** Add missing `Copy` derives to the five UUID-backed
Azure DevOps account/project/repository/team/query IDs and the three UUID-backed
role names. Existing direct UUID wrappers already implement `Copy`. Remove
redundant clones in callers, retaining shared handles and string-backed values.

**Validation:** Compile all workspace targets with Clippy to cover SDK, CLI,
tests, and example callers. Check formatting and whitespace. No live requests,
credential discovery, or new derive-only tests are needed.

**Completion evidence — 2026-11-09:** Audited all 25 direct UUID wrappers;
eight needed `Copy`. Removed 13 redundant ID clones and nine now-unnecessary
borrows across SDK, CLI, UI, and tests. Nightly formatting and whitespace checks
pass. `cargo clippy --offline --locked --workspace --all-targets -- -D clippy::clone_on_copy`
passes, compiling test and example targets; existing unrelated feature-name and
documentation warnings remain. No tests or live requests ran. Existing staging
is preserved and changes remain uncommitted.

### [x] Async callback capture refinement

**Work — 2026-11-09:** Apply the reviewed async-closure capture style throughout
SDK requests, CLI/picker callbacks, HCL/dump workflows, and synthetic tests.
Borrow original fields directly instead of creating capture-only aliases or
clones before `async move`. Keep per-call ownership needed by retained futures,
mutable factory admission snapshots, synchronous caller capture, child scopes,
complete-all results, and deferred completion ordering. Record the convention in
`AGENTS.md`; do not change the callback APIs or interactive shutdown contract.

**Completion evidence — 2026-11-09:** Workspace all-target compilation and
all-target Clippy with warnings denied across the 12 affected crates pass.
All 180 selected local tests pass: 46 execution, 105 cache, nine loopback REST,
15 command fixture, and five app cancellation process tests without auth features.
Formatting and whitespace checks pass. Existing staging is preserved; changes
remain uncommitted. No live-service or Terraform tests ran. Reproduction commands
are recorded in [scoped join review](scoped-join-review.md).

### [x] Interactive handler and background-task ownership follow-up

**Finding before implementation:** `picker_tui.rs` replaced its `FuturesUnordered`
handler collection on reload and dropped it on exit; early errors could also
abandon handlers. Entra picker
handlers call real SDK requests, which can owe response decoding/artifact
completion. These are event-driven handlers, not a finite input batch.

`AppWorkTracker::finish` also aborted work marked discardable, including the role
assignment picker's principal/RBAC loads. A cancellation request before that
abort does not preserve required descendants. Its `JoinError` path can return
before retaining all remaining jobs. Include this shutdown owner in the follow-up.

**Work:** Introduce a linked handler-generation cancellation owner and drain started
handlers on reload, accept, cancellation, and terminal/handler errors. Preserve
generation filtering and responsiveness while preventing old handlers from
publishing into the new selection. Keep the coordinator alive through required
completion without deferring every handler. Scope this separately from the finite
join/map rollout; do not mark it safe merely because its requests are reads.
Replace discardable-task aborts with cancellation and draining for SDK-backed
jobs, and retain all started jobs when another job fails or panics.

**Validation:** Use synthetic pending handlers with required completion and a fake
candidate channel to verify reload/exit/error draining and obsolete-generation
isolation. Do not resolve credentials or read service inventories.

**Completed work — 2026-11-09:** Implemented picker session/handler ownership,
explicit caller scopes (including nested terminal pickers), and application
background shutdown. Picker
callbacks receive their own scope; SDK callbacks rebind the executor while
preserving its execution restrictions. Terminal restoration retains the owner
without shielding optional handler work. The tab warning becomes an in-picker
notification so a blocking stdin reader cannot survive cancellation. Background
shutdown closes admission and drains every started job even after errors/panics.
GUI shutdown wakes idle windows and retains jobs before invoking application
callbacks. Poisoned terminal owners retire nested frames without attempting
unsafe resume handoffs. The final intent audit and independent reviews found no
remaining required work in this slice.

**Completion evidence — 2026-11-09:** All 86 focused tests pass: 64 user-input
tests, 14 execution contracts, four command background-work tests, one GUI tracker
test, two Ratatui shutdown tests, and one temporary-filesystem picker test. The
manual terminal test remains ignored. Workspace all-target compilation,
all-feature/all-target picker compilation, and all-target Clippy with warnings
denied across 11 affected crates pass. Formatting and whitespace checks pass.
No native UI was manually exercised, and no production services, credentials,
configuration, or caches were accessed. U120 committed this completed checkpoint
as `58f454e4`. Reproduction commands are in the
[scoped join review](scoped-join-review.md#interactive-lifecycle-verification).

### [x] Follow-up: consolidate UI background work

**Scope:** U120 follows the completed interactive ownership checkpoint. Reuse the
generic command `Loadable`, `LoadableWorkBuilder`, work messages, and tracker from
egui; remove its duplicate implementations without compatibility exports. Keep
UI presentation state separate from memory/disk cache reuse. Remove the unused
success-delivery toggle: a closed UI may discard a successful snapshot, while
operation failures remain observable. Deferrals, not task flags, protect writes.

**Work:** Make the tracker independent of application state and pass the tracker,
message channel, mutable UI state, and scope directly when enqueueing. Remove
the sender/acceptor indirection. Register autosaves as scoped jobs with protected
write bodies; a queued job can cancel before starting. Transfer final GUI settings
to the owning shutdown procedure, drain older jobs, then flush both settings
under required completion so cancellation cannot lose the final snapshot.

**Completion evidence — 2026-11-09:** Egui now uses the generic loading adapter,
messages, and tracker; seven duplicate modules and three sender/acceptor types
are removed. The tracker no longer carries an unused application-state type.
Autosaves use protected writes and shutdown saves the final snapshot after older
jobs drain. Six focused offline tests pass: five shared tracker tests, including
cancellation before first polling and closed-receiver handling, and one synthetic
autosave test covering protected completion and rejected admission. Workspace
all-target compilation, affected-crate all-target Clippy with warnings denied,
formatting, and whitespace checks pass. Dependency changes only remove the
deleted adapters' direct dependencies and add the existing cancellation/test
dependencies to egui; no versions changed. Independent reviews found no blocking
issues. U122 records this cleanup in the local checkpoint titled
`Consolidate UI background work and scoped autosaves`. Native UI and real configuration
were not exercised. A native panic before `on_exit` cannot hand off unsaved final
settings, but already-started protected saves still drain.

Reproduction commands are in the
[scoped join review](scoped-join-review.md#ui-adapter-consolidation).

## Phase 6 — accounts, runtime verification, requests, and remaining consumers

**Revised next comprehensive boundary:** U123–U128 insert tasks 6.0a–6.0e before
audit extraction. The current work is design only; task 6.0a closes its contracts.
Remove accidental live Cargo entrypoints, introduce named account ownership, and
complete account-aware request data/direct callers before extracting audits.
After 6.0a, test classification/runner work (6.0b) and provider/store work (6.0c)
may proceed independently against the same agreed selection contract. Complete
the integrated family cutover and evidence in 6.0d–6.0e before 6.1–6.3.
Task 6.2 closes isolated startup/runtime evidence around the already-installed executor; task 6.3
reconciles and finishes remaining consumer batches. Registry invocation, picker
child scopes, and the shared UI work lifecycle are established foundations and
should not be reimplemented. Audit extraction stays in the Azure CLI scope
described by 6.1; Azure DevOps audit extraction is not silently added to that task.

Phase 6 is complete when the account/runtime-verification/request-first
acceptance criteria in the companion hold, existing audit policies return typed
findings through one output boundary and share decoded inventories, startup with authentication
disabled has isolated runtime evidence, and every inventoried consumer has a
verified execution contract or an explicitly documented Phase 7 removal item.
Keep scope-only helpers independent of the executor, preserve existing audit
behavior, and use synthetic data only.

### [ ] 6.0a Close account and runtime-verification contracts

**Work:** Close A1–A4/V1 in the [design companion](accounts-runtime-tests-and-requests.md)
with the confirmed scoped aliases/configured defaults. Decide serialized binding
identity, provider selection, session lifecycle, CLI surface, source support,
legacy migration, verification registration/input/strictness semantics, and
effect admission. Include optional clean invalidation/replay diagnosis and
per-operation account visibility for approvals; keep credential generations
separate from durable identity. Inventory concrete store keys, account-sensitive aggregates,
live/offline Cargo tests, and forwarding versus meaningful helper functions.
Specify domain-qualified account type names, including `CloudTerrastodonAccountId`,
and inventory existing service-name inconsistencies under U129.
Preserve direct registry handles, the existing crate DAG, and scope-only helpers.

**Validation:** Review source and official provider contracts without resolving
credentials or inspecting real configuration. Check the inventory against all
SDKs, auth, CLI/UI, tools/examples, and test scripts. Audit the gates against two
accounts in one tenant, replay after rebinding, logout during a deferred mutation,
and cross-account aggregate dependencies. Review examples for missing scope or
secret-bearing fields; run `git diff --check` on the design changes.

**Completion criteria:** Every gate has a recorded decision and validation
contract; inventories classify actual behavior, not file/function names. The next
agent can implement without guessing public defaults, account identity, or test
admission. The present design review is preparation, not completion of this task.

### [ ] 6.0b Separate live verification from Cargo tests

**Work:** Convert meaningful live checks to separately registered executable test
requests composing production requests. Add metadata-only listing, explicit typed case
construction, allowed-effect subset checks, typed results, and scoped execution
through the existing executor. Add optional `--clean`, default false: warn without
it and allow eligible cached responses; when supplied, delegate invalidation to
every underlying read request, including pages and composed/non-descendant reads.
Reuse `.with_invalidation(clean)` and existing contracts without a mandatory new
cache root or unrelated purge. Design an explicit saved-response decoder path for
parser diagnosis, since typed memory hits skip parsing and failed-decode artifacts
are not published as reusable results. Preserve normal success-only cache rules,
artifact integrity/retention, and visible cache/replay/strictness provenance.
Preserve offline unit/policy/loopback/tempfile tests;
remove obsolete experiments with rationale. Retire credential-backed Cargo test
helpers/default lookups once callers are classified. Include tests that do not
use the SDK test_executor helper. Update test scripts and development guidance.

**Validation:** Inspect each migrated test before executing it. Use synthetic
registrations/providers to exercise list, missing input, disallowed Write/Solicit,
zero-case runs, failure reporting, warning/provenance without `--clean`, complete
invalidation and failed-invalidation handling with it, empty-item coverage, and cooperative
draining. Cancellation during a case must stop queued cases from starting and
produce a cancelled/partial non-success outcome after required work drains; it
must not become an ordinary failed case that permits continued scheduling.
List may inspect metadata only, with configuration/credentials disabled.
Use synthetic retained responses to prove repeated decoding uses the new decoder
without network, bypasses typed memory hits, and does not publish a failed attempt
as a normal cached success. Test repeated/dynamic child-read coverage for clean runs.
Compile live cases without invoking them. Record exact focused test commands once
targets exist and have been inspected; do not run broad Cargo suites yet.

**Completion criteria:** No Cargo test invokes a live verification entrypoint or
ambient service credentials/configuration. Meaningful former live checks have
explicit runtime cases; genuine offline coverage remains. All cases declare
actual effects and cannot bootstrap live scope via Arbitrary/discovery. Clean and
cached runs follow U130, parser diagnosis has explicit saved-response provenance,
and neither cached nor replayed success claims a fresh service exchange.

### [ ] 6.0c Implement named accounts and credential-session ownership

**Work:** Add stable account/binding vocabulary to auth_types and resource-owning
profile/session behavior to auth, using `CloudTerrastodon`-prefixed account types
to distinguish CT's model from service identities and keeping the dependency
graph acyclic. Keep approved account identity stable across valid credential
renewal and private session generations. Provide explicit account selection
within the executor's admitted sessions for mixed-account workflows; missing
accounts must not fall back to another account/default. Implement
scoped aliases, configured defaults, explicit source-aware CLI management and
legacy storage migration under A1–A4. Separate account-aware CLI/browser/PAT
storage; account for workload identity configurations and Gitea profiles. Preserve
existing conservative cache reuse and isolated CLI sessions. Define new-admission
closure separately from already-admitted required completion. Include invalidation
of retired account partitions and dependent aggregates without narrowing affected
resource invalidation across accounts.

**Validation:** Use synthetic provider sessions and temporary files only. Prove
same-tenant same-source accounts and same aliases in separate namespaces coexist;
wrong bindings fail before warm hits; default removal/rebind cannot switch an old
request; logout preserves response/artifact/affected-read completion already owed.
Prove credential renewal retains the reviewed account while principal replacement
requires a distinct approved identity/reapproval under A3's continuity rules.
Cover cross-account dependencies, alias reuse, and legacy migration failure.
Record exact inspected test targets during implementation. No login or real
credential-store probing is part of development acceptance.

**Completion criteria:** Supported source/account lifecycles and CLI behavior have
synthetic evidence, explicit default semantics, and documented separate-process
limits. No single tenant/global credential slot silently replaces another account.
Profiles carry references, not serialized secrets; requests still name authority
and service scope correctly.

### [ ] 6.0d Complete account-aware request data and direct callers

**Work:** Migrate each inventoried family and all consumers together to resolved
serializable account/source/tenant selections, including the still-context-bearing
resource-group and storage-account list requests. A reviewable operation names
its intended execution account even when different from the workflow requester;
credentials, provider handles, and login/token generations stay runtime-owned.
Remove trivial fetch/binding
functions, imports, and exports; use request literals plus `.execute(executor)`.
Preserve meaningful workflows and cancellation-only operations. Include SDK
composition, transports/aggregates, CLI, both UIs, registry, examples/tools, and
runtime test requests. Preserve Cow/Arc ownership, custom errors, lazy invalidation,
observer/caller diagnostics, and callback child executors. Do not replace helpers
with redundant constructors or compatibility exports.
Audit domain naming in the migrated request/value families. Rename
`ResourceGroupListRequest` to `AzureResourceGroupListRequest` and `StorageAccount`
to `AzureStorageAccount` together with their related types as appropriate, owning
files, consumers, registrations, tests, and examples. Record other inconsistent
families for the same treatment; classify generic infrastructure separately.
Do not change service wire properties, CLI spellings, or semantic cache scopes
merely because a Rust type is renamed.

**Validation:** Compile every affected consumer and run inspected offline behavior
tests after each family. Verify secret-free request construction/serialization,
account-aware composite identity, registration/effects, borrowing, and cancellation
lineage. Check reflected names and actual fingerprint/serialization dependencies
for the domain renames. Add a synthetic approval round-trip with admin-account read
and independently selected later operation; changing defaults must not substitute
accounts, and a missing approved account must fail admission. This does not build
a new workflow engine. Finish with `cargo check --offline --locked --workspace --all-targets`.
Classify remaining fetch/executor.execute occurrences rather than deleting by name.

**Completion criteria:** All inventoried trivial wrappers are removed; request
data contains no auth runtime or cancellation; supported callers use request-first
execution without redundant ownership conversions. Legitimate helper/infrastructure
exceptions have explicit responsibilities and no compatibility role. New account
types and inventoried renamed service families use their owning domain prefix;
no old-name compatibility aliases remain.

### [ ] 6.0e Prove integrated account and verification behavior

**Work:** Integrate account selection/defaults, normal requests, aggregate caches,
runtime test admission, and consumer outputs. Document `ct account`/`ct test`,
credential lifecycle and separate-process limits, strict/normal decoding coverage,
optional `--clean`/cached warnings and saved-response diagnosis, per-request
approval identities, full Rust API cutover, and how to run offline Cargo tests. Update the inventory,
contracts, auth docs, and Unreleased notes to the final implemented design.

**Validation:** Use two synthetic accounts in one tenant/source sharing a cache,
including mixed-account composition and rebind/logout races. Exercise metadata-only
test listing, selected read runs with denied writes/solicitation, scoped defaults,
both clean modes/replay against synthetic isolated storage, and real
`--auth-source none` dispatch without service I/O. Test an approved admin read
alongside another account's request without default substitution or credential
resources in serialized request data.
Audit all Cargo targets before admitting a workspace test command; until that
audit completes, keep the existing reviewed-target policy. Record exact commands,
platform gaps, and which layers have runtime evidence. Live API compatibility
remains a separate operator action, not an inferred test result.

**Completion criteria:** Accounts, test isolation/admission, and request-first
cutover have integrated offline evidence. Every prior live Cargo case has a
disposition; the ordinary test workflow cannot resolve ambient credentials or
access services. Existing tasks 6.1–6.3 can proceed against the settled contract.

### [ ] 6.1 Separate audit activities without centralizing their input data

**Prerequisite:** Complete 6.0a–6.0e. Audit callers use the settled account selection
and request-first API, avoiding a second broad caller migration.

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

**Foundation:** Tasks 3.2 and 4.2 now bind the application executor and propagate
it through service callers, cache list/clean paths, and both examples. This item
retains end-to-end ownership/feature validation and any remaining application
cleanup; broad executor parameter propagation is no longer pending.

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

**Verified foundation:** Task 3.3 already supplies executors to registry invocation;
U118 and U120 now supply child scopes to UI workers and share background-task
ownership. Reconcile those completed changes against the inventory rather than
repeat them. Remaining request-owned runtime fields still need classification;
for example, `crates/hcl/src/reflow/reflow_new.rs` retains an optional borrowed
Azure tenant auth context. Preserve legitimate execution contexts and scope-only
operations while closing the actual remaining consumer gaps.

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
cargo test --offline --locked -p cloud_terrastodon_registry_execution --lib
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
where replaced. Audit that 3.0/3.1 already removed the registry-values compatibility
exports, retired `registry_values`/`credentials` packages, and updated all owning
imports without dependency aliases. Remove any other remaining migration
compatibility exports and update their consumers in the same change. Keep legitimate AppContext/auth
runtime fields and unrelated pure operations; search findings require classification
rather than blind edits.
Verify 6.0d's forwarding helpers and 6.0b's live Cargo entrypoints are gone;
retain only classified operations with a genuine remaining responsibility.
Preserve output formats, operation effects, mandatory cancellation via executor,
explicit affected-read invalidation, and required-completion units in mutations.

```powershell
rg -n 'MEMORY_CACHE|bind_request_cancellation|impl_cacheable_into_future|impl_cacheable_work_into_future|AuthContext::resolve' crates examples
cargo check --offline --locked --workspace --all-targets
```

**Completion criteria:** No production caller relies on hidden execution state or
the old cache lifecycle, and no migration compatibility re-exports remain. Each
search finding is removed or justified by its actual remaining responsibility.
The full workspace compiles against the new contract.

### [ ] 7.2 Verify acceptance and document public migration/release impact

**Work:** Run the reviewed local test matrix and cross-platform checks. Update
REUSABLE_APPLICATIONS.md, affected request docs/examples, public import paths, and
the Unreleased changelog with concrete API/behavior changes. Reconcile references
in the earlier auth plan without importing its stale statements. Review new
package publication order and exact pinned dependency resolution; do not publish
or bump unrelated versions. Document direct owning-crate imports and breaking API
changes without compatibility re-exports. Update this plan's evidence and privacy
audit.

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
| Shared cache handle and executor clones | Coordinated within one owner | Concurrent typed sharing, per-key ownership, generation/dependency races | Tasks 2.3–2.4 cache races and task 3.2's shared decoding, ancestry, admission, and caller-cancellation tests passed |
| Independent handles/processes sharing disk | No single-flight/coherent invalidation guarantee yet | Document limits; phase 8 supplies stronger proof later | Deferred |
| Command, REST, in-process reads | One lifecycle, canonical memoization | Local adapter + cache tests; different validators still run | Task 4.1 Command checks and task 4.2 REST typed-sharing/caller-policy loopback checks passed; all callers compile |
| Raw artifact storage | Isolated complete attempts and guarded selection within one handle | Local storage/integrity, freshness, and worker-drop tests | Core storage and Command checks passed; task 4.2 REST failure/outcome/sensitive-retention tests passed |
| SDK/transport namespaces and ownership | Separate concrete entries; shared logical invalidation scope | Synthetic wrapper succeeds; same-namespace owner/type collisions bail before execution/reuse; invalidation covers all kinds | Core namespace/claim/invalidation tests and REST wrapper/envelope loopback scenario passed; all callers compile |
| Mutations, zero-duration and uncached work | Execute each invocation; retain required artifacts | Cancellation and artifact/invalidation tests | Executor/Command checks and REST independent-mutation/required-completion loopback checks passed |
| Graph principals, ARM RBAC, Azure DevOps projects | Explicit auth/scope; shared assembled results | Typed local composition/discovery tests and compile checks | Explicit executor/auth binding and discovery checks passed in 4.2; shared aggregates and dependency freshness remain phase 5 |
| Azure CLI bearer, browser, workload identity | Explicit request tenant/source | Offline selection/command construction; no live credential tests | Auth/runtime selection and task 4.2 caller-provider rebinding checks passed; no live credential validation |
| PAT and local-only/disabled auth | PAT tenant-independent; local work credential-free | Selection/admission tests | Provider policy, CLI early admission, raw REST disabled-auth, group-management disabled-auth, and small-stack dispatch checks passed |
| Named accounts and scoped aliases/defaults | Multiple identities per tenant/service; explicit immutable request selection | Same-source/account isolation, default resolution, binding replay, logout/rebind completion and dependency tests | U123/U127/U128 design recorded; 6.0a/6.0c/6.0e pending |
| Cargo tests and registered runtime verification | Offline Cargo workflow; deliberate runtime checks with optional --clean and warning/provenance for cache use | Classified test inventory, offline listing, effect/input admission, clean/cached/replay/strictness reporting and synthetic runner checks | U130/U132 refine the design; 6.0b/6.0e pending; no live run authorized |
| Direct request callers | Trivial fetch wrappers removed; behavior/borrowing preserved | Per-family full consumer compilation and inspected offline behavior checks | ExecutableRequest::execute exists; 6.0d pending |
| Registry, UI, discovery, CLI output | Explicit execution; offline planning preserved | Synthetic invocation/discovery and scoped UI/output checks | Task 3.3's registry, shared-output discovery, UI lifecycle/export, and entrypoint dispatch checks passed; broader service/output consumer migration remains phase 6 |
| App with auth and without auth; workspace examples | Supported feature boundaries; no production inputs | Both process-suite feature configurations, `--auth-source none` local handler with isolated storage, and consumer compile checks | Task 3.2's no-auth library and auth-enabled binding tests plus workspace compilation passed; later consumer/process acceptance remains task 6.2 |
| Windows | Primary local validation platform | Existing linker/toolchain; local PowerShell fixture suites | Command phase 4.1's 51 local tests and 16 focused policy tests passed; later migration phases remain open |
| Linux | Preserve documented consumer support | Repeat relevant compilation/tests when environment available | Pending |
| Live cloud and pipeline use | Outside this migration's development validation; no production data access or ingestion | Local synthetic fixtures and compile checks only; live compatibility is not established | Not run |

## Overall initial-migration completion criteria

- [ ] Every active U01–U132 requirement has task evidence; G1–G6 and A1–A4/V1 are closed and
  phases 1–7 complete. Phase 8 remains explicitly deferred.
- [ ] Requests contain explicit operation/auth selection data and no execution
  resources; both execute spellings and registry invocation share one contract.
- [ ] All moved-type consumers use the owning crate/API; no migration compatibility
  re-exports remain, and examples and public documentation show the new imports.
- [ ] Pure `registry` and executor-aware `registry_execution` own their respective
  registrations; runtime `auth` and vocabulary `auth_types` replace `credentials`.
  `AuthSource` belongs to `auth_types`, `AzureTenantId` remains in `azure_types`,
  and `registry_values`/old package aliases are absent. The normal/build graph
  remains acyclic and the integrated catalogue has no duplicate registrations.
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
- [ ] Multiple accounts in one tenant/service, scoped aliases, configured defaults,
  binding replacement/logout, and account-dependent cache reuse satisfy the
  companion contracts without ambient account fallback or cross-process overclaims.
- [ ] Cargo tests are isolated from live credentials/services. Registered runtime
  verification uses explicit scope, actual effect admission, optional `--clean`
  with complete underlying-read invalidation, warnings for cached runs, deliberate
  saved-response diagnosis, and honest build-mode/coverage reporting; cached or
  replayed success does not imply current live compatibility.
- [ ] Approval surfaces can inspect each request's selected account/source/tenant;
  execution cannot substitute a requester's/default account. Credential renewal
  is distinct from changing the approved principal, and requests carry no runtime
  auth resources or mandatory token/session generations.
- [ ] Operation call sites construct requests and use `.execute(executor)`;
  forwarding helpers are gone and meaningful scope-only/workflow helpers remain.
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
| Named profile rebind redirects serialized request or cached aggregate | A2/A3 immutable binding intent; 6.0c/6.0d generation/dependency proof, including mixed-account parents |
| Logout turns a successful write into an early error before cleanup | A3 distinguishes closed new admission from admitted required completion; 6.0c mutation/response/artifact/invalidation drain tests |
| Cached/replayed test result mistaken for fresh service coverage | U130/V1; optional clean invalidation, warning/provenance, saved-response versus typed-hit distinction, coverage/strictness metadata, no verdict memoization, explicit zero-case behavior |
| Root invalidation leaves child data cached or parser retry silently refetches | U130/6.0b; underlying read-contract delegation across branches, dynamic-child coverage, explicit diagnostic artifact replay without normal success publication |
| Reviewed operation executes under a different account/default | U131/A2/A3; serialized stable account/source/tenant, exact provider admission, credential-renewal versus principal-change tests |
| Test marker or Read intersection admits writes/solicitation | V1 separate purpose metadata and full allowed-effects subset; offline admission tests |
| Broad helper deletion adds clones, loses typed errors, or bypasses child ownership | 6.0d classification and complete per-family caller validation |
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
| Runtime observer closures silently skipped or captured in cached policy | G4; 3.2 borrowed caller-wrapper lifetime checks; phase 4 and SDK slices own production callback/observation behavior and tests |
| New executor creates command/auth or registry cycles | G1; 3.0/3.1 normal/build DAG checks including pure conversions and metadata; staged checks and release-order review |
| Legacy timestamps, metadata, or cache paths misrepresent freshness | G5; 2.2 legacy/expiry policy; preserve diagnostic inspection |
| Claimed thread safety mistaken for cross-process guarantees | U27; acceptance matrix and separate phase 8 |
| Tests or cache inspection expose production data to tools/agent context | U29; reviewed local targets, auth-feature startup with none, isolated synthetic storage, no test.ps1/workspace suite/ignored tests |
| Plan drifts or includes identifying local output | Stable ledger, three-pass audit, relative evidence, privacy scan on update |
