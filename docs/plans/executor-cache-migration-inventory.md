# Executor/cache migration caller inventory

**Inventory date:** 2026-10-30

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
The single `azure_resource_types` adapter hit is an ignored, test-only source
generator, classified below. Zeros do not establish absence of consumers.

To enumerate a metric's files rather than its counts, use the same glob policy:

```powershell
rg -l 'CacheableCommand for|CacheableWorkRequest for|impl_cacheable_into_future!|impl_cacheable_work_into_future!|impl.*IntoFuture|register_into_future!' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'pub fn (receive|run)|pub async fn (receive|run)' crates/command/src/command.rs crates/rest/src/rest_request.rs
rg -n 'receive_raw|receive_with_mapper|receive_with_validator|run_raw|run_with_mapper|run_with_validator|run_polite' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'get_cached_output|write_output|write_failure|put_memory_cache_entry|bust_cache|invalidate_cache|discover_caches|with_invalidation' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n 'invoke_value_boxed|invoke_result_future|bind_request_cancellation|REGISTERED_INTO_FUTURE' crates examples -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
```

## Shared execution, auth, and invalidation boundaries

| Batch | Owning tasks | Source and responsibility | Completion obligation |
| --- | --- | --- | --- |
| B01 Cache owner and artifact storage | 2.1–2.4 | [artifact_cache.rs](../../crates/command/src/artifact_cache.rs), [cache_key.rs](../../crates/command/src/cache_key.rs), [cache_invalidatable.rs](../../crates/command/src/cache_invalidatable.rs), [command_cache_discovery.rs](../../crates/command/src/command_cache_discovery.rs) | Replace global memory ownership; bind invalidation and discovery to the explicit cache root/owner; preserve inspectable artifacts, validity, readable scope hierarchy, and live-entry coordination. |
| B02 Shared work and subprocess adapter | 2.1, 2.3, 4.1 | [cacheable_work.rs](../../crates/command/src/cacheable_work.rs), [cachable_command.rs](../../crates/command/src/cachable_command.rs), [command.rs](../../crates/command/src/command.rs), [process_fingerprint.rs](../../crates/command/src/process_fingerprint.rs), [from_command_output.rs](../../crates/command/src/from_command_output.rs) | Migrate both request traits/macros and all subprocess entrypoints to one explicit lifecycle. The existing spelling `cachable_command.rs` is intentional in this reference. Preserve complete fingerprints, retries, raw outputs, per-caller policy, caller/span diagnostics, and cancellation. |
| B03 REST adapter | 4.2 | [rest_request.rs](../../crates/rest/src/rest_request.rs), [request_execution.rs](../../crates/rest/src/request_execution.rs), [tenant_inference.rs](../../crates/rest/src/tenant_inference.rs) | Move auth/scope resources to execution; cover cached and uncached paths, raw and typed outputs, decoders, validators, metadata mappers, continuation headers, and explicit auth selection. |
| B04 Credential selection and runtime | 3.1, 3.2 | [auth_context.rs](../../crates/credentials/src/auth_context.rs), [azure_tenant_auth_context.rs](../../crates/credentials/src/azure_tenant_auth_context.rs), [azure_devops_auth_context.rs](../../crates/credentials/src/azure_devops_auth_context.rs), [azure_access_token.rs](../../crates/credentials/src/azure_access_token.rs), [tracked_tenants.rs](../../crates/azure/src/tracked_tenants.rs) | Add serializable selection and explicit bearer tenant; retain runtime token/session ownership, WIF tenant checks, headless policy, and PAT independence. Credential acquisition's subprocess is an infrastructure dependency, not a new domain-data dependency. |
| B05 Registry invocation and discovery | 3.3, 6.3 | [registry lib](../../crates/registry/src/lib.rs), [bind_request_cancellation.rs](../../crates/registry/src/bind_request_cancellation.rs), [default_production.rs](../../crates/registry/src/default_production.rs), [function_trait_contract.rs](../../crates/registry/src/function_trait_contract.rs), [discovery_reflection.rs](../../crates/registry/src/discovery_reflection.rs) | Replace reflected scope injection with explicit runtime invocation without adding a registry-to-concrete-executor cycle; preserve pure conversions, effects/admission, registered outputs, and offline planning/simulation. |

