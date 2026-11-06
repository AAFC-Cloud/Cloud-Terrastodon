# Executor/cache migration caller inventory

**Inventory date:** 2026-10-30

**Boundary decision update:** 2026-11-05; approved option B and `auth`/`auth_types`
naming. Task 3.0's registry cutover and task 3.1's auth rename/extraction are
implemented. Task 3.2's lower executor, injected provider, and minimal app binding
are implemented and verified offline in the owning plan. The snapshot
counts remain the original baseline.

**Owning work item:** [Migration plan, task 1.2](executor-cache-migration.md)

**Status:** Source inventory complete; implementation and runtime validation are
not performed by this inventory. The owning plan records baseline checks and
contract decisions separately.

**Scope:** Repository source, manifests, and instructions only. No production
services, credentials, caches, logs, generated production artifacts, or user
configuration were read. No build, test, or request was executed for this inventory.
Repository-relative paths and source-derived counts below contain no local account
or machine information. Only the root [AGENTS.md](../../AGENTS.md) was found.

## How to use and refresh this inventory

This supplements the plan rather than replacing its requirements ledger or gates.
Each batch below identifies its owning plan tasks and its exact responsibility.
Before implementing a batch, rerun the searches, inspect new hits, and record its
completion in the owning task. Add newly discovered surfaces to the appropriate
batch; do not treat a missing search hit as an exemption. One implementation owner
should take a batch at a time unless the main plan explicitly assigns independent
work. Batch IDs are stable and are not claims that the batch has been implemented.

The searches intentionally inventory syntax, not Rust semantics. Inline test
modules, declarations, documentation examples, macro definitions, and parameter
types can match. Consequently, counts are **candidate files**, not request counts
or counts of production structs. Dedicated `tests` directories and files ending
in `tests.rs` are excluded from the snapshot; their compilation and relevant
offline integration coverage still belong to the owning implementation batch.

## Reproducible source snapshot

Task 4.1b's Command cutover is narrowed by U87/U88 and task 4.1c: local process
and cancellation-only helpers take scopes directly; artifact/cache-managed
operations take explicit executors. `CommandBuilder` no longer appears in `azure/src`; the
DevOps SDK uses it only for local CLI configuration. Auth-owned account metadata
and logout processes bypass semantic caching and never start implicit login.
Terraform/HCL, cancellation-only menu/workflow code, and egui use scopes directly.
Dispatch retains the application executor for genuine managed requests. Gitea
selects an explicit matching tea profile and uses managed owned results because
the external tool exposes no credential generation. Fresh local configuration
helpers use scopes; registered cached configuration requests retain the executor.

The formerly ambient Azure/DevOps operations listed in this baseline now use
explicit-context REST: current user, group membership, role assignments, resource
name availability, retail pricing, Key Vault metadata, registry repositories/tags,
DevOps agent packages/pools, group entitlements/membership, repositories, teams,
service endpoints, queries, and license updates. ACR endpoint discovery uses the
actual login server; credential exchanges do not enter the raw artifact store.
Core DevOps routes preserve Server collection identity. New response records live
in the domain types crates. Command service authentication is no longer inferred
from an ambient CLI session.

Task 4.2 now routes REST through the explicit executor and modern cache owner.
The transport retains nonsecret authentication selection rather than runtime
credentials or cancellation. Cached receives use canonical operation owners and
decoded `Arc` values; owned receives execute independently. SDK and consumer
callers use bound execution and modern invalidation contracts. The old Command
global cache, `CacheableCommand`, and its request macros are removed.
Some SDK request objects still retain authentication contexts pending phase 5's
request-data refinement. Task 4.2b now carries cached response ownership through
SDK consumers: singleton reads return `Arc<T>`, single-page collections can
return `Arc<Vec<T>>`, and paginated/composed inventories retain records as
`Vec<Arc<T>>`. It removes deep-copy adapters and response `Clone` derives/bounds
used solely for those copies. Owned ID/name/path projections remain appropriate
when they build a distinct domain value; consumers borrow or retain cached
handles for ordinary selection and presentation. Task 4.2b records 46 passing
offline tests and successful workspace all-target check/Clippy for this refinement.
Sharing is not proof of aggregate memoization or
dependency freshness. Existing uncached Work Item
Tracking reads retain that policy. Unknown-ID `PrincipalRequest` batches share
the typed envelope with opaque success bodies, then decode the ID-selected
principal type on each invocation; that remaining concrete decode belongs to
phase 5's canonical principal result.
The new local configuration files are `azure_devops_cli_config_request.rs`,
`default_azure_devops_organization_url_request.rs`, and
`default_azure_devops_project_name_request.rs`.

Source searches and compile/test evidence for the cutover are recorded in the
owning plan. No production service, credential, default configuration, or existing
cache was consulted for validation. The table below retains historical counts.
Run from repository root in PowerShell. These commands inspect tracked workspace
source and do not invoke any application or inspect its cache/configuration.

