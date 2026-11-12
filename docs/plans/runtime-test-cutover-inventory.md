# Runtime verification and Cargo-test cutover inventory

The production catalogue contains explicitly registered verification cases. This companion records the source inventory for migration tasks 6.0a, 6.0b and 6.0e. It contains no live service data. No runtime verification case was executed against real accounts during implementation.

## Contract

- `ct test list [pattern]` inspects static registration metadata only.
- `ct test run <pattern>` matches full registry names using `*` and `?`; cases are not CLI subcommands.
- Inputs come from explicit tenant/account/org/project/instance flags and optional per-case JSON fields. The selected request decoder validates remaining fields; no arbitrary values or discovery-produced scopes are used.
- Runtime test registration is separate from ordinary operation discovery. Existing SDK cases reference their exact executable handle. Verification-only workflows do not enter the ordinary operation catalogue.
- Every declared effect must be admitted; default admission is Read.
- `--clean` only chooses normal `.with_invalidation(clean)` execution. There is no decoding-only or response replay mode. Without it the CLI warns about eligible cache reuse.
- Case errors are reported as data. Serial admission awaits every started request, then stops admitting new work on cancellation. The report must still be emitted before a nonzero outcome.
- Empty API collections are accepted: successful execution proves only the response path exercised, not item-schema coverage for absent items. Memory hits do not re-run decoding; cache initializer counts are not network-call counts. Reports count directly returned collection entries through ownership wrappers without retaining payloads. A zero count exposes the lack of item observations; other shapes are marked Unmeasured, and positive counts do not imply every nested schema was exercised. Debug builds enable existing strict response decoding.

## Removed Cargo entrypoints

Former nonempty/population-count assertions were tenant-specific expectations, not service contracts. Explicit runtime cases retain request execution and decoding while dropping those assumptions. Former exploratory UI/process/install demonstrations are removed where no repeatable assertion existed; they are not disguised as ignored Cargo tests.