Subprocess methods requiring an explicit migration disposition are `run_raw`,
`run_raw_polite`, `run`, `run_polite`, `run_with_validator`, `run_with_mapper`,
and public `run_with_validator_from`. `get_cached_output`, `write_output`,
`write_failure`, and `bust_cache` are also public artifact/cache boundaries.
The last writes a disk `busted` marker today; [oauth2_permission_grants.rs](../../crates/azure/src/oauth2_permission_grants.rs)
calls it directly. Include this path in owner-aware invalidation even though the
snapshot's narrower `Invalidation` pattern does not find it. Internal retry calls reenter raw execution today; they must not
reacquire a non-reentrant key lock while the outer operation owns it.

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
| B11 ARM inventories | 6.3, 7.1 | Azure `subscriptions`, `resource_group_list`, `resources`, `management_groups`, `policy_*`, network/VM/storage/compute families, app-service/application-gateway/container/cognitive-services/key-vault inventories | Migrate one resource family at a time through its current Resource Graph, REST, or command boundary. Preserve its identity filters/limits and output. Protocol replacement is not part of this migration. |
| B12 Azure direct helpers and mutations | 6.3, 7.1 | Azure `batch_api`, `cost_management`, `metrics`, `tags`, `storage_account_name_availability`, `virtual_machine_prices/sizes`, `create_role_assignment`, `governance_role_assignments`, `eligible_child_resources`, `role_eligibility_schedules`, `role_management_policy_assignments`, `evaluate_policy_assignment_compliance`, `remediate_policy_assignment`, `entra_group_member_add/remove`, OAuth grant create/update/remove, and PIM activation | Inventory plain functions and `.invoke()` helpers as well as request macros. Read-only batches opt in explicitly; writes execute every time and preserve required response/artifact/invalidation work. |
| B13 Azure DevOps builds and folders | 6.3, 7.1 | `azure_devops_build_*_request.rs` and corresponding [build CLI](../../crates/azure_devops_cli/src/build) | Preserve list/get filters and revision identity, preflight freshness, folder prune/delete deferrals, and visible affected-read invalidation. |
| B14 Azure DevOps work items | 6.3, 7.1 | `azure_devops_work_item*`, `work_item_copy*`, `rest_work_item_copy_writer.rs`, and corresponding work-item CLI | Migrate schema/query reads, mutations, copy planner, writer, and durable journal workflows in separate slices. Preserve effect declarations, journaling/required completion, and existing copy-plan decisions. |
| B15 Other Azure DevOps inventories/projections | 6.3, 7.1 | `azure_devops_agent_*`, group/team/member families, service endpoints, test plans/suites/cases, user/group licensing, project members, onboarding statuses, and `azure_devops_repos.rs` | Include plain async repository helpers and non-Send projection futures. Reuse inventories without suppressing explicit invalidation or changing source policy. Licensing writes remain mutations. |
| B16 Gitea requests | 6.3, 7.1 | [gitea_api_support.rs](../../crates/gitea/src/gitea_api_support.rs) and `gitea_*_request.rs` under [Gitea SDK](../../crates/gitea/src) | Preserve command-backed pagination, uncached pacing, best-effort error behavior, tenant identity, login-list handling, and repository scan/enumeration dependencies. Move execution resources explicitly without introducing Azure authentication. |

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

Today `CacheableCommand::run` executes the SDK body without caching the assembled
SDK result. Therefore a request can pass its own key to a transport without an
outer key lock. That becomes unsafe if SDK memoization is introduced mechanically.
These are source-established examples, not failures reproduced against real caches:

