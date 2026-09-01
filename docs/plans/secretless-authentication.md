# Secretless authentication and audit pipeline support

**Plan status:** Active — browser/typed-project/audit context slice implemented; live WIF validation deferred
**Primary implementation root:** main in ~/repos/Cloud-Terrastodon (WSL)
**Last updated:** 2026-08-27
**Intent audit:** Passed 2026-08-21 against the full conversation through the request to create this local plan  
**Public issue:** Deferred; do not create or post a GitHub issue until this plan and its first milestone are reviewed  

## Current implementation slice

The local Rust track now supports the first interactive Linux handoff without
calling Azure CLI for the migrated path:

- `ct az tenant login <tenant-or-alias>` uses authorization-code + PKCE by
  default (device code remains an explicit PIM-only fallback) and persists a
  refresh token in the platform-appropriate local store.
- The browser consent request covers the normal Graph, ARM, and Azure DevOps
  delegated resources in one handoff; code redemption still requests one
  resource at a time, and the callback URL/port is printed for WSL forwarding.
- `ct az devops project list` receives the entrypoint `AuthContext`; every
  continuation page uses that context and its per-page cache key. It performs
  the resource-token preflight before resolving organization metadata or
  consulting the project-list cache, then reuses that token for all pages.
- `ct audit azure` and `ct audit azure-devops` now receive the entrypoint
  context, derive their default tenant from WIF/browser credentials, and pass
  the same context through nested and parallel request families.
- Context-aware typed request objects own a public invocation `AuthContext`
  field, and `AuthContext` itself implements Facet with credential caches marked
  opaque/sensitive. Their `IntoFuture` implementations execute with the
  request's context rather than resolving an implicit global or passing `None`.
  Legacy constructors use an explicit Azure CLI compatibility context until
  their callers are migrated to the entrypoint context.
- `auto` selects workload identity first, then a stored browser session for an
  interactive local invocation, and finally Azure CLI compatibility. It does
  not start a new browser login; that handoff is explicit through tenant login
  or `--auth-source browser`. CLI token acquisition uses fail-fast retry
  behavior so a missing session cannot launch an implicit device-code login.

Live browser callback, tenant permissions, and WIF pipeline validation remain
external checks and are intentionally not run during local implementation.

## How to update this plan

- [ ] Not started
- [~] In progress
- [x] Complete
- [!] Blocked — include the exact blocker, last evidence, and unblocking condition

Put a work item's status in its heading. Update its heading and completion notes
together. A phase is complete only when every work item in it is [x]. Record
design decisions, affected paths, validation results, and follow-ups below the
task they affect; do not append a detached chronological work log.

Implementation commits require explicit user approval. This plan is currently an
uncommitted local artifact.

## Authoritative user guidance ledger

| ID | Active guidance | Required plan consequence | Superseded by |
| --- | --- | --- | --- |
| U1 | Cloud Terrastodon must build and run on Linux; Windows-only command assumptions were encountered. | Preserve Linux command defaults and include Linux in the first authentication target. | — |
| U2 | Do not commit changes unless explicitly requested. | Keep implementation and this plan uncommitted unless the user later authorizes a commit. | — |
| U3 | Authentication flows should not require device-code flow. | Keep browser authorization-code + PKCE as the preferred local delegated flow; device code may remain an explicit fallback, not a prerequisite. | — |
| U4 | Azure DevOps pipelines must work without an interactive signed-in user. | Add workload identity federation using the Azure DevOps/Entra trust relationship and short-lived tokens. | — |
| U5 | Support Graph, Azure Resource Manager, and Azure DevOps through a shared authentication story. | Extend credentials and REST boundaries; keep resource audiences distinct. | — |
| U6 | Accept both canonical Azure-style environment names and Azure DevOps task names. | Support AZURE_CLIENT_ID/servicePrincipalId, AZURE_TENANT_ID/tenantId, and AZURE_FEDERATED_TOKEN/idToken; define conflict and partial-input behavior. | — |
| U7 | Use one app registration/service connection for Graph, ARM, and Azure DevOps. | Design one-registration support while keeping delegated permissions, application permissions, RBAC, and token audiences explicit and separate. | — |
| U8 | PIM is an interactive-only need; pipelines doing PIM are not anticipated. | Keep current PIM operations delegated and interactive; explicitly exclude pipeline PIM from the first milestone. | — |
| U9 | Audit is the target surface for the first migration because it is useful in a pipeline. | Make ct audit azure and ct audit azure-devops the first end-to-end acceptance commands. | — |
| U10 | There are many CLI-backed commands; sample the inventory before committing to all migrations. | Record the inventory and migrate only the audit dependency slice first; classify remaining CLI uses. | — |
| U11 | A global --auth-source mechanism is appropriate; --debug should not carry authentication semantics. | Add a dedicated global authentication preference and propagate it to the shared credential/REST layer. | — |
| U12 | REST and command helpers should centralize behavior; migrate commands away from Azure CLI where an API exists. | Route typed requests and ct rest through the provider; replace applicable direct CLI wrappers with Graph, ARM, or Azure DevOps REST. | — |
| U13 | Work locally for now; do not pursue the public GitHub issue yet. | This file is the authoritative working plan. A public issue is a later coordination artifact. | Earlier issue-draft guidance is superseded. |

## Guidance traceability

