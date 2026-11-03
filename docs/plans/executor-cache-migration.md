# Explicit execution and shared typed cache migration

**Plan status:** Active migration; tasks 2.6–2.8 and their tooling refinements were
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
4.1b with executor-bound raw artifacts. The production Command adapter and
caller cutover remain unfinished; REST follows in 4.2.
U78 authorizes the local checkpoint titled
`Add scoped executor with shared cache and injected authentication`, including
U76's execution-mode names and U77's startup diagnostics.
That checkpoint is `3478e3bb`; U79 begins from its clean working tree.
The registry checkpoint is `09d8bb11`.

**Primary implementation root:** Repository root (`.`); no branch change prescribed.

**Last updated:** 2026-11-03

**Source baseline:** `26e14bcc`; tracked files were initially clean. Pre-existing
untracked notes are preserved.

**Intent audit:** Updated 2026-11-03 using the preserved ledger and available user
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

**Current authorization:** The user authorized proceeding with the migration on
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

## Authoritative user guidance ledger

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
5 SDK slices → 6 audit/app/consumers → 7 cutover/acceptance`; phase 8 is deferred.
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

### [~] 4.1 Route subprocess execution through shared work

**Active task 4.1b (U83):** The first slice, 4.1a, is complete and committed below.
Production Command callers still use the existing cache boundary until 4.1b
completes the explicit-executor cutover. The original completion criteria below
remain required for task 4.1.

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

#### [~] 4.1b Bind Command artifacts and decoded results to the executor

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
lockfile entries changed. The new implementation and documentation remain
uncommitted. These are synthetic local Windows checks; no credentials,
production data, or remote services were accessed. They establish the executor
boundary, not a completed Command/REST transport cutover. Source review found
no outstanding correctness issue in this prerequisite.

**Remaining work:** Route canonical command decoding through shared work, apply per-caller
validators on hits, and migrate actual callers to an explicitly supplied executor
and operation owner. The current Command APIs accept only a cancellation scope;
the source inventory finds 42 production caller files across SDK, auth, HCL,
Gitea, and entrypoint code. There must be no implicit default executor or second
adapter left alongside the old path.

The owned uncached path must preserve non-`Sync` output and explicit semantic
admission: a reusable initializer must reject nonreusable child work even when
there is no shared result to retain. Do not force that API through the shared
method's `Sync` bound or silently bypass parent dependency admission. Keep the
Command API and real caller migration together; constructing an executor from
only a cancellation scope or retaining a parallel legacy adapter is not a cutover.

This slice still owes typed `Arc` reuse, operation/type collision enforcement,
decoded acceptance before reusable publication, sensitive-output policy,
generation/dependency invalidation, and provider-bound authenticated subprocess
admission. The current raw-success-before-typed-decode behavior and legacy CLI
reauthentication remain scheduled for this cutover. Existing isolated credential
provider subprocesses remain outside ordinary work caching. Once these contracts
and the original 4.1 completion criteria pass, begin REST task 4.2.

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
| Command, REST, in-process reads | One lifecycle, canonical memoization | Local adapter + cache tests; different validators still run | Pending |
| Raw artifact storage | Isolated complete attempts and guarded selection within one handle | Local storage/integrity, freshness, and worker-drop tests | Tasks 2.2–2.4 storage, generation, quarantine, and bound import checks passed; production adapters pending |
| SDK/transport namespaces and ownership | Separate concrete entries; shared logical invalidation scope | Synthetic wrapper succeeds; same-namespace owner/type collisions bail before execution/reuse; invalidation covers all kinds | Pending |
| Mutations, zero-duration and uncached work | Execute each invocation; retain required artifacts | Cancellation and artifact/invalidation tests | Pending |
| Graph principals, ARM RBAC, Azure DevOps projects | Explicit auth/scope; shared assembled results | Typed local composition/discovery tests and compile checks | Pending |
| Azure CLI bearer, browser, workload identity | Explicit request tenant/source | Offline selection/command construction; no live credential tests | Pending |
| PAT and local-only/disabled auth | PAT tenant-independent; local work credential-free | Selection/admission tests | Task 3.2's provider policy and no-auth app binding passed; production adapter integration remains phase 4 |
| Registry, UI, discovery, CLI output | Explicit execution; offline planning preserved | Synthetic invocation/discovery and scoped UI/output checks | Task 3.3's registry, shared-output discovery, UI lifecycle/export, and entrypoint dispatch checks passed; broader service/output consumer migration remains phase 6 |
| App with auth and without auth; workspace examples | Supported feature boundaries; no production inputs | Both process-suite feature configurations, `--auth-source none` local handler with isolated storage, and consumer compile checks | Task 3.2's no-auth library and auth-enabled binding tests plus workspace compilation passed; later consumer/process acceptance remains task 6.2 |
| Windows | Primary local validation platform | Existing linker/toolchain; local PowerShell fixture suites | Pending |
| Linux | Preserve documented consumer support | Repeat relevant compilation/tests when environment available | Pending |
| Live cloud and pipeline use | Outside this migration's development validation; no production data access or ingestion | Local synthetic fixtures and compile checks only; live compatibility is not established | Not run |

## Overall initial-migration completion criteria

- [ ] Every active U01–U83 requirement has task evidence; G1–G6 are closed and
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
| Runtime observer closures silently skipped or captured in cached policy | G4; 3.2 borrowed caller-wrapper lifetime checks; phase 4 and SDK slices own production callback/observation behavior and tests |
| New executor creates command/auth or registry cycles | G1; 3.0/3.1 normal/build DAG checks including pure conversions and metadata; staged checks and release-order review |
| Legacy timestamps, metadata, or cache paths misrepresent freshness | G5; 2.2 legacy/expiry policy; preserve diagnostic inspection |
| Claimed thread safety mistaken for cross-process guarantees | U27; acceptance matrix and separate phase 8 |
| Tests or cache inspection expose production data to tools/agent context | U29; reviewed local targets, auth-feature startup with none, isolated synthetic storage, no test.ps1/workspace suite/ignored tests |
| Plan drifts or includes identifying local output | Stable ledger, three-pass audit, relative evidence, privacy scan on update |