| Existing chain | Current key relationship | Required classification |
| --- | --- | --- |
| [EntraUserGetRequest](../../crates/azure/src/entra_user_get_request.rs) → REST receive | Assigns `Some(self.cache_key())` directly to RestRequest | SDK and REST must have distinct execution owners/identities, even if they decode the same user type. |
| [EntraGroupGetRequest](../../crates/azure/src/entra_group_get_request.rs), service-principal/application get → Graph `fetch_one` → REST | Graph forwards the caller key unchanged for singleton requests | A generic Graph location is not an owner identifier; classify all singleton helper callers. |
| [PrincipalRequest](../../crates/azure/src/principal_get_request.rs) for unknown ID → Graph batch | Batch receives the principal request key but returns a different intermediate shape | Separate transport batch artifacts from canonical principal result; preserve dependencies for known-ID child requests too. |
| [ComputeSkuPricesRequest](../../crates/azure/src/compute_sku_prices.rs) → CommandBuilder | `cmd.cache(self.cache_key())` | A REST-only prefix does not prevent SDK/command collisions. |
| Azure DevOps agent/group/team/service-endpoint/legacy query requests → CommandBuilder | Multiple wrappers attach `self.cache_key()` directly | Apply shared central ownership checks and namespaces to command-backed wrappers as well as REST. |
| [Gitea singleton get](../../crates/gitea/src/gitea_user_get_request.rs) → `gitea_api_get` → command | Helper forwards its optional key to `cmd.use_cache` | Distinguish SDK result ownership from raw command execution; preserve best-effort callers separately. |
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
| B19 Reusable app and examples | 3.2, 6.2 | [App](../../crates/app/src/lib.rs), [standalone example](../../examples/standalone-cli/src/main.rs), [group-management example](../../examples/group-management/src/main.rs), its `desire_document/try_plan.rs`, facade re-exports | Construct shared resources inside the bounded invocation scope; keep auth optional, local-only execution credential-free, and child-scope binding explicit. Update both examples and public docs; do not rely on counts to discover AppContext users. |
| B20 Ratatui and discovery consumers | 3.3, 6.3 | [invocation_controller.rs](../../crates/ui_ratatui/src/object_explorer/invocation_controller.rs), `production_controller.rs`, `explorer_engine.rs`, `value_builder.rs`, browser event/snapshot code, [dump_plan_cli.rs](../../crates/entrypoint/src/cli/command/dump/dump_plan_cli.rs) | Pass explicit resources to registry invocation; seed selection data rather than runtime auth objects; preserve ownership/lease behavior, offline simulation, effects, and terminal coordination. |
| B21 egui workers | 6.3 | [app.rs](../../crates/ui_egui/src/app.rs), [run_app.rs](../../crates/ui_egui/src/run_app.rs), [workers](../../crates/ui_egui/src/workers), [resources_window.rs](../../crates/ui_egui/src/windows/resources_window.rs) | Cloned workers share cache/auth resources but receive their own linked child scope. Migrate subscription/resource-group/resource loads and launch callers together. |
| B22 HCL and raw process consumers | 6.3 | [reflow_new.rs](../../crates/hcl/src/reflow/reflow_new.rs), HCL writer/importer/import-builder/provider/work-directory lifecycle, [strings.rs](../../crates/hcl_types/src/strings.rs), entrypoint Terraform/process-generated helpers | Separate reflow request data from optional auth/scope; pass explicit process executor to raw command helpers while preserving cleanup and subprocess side effects. HCL transformations themselves are not redesigned. |
| B23 Source scanning and local exports | 3.3, 6.3 | [SourceHuntRequest](../../crates/source/src/source_hunt_request.rs), source CLI, [DiscoveryPlanBuildRequest](../../crates/registry/src/discovery_plan_build_request.rs), [ProduceJsonRequest](../../crates/ui_ratatui/src/object_explorer/produce_json_request.rs), [ProduceExcalidrawDumpPlanRequest](../../crates/excalidraw/src/produce_excalidraw_dump_plan_request.rs) | Move request-owned cancellation/progress handles to bound execution/observation without artificial auth. Preserve blocking-worker checkpoints and required filesystem publication. These execute through the contract but do not become reusable cloud reads. |
| B24 Cache inspection and refresh UX | 2.1, 6.2 | [cache list](../../crates/entrypoint/src/cli/command/cache/list/cache_list_cli.rs), [cache clean](../../crates/entrypoint/src/cli/command/cache/clean_tui_cli.rs), `with_invalidation` callers from snapshot | Preserve readable discovered scopes and use the owning cache handle for memory/disk invalidation. Test against isolated synthetic storage only. |

Raw-result callers require deliberate preservation of their interpretation step.
Examples include Resource Graph and cost management custom decoders, batch
response validators, build-definition validators, Gitea best-effort parsing,
application-gateway/deployment raw response handling, build-folder deletion,
and generic REST CLI output. The shared cache must not turn caller-specific
validation/projection into canonical decoding merely because the Rust types match.

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
- The `azure_resource_types/src/resource_types_generator.rs` hit is entirely
  under `#[cfg(test)]`; its generator is ignored and contacts a service. Adapt its
  compile-time API calls if required, keep it ignored, and do not execute it for
  this migration. Other inline/live tests likewise need compile adaptation, not
  authorization to run against real systems.
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