```powershell
$inventoryPatterns = [ordered]@{
    Request = 'CacheableCommand for|CacheableWorkRequest for|impl_cacheable_into_future!|impl_cacheable_work_into_future!|impl.*IntoFuture|register_into_future!'
    Runtime = '^\s*(?:pub(?:\([^)]*\))?\s+)?(?:cancellation|auth_context|auth):[^\r\n]*(?:CancellationScope|AuthContext)'
    Adapter = 'RestRequest::|CommandBuilder::'
    Invalidation = '\.invalidate\(|with_invalidation\(|invalidate_cache|CacheInvalidatable'
    Constructor = '^pub (async )?fn (fetch_|get_default_|resolve_|pick_)'
}
foreach ($entry in $inventoryPatterns.GetEnumerator()) {
    $matchedFiles = @(rg -l $entry.Value crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs')
    [pscustomobject]@{
        Metric = $entry.Key
        Files = $matchedFiles.Count
        Groups = @($matchedFiles |
            Group-Object { ($_ -split '[\\/]')[1] } |
            Sort-Object Name |
            ForEach-Object {
                [pscustomobject]@{ PackageDirectory = $_.Name; Files = $_.Count }
            })
    } | ConvertTo-Json -Depth 4 -Compress
}
```

`Runtime` includes typed function parameters as well as fields. `Request` includes
explicit and macro-generated execution surfaces. `Constructor` is a naming-based
backstop for plain functions missed by request-registration searches. Directory
names in this table are not necessarily Cargo package names.

| Package directory | Request | Runtime | Adapter | Invalidation | Constructor |
| --- | ---: | ---: | ---: | ---: | ---: |
| app | 0 | 5 | 0 | 0 | 0 |
| azure | 79 | 102 | 46 | 7 | 80 |
| azure_cli | 0 | 152 | 1 | 10 | 1 |
| azure_devops | 50 | 55 | 43 | 12 | 28 |
| azure_devops_cli | 0 | 46 | 1 | 7 | 0 |
| azure_resource_types | 0 | 0 | 1 | 0 | 0 |
| command | 2 | 2 | 2 | 4 | 1 |
| credentials | 0 | 6 | 1 | 0 | 2 |
| entrypoint | 0 | 65 | 11 | 4 | 0 |
| excalidraw | 1 | 1 | 0 | 0 | 0 |
| gitea | 16 | 18 | 2 | 1 | 14 |
| group-management example | 0 | 4 | 0 | 1 | 0 |
| hcl | 1 | 7 | 5 | 0 | 0 |
| hcl_types | 0 | 1 | 1 | 0 | 0 |
| registry | 2 | 10 | 0 | 0 | 0 |
| rest | 1 | 2 | 1 | 1 | 0 |
| source | 1 | 3 | 0 | 0 | 0 |
| source_cli | 0 | 1 | 0 | 0 | 0 |
| ui_egui | 0 | 3 | 0 | 0 | 0 |
| ui_ratatui | 5 | 11 | 0 | 0 | 0 |
| **Total files matched per metric** | **158** | **494** | **115** | **47** | **126** |

The standalone example has no match in these particular patterns because it uses
`AppContext` and fetch helpers directly; it is nevertheless explicitly in B19.
At this inventory's baseline, the single `azure_resource_types` adapter hit was
an ignored, test-only source generator. Task 2.7 extracts it into standalone
tooling; these baseline counts are not current dependency guarantees. Zeros do
not establish absence of consumers.

To enumerate a metric's files rather than its counts, use the same glob policy:

```powershell
rg -l 'CacheableCommand for|CacheableWorkRequest for|impl_cacheable_into_future!|impl_cacheable_work_into_future!|impl.*IntoFuture|register_into_future!' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'pub fn (receive|run)|pub async fn (receive|run)' crates/command/src/command.rs crates/rest/src/rest_request.rs
rg -n 'receive_raw|receive_with_mapper|receive_with_validator|run_raw|run_with_mapper|run_with_validator|run_polite' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'get_cached_output|write_output|write_failure|put_memory_cache_entry|bust_cache|invalidate_cache|discover_caches|with_invalidation' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'invoke_value_boxed|invoke_result_future|bind_request_cancellation|REGISTERED_INTO_FUTURE' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
```

## Shared execution, auth, and invalidation boundaries

Task 3.2 adds the [executor contract](../../crates/execution/src/cloud_terrastodon_executor.rs)
and [request contract](../../crates/execution/src/executable_request.rs) beneath
the runtime credential implementation. B04 owns its provider adapter and B19 owns
application binding. Task 3.3 implements B05's borrowed invocation ABI and the
direct UI consumers, with validation recorded in the owning plan. B02/B03's
production transport lifecycle belongs to phase 4. B02's Command cutover is
complete in 4.1b/4.1c; B03's REST cutover and caller propagation are complete
in 4.2, with workspace all-target check/Clippy, focused offline tests, and final
caller review passed. Legacy registry IntoFuture callbacks remain separately tagged for
remaining local/request families; their presence does not imply a legacy REST
cache path.