| Former source | Former test | Replacement or disposition |
| --- | --- | --- |
| `crates/auth/src/azure_access_token.rs` | `it_works_profiles` | authenticated Azure DevOps runtime cases |
| `crates/auth/src/azure_access_token.rs` | `it_works_projects` | authenticated Azure DevOps runtime cases |
| `crates/auth/src/azure_devops_pat.rs` | `non_empty` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/auth/src/azure_devops_rest_client.rs` | `it_works` | authenticated Azure DevOps runtime cases |
| `crates/auth/src/jwt.rs` | `it_works` | authenticated Azure runtime cases (offline JWT claim parser tests retained) |
| `crates/azure_devops_cli/src/azure_devops_rest_command_cli.rs` | `it_works` | azure-devops-projects-list |
| `crates/azure_devops/src/azure_devops_agent_packages.rs` | `it_works` | azure-devops-agent-packages |
| `crates/azure_devops/src/azure_devops_agent_pool_entitlements_for_pool.rs` | `it_works` | azure-devops-agent-pool-entitlement-list-for-pool (explicit organization and pool selection) |
| `crates/azure_devops/src/azure_devops_agent_pool_entitlements_for_project.rs` | `it_works` | azure-devops-agent-pool-entitlement-list-for-project |
| `crates/azure_devops/src/azure_devops_agent_pools.rs` | `it_works` | azure-devops-agent-pool-list |
| `crates/azure_devops/src/azure_devops_group_license_entitlements.rs` | `it_works` | azure-devops-group-license-entitlements |
| `crates/azure_devops/src/azure_devops_group_member.rs` | `it_works` | azure-devops-group-members-list, azure-devops-group-members-v2 |
| `crates/azure_devops/src/azure_devops_group_member.rs` | `it_works_v2` | azure-devops-group-members-list, azure-devops-group-members-v2 |
| `crates/azure_devops/src/azure_devops_group.rs` | `it_works` | azure-devops-groups-list |
| `crates/azure_devops/src/azure_devops_groups_for_member.rs` | `it_works` | azure-devops-groups-for-member |
| `crates/azure_devops/src/azure_devops_projects.rs` | `test_fetch_all_azure_devops_projects` | azure-devops-projects-list |
| `crates/azure_devops/src/azure_devops_repos.rs` | `test_fetch_all_azure_devops_repos` | azure-devops-repositories-list |
| `crates/azure_devops/src/azure_devops_service_endpoint.rs` | `it_works` | azure-devops-service-endpoints-list |
| `crates/azure_devops/src/azure_devops_team_member.rs` | `it_works` | azure-devops-team-members |
| `crates/azure_devops/src/azure_devops_team.rs` | `it_works` | azure-devops-teams-for-project |
| `crates/azure_devops/src/azure_devops_test_plans.rs` | `it_works` | azure-devops-test-plan-list |
| `crates/azure_devops/src/azure_devops_test_suite_cases.rs` | `it_works` | azure-devops-test-suite-cases-list |
| `crates/azure_devops/src/azure_devops_test_suites.rs` | `it_works` | azure-devops-test-suite-list |
| `crates/azure_devops/src/azure_devops_user_license_entitlements.rs` | `it_works` | azure-devops-user-license-entitlement-list |
| `crates/azure_devops/src/azure_devops_work_item_queries.rs` | `it_works` | azure-devops-work-item-queries-for-project |
| `crates/azure_devops/src/azure_devops_work_items_for_query.rs` | `it_works` | azure-devops-work-items-for-query |
| `crates/azure_devops/src/default_organization_tui.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/azure_devops/src/work_item_tests.rs` | `live_work_item_deserialization` | azure-devops-work-item-batch-get, azure-devops-work-item-type-list and associated work-item cases with explicit inputs |
| `crates/azure/src/app_service_list_request.rs` | `it_works` | app-service-list |
| `crates/azure/src/application_gateway_list_request.rs` | `it_works` | application-gateway-list |
| `crates/azure/src/application_registration_list_request.rs` | `list_application_registrations` | application-registration-list |
| `crates/azure/src/application_registration_search_request.rs` | `application_search_filters_are_accepted_by_graph` | application-registration-search |
| `crates/azure/src/cognitive_services_account_list_request.rs` | `it_works` | cognitive-services-account-list |
| `crates/azure/src/compute_publisher_vm_image_offer_sku_versions.rs` | `it_works` | compute-publisher-image-offer-sku-version-list |
| `crates/azure/src/compute_publisher_vm_image_offer_skus.rs` | `it_works` | compute-publisher-image-offer-sku-list |
| `crates/azure/src/compute_publisher_vm_image_offers.rs` | `it_works` | compute-publisher-image-offer-list |
| `crates/azure/src/compute_publishers.rs` | `it_works` | compute-publishers-list |
| `crates/azure/src/compute_sku_prices.rs` | `it_works` | compute-sku-prices |
| `crates/azure/src/compute_skus.rs` | `it_works` | compute-sku-list |
| `crates/azure/src/conditional_access_named_locations.rs` | `it_works` | conditional-access-named-location-list |
| `crates/azure/src/conditional_access_policies.rs` | `disallowed_ips` | conditional-access-policy-list |
| `crates/azure/src/conditional_access_policies.rs` | `it_works` | conditional-access-policy-list |
| `crates/azure/src/container_instance_list_request.rs` | `it_works` | container-instance-list |
| `crates/azure/src/container_registry.rs` | `it_works` | container-registry-list |
| `crates/azure/src/container_registry.rs` | `it_works2` | container-registry-list |
| `crates/azure/src/cost_management.rs` | `it_works1` | cost-query (explicit query body replaces three ambient current-month variants) |
| `crates/azure/src/cost_management.rs` | `it_works2` | cost-query (explicit query body replaces three ambient current-month variants) |
| `crates/azure/src/cost_management.rs` | `it_works3` | cost-query (explicit query body replaces three ambient current-month variants) |
| `crates/azure/src/eligible_child_resources.rs` | `it_works` | eligible-child-resource-list |
| `crates/azure/src/eligible_child_resources.rs` | `it_works_interactive` | eligible-child-resource-list |
| `crates/azure/src/eligible_child_resources.rs` | `it_works_interactive2` | eligible-child-resource-list |
| `crates/azure/src/eligible_child_resources.rs` | `it_works2` | eligible-child-resource-list |
| `crates/azure/src/eligible_child_resources.rs` | `it_works3` | eligible-child-resource-list |
| `crates/azure/src/eligible_child_resources.rs` | `it_works4` | eligible-child-resource-list |
| `crates/azure/src/entra_group_list_request.rs` | `list_groups` | entra-group-list |
| `crates/azure/src/entra_group_members.rs` | `list_group_members` | entra-group-members-list |
| `crates/azure/src/entra_group_owners.rs` | `list_group_owners` | entra-group-owner-list |
| `crates/azure/src/entra_user_list_request.rs` | `it_works` | entra-user-list |
| `crates/azure/src/entra_user_search_request.rs` | `it_finds_the_current_user_by_user_principal_name` | entra-user-search |
| `crates/azure/src/governance_role_assignments.rs` | `it_works` | governance-role-assignment-list |
| `crates/azure/src/key_vaults.rs` | `it_works` | key-vault-list |
| `crates/azure/src/locations.rs` | `it_works` | location-list |
| `crates/azure/src/management_groups.rs` | `it_works` | management-group-list |
| `crates/azure/src/metrics.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/azure/src/network_interface_list_request.rs` | `it_works` | network-interface-list |
| `crates/azure/src/oauth2_permission_grants.rs` | `it_works` | oauth2-permission-grant-list |
| `crates/azure/src/oauth2_permission_scopes.rs` | `it_works` | entra-oauth2-permission-scope-list |
| `crates/azure/src/pim_entra_role_assignments.rs` | `it_works` | entra-pim-role-assignments-list-mine |
| `crates/azure/src/pim_entra_role_definitions.rs` | `it_works` | pim-entra-role-definition-list |
| `crates/azure/src/pim_entra_role_settings.rs` | `it_works` | entra-pim-role-settings |
| `crates/azure/src/policy_assignments.rs` | `it_works` | policy-assignment-list |
| `crates/azure/src/policy_definitions.rs` | `it_works` | policy-definition-list |
| `crates/azure/src/policy_set_definitions.rs` | `it_works` | policy-set-definition-list |
| `crates/azure/src/policy_set_definitions.rs` | `it_works_v2` | policy-set-definition-list |
| `crates/azure/src/principals.rs` | `it_works` | principal-list |
| `crates/azure/src/private_endpoint_list_request.rs` | `it_works` | private-endpoint-list |
| `crates/azure/src/public_ip_list_request.rs` | `it_works` | public-ip-list |
| `crates/azure/src/resource_graph_helper.rs` | `it_works` | resource-group-list and other typed Resource Graph cases |
| `crates/azure/src/resource_group_list_request.rs` | `invalidation` | resource-group-list |
| `crates/azure/src/resource_group_list_request.rs` | `it_works` | resource-group-list |
| `crates/azure/src/resource_group_list_request.rs` | `pick` | resource-group-list |
| `crates/azure/src/resources.rs` | `count` | resource-list |
| `crates/azure/src/resources.rs` | `it_works` | resource-list |
| `crates/azure/src/resources.rs` | `resource_groups` | resource-list |
| `crates/azure/src/role_assignments.rs` | `it_works` | role-assignment-list |
| `crates/azure/src/role_definitions_and_assignments.rs` | `it_works` | role-definitions-and-assignments-list |
| `crates/azure/src/role_definitions.rs` | `it_works` | role-definition-list |
| `crates/azure/src/role_definitions.rs` | `key_vaults` | role-definition-list |
| `crates/azure/src/role_eligibility_schedules.rs` | `it_works` | entra-role-eligibility-schedule-list-mine |
| `crates/azure/src/role_management_policy_assignments.rs` | `it_works` | role-management-policy-assignments-for-role |
| `crates/azure/src/role_operations.rs` | `includes_microsoft_authorization_provider` | role-operation-list |
| `crates/azure/src/role_operations.rs` | `it_works` | role-operation-list |
| `crates/azure/src/route_table.rs` | `it_works` | route-table-list |
| `crates/azure/src/security_groups.rs` | `it_works` | security-group-list |
| `crates/azure/src/service_groups.rs` | `it_works` | service-group-list |
| `crates/azure/src/service_principals.rs` | `it_works` | service-principal-list |
| `crates/azure/src/storage_accounts.rs` | `it_works` | storage-account-list |
| `crates/azure/src/subscriptions.rs` | `get_active` | Removed ambient Azure CLI subscription-selection probe; no isolated assertion beyond nonempty display. |
| `crates/azure/src/subscriptions.rs` | `it_works` | subscription-list |
| `crates/azure/src/subscriptions.rs` | `resolves_tenant_for_subscription_id` | subscription-list |
| `crates/azure/src/tags.rs` | `get_tags_test` | resource-tags (explicit selected resource ID) |
| `crates/azure/src/tenant_default.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/azure/src/tenant_details.rs` | `it_works` | tenant-details |
| `crates/azure/src/tenant_licenses.rs` | `it_works` | tenant-license-list |
| `crates/azure/src/unified_role_assignments.rs` | `it_works` | unified-role-assignment-list |
| `crates/azure/src/unified_role_definition.rs` | `it_works_single` | unified-role-definition |
| `crates/azure/src/unified_role_definition.rs` | `it_works_single_deep` | unified-role-definition |
| `crates/azure/src/unified_role_definitions_and_assignments.rs` | `it_works` | unified-role-definitions-and-assignments-list |
| `crates/azure/src/unified_role_definitions.rs` | `it_works` | unified-role-definition-list |
| `crates/azure/src/virtual_machine_prices.rs` | `it_works` | compute-sku-prices |
| `crates/azure/src/virtual_machine_sizes.rs` | `it_works` | virtual-machine-size-list |
| `crates/azure/src/virtual_machines.rs` | `it_works` | virtual-machine-list |
| `crates/azure/src/virtual_network.rs` | `it_works` | virtual-network-list |
| `crates/azure/tests/predict_key_vault_access.rs` | `predict_would_secret_list_succeed` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/command/tests/command_azure_tests.rs` | `local_cli_metadata_can_share_a_canonical_raw_result` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/command/tests/command_azure_tests.rs` | `local_cli_metadata_uses_explicit_execution` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/command/tests/command_terraform_tests.rs` | `send_stdin_terraform_fmt` | terraform-format |
| `crates/entrypoint/src/interactive/create_new_action_variant.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/gitea/src/lib.rs` | `it_analyzes_repo_enumeration_methods` | gitea-repo-list (explicit instance/account) |
| `crates/gitea/src/lib.rs` | `it_fetches_repositories_without_empty_ids` | gitea-repo-list (explicit instance/account) |
| `crates/hcl/src/as_formatted_hcl_string.rs` | `send_stdin_fmt` | terraform-format |
| `crates/hcl/src/importer.rs` | `terraform_concurrent_init_fail` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/hcl/src/importer.rs` | `terraform_concurrent_init_happy` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/hcl/src/provider_manager.rs` | `install_default_providers` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/hcl/src/provider_manager.rs` | `install_missing_providers` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/hcl/src/provider_manager.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/hcl/src/work_dir_lifecycle.rs` | `terraform_concurrent_init_happy` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/ui_ratatui/src/role_assignment_picker_app.rs` | `manual_pick` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/user_input/src/message_box.rs` | `manual_run` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/user_input/tests/pick_inference.rs` | `it_works` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/zombies/src/lib.rs` | `kill` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |
| `crates/zombies/src/lib.rs` | `main` | Removed ambient/configuration or manual exploratory demonstration; meaningful offline policy tests retained. |


