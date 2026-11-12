# Account authentication resolution and CLI ownership cutover

**Plan status:** Complete; comprehensive Windows offline acceptance passed.
Changes remain uncommitted.
**Primary implementation root:** Repository root (`.`).
**Last updated:** 2026-11-10
**Intent audit:** Passed against the available post-U135 design conversation and
the existing migration ledger; earlier implementation evidence remains historical.

## How to update this plan

- `[ ]` Not started; `[~]` in progress; `[x]` complete; `[!]` blocked.
- Update task headings and their local completion evidence together. Do not mark
  a phase complete while any of its required behavior or validation remains.
- Parallel ownership is intentional: account model/store/provisioning; Azure CLI;
  Azure DevOps CLI; executor/application integration and final validation.

## Guidance ledger and traceability

| ID | Active guidance | Implementation and proof |
| --- | --- | --- |
| R1 | A shared executor has no current-account setting. Requests explicitly select accounts; CLI leaves do not construct AuthContext or replace providers. | 2, 3; mixed-account execution and retirement tests |
| R2 | Bearer SDK requests serialize concrete account ID, source and tenant. Reuse AzureBearerAuthenticationSelection in both authentication enums. | 1, 2; compilation and selection validation |
| R3 | Preserve tenant-scoped account aliases and defaults. Tenant/account aliases are separate. A tenant can default in CLI input but must be concrete in SDK request data. | 1, 3; scoped selector tests |
| R4 | CLI account is Option<CloudTerrastodonAccountArgument>; omission resolves the configured scoped default. No Default account-selector variant or alias fallback. Reserve the name default, case-insensitively. | 1, 3; constructor/default failure tests |
| R5 | One account can use browser or Azure CLI mechanisms for the same principal. Source has an account-configured preference and explicit per-invocation override. Never probe sources until one works. | 1, 2; principal continuity and mechanism selection tests |
| R6 | Flatten AzureAuthenticationArgs / AzureDevOpsAuthenticationArgs into service leaves. They own field documentation, metadata resolution and coherence checks. | 3; representative leaf integrations |
| R7 | Azure DevOps org/project stay explicit leaf operation fields. Tenant selects bearer namespace; no tenant selects org PAT namespace. PAT plus tenant and bearer without tenant fail. --tenant default can explicitly resolve the configured tenant. | 3; argument resolution policy tests |
| R8 | Output-producing leaves return CliOutput; shared output boundary emits once. Keep Cow inputs, Arc outputs and typed errors. | 3, 4; output/compilation checks |
| R9 | Auth/source/account arguments leave GlobalArgs and App startup. Local commands do not resolve accounts or credentials. Auth-sensitive runtime tests, interactive/UI launchers and tools own their selection boundary. | 4; local App/process tests and consumer audit |
| R10 | Preserve leases, account/source cache isolation, cooperative required completion and no authorization reentry. No provider reset per nested request. | 2; retirement and execution contract tests |
| R11 | Full cutover; no compatibility re-exports or temporary parallel production paths. Preserve existing uncommitted work; no commits, staging or live/production access. Plans contain no personal paths/PII. | All tasks; final repository audit |
| R12 | Update plans before implementation and complete all affected callers and validation, not only demonstration leaves. | 0, 1–5 |

Superseded proposals: globally unique account aliases; removing tenant/source
from request data; an account Default variant; alias-default fallback; mandatory
tenant flags for every Azure invocation; leaf-specific executor binding; and
flat repeated authentication fields. R6 explicitly authorizes the shared CLI
input contract despite the general convention against convenience field bags.

## Intent audit evidence

- **Extraction:** Captured the available discussion from moving global arguments
  to leaves through the Azure/DevOps flattened examples and implementation request
  as R1–R12. Earlier foundations are preserved by the U1–U135 migration ledger.
- **Traceability:** Each requirement maps to a task and an appropriate policy,
  lifecycle, integration or repository check; no production data is needed.
- **Adversarial omission:** Rechecked the changes of mind: tenant remains explicit
  in requests but CLI may default; aliases remain scoped; account omission uses
  Option; same account supports multiple verified mechanisms; final flattening
  direction supersedes field duplication; DevOps PAT never gains a fictitious
  tenant; changes stay uncommitted.
- **Source limitation:** Earlier work predating the available design conversation
  is supported by the existing ledger and recorded evidence, not re-audited as
  newly executed work.

## Foundation and decisions

The existing cache, request-first SDK, runtime verification registry and offline
acceptance are foundations. This refinement does not add cross-process cache
coordination, change service endpoints, or introduce workflow approval machinery.