| Guidance | Plan coverage | Evidence when complete |
| --- | --- | --- |
| U1 | Foundation, acceptance matrix, phases 2–6 | Linux build/check and pipeline evidence |
| U2 | Metadata, update rules, handoff constraints | No commit without explicit approval |
| U3 | Scope, phases 2.2 and 5.1, acceptance matrix | Browser PKCE and explicit device behavior tested/documented |
| U4 | Purpose, phases 2–4, phase 6.2 | Headless WIF pipeline succeeds without user/device/PAT |
| U5 | Gates 1–3, phases 2–4 | Graph, ARM, and Azure DevOps use the shared provider |
| U6 | Gate 1.2, phases 2.1–2.2 | Both environment contracts and invalid-input tests pass |
| U7 | Constraints, gate 1.3, phase 1.3 | One-registration permission/federated-credential matrix validated |
| U8 | Scope/non-goals, gate 1.3, phase 4 | PIM remains delegated/interactive; pipeline PIM unsupported |
| U9 | Purpose, phase 4, acceptance matrix | Both audit commands pass in WIF pipeline |
| U10 | Inventory, phase 4.2, phase 5.2 | Every remaining CLI use is classified |
| U11 | Gate 1.1, phase 2.2, phase 5.1 | Global auth-source selection is parser- and behavior-tested |
| U12 | Scope, gate 1.4, phases 3–5 | Applicable CLI wrappers use shared REST paths |
| U13 | Metadata and phase 6.3 | Plan remains local; issue creation is separately authorized |

## Intent audit evidence

- **Pass 1 — extraction:** Reread the original user messages from the Linux build and Windows command assumptions through the WIF discussion, issue-draft review, decisions about environment aliases, one registration, delegated PIM, audit scope, auth-source, and the final local-plan request. The ledger preserves the build constraint, authentication goals, no-user pipeline qualifier, PIM exclusion, migration scope, global preference, no-issue direction, and no-commit constraint.
- **Pass 2 — traceability:** Mapped every active ledger entry to scope, constraints, design gates, work items, validation, acceptance criteria, or an explicit non-goal. The audit target is intentionally narrower than the full CLI inventory, and the plan records why.
- **Pass 3 — adversarial omission:** Rechecked the qualifiers both environment contracts, one registration, interactive-only PIM, audit as the first surface, without device/user/PAT/CLI in pipelines, and for now regarding the public issue. The plan keeps Azure CLI as a local compatibility source rather than silently removing it.
- **Known source limitation:** No user-instruction source was unavailable. Exact tenant permissions, Azure DevOps service-connection configuration, and live pipeline availability remain external design/validation gates.

## Purpose

Enable ct audit azure and ct audit azure-devops to run in an Azure DevOps
pipeline without an interactive user, device-code flow, embedded client secret,
or PAT. The pipeline authenticates the one configured app registration through
workload identity federation and uses short-lived, resource-specific Entra
tokens. Local delegated PIM remains browser-based and interactive.

Authentication is a shared concern of credentials and REST layers. Audit request
types must not parse pipeline variables, decide token audiences, or invoke
interactive login.

## Scope

### In scope

- A shared authentication source/provider in crates/credentials.
- Workload identity federation for Azure DevOps pipeline execution.
- Both canonical and Azure DevOps task environment-variable contracts.
- A global --auth-source preference with a safe auto mode.
- ARM and non-PIM Graph token acquisition through the shared provider.
- Azure DevOps Entra token acquisition and REST authorization.
- Correct distinction between Entra Bearer tokens and actual PAT Basic authentication.
- ct audit azure as the first ARM/Graph end-to-end surface.
- ct audit azure-devops as the first Azure DevOps migration surface.
- Migration of the five direct Azure DevOps CLI-backed request families used by audit_azure_devops.
- Headless-mode errors that never silently launch az login or device-code authentication.
- Unit, focused integration, pipeline validation, and authentication documentation.

### Out of scope for the first milestone

- PIM operations in a pipeline. Existing PIM remains delegated and interactive-only.
- Granting application permissions to make pipeline PIM work.
- Migrating every direct CommandKind::AzureCLI use.
- Removing Azure CLI completely from local compatibility workflows.
- Replacing Terraform/app-registration configuration before its permission contract is reviewed.
- Creating or posting the public GitHub issue.
- Embedding or persisting client secrets, OIDC assertions, or access tokens.

## Established foundation

These facts were verified in the baseline checkout before this implementation pass:

- The checkout is clean on main and one commit ahead of origin/main; the latest local commit is 37aa9978 fix linux exe. Linux command defaults are cfg-gated in crates/config/src/commands_config.rs.
- crates/credentials/src/pim_graph_access_token.rs:81-109 already prefers browser authorization-code + PKCE and only selects device code when CLOUD_TERRASTODON_PIM_AUTH_FLOW=device_code is explicitly set.
- crates/credentials/src/azure_access_token.rs:9-20 shells out to az account get-access-token for every resource.
- crates/rest/src/request_execution.rs:37-150 centralizes Graph and ARM bearer requests and recognizes Azure DevOps as a REST service, but its Azure DevOps path at :80-99 reads the PAT/credential-manager path directly.
- crates/credentials/src/azure_devops_pat.rs:38-45 reads AZDO_PERSONAL_ACCESS_TOKEN on non-Windows and Windows Credential Manager on Windows.
- crates/credentials/src/azure_access_token.rs:29-34 calls an Entra token an AzureDevOpsPersonalAccessToken; the downstream helper uses the PAT-shaped type. Token type and header semantics must be corrected before WIF.
- crates/entrypoint/src/cli/global_args.rs:4-29 has logging/backtrace options only. --debug is not an authentication preference.
- crates/entrypoint/src/cli/command/rest/rest_cli.rs:37-113 infers Graph, ARM, and Azure DevOps services and delegates to RestRequest.
- crates/command/src/command.rs:418-504 retries some failures by invoking az account list and az login. That behavior must be gated by authentication source and disabled for headless WIF.
- Inventory found 40 Rust files and 58 CommandKind::AzureCLI references. Excluding tests/examples leaves 37 files and 46 references; 13 are in crates/azure/src and 18 in crates/azure_devops/src. These are syntactic references, not unique public commands.
- crates/entrypoint/src/noninteractive/audit_azure.rs calls typed ARM/Graph request helpers and contains no direct CLI-backed domain request; its remaining CLI dependency is centralized token acquisition.
- crates/entrypoint/src/noninteractive/audit_azure_devops.rs uses five direct CLI-backed request families: user license entitlements, projects, test plans, test suites, and project groups. Its member-group request already uses RestRequest but currently inherits the PAT authentication path.
- check-all.ps1 is the repository validation script. It runs cargo check --all --tests --examples --workspace, nightly cargo fmt --all, and cargo clippy --all-targets --all-features -- -D warnings; its test section is currently commented out.