| Batch | Owning tasks | Source and responsibility | Completion obligation |
| --- | --- | --- | --- |
| B01 Cache owner and artifact storage | 2.1–2.4, 4.2 | [work_cache.rs](../../crates/cache/src/work_cache.rs), [cache_key.rs](../../crates/cache/src/cache_key.rs), [work_cache_invalidation.rs](../../crates/cache/src/work_cache_invalidation.rs), [work_cache_inventory.rs](../../crates/cache/src/work_cache_inventory.rs) | Explicit cache ownership replaces the removed Command global cache. Modern inventory reads addresses and metadata under the artifact gate, without loading response payloads or credential inputs; list/clean consumers invalidate through that same owner. Preserve inspectable artifacts, validity, readable scopes, and in-flight coordination. |
| B02 Shared work and subprocess adapter | 2.1, 2.3, 4.1, 4.2 | [cached_work_spec.rs](../../crates/cache/src/cached_work_spec.rs), [command.rs](../../crates/command/src/command.rs), [process_fingerprint.rs](../../crates/command/src/process_fingerprint.rs), [from_command_output.rs](../../crates/command/src/from_command_output.rs) | Tasks 4.1b/4.1c use explicit managed entrypoints for artifacts and cached results, with scope-only local process methods beneath them. Preserve fingerprints, retries, raw outputs, per-caller policy, caller/span diagnostics, and cancellation. Task 4.2 removes the residual Command cacheable-request traits/macros after migrating their REST/SDK consumers. |
| B03 REST adapter | 4.2 | [rest_request.rs](../../crates/rest/src/rest_request.rs), [request_execution.rs](../../crates/rest/src/request_execution.rs), [container_registry_request_execution.rs](../../crates/rest/src/container_registry_request_execution.rs), [tenant_inference.rs](../../crates/rest/src/tenant_inference.rs) | Implemented explicit executor and authentication-selection boundaries, independent owned receives, canonical shared decoding, per-caller policy, metadata/continuation mapping, and provider-owned ACR authorization. Final local regression evidence belongs to task 4.2; no live compatibility claim. |
| B04 Credential selection and runtime | 3.1, 3.2 | [auth_source.rs](../../crates/auth_types/src/auth_source.rs), [auth_context.rs](../../crates/auth/src/auth_context.rs), [credential_authentication_provider.rs](../../crates/auth/src/credential_authentication_provider.rs), [azure_tenant_auth_context.rs](../../crates/auth/src/azure_tenant_auth_context.rs), [azure_devops_auth_context.rs](../../crates/auth/src/azure_devops_auth_context.rs), [azure_access_token.rs](../../crates/auth/src/azure_access_token.rs), [tracked_tenants.rs](../../crates/azure/src/tracked_tenants.rs) | Completed 3.1 renames runtime `credentials` to `auth` and moves selections to `auth_types` without compatibility aliases; `AzureTenantId` stays in `azure_types`. Task 3.2 adapts the owned auth context to the lower `AuthenticationProvider` interface, preserving source/tenant admission, session identity, sensitive token caches, and PAT audience policy. Credential acquisition is infrastructure, not an inventory dependency or general-cache result. |
| B05 Registry invocation and discovery | 3.0, 3.3, 6.3, 7.1 | [pure registry lib](../../crates/registry/src/lib.rs), [invocation registry lib](../../crates/registry_execution/src/lib.rs), [function.rs](../../crates/registry_execution/src/function.rs), [bind_request_cancellation.rs](../../crates/registry_execution/src/bind_request_cancellation.rs), [default_production.rs](../../crates/registry_execution/src/default_production.rs), [function_trait_contract.rs](../../crates/registry/src/function_trait_contract.rs), [discovery_reflection.rs](../../crates/registry_execution/src/discovery_reflection.rs) | Completed 3.0 makes `registry` the pure foundation, absorbing `registry_values` and pure conversions/metadata/planning; runtime invocation lives in `registry_execution` with no old package/import compatibility surfaces. One authoritative registration per operation supplies metadata and runtime catalogue views. Task 3.3 adds the borrowed executable callback, direct registration handle, executor-bound default production, and offline shared-output discovery without adding an upward dependency from the foundation. The separately tagged legacy callback is removed with its remaining request families in phases 5–7. Task 7.1 audits residual old references rather than postponing this cutover. |

Task 4.1c corrects the overly broad 4.1b dependency: `run_raw`, `run_raw_polite`,
`run`, `run_polite`, `run_with_validator`, `run_with_mapper`, and
`run_with_validator_from` take cancellation scopes. Explicit artifact variants
retain executor admission; cached variants require a canonical owner and return
shared values. The former public Command cache
methods `get_cached_output`, `write_output`, `write_failure`, and `bust_cache`
are removed. [oauth2_permission_grants.rs](../../crates/azure/src/oauth2_permission_grants.rs)
uses its read-request invalidation contract. Internal retries stay within the
admitted uncached process attempt and never reacquire its key lock.

REST methods requiring coverage are `receive_raw_with_decoder`, `receive_raw`,
`receive`, `receive_with_validator`, `receive_with_mapper`, and
`receive_with_ms_continuation_token`, plus `IntoFuture` and
`CacheInvalidatableIntoFuture`. A raw/typed method switch on the same request is
not permission to replace its canonical cache owner silently.