| Gate | Decision |
| --- | --- |
| Identity | Account identifies a principal in its namespace; mechanism registrations and sessions belong to it. First verified provisioning establishes identity; additional mechanisms must prove continuity. Email/display-name matching and unsigned JWT decoding are insufficient. |
| Provider | One invocation-owned account-aware provider lazily initializes and retains sessions by account/source; clones share ownership. Metadata selectors never acquire credentials. |
| Cache | Account, source and session/generation admission remain distinct. Existing affected-read/dependency invalidation crosses appropriate view variants. |
| Defaults | Resolve tenant, account, source in that order. Defaults are configured deliberately; errors do not trigger fallback. Explicit source override must be configured and usable. |
| Runtime boundary | Generic execution receives pure authentication types. CLI selectors and runtime credential implementation stay outside the execution crate. |
| Credential replacement | Verified same-principal provisioning may add mechanisms without changing account ID. Replacement cannot silently change principal. Logout/removal retains required completion and scoped cleanup. |
| Persistence | Reject the earlier unverified single-source draft schema with an actionable migration error. Preserve its metadata, IDs and credential files untouched; never silently assign a verified principal, import ambient credentials, or erase credentials. Future explicit conversion must preserve account IDs and verify identity. |

## [x] 0. Record the revised implementation contract

**Completion notes:** R1–R12 and the superseded directions above reflect the
accepted examples. Initial source inspection found profiles storing one credential
and session, providers supporting explicit account maps, and CLI leaves rebinding
providers. Those baseline limitations are addressed by tasks 1–4.

## [x] 1. Separate account identity from credential mechanisms

**Completion notes:** Multi-source model/store/provisioning implemented. Account
IDs survive source attachment and replacement. Earlier draft records without a
verified principal are rejected without changing their IDs or credential files.
Focused source-lifecycle, metadata-resolution and headless-session tests passed
with the 61-test auth suite, including a selected-generation regression:
aliases/preferences/sibling changes retain the session, while selected credential
replacement rejects new admission and preserves existing leases.

**Work:** Add coherent multi-source registrations/preference and verified principal
continuity; reserve default alias; adapt account add/login/import/preference/logout
commands and persisted catalogue handling. Verify new identity acquisition against
official documentation; test with synthetic providers only.

**Validation:** Focused auth_types/auth tests using temporary account stores;
identity mismatch, explicit/default source selection, alias/namespace errors,
source lifecycle, persisted validation and retirement tests.

**Completion criteria:** Same account can use distinct verified sources; mismatched
principal/tenant/source cannot become active or reuse cached results.

## [x] 2. Resolve request-selected sessions through one shared provider

**Completion notes:** Shared lazy account/source provider, tuple enum payloads,
metadata resolution, retirement and source-specific leases implemented. Core
auth/execution tests passed as part of 130 focused tests, and the workspace
all-feature/all-target check passed. Core all-target Clippy passed with warnings
denied. Logout/remove now retire and drain through the shared executor before
durable credential cleanup; source retirement preserves sibling mechanisms.

**Work:** Consolidate bearer enum payloads; add pure metadata resolution boundary;
retain shared lazy per-account/source initialization and top-level execution leases;
remove user-facing provider-binding ceremony. Internal request admission hooks
retain leases across required completion and preserve each request's custom error
type; they do not select a current account or create replacement sessions.

**Validation:** Focused execution/auth tests for concurrent different-account and
different-source requests, single initialization, failures/retirement before cache
reuse, nested lease reuse and required completion.

**Completion criteria:** Request identity determines authentication through the
same executor; no current-account state or per-child session reconstruction.

## [x] 3. Cut over Azure and Azure DevOps CLI leaves

**Completion notes:** Both CLI trees use the shared `auth_cli` contracts and
return owned `CliOutput` values. Selected output records retain shared handles;
the non-Clone claims inventory retains its owner and renders sorted borrowed
views. Workspace all-target/all-feature compilation passed.

**Work:** Implement shared flattened authentication argument contracts and methods;
replace incoming AuthContext parameters/provider rebinding throughout leaves and
dispatchers; retain operation scope fields and return CliOutput.

**Validation:** Compile every CLI target and exercise actual default/source/scope
policy using temporary catalogues; no tests merely restating parser declarations.

**Completion criteria:** Both service CLI trees follow the accepted examples,
including nonstandard workflows and audit/picker helpers.

## [x] 4. Remove global selection and update remaining consumers