Implementation evidence from this pass:

- crates/credentials/src/auth_source.rs defines the shared `AuthSource` contract; `AuthContext` resolves the source and owns short-lived credential caches; crates/credentials/src/workload_identity.rs normalizes both environment contracts and provides the injectable Entra assertion exchange boundary; crates/credentials/src/azure_bearer_token.rs separates Entra bearer tokens from PATs.
- crates/azure_devops/src/azure_devops_rest.rs centralizes encoded Azure DevOps REST URL construction. The five audit request modules no longer contain `CommandKind::AzureCLI`; the existing dump-oriented group-member list remains intentionally outside the first audit slice.
- `cargo check --all --tests --examples --workspace` passes. Deterministic tests pass for auth-source parsing, WIF normalization/form construction, Bearer-vs-PAT headers, headless reauthentication gating, REST URL/continuation parsing, and token refresh boundaries. No intentional live tenant, subscription, organization, pipeline, or issue-tracker validation was performed; the one accidental pre-existing profile-test invocation failed locally before producing validation evidence and is now ignored.

## Confirmed constraints

- The first pipeline identity is the one app registration/service connection. One registration may carry delegated public-client settings for local PIM and federated/application access required by pipeline resources, but each resource still receives a distinct token audience.
- PIM remains a user-delegated workflow. The pipeline milestone must not attempt to make PIM activation act as a service principal.
- The pipeline has no signed-in user. Pipeline operations act as the service connection identity: ARM uses its RBAC assignments, Graph uses its Entra application permissions, and Azure DevOps requires explicit organization membership, licensing, and organization permissions for the service principal.
- Both environment naming conventions are supported. Values are sensitive and must never be logged, included in command summaries, or written to cache artifacts.
- Device code is not a prerequisite. Browser PKCE remains the preferred local delegated flow; device code remains opt-in only if retained.
- auto must not trigger interactive login when a pipeline/WIF context is detected. Missing or invalid WIF input must fail clearly in headless mode.
- Azure CLI may remain available for local compatibility, but pipeline acceptance must not depend on it.
- No implementation commit or public issue is authorized by this plan alone.

## Design questions that must be closed before implementation

| Gate | Required decision | Acceptance consequence |
| --- | --- | --- |
| Auth-source contract | Confirm exact enum values and default behavior for --auth-source. Implemented values: auto, workload-identity, browser, azure-cli, and pat compatibility mode. | Parser tests cover every value, invalid input, and default. |
| Source precedence | Decide interaction between explicit flag, environment contract, local refresh token, and CLI fallback. Recommended: explicit flag wins; auto prefers complete WIF, then local delegated browser/refresh-token paths, then CLI only when interactivity is allowed. | Selection tests cover complete, partial, conflicting, and absent sources. |
| Environment aliases | Confirm aliases and behavior when they disagree or only part of a pair is present. Recommended: accept either complete set, reject disagreement, and return a redacted actionable error for partial WIF input. | Unit tests assert normalization and conflict rejection. |
| OIDC assertion acquisition | Decide whether the first slice consumes an exposed assertion only or also refreshes through Azure DevOps’ OIDC request endpoint. | Pipeline tests prove freshness/expiry behavior. |
| Token model and headers | Define resource-specific credentials. Entra uses Bearer; actual PAT uses Basic only in compatibility mode. | Header tests prevent an Entra token from using the PAT client. |
| One-registration permissions | Document delegated Graph PIM permissions, non-PIM Graph application permissions needed by audit_azure, ARM RBAC, Azure DevOps access/licensing, and federated credentials. | Permission matrix and live check prove least-privilege audit access. |
| Auth context propagation | Keep authentication invocation-owned and explicit: resolve once at the entrypoint, store it on migrated request objects, and pass it through request builders. Do not use process-global state or `None` as an implicit resolver. | `ct rest`, typed requests, and migrated helpers share auth without parsing flags or silently selecting another source. |
| Audit migration boundary | Confirm the five Azure DevOps request families used by audit_azure_devops as the first direct CLI migrations. | Focused audit run proves no direct Azure CLI process is needed. |
| CLI fallback behavior | Decide whether azure_cli is explicit-only or a final local auto fallback. Recommended: explicit in headless mode; compatibility fallback only when no WIF/browser context exists and interactivity is allowed. | Retry logic cannot invoke az login in WIF/headless mode. |

## Source and implementation references

### Repository paths

