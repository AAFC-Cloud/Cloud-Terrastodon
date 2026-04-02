---
name: add-azure-resource-support
description: 'Add support for a new Azure resource in cloud-terrastodon. Use when creating Azure resource newtypes, typed resource/id/name models, naming-rule validation, Resource Graph list requests, Azure CLI list/show commands, scope plumbing, outage/discovery matching, and repo-standard validation with .\\check-all.ps1.'
argument-hint: 'Resource kind to add, for example: network interface, load balancer, subnet child resource'
user-invocable: true
---

# Add Azure Resource Support

Use this skill when extending `cloud-terrastodon` to support a new Azure resource end to end.

This workflow captures the repo pattern used for resources like:
- Azure Public IP
- Azure Application Gateway
- Azure Application Gateway backend health
- Azure Network Interface

## What This Produces
- Strong `azure_types` models for the resource
- Typed `Id` and `Name` newtypes with Azure naming validation
- `Scope` integration when the resource has a first-class ARM resource id
- A typed request in `crates/azure` for listing or fetching the resource
- `ct az ... list` and `ct az ... show` CLI plumbing in `crates/entrypoint`
- Matching logic for resource id, name, and useful secondary identifiers like IP or FQDN
- Tests and repo validation via `./check-all.ps1`

## When To Use
- Add a new top-level Azure ARM resource to the repo
- Replace `String` ids or names with strong newtypes
- Add Azure discovery support for outage investigation or correlation workflows
- Mirror an existing Azure resource pattern for consistency

## Procedure

### 1. Pick the Closest Existing Pattern
Start by finding the nearest comparable resource already in the repo.

Good reference points:
- `crates/azure_types/src/azure_public_ip_resource.rs`
- `crates/azure_types/src/azure_public_ip_resource_id.rs`
- `crates/azure_types/src/azure_public_ip_resource_name.rs`
- `crates/azure/src/public_ip_list_request.rs`
- `crates/entrypoint/src/cli/command/azure/public_ip/`
- `crates/azure_types/src/azure_application_gateway_resource.rs`
- `crates/azure/src/application_gateway_backend_health_request.rs`

Choose the reference based on shape:
- Top-level ARM resource with list/show only: mirror Public IP
- Top-level ARM resource with additional action endpoint: mirror Application Gateway plus backend health
- Correlation/discovery data source: mirror how `outage_investigate.rs` enriches and matches data

### 2. Define The Resource Surface
Decide what support is needed right now.

Questions to answer:
- Do we need a top-level resource type?
- Do we need `Id` and `Name` newtypes?
- Is there a separate action response type, like backend health?
- Do users need `list`, `show`, or both?
- Should `show` match only id/name, or also IP/FQDN/child-resource references?
- Does this resource belong in `ScopeImplKind` and `ScopeImpl`?

Default rule:
- If the resource has a stable ARM id and will be passed around in code, add strong `Id` and `Name` types.

### 3. Add `Name` Newtype In `azure_types`
Create `crates/azure_types/src/<resource>_name.rs`.

Follow the repo slug pattern:
- Implement `Slug`
- Validate using Azure naming rules from Microsoft docs
- Add `FromStr`, `Display`, serde, `Deref`, `TryFrom<CompactString>`
- Add property tests or fuzz tests when the adjacent resources do so

Naming rules checklist:
- Confirm min/max length
- Confirm allowed characters
- Confirm start/end constraints
- Add the Microsoft naming-rules documentation URL in wrapped errors

### 4. Add `Id` Newtype In `azure_types`
Create `crates/azure_types/src/<resource>_id.rs`.

Use the repo resource-id pattern:
- Add the ARM suffix constant like `.../providers/Microsoft.Network/...`
- Store `resource_group_id` plus typed name for resource-group-scoped resources
- Implement `HasSlug`, `AsRef`, `NameValidatable`, `HasPrefix`, `TryFromResourceGroupScoped`
- Implement `Scope`
- Implement serde via expanded ARM id strings
- Add round-trip and constructor tests

If the resource is a first-class scope:
- Add it to `ScopeImplKind`
- Add it to `ScopeImpl`
- Update scope parsing, display, `expanded_form`, `short_form`, and kind mapping in `crates/azure_types/src/scopes.rs`

### 5. Add The Typed Resource In `azure_types`
Create `crates/azure_types/src/<resource>.rs`.

Model the JSON returned by Resource Graph or the ARM API.

Rules:
- Use typed `id` and `name` fields when available
- Preserve `tenant_id` when the repo pattern expects it
- Use `deserialize_default_if_null` for arrays and maps that Azure may return as `null`
- Prefer typed nested structs for fields the repo will actually use
- Leave unknown or low-value areas as `serde_json::Value` until there is a clear need

When choosing how much to model:
- Strong-type fields needed for CLI filtering, correlation, or follow-up requests
- Keep the first pass focused on fields the workflow needs now
- Expand later when real payloads justify more structure

### 6. Export The New Types
Update `crates/azure_types/src/lib.rs`.

Required changes:
- Add `mod ...;`
- Add `pub use crate::...::*;`

### 7. Add The Azure Request In `crates/azure`
For list support, add a file like `crates/azure/src/<resource>_list_request.rs`.