**Completion notes:** Global auth flags and App auth feature removed. App accepts
one injected provider and defaults to anonymous. Entry point, runtime tests,
UI boundaries, tools and examples passed all-target compilation. Old runtime
context and binder source consumers are gone. Interactive DevOps actions now
prompt for organization, tenant/PAT scope, account and source before metadata
resolution, so they do not force bearer authentication on PAT users.

**Work:** Simplify GlobalArgs/App/AppContext and entrypoint startup; update Gitea,
runtime test runner, interactive/GUI boundaries, registry workflows, examples and
resource-type tooling. Remove obsolete context APIs/dependencies after callers move.
Retain explicit disable-authentication behavior at relevant selection boundaries.

**Validation:** Workspace compile, App/process and runtime-test policy tests;
search for old globals, contexts and bind helpers across production consumers.

**Completion criteria:** Local commands need no auth selection, all service entry
points own explicit selection, and no compatibility plumbing remains.

## [x] 5. Validate the complete cutover and reconcile public documentation

**Completion notes:** Workspace all-target/all-feature check and Clippy with
warnings denied passed. The full workspace test sweep exposed four failing
targets; all 398 tests in those targets now pass after correcting conditional
discovery assertions, the Cognitive Services CLI resource/authentication argument
collision, UI navigation of the shared bearer payload, and random HCL fixtures.
Public authentication/application/runtime-verification docs, examples, changelog
and inventories match the implemented APIs. Doctests, minimal App acceptance and
final formatting passed. No source-level acceptance failures remain.

**Work:** Update authentication/application/runtime-test docs, examples, changelog,
root migration plan and inventory; run focused and full offline acceptance after
reviewing tests for external effects. Preserve tracked and staged user changes.

**Validation:** Use these resource-conserving environment settings for all Cargo work:

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
cargo test --offline --locked --workspace --all-features --lib --tests --no-fail-fast
cargo clippy --offline --locked --workspace --all-targets --all-features -- -D warnings
cargo test --offline --locked --workspace --all-features --doc
cargo test --offline --locked -p cloud_terrastodon_app --no-default-features --lib
cargo +nightly fmt --all -- --check
git diff --check
```

**Completion criteria:** Current-tree Windows checks pass; documentation matches
implemented ownership/commands; remaining platform/live-service limitations are
explicit. No production data or credentials were accessed. Changes uncommitted.

### Final acceptance evidence (2026-11-10)

| Check | Result |
| --- | --- |
| Workspace all-feature/all-target compilation | Passed; final Clippy also checked every target after the corrections. |
| Full workspace library/integration sweep and corrected-target rerun | 88 targets covered. Initial sweep: 1,647 passed, 12 failed, 5 existing ignored. All four affected targets were corrected and rerun together: 398 passed, none failed/ignored, including one additional discovery-policy regression. Combined current coverage: 1,660 passing tests and 5 existing ignored. |
| Workspace all-feature/all-target Clippy | Passed with `-D warnings`. |
| Workspace doctests | 7 passed, 1 existing ignored, none failed. |
| App without default/optional features | 33 passed, none failed/ignored. |
| Nightly formatting and whitespace | `cargo +nightly fmt --all -- --check` and `git diff --check` passed. |
| Dependency/source audit | Only internal workspace dependency edges and the new `auth_cli` package changed in the lockfile; external versions/sources/checksums unchanged. Obsolete context binders have no production callers. |
| Data and repository ownership | Used synthetic fixtures, temporary stores, loopback HTTP and controlled local processes. No live credentials or production data accessed. No staging or commits performed; existing staging preserved. |

Corrected targets were rerun with:

```powershell
cargo test --offline --locked -p cloud_terrastodon_azure_devops -p cloud_terrastodon_entrypoint -p cloud_terrastodon_hcl -p cloud_terrastodon_ui_ratatui --all-features --lib --test discovery_plan --no-fail-fast
```

Local logs are under `target/account-auth-*.log` and are generated artifacts, not
tracked documentation. The earlier single-credential draft catalogue is rejected
without rewriting or deleting it; automatic migration is not provided. Real
identity-provider/service compatibility and Linux execution remain unvalidated.

## Risks and completion gates

- Identity verification is a provisioning gate, not a metadata-lookup assertion.
- Account/source initialization and retirement must share owners to avoid stale
  cache admission or deadlocks during required mutation completion.
- Windows all-target builds need bounded debug artifacts; do not run parallel Cargo
  jobs from independent agents against the same target directory.
- Prior Windows acceptance proves the foundation, not this refinement. Linux and
  live-service compatibility must not be claimed from synthetic tests.