- crates/credentials/src/azure_access_token.rs — current Azure CLI token acquisition and Azure DevOps token naming.
- crates/credentials/src/auth_source.rs and crates/credentials/src/auth_context.rs — authentication-source contract, entrypoint resolution, and invocation-owned credential state.
- crates/credentials/src/workload_identity.rs — dual environment normalization and federated assertion exchange.
- crates/credentials/src/azure_bearer_token.rs — sensitive Entra bearer-token wrapper.
- crates/credentials/src/azure_devops_pat.rs — PAT environment and Windows credential-manager fallback.
- crates/credentials/src/auth_bearer_ext.rs — current Basic header encoding for PAT-shaped values.
- crates/credentials/src/pim_graph_access_token.rs — browser PKCE default and explicit device-code fallback.
- crates/credentials/src/azure_rest_resource.rs — Graph/ARM/Azure DevOps resource enumeration.
- crates/rest/src/request_execution.rs — shared REST authentication dispatch.
- crates/rest/src/rest_request.rs and crates/entrypoint/src/cli/command/rest/rest_cli.rs — REST plumbing and service inference.
- crates/entrypoint/src/cli/global_args.rs and crates/entrypoint/src/entrypoint.rs — global CLI options and initialization boundary.
- docs/authentication.md — implemented source-selection, WIF environment, pipeline identity, and PIM support policy.
- crates/command/src/command.rs and crates/command/src/command_kind.rs — command execution, CLI retry/login behavior, and configured executables.
- crates/entrypoint/src/noninteractive/audit_azure.rs — ARM/Graph audit target.
- crates/entrypoint/src/noninteractive/audit_azure_devops.rs — Azure DevOps audit target.
- crates/azure_devops/src/azure_devops_rest.rs — shared encoded Azure DevOps REST URL construction, pagination-header parsing, per-page cache keys, and offline response fixture boundary.
- crates/azure_devops/src/azure_devops_user_license_entitlements.rs — typed REST entitlement request.
- crates/azure_devops/src/azure_devops_projects.rs — typed REST project request.
- crates/azure_devops/src/azure_devops_test_plans.rs — typed REST test-plan request.
- crates/azure_devops/src/azure_devops_test_suites.rs — typed REST test-suite request.
- crates/azure_devops/src/azure_devops_group.rs — typed REST project-group request.
- crates/azure_devops/src/azure_devops_groups_for_member.rs — REST member-group request with explicit context propagation for the audit path.
- crates/azure_devops/src/azure_devops_group_member.rs — direct CLI group-membership request used by dump flows.
- check-all.ps1 — current check, format, and Clippy validation.

### Inventory commands

    rg -l 'CommandKind::AzureCLI' crates -g '*.rs'
    rg -l 'CommandKind::AzureCLI' crates -g '*.rs' | rg -v '/tests/|/examples/'
    rg -n -C 2 'CommandKind::AzureCLI' crates/azure/src crates/azure_devops/src

### Authoritative external references