`CacheInvalidatable` currently derives behavior from `HasCacheKey`; macros call
`invalidate()` without an explicit owner. Custom invalidators also exist on
composed inventories, picker requests, project-member discovery, and legacy
default-resolution requests. Migrate their semantic invalidation scopes, not just
the trait signatures. The cache-clean UI directly discovers disk keys and calls
`key.invalidate()`; it must address the same owner used by execution.

Credential runtime helpers also include `auth_bearer_ext.rs`, browser/WIF token
providers, PAT retrieval, and PIM configuration/token helpers. They remain runtime
resources and are not themselves serialized into ordinary SDK requests. Preserve
their existing protocol behavior. Do not require Azure tenants for local-only or
Gitea work merely because those operations acquire an executor parameter.

### Approved crate-boundary cutover inventory

The following source paths are **historical locations at the boundary decision**.
Tasks 3.0 and 3.1 have implemented the registry and auth destinations. Pure and
runtime registrations have different owning crates, so future
callers must choose their imports by responsibility.

| Current source | Approved target | Required classification |
| --- | --- | --- |
| `crates/registry_values` | `crates/registry`, package `cloud_terrastodon_registry` | Reflection, runtime value representation, registration sites, offline arbitrary constructors, and their invariants stay below execution. Remove the old `registry_values` package and compatibility exports after consumer migration. |
| `crates/registry` | Split between the pure `registry` and `crates/registry_execution`, package `cloud_terrastodon_registry_execution` | Keep context-free constructors, conversions, metadata, and planning in the foundation; place executor-dependent invocation and its adapters in `registry_execution`. Split mixed modules by responsibility, including `Function` metadata versus invocation. A function is not automatically an execution-layer operation. |
| `crates/credentials` | `crates/auth`, package `cloud_terrastodon_auth` | Preserve credential acquisition, browser/workload sessions, token caches, logout, and OS credential integration. The runtime implements the provider interface owned by `execution`; execution does not import this concrete implementation. |
| `crates/credentials/src/auth_source.rs` and new resolved selection vocabulary | `crates/auth_types`, package `cloud_terrastodon_auth_types` | Move the existing `AuthSource` preference enum with its parsing and wire behavior; keep `Auto` resolution in runtime startup. Resolved request selections remain explicit and token-free. |
| `crates/azure_types/src/tenant_id.rs` | Remains in `crates/azure_types` | Keep one `AzureTenantId` definition and registration. `auth_types` may depend on `azure_types`; `azure_types` must not acquire a reverse auth/runtime dependency. |

The `azure_types` registration surface includes `register_thing!`,
`register_arbitrary!`, and six consuming `From`/`TryFrom` registrations in
[entra_principal.rs](../../crates/azure_types/src/entra_principal.rs). The latter
are pure conversions and now use lower `PureFunction` registrations. Task 3.0
preserved their discoverability, ownership, failure behavior, and query coverage
while moving descriptor/invocation support down. Metadata consumers
must not need `registry_execution` solely to inspect registered types or pure
conversion relationships. Executor-dependent invocation and admission belong in
the higher layer. Existing discovery simulation remains offline and never invokes
service requests; context-free metadata and planning belong in the foundation.
Classification cannot be based only on a filename or an `async` signature.