Use the `ResourceGraphHelper` pattern:
- Define a request struct holding `tenant_id`
- Expose `fetch_all_<resources>(tenant_id)`
- Implement `CacheableCommand`
- Build a Resource Graph query selecting the typed fields you modeled
- Return `Vec<YourResourceType>`
- Add an integration-style test if similar nearby files do

For action endpoints or non-graph fetches:
- Follow the backend-health pattern
- Use typed REST responses instead of raw `Value` when possible
- Reuse shared REST plumbing from `cloud_terrastodon_credentials`

### 8. Export The Request From `crates/azure`
Update `crates/azure/src/lib.rs`.

Required changes:
- Add the new `mod`
- Re-export it with `pub use`

### 9. Add CLI Module Plumbing In `entrypoint`
Create a folder under `crates/entrypoint/src/cli/command/azure/` when the resource deserves its own command group.

Typical files:
- `mod.rs`
- `azure_<resource>.rs`
- `azure_<resource>_list.rs`
- `azure_<resource>_show.rs`

Then wire them into:
- `crates/entrypoint/src/cli/command/azure/mod.rs`
- `crates/entrypoint/src/cli/command/azure/azure_command.rs`

Alias guidance:
- Add a short alias only when it is obvious and consistent, like `agw` or `nic`

### 10. Implement `list`
Mirror the public-IP list command.

Expected behavior:
- Resolve `tenant`
- Call the typed request
- Log count with `tracing::info!`
- Print pretty JSON to stdout

### 11. Implement `show`
Mirror the public-IP or application-gateway show command.

Base matching always includes:
- Full resource id
- Resource name, case-insensitive

Optional matching may include:
- IP addresses
- FQDNs
- Attached public IP resource ids
- Child resource ids

Behavior rules:
- `0` matches: clear not-found error
- `1` match: print JSON
- `>1` matches: sort by expanded id and ask for full resource id

### 12. Add Correlation Hooks If The Resource Supports Discovery Workflows
If the new resource helps outage investigation or automated digging, update the relevant workflow.

Examples:
- Public IP -> Application Gateway frontend
- Backend server IP -> Network Interface
- Network Interface -> VM

When enriching a report:
- Deduplicate expensive follow-up lookups
- Preserve partial success by attaching an `error` field instead of failing the whole report when helpful
- Use typed response models in the report output

### 13. Use Strong Types Instead Of `String` Where Worthwhile
Prefer strong types for:
- ARM resource ids
- Resource names
- Enumerated health states or status values

Use tolerant enums when Azure responses are inconsistent across docs and real payloads.

Example strategy:
- Add known enum variants
- Add `Other(String)` to preserve forward compatibility

### 14. Test At The Right Level
Add or update tests in the new files.

Common test categories:
- Resource name validation
- Resource id round-trip parsing/serialization
- Typed resource deserialization from a realistic payload sample
- CLI matching behavior helpers
- Case-insensitive header or id lookups if relevant

Use real payload fragments whenever possible.

### 15. Validate With Repo Standard Checks
Always run the repo’s preferred validation flow:

```powershell
.\check-all.ps1
```

When tests use `CommandKind::CloudTerrastodon`, remember the repo note:
- build the binary first if CLI command shapes changed

## Decision Points

### Whether To Add `Id` And `Name` Types
Add them when:
- The resource is a first-class Azure resource
- The resource appears in CLI input or output
- The resource will be referenced by other typed structs

Skip or defer only when:
- The data is purely nested and not reused
- The workflow is exploratory and not stable yet

### Whether To Add Scope Support
Add scope support when:
- The resource has a standard ARM id
- The repo benefits from parsing or displaying it generically
- Other commands may accept it as a scope-like input later

### Whether To Match On Secondary Identifiers In `show`
Add secondary matching when users naturally search that way.

Examples:
- Public IP `show` by IP or FQDN
- NIC `show` by private IP or attached public IP resource id

Do not add overly broad fuzzy matching that can create noisy ambiguity.

### Whether To Strong-Type A Field Now
Strong-type it now when:
- Code branches on it
- It appears in multiple workflows
- It drives filtering or diagnostics

Leave it as `String` or `Value` for now when:
- Only displayed raw
- Documentation and observed payloads diverge too much and no behavior depends on it yet
- Real-world examples are still sparse

## Completion Criteria
A resource addition is complete when all of the following are true:
- Typed resource compiles and deserializes from realistic payloads
- `Id` and `Name` newtypes exist and validate correctly
- `azure_types` and `azure` exports are wired
- CLI command is reachable from `ct az ...`
- `list` and/or `show` behave like sibling commands in the repo
- Scope plumbing is updated if needed
- New matching logic is covered by tests where practical
- `./check-all.ps1` passes

## Examples
- Add support for Azure load balancers using the Public IP and Network Interface patterns
- Add `ct az nic show <private-ip>` and thread NIC matches into outage investigation
- Add a typed action response like Application Gateway backend health instead of returning raw `Value`

## Notes For This Repo
- Prefer `apply_patch` for focused file edits
- Prefer ASCII in new files
- Reuse existing repo patterns rather than inventing a new abstraction layer
- Do not duplicate REST response types already provided by `cloud_terrastodon_credentials`
- Use `deserialize_default_if_null` aggressively for Azure arrays and maps that may come back as `null`
- Validate end-to-end with `./check-all.ps1`