- [Microsoft Entra workload identity federation](https://learn.microsoft.com/en-us/entra/workload-id/workload-identity-federation)
- [OAuth 2.0 client credentials with federated assertions](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-client-creds-grant-flow)
- [Azure DevOps Microsoft Entra authentication](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra?view=azure-devops)
- [Azure DevOps pipeline WIF updates](https://learn.microsoft.com/en-us/azure/devops/release-notes/2026/pipelines/sprint-275-update)
- [AzureCLI@2 service-principal environment variables](https://learn.microsoft.com/en-us/azure/devops/pipelines/tasks/reference/azure-cli-v2?view=azure-pipelines)
- [Microsoft Graph role-assignment schedule request permissions](https://learn.microsoft.com/en-us/graph/api/rbacapplication-post-roleassignmentschedulerequests?view=graph-rest-1.0) — newer schedule APIs list both delegated and application permissions; current CT PIM behavior remains delegated by project decision.

## Execution order

    baseline/inventory
        -> close auth contract and permission gates
        -> implement WIF/source-selection credential core
        -> route Graph/ARM/Azure DevOps REST through provider
        -> migrate five Azure DevOps audit request families
        -> wire global --auth-source and headless retry behavior
        -> validate both audit commands in a WIF pipeline
        -> document support boundaries and revisit public issue publication

## Phases and work items

### Phase 0 — Baseline and scope [x]

### [x] 0.1 Record the current authentication and CLI inventory

**Work:**

- Inspect credentials, REST dispatch, PIM flow, global CLI args, retry path, audit entrypoints, and direct CLI references.
- Record counts as a syntactic inventory only.

**Validation:**

    git status --short --branch
    rg -l 'CommandKind::AzureCLI' crates -g '*.rs' | wc -l
    rg -l 'CommandKind::AzureCLI' crates -g '*.rs' | rg -v '/tests/|/examples/' | wc -l

**Completion criteria:** The plan identifies shared auth boundaries, five direct Azure DevOps audit dependencies, and PAT/CLI paths.

**Completion notes:** Current checkout reports 40 Rust files/58 references overall and 37 files/46 references excluding tests/examples. Domain counts are 13 Azure files and 18 Azure DevOps files. Checkout is clean on main and one commit ahead of origin/main.

### [x] 0.2 Record the first milestone and non-goals

**Work:**

- Make ct audit azure and ct audit azure-devops the first end-to-end targets.
- Keep PIM interactive-only and defer the public GitHub issue.

**Validation:**

    sed -n '1,240p' crates/entrypoint/src/noninteractive/audit_azure.rs
    sed -n '1,260p' crates/entrypoint/src/noninteractive/audit_azure_devops.rs

**Completion criteria:** Audit surfaces, PIM exclusion, no-issue direction, and no-commit constraint are recorded in the ledger, scope, and acceptance criteria.

**Completion notes:** User confirmed the audit target, global --auth-source, one registration, both environment contracts, delegated interactive-only PIM, and local-plan-only workflow.

### Phase 1 — Close the authentication contract [~]

### [x] 1.1 Define the AuthSource public contract and initialization boundary

**Work:**

- Add a global authentication preference separate from --debug.
- Define auto, WIF, browser/delegated, and Azure CLI behavior, including headless detection and explicit override semantics.
- Decide how GlobalArgs initializes the shared provider without making each request type parse CLI flags.

**Validation:**

    cargo test -p cloud_terrastodon_entrypoint
    cargo test -p cloud_terrastodon_credentials

**Completion criteria:** Enum/value names, precedence, default, errors, and context propagation are documented and parser/provider-tested.

**Completion notes:** Added `AuthSource` values `auto`, `workload-identity`, `browser`, `azure-cli`, and explicit `pat` compatibility mode. `GlobalArgs` exposes `--auth-source`; the entrypoint resolves one `AuthContext` and passes it through request trees without making request types parse CLI arguments or relying on process-global authentication state. Auto refuses interactive Azure CLI authentication in CI/headless contexts.

### [x] 1.2 Define and normalize both WIF environment contracts

**Work:**

- Accept AZURE_CLIENT_ID and servicePrincipalId.
- Accept AZURE_TENANT_ID and tenantId.
- Accept AZURE_FEDERATED_TOKEN and idToken.
- Define complete-set, partial-set, disagreement, and redacted-diagnostic behavior.
- Decide whether the first slice also refreshes assertions through Azure DevOps’ OIDC endpoint.

**Validation:**

    cargo test -p cloud_terrastodon_credentials

**Completion criteria:** Provider tests prove both naming conventions and reject ambiguous/incomplete input without logging sensitive values.

**Completion notes:** `WorkloadIdentityConfig` accepts both aliases, rejects disagreement/partial input, redacts assertion debug output, and has deterministic tests for canonical/task contracts and resource-specific form construction.

### [!] 1.3 Document the one-registration permission and service-connection matrix

**Work:**

- Record the single app registration/client ID used for Graph, ARM, and Azure DevOps.
- Separate delegated Graph permissions for local PIM from application permissions required by non-PIM pipeline audit reads.
- Record ARM RBAC scope, Azure DevOps access/licensing, and federated credential issuer/subject/audience.
- Identify required Terraform changes in the external Cloud Terrastodon PIM configuration without editing that separate project as part of plan creation.

**Validation:**

    Review the app-registration/service-connection matrix with a real tenant and pipeline owner.
    Verify the least-privilege token audiences required by both audit commands.

**Completion criteria:** The matrix distinguishes delegated PIM from app-only audit access and contains no client-secret requirement.

**Blocker:** The exact Graph/ARM/Azure DevOps permission and federated-credential matrix requires review against a real tenant and service connection. This local-only goal deliberately does not access those systems. Unblock by reviewing the matrix with the pipeline owner before live validation.

### [x] 1.4 Define token type, audience, and authorization-header model

**Work:**

- Represent Graph, ARM, and Azure DevOps access tokens as resource-specific credentials.
- Send Entra access tokens as Bearer credentials.
- Retain Basic only for a deliberately selected PAT compatibility source.
- Mark assertions/tokens sensitive and exclude them from logs/cache fingerprints.

**Validation:**

    cargo test -p cloud_terrastodon_credentials
    cargo test -p cloud_terrastodon_rest

**Completion criteria:** Tests fail if a WIF/Entra token uses the PAT Basic-header path or appears in diagnostics.

**Completion notes:** Added `AzureBearerToken`, resource-specific WIF scopes, Bearer header handling for Entra tokens, Basic handling only for PAT values, and deterministic header/redaction tests.

### Phase 2 — Implement shared credential acquisition [x]

### [x] 2.1 Implement the WIF client-assertion exchange

**Work:**

- Exchange the Azure DevOps OIDC assertion at the tenant token endpoint using the client-credentials/federated-assertion flow.
- Request one resource audience at a time for ARM, Graph, or Azure DevOps.
- Parse expiry, return actionable Entra errors, and never log assertion/access-token values.

**Validation:**

    cargo test -p cloud_terrastodon_credentials

Add a deterministic token-endpoint mock boundary rather than depending on a live tenant for unit tests.

**Completion criteria:** A focused test proves normalized WIF input produces correct form fields, audience, expiry, and redacted errors.

**Completion notes:** Implemented the injectable `exchange_workload_identity_assertion` boundary and form/response parsing. Local tests cover form fields and redaction without contacting Entra; live endpoint behavior remains deferred.

### [x] 2.2 Implement source selection, caching, and refresh

**Work:**

- Implement explicit source selection and auto precedence from gate 1.1.
- Cache by tenant/resource/source and refresh before expiry.
- Use browser PKCE/refresh-token behavior for local delegated PIM and normal
  Graph, ARM, and Azure DevOps REST requests; keep PIM's additional Graph
  scopes explicit.
- Prevent auto from falling into interactive login when WIF is present or process is headless.

**Validation:**

    cargo test -p cloud_terrastodon_credentials
    cargo test -p cloud_terrastodon_command

**Completion criteria:** Repeated requests reuse valid tokens, expired tokens refresh, and headless WIF failures never attempt device/browser/CLI login.

**Completion notes:** Added invocation-owned source selection, WIF token caching keyed by tenant/resource with a two-minute refresh buffer, browser PKCE/refresh-token caching for normal delegated resources, headless refusal, and deterministic expiry-boundary tests. The cache lock serializes refreshes for a tenant/resource key; live concurrent pipeline behavior remains part of phase 6.2.

### [x] 2.3 Replace fetch_azure_access_token’s unconditional CLI dependency

**Work:**

- Route ARM and non-PIM Graph token requests through the shared provider.
- Preserve an explicitly selected Azure CLI source for local compatibility.
- Keep tenant/resource abstraction without assuming every token comes from az account get-access-token.

**Validation:**

    cargo test -p cloud_terrastodon_credentials
    cargo test -p cloud_terrastodon_rest
    cargo test -p cloud_terrastodon_azure

**Completion criteria:** execute_azure_bearer_request can acquire ARM/Graph tokens through WIF without spawning Azure CLI, while explicit local CLI fallback remains testable.

**Completion notes:** REST bearer requests now use `fetch_azure_bearer_access_token`; WIF never invokes Azure CLI, while `--auth-source azure-cli` retains local compatibility.

### Phase 3 — Route REST requests through the provider [x]

### [x] 3.1 Migrate Azure DevOps REST authentication off the PAT-only path

**Work:**

- Change execute_azure_devops_request to use the shared provider for Entra/WIF.
- Keep AZDO_PERSONAL_ACCESS_TOKEN and Windows Credential Manager only as explicit PAT compatibility sources.
- Refactor the REST client and types so Entra Bearer and PAT Basic credentials cannot be confused.
- Update fetch_azure_devops_groups_for_member to use the shared provider.

**Validation:**

    cargo test -p cloud_terrastodon_credentials
    cargo test -p cloud_terrastodon_rest
    cargo test -p cloud_terrastodon_azure_devops

**Completion criteria:** WIF-selected Azure DevOps REST requests do not read PAT storage and send Bearer tokens; explicit PAT mode remains isolated and tested.

**Completion notes:** Azure DevOps REST dispatch selects WIF/Entra Bearer credentials when WIF is present, keeps PAT Basic auth under explicit/interactive compatibility behavior, and routes the existing member-group REST request through `RestRequest`.

### [x] 3.2 Make ct rest the observable shared-auth smoke surface

**Work:**

- Ensure service inference and tenant behavior use the shared provider for Graph, ARM, and Azure DevOps.
- Keep explicit bearer-token behavior while applying correct resource/header semantics for provider-acquired tokens.
- Add redacted source/audience diagnostics useful for pipeline failures.

**Validation:**

    cargo test -p cloud_terrastodon_rest
    cargo test -p cloud_terrastodon_entrypoint

**Completion criteria:** Focused ct rest tests cover all three services and prove selected auth source without exposing token material.

**Completion notes:** Service inference and bearer dispatch compile through the shared provider. Azure DevOps source selection has a live-free precedence harness, and resource-specific provider tests cover Graph, ARM, and Azure DevOps audiences. Network request execution remains intentionally uncalled locally.

### [x] 3.3 Gate command retry/reauthentication by auth source

**Work:**

- Update crates/command/src/command.rs so WIF/headless failures do not invoke az account list or az login.
- Route refresh to the shared provider where possible.
- Preserve local interactive Azure CLI reauthentication only for explicit/compatible CLI source.

**Validation:**

    cargo test -p cloud_terrastodon_command

**Completion criteria:** Simulated WIF auth failure returns a provider error and never launches an interactive process.

**Completion notes:** Command retry now rejects interactive reauthentication for explicit WIF, browser/delegated, PAT, any WIF input, CI/build variables, or a non-terminal stdin. Local deterministic tests cover WIF/headless denial and explicit local CLI allowance.

### Phase 4 — Migrate the audit dependency slice [~]

### [~] 4.1 Validate ct audit azure through WIF-backed REST

**Work:**

- Exercise the existing typed ARM/Graph request graph used by audit_azure.
- Confirm least-privilege application access needed for non-PIM audit reads.
- Keep PIM activation and delegated user operations out of this path.

**Validation:**

    cargo test -p cloud_terrastodon_azure

Run a Linux pipeline smoke command with the real WIF service connection when the external prerequisite is available.

**Completion criteria:** ct audit azure completes headlessly without Azure CLI, device code, PAT, or signed-in user.

**Completion notes:** The audit's typed ARM/Graph dependency graph now flows through the entrypoint-owned context and shared REST provider, including Graph pagination and Resource Graph batches. End-to-end completion requires the deferred external WIF pipeline run.

### [x] 4.2 Replace the five direct Azure DevOps CLI-backed audit request families

**Work:**

Migrate these request families to typed RestRequest/Azure DevOps REST implementations:

- crates/azure_devops/src/azure_devops_user_license_entitlements.rs
- crates/azure_devops/src/azure_devops_projects.rs
- crates/azure_devops/src/azure_devops_test_plans.rs
- crates/azure_devops/src/azure_devops_test_suites.rs
- crates/azure_devops/src/azure_devops_group.rs for project groups

Preserve pagination, route parameters, response typing, caching, and current
audit output semantics. Do not migrate unrelated dump or interactive command
families in this work item.

**Validation:**

    cargo test -p cloud_terrastodon_azure_devops

Add focused response fixtures for each API shape and run existing tests that do
not require a live tenant.

**Completion criteria:** These five audit dependencies contain no direct
CommandKind::AzureCLI invocation and retain typed outputs/cache behavior.

**Completion notes:** Migrated user license entitlements, projects, test plans, test suites, and project groups to typed `RestRequest` implementations with encoded URLs, pagination fields, and per-page cache entries beneath the command's base key. Invalidating the base key invalidates continuation pages too. The five production modules contain no `CommandKind::AzureCLI` reference.

### [~] 4.3 Validate ct audit azure-devops end to end

**Work:**

- Run the audit with explicit organization URL and tenant rather than relying on a local Azure CLI default account.
- Confirm Entra user lookup and Azure DevOps requests use the shared provider.
- Verify concurrent requests reuse tokens safely and do not reintroduce PAT/CLI access.

**Validation:**

    cargo test -p cloud_terrastodon_entrypoint
    cargo test -p cloud_terrastodon_azure_devops

Run the real Linux Azure DevOps pipeline smoke test when service connection and organization permissions are available.

**Completion criteria:** ct audit azure-devops completes headlessly with WIF and no Azure CLI, device-code, or PAT dependency.

**Completion notes:** The audit request graph is CLI-free for the five migrated families and passes one entrypoint-owned context through Graph lookup, Azure DevOps pagination, and parallel work. Shared page parsing has an offline response fixture covering JSON envelopes and continuation headers; the external pipeline run remains outstanding.

### Phase 5 — Global preference and remaining CLI classification [x]

### [x] 5.1 Add and document the global --auth-source option

**Work:**

- Add the option to GlobalArgs and initialize shared authentication context at entrypoint.
- Document auto, WIF, browser/delegated, and Azure CLI behavior.
- Keep --debug as logging/backtrace configuration only.

**Validation:**

    cargo test -p cloud_terrastodon_entrypoint
    cargo test -p cloud_terrastodon_credentials

**Completion criteria:** Help output, parser tests, and focused runtime tests agree on the public authentication contract.

**Completion notes:** `cargo run -- --help` exposes `--auth-source <SOURCE>` with default `auto`; parser/source unit tests pass. `--debug` remains logging-only.

### [x] 5.2 Classify remaining direct Azure CLI uses

**Work:**

- Re-run the inventory after audit migration.
- Classify each remaining production use as migrated, explicit local fallback, no suitable REST equivalent yet, or unsupported in headless mode.
- Prioritize az ad operations for Graph migration and az rest operations for direct REST migration.
- Treat dump_azure_devops, dump_everything, resource-name checks, subscription/account discovery, and interactive login as follow-on slices unless required by a concrete pipeline command.

**Validation:**

    rg -n -C 2 'CommandKind::AzureCLI' crates/azure/src crates/azure_devops/src crates/credentials/src crates/entrypoint/src

**Completion criteria:** No remaining use is silently assumed to be pipeline-safe; every use has a support status and follow-up boundary.

**Completion notes:** After the five audit migrations, the inventory contains 32
production files and 41 `CommandKind::AzureCLI` references. The remaining uses
are classified below; this is a status inventory, not a promise that all
follow-on work belongs in the first milestone.

| Status | Remaining production uses | Boundary |
| --- | --- | --- |
| Shared compatibility plumbing | `crates/credentials/src/azure_access_token.rs`; `crates/command/src/command.rs`; `crates/command/src/command_kind.rs` | Explicit Azure CLI token acquisition, executable configuration, debug argument handling, and local interactive reauthentication. WIF paths do not use this provider. |
| Explicit local/interactive compatibility | `crates/azure/src/auth.rs`; `crates/azure/src/accounts.rs`; `crates/azure/src/subscriptions.rs`; `crates/azure_devops/src/default_organization.rs`; `crates/azure_devops/src/azure_devops_configure.rs`; `crates/entrypoint/src/cli/command/azure/tenant/azure_tenant_login.rs`; `crates/entrypoint/src/interactive/copy_azurerm_backend_menu.rs` | User login, account/subscription discovery, CLI configuration, tenant login, and interactive backend convenience. Not pipeline-safe without an explicit local CLI source. |
| REST migration candidates, follow-on | `crates/azure/src/compute_sku_prices.rs`; `crates/azure/src/container_registry.rs`; `crates/azure/src/create_role_assignment.rs`; `crates/azure/src/entra_group_member_add.rs`; `crates/azure/src/entra_group_member_remove.rs`; `crates/azure/src/key_vault_secrets.rs`; `crates/azure/src/key_vaults.rs`; `crates/azure/src/remediate_policy_assignment.rs`; `crates/azure/src/storage_account_blob_container_names.rs`; `crates/azure/src/storage_account_name_availability.rs`; `crates/azure_devops/src/azure_devops_agent_packages.rs`; `crates/azure_devops/src/azure_devops_agent_pool_entitlements_for_project.rs`; `crates/azure_devops/src/azure_devops_agent_pools.rs`; `crates/azure_devops/src/azure_devops_group_license_entitlements.rs`; `crates/azure_devops/src/azure_devops_group_member.rs`; `crates/azure_devops/src/azure_devops_repos.rs`; `crates/azure_devops/src/azure_devops_service_endpoint.rs`; `crates/azure_devops/src/azure_devops_team.rs`; `crates/azure_devops/src/azure_devops_team_member.rs`; `crates/azure_devops/src/azure_devops_user_license_entitlement_update.rs`; `crates/azure_devops/src/azure_devops_work_item_queries.rs` | API-backed resource, membership, mutation, dump, and discovery operations. They remain outside the audit slice until typed REST routes and response fixtures are added. |
| Development-only tooling | `crates/azure_resource_types/src/resource_types_generator.rs` | Build-time resource metadata generation; not a runtime pipeline command. |

The five migrated audit modules (`user_license_entitlements`, `projects`,
`test_plans`, `test_suites`, and `group`) no longer contain a direct Azure CLI
invocation. The inventory should be rerun whenever another migration slice is
started.

### [x] 5.3 Document local fallback and headless failure policy

**Work:**

- Explain when Azure CLI is allowed locally and how users select it explicitly.
- Explain that WIF represents the service connection identity, not a user.
- Explain that PIM remains interactive/delegated-only.
- Document environment aliases without examples containing real tokens.

**Validation:**

    rg -n 'auth-source|workload identity|PIM|device|AZURE_CLIENT_ID|servicePrincipalId' README.md docs crates

**Completion criteria:** Documentation matches implemented precedence, support targets, and non-goals.

**Completion notes:** Added [`docs/authentication.md`](../authentication.md)
with the `--auth-source` values, both WIF environment contracts, resource and
identity boundaries, local CLI/PAT compatibility, and the interactive-only PIM
policy. It contains no live credentials or tenant-specific examples.

### Phase 6 — Acceptance, propagation, and handoff [~]

### [~] 6.1 Run focused and repository validation

**Work:**

- Run focused tests for credentials, REST, Azure, Azure DevOps, command, and entrypoint.
- Run the existing check script and record environment-specific limitations.
- Run the full workspace test suite if external integration tests are available; otherwise record exact skips.

**Validation:**

    cargo test -p cloud_terrastodon_credentials
    cargo test -p cloud_terrastodon_rest
    cargo test -p cloud_terrastodon_azure
    cargo test -p cloud_terrastodon_azure_devops
    cargo test -p cloud_terrastodon_command
    cargo test -p cloud_terrastodon_entrypoint
    cargo check --all --tests --examples --workspace
    pwsh ./check-all.ps1

**Completion criteria:** Focused tests and check-all.ps1 pass on the current source tree, or unavailable external tests are explicitly recorded with prerequisites.

**Completion notes:** `cargo check --workspace --all-targets`, `cargo build --workspace`, stable `cargo fmt --all -- --check`, and `cargo clippy --all-targets --all-features -- -D warnings` pass on Linux. Focused offline suites pass for credentials/browser OAuth, REST, command retry gating, entrypoint project-list auth preflight, entrypoint CLI schema, and Azure DevOps URL/pagination construction; the Azure DevOps test binary also compiles with `--no-run`. Its default suite contains legacy live tests, so it is not used as local validation. Live browser callback, tenant, organization, and WIF pipeline validation remain external and were not run.

### [!] 6.2 Run the Linux WIF pipeline acceptance matrix

**Work:**

- Run ct audit azure with the ARM-capable service connection.
- Run ct audit azure-devops with the Azure DevOps-capable service connection/organization.
- Confirm no az process is required by Cloud Terrastodon, no interactive prompt occurs, and no PAT is present.
- Capture only redacted logs and status evidence.

**Validation:**

    Azure DevOps Linux agent with WIF service connection and the two audit commands.
    Inspect process/log output without printing OIDC assertions or access tokens.

**Completion criteria:** Both audits complete under the supported pipeline contract with one app registration and least-privilege permissions.

**Blocker:** This requires a real Azure DevOps WIF service connection, tenant permissions, organization access/licensing, and a Linux agent. The local-only goal forbids those external interactions. Unblock only with a separately authorized live validation step.

### [x] 6.3 Update this plan and decide on public issue publication

**Work:**

- Record evidence beside each work item; do not append a detached work log.
- Reconcile CLI inventory and acceptance matrix.
- Prepare a concise public issue only after the local plan and first milestone are reviewed.

**Validation:**

    git diff --check
    git status --short

**Completion criteria:** The plan is resumable by a fresh agent, all active user guidance has evidence, and issue publication is a separate explicit decision.

**Completion notes:** This plan remains the local source of truth; no issue was created and no commit was made.

## Acceptance matrix

| Target | Support status | Required validation | Evidence |
| --- | --- | --- | --- |
| Linux local, delegated PIM | Supported | Browser PKCE/refresh-token focused tests; device code remains opt-in | Pending |
| Linux local, explicit Azure CLI source | Supported compatibility path | Existing command/auth tests and documented fallback | Pending |
| Linux Azure DevOps pipeline, ct audit azure | First milestone target | Real WIF pipeline smoke test with ARM/Graph permissions | Pending |
| Linux Azure DevOps pipeline, ct audit azure-devops | First milestone target | Real WIF pipeline smoke test with Azure DevOps permissions | Pending |
| Windows local | Compatibility target | Workspace compile/check and Windows credential fallback tests | Pending |
| Windows Azure DevOps pipeline | Intended follow-on target | Focused WIF tests plus live agent validation when available | Pending |
| Any pipeline PIM activation | Explicitly unsupported | Documented rejection/non-goal; no application-permission implementation | Pending |

## Overall completion criteria

- [x] Shared provider supports both environment contracts and resource-specific WIF exchange.
- [x] --auth-source is documented; --debug remains unrelated to authentication.
- [x] auto never triggers interactive/device-code/Azure CLI login in detected headless WIF context.
- [x] Graph, ARM, and Azure DevOps token/header semantics are distinct and tested.
- [!] ct audit azure runs in a Linux WIF pipeline without user or Azure CLI dependency — requires the external tenant/service-connection acceptance run.
- [!] ct audit azure-devops runs after the five direct CLI request families are migrated — code migration is complete; live execution remains external.
- [x] PIM remains delegated and interactive-only; pipeline PIM is unsupported.
- [x] Remaining direct CLI uses are classified rather than silently treated as pipeline-safe.
- [x] Focused validation and repository checks are recorded; live pipeline evidence is explicitly deferred.
- [x] Documentation and support limits match implementation; tenant-specific permission requirements remain an external gate.
- [x] No public issue is created without a separate explicit decision.

## Risk register

| Risk | Guardrail/mitigation | Validation |
| --- | --- | --- |
| Entra token is sent as a PAT Basic credential. | Separate credential/token types and authorization schemes. | Header-selection tests and Azure DevOps REST smoke test. |
| WIF assertion or access token leaks through logs, cache fingerprints, command summaries, or failure files. | Mark secrets sensitive, redact diagnostics, exclude values from fingerprints. | Redaction tests and artifact inspection. |
| auto unexpectedly launches device/browser/Azure CLI login in CI. | Detect headless WIF, define precedence, gate retry login by source. | Simulated auth failures and live process inspection. |
| One app registration receives excessive permissions. | Least-privilege matrix separating delegated PIM and app-only audit access. | Tenant/service-connection review and limited smoke test. |
| Audit migrations diverge from Azure DevOps CLI response semantics. | Preserve typed outputs, route parameters, pagination, caching, and fixtures. | Per-request fixtures and audit smoke run. |
| Broad CLI inventory causes scope expansion. | Treat 37 non-test/example files as classification inventory; audit is only first milestone. | Inventory reconciliation in phase 5.2. |
| Token expiry or concurrent refresh causes intermittent failures. | Cache by audience/source, refresh before expiry, serialize refresh by credential key. | Expiry/concurrency tests and multi-request audit run. |
| External tenant, service connection, or licensing prerequisites are unavailable. | Deterministic provider tests and explicit live-test prerequisites/skips. | Focused tests plus recorded pipeline evidence. |
| Global auth context leaks between tests or commands. | Immutable per-process context and injectable test seams. | Parallel test isolation and repeated invocation tests. |