## Retained Cargo coverage

The latest inventory covered 1,651 test functions throughout `crates`, `tools`, and `examples`, using lexical extraction followed by review of credential/configuration, external command, network, and interactive candidates. Test helpers were examined where a call could hide those effects. There are no custom `harness = false` test targets in workspace manifests.

- Domain/parser/policy/registry tests retain synthetic values and do not establish live API compatibility.
- Cache/execution/REST protocol tests use temporary storage, injected authentication, and loopback transports.
- Azure/DevOps aggregate tests use fake providers and explicit synthetic requests. Rejection tests stop before credential acquisition/network.
- Auth tests retain temporary session stores, fixed placeholder credentials and pre-network policy checks. Ambient credential probes were removed.
- Source discovery tests create disposable local repositories; UI/excalidraw tests use in-memory state or temporary files.
- Provider inference tests now use an owned OS temporary directory instead of configured application storage.
- Config load/upgrade integration tests override `Config::config_path` through a test-only Tokio task-local root, with a distinct owned temporary directory per test. Both schema versions share only that test's path. No process environment is changed, and missing test setup fails instead of falling back to application configuration. Full-workspace execution exposed these three fixture types as an isolation gap in the initial inventory; it is now corrected. The separate egui autosave fixture overrides `save` entirely and performs no file I/O.
- Controlled process tests use explicit shell programs or local test executables. Tests that resolved configured Azure CLI/Terraform programs moved out of Cargo.
- Existing ignored, offline development previews/unfinished pure policy assertions remain unrelated to live verification.