Audit more than Rust imports. The root [Cargo.toml](../../Cargo.toml) now names
both registry packages and the `auth`/`auth_types` packages, with `auth` and
`auth-types` features. [src/lib.rs](../../src/lib.rs) exposes their owning facade
modules. [app's manifest](../../crates/app/Cargo.toml) retains its optional `auth`
feature and now depends on the renamed runtime package. The cutover audit covers
workspace dependency keys, member paths, package names, affected lockfile entries,
feature references, macro paths, facade exports, tests, examples, standalone tools,
source configuration types, CI/build scripts where present, and maintained
documentation. Existing configured auth-source spellings are preserved. U67/U68
separately authorize the identity and persistent-session changes described in
[G3](executor-cache-contracts.md); the package rename alone does not authorize
behavior changes. Historical completion evidence may retain an old name with its date.

Refresh this inventory with source-only searches before each boundary cutover:

```powershell
rg -n 'cloud_terrastodon_(registry_values|registry|registry_execution|credentials|auth_types|auth)|crates/(registry_values|credentials)|\bAuthSource\b|\bAzureTenantId\b' Cargo.toml Cargo.lock src crates examples tools docs .github -g '*.rs' -g '*.toml' -g '*.lock' -g '*.md' -g '*.ps1' -g '*.yml' -g '*.yaml' -g '!docs/plans/context rework.md'
rg -n 'register_from!|register_try_from!|register_arbitrary!|functions_from_to|invoke_value_boxed|bind_request_cancellation' crates -g '*.rs'
```

Record the final dependency graph in tasks 3.0/3.1, not just successful import
replacement. `registry` must have no dependency on `registry_execution`,
`execution`, or runtime `auth`; `azure_types` must stay on the pure foundation;
`execution` must not depend on concrete `auth`. Executable registration consumers
use `registry_execution` directly, with no old-name facade or compatibility
re-export that silently restores a removed dependency edge.

## SDK batches and ownership changes

Each batch covers its named SDK family, matching fetch constructors, registry
registrations, and direct consumers. For broad remaining families, split by the
listed subfamily before editing; do not convert an entire SDK in one unreviewed
patch. No batch authorizes a new endpoint, live inventory, or unrelated policy.

| Batch | Owning tasks | Bounded source family | Required migration behavior |
| --- | --- | --- | --- |
| B06 Graph principal vertical slice | 5.1 | [entra_user_list_request.rs](../../crates/azure/src/entra_user_list_request.rs), [security_groups.rs](../../crates/azure/src/security_groups.rs), [service_principals.rs](../../crates/azure/src/service_principals.rs), [principals.rs](../../crates/azure/src/principals.rs), [microsoft_graph.rs](../../crates/azure/src/microsoft_graph.rs) | Constructor data includes source/tenant; fetch receives executor; progress observation is bound separately; pages and aggregate have distinct owners; record all child dependencies. |
| B07 ARM RBAC vertical slice | 5.2 | [role_definitions.rs](../../crates/azure/src/role_definitions.rs), [role_assignments.rs](../../crates/azure/src/role_assignments.rs), [role_definitions_and_assignments.rs](../../crates/azure/src/role_definitions_and_assignments.rs), [resource_graph.rs](../../crates/azure/src/resource_graph.rs) | Preserve read-only POST semantics, rate limits, query/page identity, combined validation, and cross-branch dependency invalidation. |
| B08 Azure DevOps project vertical slice | 5.3 | [azure_devops_projects.rs](../../crates/azure_devops/src/azure_devops_projects.rs), project-list CLI, project picker and direct project-list consumers | Preserve explicit organization, explicit bearer tenant, PAT selection, continuation metadata, cache-hit behavior, and chosen shared-record output shape. Picker interaction itself is not memoized. |
| B09 Remaining Graph identity reads | 6.3, 7.1 | `entra_user_get/search`, `entra_group_get/list/members/members_batch/owners`, `entra_groups_for_member`, `entra_directory_objects_by_ids`, `principal_get`, `service_principal_get`, and `microsoft_graph_batch` under [Azure SDK](../../crates/azure/src) | Move runtime construction/fetching; preserve mixed known/unknown principal resolution, batching, absence semantics, and pagination. Revisit owned conversions only where the selected shared-record boundary requires it. |
| B10 Graph application/tenant/security reads | 6.3, 7.1 | Azure `application_registration_*`, `application_role_list`, `oauth2_permission_grant_list`, `oauth2_permission_scopes`, `conditional_access_*`, `tenant_details/licenses`, `service_groups`, `unified_role_*`, and `pim_entra_role_*` | Migrate each subfamily independently; preserve bearer audience/tenant, child dependencies, and protected policy boundaries. Unified role aggregates require the same cross-branch invalidation treatment as RBAC. |
| B11 ARM inventories | 4.1b, 6.3, 7.1 | Azure `subscriptions`, `resource_group_list`, `resources`, `management_groups`, [resource_provider_list_request](../../crates/azure/src/resource_provider_list_request.rs), `policy_*`, network/VM/storage/compute families, app-service/application-gateway/container/cognitive-services/key-vault inventories | Migrate one resource family at a time through its current Resource Graph or REST boundary. Preserve its identity filters/limits and output. Approved protocol replacements include task 3.1a's remediation/container ARM cutover (U71/U72) and task 4.1b's remaining service CLI replacements (U86). |
| B12 Azure direct helpers and mutations | 6.3, 7.1 | Azure `batch_api`, `cost_management`, `metrics`, `tags`, `storage_account_name_availability`, `virtual_machine_prices/sizes`, `create_role_assignment`, `governance_role_assignments`, `eligible_child_resources`, `role_eligibility_schedules`, `role_management_policy_assignments`, `evaluate_policy_assignment_compliance`, `remediate_policy_assignment`, `entra_group_member_add/remove`, OAuth grant create/update/remove, and PIM activation | Inventory plain functions and `.invoke()` helpers as well as request macros. Read-only batches opt in explicitly; writes execute every time and preserve required response/artifact/invalidation work. |
| B13 Azure DevOps builds and folders | 6.3, 7.1 | `azure_devops_build_*_request.rs` and corresponding [build CLI](../../crates/azure_devops_cli/src/build) | Preserve list/get filters and revision identity, preflight freshness, folder prune/delete deferrals, and visible affected-read invalidation. |
| B14 Azure DevOps work items | 6.3, 7.1 | `azure_devops_work_item*`, `work_item_copy*`, `rest_work_item_copy_writer.rs`, and corresponding work-item CLI | Migrate schema/query reads, mutations, copy planner, writer, and durable journal workflows in separate slices. Preserve effect declarations, journaling/required completion, and existing copy-plan decisions. |
| B15 Other Azure DevOps inventories/projections | 6.3, 7.1 | `azure_devops_agent_*`, group/team/member families, service endpoints, test plans/suites/cases, user/group licensing, project members, onboarding statuses, and `azure_devops_repos.rs` | Include plain async repository helpers and non-Send projection futures. Reuse inventories without suppressing explicit invalidation or changing source policy. Licensing writes remain mutations. |
| B16 Gitea requests | 4.1b; final audit 7.1 | [gitea_api_support.rs](../../crates/gitea/src/gitea_api_support.rs) and `gitea_*_request.rs` under [Gitea SDK](../../crates/gitea/src) | Task 4.1b completes data-only executable requests and explicit executor propagation, preserving command-backed pagination, uncached pacing, best-effort behavior, tenant identity, and scan/enumeration workflows. Tea receives exactly one matching profile explicitly; service reads execute independently because profile names do not establish credential generations. No Azure authentication is introduced. |

The `B11` remainder is resolved by the complete request/constructor searches over
`crates/azure/src`; any hit not named above must be assigned to a subfamily or an
explicit exclusion before final cutover. `B15` performs the corresponding remainder
classification for `crates/azure_devops/src`. Supporting request bodies, pages,
reports, selectors, and journals are data objects; only their affected ownership,
serialization, or dependencies change, not their unrelated domain rules.

Shared-record changes also reach [EntraPrincipal](../../crates/azure_types/src/entra_principal.rs),
[EntraPrincipalCollection](../../crates/azure_types/src/entra_principal_collection.rs),
and [RoleDefinitionsAndAssignments](../../crates/azure_types/src/role_definitions_and_assignments.rs).
They currently contain boxed/owned records, consuming conversions, mutable
collection access, or cross-record validation. Decide these contracts in G4;
an `Arc<Vec<T>>` alone does not allow consuming the elements retained by the cache.
Do not turn the optional `Vec<Arc<T>>` design into a blanket domain-model rewrite.

## Existing key reuse that must not become nested initialization

At the original inventory baseline, `CacheableCommand::run` executed SDK bodies
without caching assembled results. Task 4.2 removes that contract: executable SDK
procedures call canonical REST work under the Rest namespace, and most assembled
SDK results remain owned. The historical forwarding examples below still guide
phase 5's separate InProcess owners and dependency tracking; do not add an outer
initializer using its child's concrete identity. These are source-established
relationships, not failures reproduced against real caches:

| Existing chain | Current key relationship | Required classification |
| --- | --- | --- |
| [EntraUserGetRequest](../../crates/azure/src/entra_user_get_request.rs) → REST receive | Shares its logical scope with canonical REST work; assembled SDK output remains owned | SDK aggregate and REST must have distinct execution owners/identities, even if they decode the same user type. |
| [EntraGroupGetRequest](../../crates/azure/src/entra_group_get_request.rs), service-principal/application get → Graph `fetch_one` → REST | Graph forwards the caller key unchanged for singleton requests | A generic Graph location is not an owner identifier; classify all singleton helper callers. |
| [PrincipalRequest](../../crates/azure/src/principal_get_request.rs) for unknown ID → Graph batch | Batch receives the principal request key but returns a different intermediate shape | Separate transport batch artifacts from canonical principal result; preserve dependencies for known-ID child requests too. |
| [ComputeSkuPricesRequest](../../crates/azure/src/compute_sku_prices.rs) → REST pages | Task 4.1b replaces Command with anonymous pricing; 4.2 uses explicit shared REST work | Keep the canonical REST owner separate from the future memoized SDK result. |
| Azure DevOps agent/group/team/service-endpoint/legacy query requests → REST | Task 4.2 uses modern keys, central REST namespace, and explicit canonical owners | Preserve scoped invalidation and child dependencies when phase 5 adds aggregate owners. |
| [Gitea singleton get](../../crates/gitea/src/gitea_user_get_request.rs) → `gitea_api_get` → command | Task 4.1b uses independently executed commands with zero-validity artifacts | SDK requests remain nonreusable without credential-generation identity; best-effort raw parsing has a distinct artifact scope. |
| Graph/Resource Graph/project lists → page requests | Child pages usually append a page index; aggregate is currently not memoized | Preserve logical subtree invalidation while separating namespaces/owners; absence of exact-key recursion does not supply aggregate dependency freshness. |

Reproduce the forwarding inventory, then inspect helper propagation rather than
assuming every textual hit is a collision:

```powershell
rg -n 'Some\(self\.cache_key\(\)\)|\.cache\(self\.cache_key\(\)\)|\.use_cache\(|cache_key = Some\(self\.cache_key\(\)\)' crates/azure/src crates/azure_devops/src crates/gitea/src
rg -n 'cache_key|cache_root|page_cache_key|get_request|fetch_one|use_cache' crates/azure/src/microsoft_graph.rs crates/azure/src/resource_graph.rs crates/azure/src/microsoft_graph_batch.rs crates/gitea/src/gitea_api_support.rs
```

An additional identity defect requires attention during B06/G3:
`SecurityGroupListRequest::cache_key` currently includes `ms/graph/GET/security_groups`
but no tenant. Preserve neither this omission nor account ambiguity when building
the new logical identity. Transport fingerprints do not justify sharing a typed
aggregate across different auth selections.

Picker requests and projection wrappers also reuse child **invalidation scopes**
without owning the child's execution identity. Examples are
[EntraUserPickRequest](../../crates/azure/src/entra_user_pick_request.rs),
[AzureDevOpsProjectPickRequest](../../crates/azure_devops/src/azure_devops_project_pick_request.rs),
[license-entitlement show](../../crates/azure_devops/src/azure_devops_user_license_entitlement.rs),
and [pool-entitlement projection](../../crates/azure_devops/src/azure_devops_agent_pool_entitlements_for_pool.rs).
Keep those semantic invalidation relationships. A soliciting picker must still
solicit; a fresh projection must honor its requested child refresh. Do not assign
them the inventory's memoized owner merely because they invalidate it.

## Consumers, local work, and application ownership

| Batch | Owning tasks | Source surface | Required disposition |
| --- | --- | --- | --- |
| B17 Audit activities | 6.1 | [audit_azure.rs](../../crates/azure_cli/src/audit_azure.rs), Azure DevOps audit code, and direct inventory consumers | Split existing Azure audit policy into cohesive files; each activity obtains its own dependencies via scoped executor and explicit scope. Preserve audit policy and one output boundary. |
| B18 Service CLI and entrypoint | 6.2, 6.3 | [Azure CLI](../../crates/azure_cli/src), [Azure DevOps CLI](../../crates/azure_devops_cli/src), [entrypoint commands](../../crates/entrypoint/src/cli/command), interactive and noninteractive entrypoint modules | Move one command family with its SDK batch; preserve explicit org/project, tenant selection, boxed dispatch futures, output, --no-cache, menus, and protected workflows. Include raw `rest`, `terraform`, login, and Gitea commands. |
| B19 Reusable app and examples | 3.1, 3.2, 6.2, 7.1 | [App](../../crates/app/src/app.rs), [AppContext](../../crates/app/src/app_context.rs), [standalone example](../../examples/standalone-cli/src/main.rs), [group-management example](../../examples/group-management/src/main.rs), its `desire_document/try_plan.rs`, public facade exports | Task 3.2 binds the context's current auth/cancellation to one invocation cache through `executor()`, preserving the existing public fields for scheduled consumers and using a disabled provider without the auth feature. Construction performs no cache I/O or token acquisition. Keep child ownership explicit; task 6.2 completes examples and broader app consumers. Do not rely on counts to discover AppContext users. |
| B20 Ratatui and discovery consumers | 3.3, 6.3 | [invocation_controller.rs](../../crates/ui_ratatui/src/object_explorer/invocation_controller.rs), `production_controller.rs`, `explorer_engine.rs`, `value_builder.rs`, browser event/snapshot code, [dump_plan_cli.rs](../../crates/entrypoint/src/cli/command/dump/dump_plan_cli.rs) | Task 3.3 threads the app-bound executor through Ratatui, the invocation host, and default production; each job has an independently cancellable linked child, and borrowed invocations remain inside the owning worker. Offline fixtures seed selection data and shared outputs. Preserve remaining service-request migration and runtime discovery admission under 6.3. |
| B21 egui workers | 4.1c, 6.3 | [app.rs](../../crates/ui_egui/src/app.rs), [run_app.rs](../../crates/ui_egui/src/run_app.rs), [workers](../../crates/ui_egui/src/workers), [resources_window.rs](../../crates/ui_egui/src/windows/resources_window.rs) | Task 4.1c removes incidental executor storage and passes each worker its linked scope. Task 6.3 introduces shared executor resources only for migrated service requests that actually require them; local GUI work stays scope-only. Migrate subscription/resource-group/resource loads and launch callers together. |
| B22 HCL and raw process consumers | 2.6, 4.1b, 4.1c; remaining request separation 6.3 | [reflow_new.rs](../../crates/hcl/src/reflow/reflow_new.rs), HCL writer/importer/import-builder/provider/work-directory lifecycle, [as_formatted_hcl_string.rs](../../crates/hcl/src/as_formatted_hcl_string.rs), entrypoint Terraform/process-generated helpers | Task 2.6 moves formatting from `hcl_types` to `hcl`; task 4.1c corrects 4.1b's broad executor dependency with scope-only process/helpers, preserving cleanup and effects. HCL and egui no longer depend directly on execution/cache crates. Task 6.3 still separates remaining reflow request data from optional auth/scope. HCL transformations themselves are not redesigned. |
| B23 Source scanning and local exports | 3.3, 6.3 | [SourceHuntRequest](../../crates/source/src/source_hunt_request.rs), source CLI, [DiscoveryPlanBuildRequest](../../crates/registry_execution/src/discovery_plan_build_request.rs), [ProduceJsonRequest](../../crates/ui_ratatui/src/object_explorer/produce_json_request.rs), [ProduceExcalidrawDumpPlanRequest](../../crates/excalidraw/src/produce_excalidraw_dump_plan_request.rs) | Task 3.3 migrates DiscoveryPlanBuildRequest to data-only ExecutableRequest. Remaining source/export requests move cancellation/progress handles to bound execution/observation in 6.3, without artificial auth. Preserve blocking-worker checkpoints and required filesystem publication. These execute through the contract but do not become reusable cloud reads. |
| B24 Cache inspection and refresh UX | 2.1, 6.2 | [cache list](../../crates/entrypoint/src/cli/command/cache/list/cache_list_cli.rs), [cache clean](../../crates/entrypoint/src/cli/command/cache/clean_tui_cli.rs), `with_invalidation` callers from snapshot | Preserve readable discovered scopes and use the owning cache handle for memory/disk invalidation. Test against isolated synthetic storage only. |

Raw-result callers require deliberate preservation of their interpretation step.
Examples include Resource Graph and cost management custom decoders, batch
response validators, build-definition validators, Gitea best-effort parsing,
application-gateway/deployment raw response handling, build-folder deletion,
and generic REST CLI output. The shared cache must not turn caller-specific
validation/projection into canonical decoding merely because the Rust types match.
Task 4.2b adds a distinct fixed wire-to-domain conversion boundary to REST:
Resource Graph rows are decoded and converted once before publishing the shared
domain result. Wire and target types both participate in the cache contract, and
conversion failures retain response artifacts. Arbitrary per-caller validators
and mappers still run separately and do not become canonical solely because
their return types match.

## Explicit exclusions and exceptions

These are responsibility exclusions, not permanent old execution paths:

- Pure domain values, API bodies/responses, validated scalar parsers, arbitrary
  generators, and `From`/`TryFrom` conversions do not receive an executor solely
  because they appear in the registry. Shared output changes may still require
  updates to their ownership and registrations.
- `AppContext`, credential token/session owners, worker coordinators, and bounded
  invocation state legitimately contain runtime resources. Removing auth/scope
  from **request data** does not mean eliminating them from their real owners.
- Existing ordinary internal helpers can continue receiving the exact local
  fields or scope they need. Do not replace every `CancellationScope` occurrence
  with the full executor or create a new parameter bag just to shorten signatures.
- Unrelated UI event handling, terminal resource ownership, HCL rewriting rules,
  source scan algorithms, and domain validation remain unchanged. Their actual
  execution/observation entrypoints still migrate in B20–B23.
- Default-organization/project discovery, Azure login/account discovery, tracked
  tenant management, and Gitea default-tenant management remain explicit
  management operations where applicable. They cannot silently supply missing
  org/project/tenant to ordinary migrated service requests. Preserve intentional
  management actions without resurrecting authenticated-request defaults.
- Task 2.7 replaces the ignored resource-type generator with
  [standalone tooling](../../tools/resource-types-cli/README.md). Offline catalogue
  generation/checking is separate from live refresh with an explicit credential
  source and resolved tenant. U56 allows this tool to use the shared default,
  UUID, or alias selector before constructing its explicitly tenant-bound SDK
  request; it does not add implicit tenant selection to service request data. Task
  2.8 moves provider records into `azure_types` and the cached tenant list request
  into `azure`; this SDK request joins the remaining request migration in 6.3.
  Only synthetic offline operations run during this migration; compile adaptation of
  refresh and other inline/live tests does not authorize real-system execution.
- A mutable application workflow, user selection, filesystem export, or
  zero-duration mutation is not automatically memoized merely because it adopts
  the execution trait. Preserve its effects and completion contract.
- Published consumer compatibility and cross-process coordination are not proven
  by this inventory. Phase 7 documents Rust API changes; phase 8 owns the deferred
  process-coordination contract. No adjacent repository is an implementation target.

## Omission risks and cutover audit

The main risks are unregistered plain functions, progress/watch handles hidden by
Facet attributes, same-key nested wrappers, raw decoder paths, custom invalidators,
local-only requests, and consumers which only access AppContext. The named batches
cover these cases in addition to the macro/field searches.

Re-run the snapshot and backstop searches before task 7.1 completion. Classify
each remaining hit as a migrated execution boundary, legitimate runtime owner,
pure/data-only object, test-only fixture, or temporary explicit adapter with an
identified removal task. None may remain an unexplained hidden-cache fallback.

```powershell
rg -n 'MEMORY_CACHE|bind_request_cancellation|impl_cacheable_into_future|impl_cacheable_work_into_future|AuthContext::resolve' crates examples
rg -n 'facet\((opaque|skip)|progress_hook|watch::Sender|auth_context:|cancellation:' crates/azure/src crates/azure_devops/src crates/source/src crates/hcl/src/reflow
rg -n 'fetch_|RestRequest::|CommandBuilder::|AppContext|context\.auth|context\.cancellation' crates/ui_egui/src crates/ui_ratatui/src crates/entrypoint/src examples -g '*.rs'
```

Validation remains the parent plan's responsibility: this inventory authorizes no
test execution, cache inspection, or live requests. Its counts are evidence of
source coverage on the recorded date, not evidence that migration behavior works.