## Verification evidence

The four runtime-registry policy and output-coverage tests passed, covering effect admission and glob selection, normal execution with optional invalidation, failure/report outcomes, and verification-only registration isolation. Three additional credential-retirement cache regressions passed, covering persisted entries, late publication and cross-account dependencies. An earlier build exhausted disk during compilation before executing tests; generated build artifacts were cleaned and remaining checks disable incremental compilation. The complete cache, registry, registry-execution, Azure and Azure-CLI test suites plus doctests passed (279 checks in that batch), including five audit-policy and shared-inventory regressions. Workspace Clippy passed with all targets, all features, and warnings denied. The final clean-rebuild workspace suite passed 1,668 tests with five existing offline tests ignored; all seven runnable/compile-fail doctests passed with one existing example ignored. The no-default-features app suite passed 47 tests. All-target/all-feature Clippy and formatting passed. See the [cutover evidence](executor-cache-cutover-validation.md) for commands, review corrections and platform limits; static inventory alone does not establish these results.


## Retired execution bridges

Registry invocation and discovery now accept execution resources only through `ExecutableRequest`. The legacy `register_into_future` macro, reflected cancellation-field replacement, corresponding contract variant and host-supplied discovery-field exception were removed. UI fixtures use the same explicit execution path. Runtime verification cancellation is sticky across the selected run: a cancelled account-specific case closes later admission even if the root cancellation scope remains active.
SDK decoder checks register the existing typed request directly: the case's generic execution callback decodes that input and executes its normal invalidation and execution contract. A redundant forwarding request would add no verification behavior. Dedicated verification request types remain appropriate for actual multi-step or local-tool checks, such as Terraform formatting; those cases keep their own typed inputs and purpose-specific metadata outside ordinary operation discovery.

## Final source review

The runner rejects unknown per-case input names before account resolution, preventing misspelled manifest entries from silently discarding supplied identities or scope. A manifest may still contain other registered cases outside the selected pattern. Gitea cases reject `None` and foreign credential-source overrides before configuration lookup.

Composed refresh policies include current-user discovery for PIM. ACR repository and tag requests now take a resolved endpoint directly; callers can obtain it from an existing registry record or explicitly execute endpoint discovery. Entra group membership now gives the member-ID API its own cached request; the previous path attempted a cached receive without attaching a key. Refresh obtains fresh member IDs and invalidates the exact group lookup batch subsequently consumed. The group batch request accepts a BTreeSet of IDs, making ordering and deduplication intrinsic to its input and cache identity. Azure DevOps project membership likewise resolves its selected project before refreshing the actual ID-scoped groups and reachable membership requests. These changes passed workspace compilation, warnings-denied Clippy, and the Azure/DevOps regressions in the clean workspace rebuild. The earlier disk-exhausted attempt is superseded by the results in the [cutover evidence](executor-cache-cutover-validation.md). No live data was accessed to validate them.
