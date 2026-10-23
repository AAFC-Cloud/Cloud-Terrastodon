# Planning cloud inventories

`ct dump plan` draws the registered requests and inputs that could produce the
types needed by a cloud inventory. It inspects the linked registry and Facet
metadata locally. It does not resolve credentials, call APIs, execute conversions,
or fabricate response values.

Plan the Azure DevOps project, service connection, and build definition chain:

```powershell
ct dump plan --org example --auth-source azure-cli `
    --target AzureDevOpsServiceEndpoint `
    --target AzureDevOpsBuildDefinition `
    --diagram plan.excalidraw
```

Open `plan.excalidraw` in Excalidraw to inspect and edit the request graph. The
diagram contains the same plan as the command output: seed and target shapes,
requests, conversions, reflected outputs, and blocked inputs. Export the typed
report as JSON with the shared output flag:

```powershell
ct dump plan --org example --auth-source azure-cli `
    --target AzureDevOpsServiceEndpoint `
    --target AzureDevOpsBuildDefinition `
    --output-format json > plan.json
```

Targets use registry type names, including `AzureDevOpsProject`,
`AzureDevOpsServiceEndpoint`, `AzureDevOpsBuildDefinitionReference`,
`AzureDevOpsBuildDefinition`, `Resource`, and `RoleAssignment`. Service connections
are called service endpoints in the current SDK model. Target names match the
registry's displayed type names exactly, including case. Repeat `--target` to request
several shapes. Unknown type names and known types with missing producers appear
as blocked targets with diagnostics. `ct registry entry list --requests-only`
shows the linked request signatures and effects.

Azure resources, Azure RBAC assignments, and Entra identities require a separate,
explicit tenant scope. An organization alone does not supply that scope:

```powershell
ct dump plan --org example `
    --tenant 11111111-1111-1111-1111-111111111111 `
    --auth-source azure-cli `
    --target AzureDevOpsServiceEndpoint `
    --target Resource --target RoleAssignment `
    --diagram cloud-plan.excalidraw
```

`--tenant` and `--org` supply independent scopes:

- `--tenant` supplies the authentication scope for Azure resources, Azure RBAC,
  and Entra identities.
- `--org` supplies the Azure DevOps organization URL for project, service
  connection, and build-definition inventory.

A plan targeting only Azure resources or Entra identities can omit `--org`.
Azure DevOps inventory still needs an explicit `--org`; supplying `--tenant`
does not discover or select any Azure DevOps organizations. Likewise, the planner
does not infer a tenant from an organization URL. Supplying both flags does not
establish a relationship between that organization and tenant.

Authentication selection is an input assumption in the plan; it is not checked
against local credentials or cloud permissions. `--auth-source none` withholds authentication
seed shapes, blocking requests that require those inputs, including the project
inventory. A DevOps personal access token does not provide Azure tenant
authentication. Tenant IDs found in service connection records do not implicitly
authorize another tenant inventory.

The planner understands declared request fields, not the authentication behavior
inside request implementations. For example, the current service-endpoint list
request invokes Azure CLI and has no authentication-context field. Its presence
in a plan does not establish support for the chosen authentication source. A live
runner will need an additional admission policy for such operations.

## What the graph establishes

The planner admits reads that do not solicit user input and pure synchronous
conversions/projections. Mutating requests, interactive selections, and Arbitrary
constructors cannot supply planned data. Effects are registry declarations, so
their accuracy remains part of the registry audit.

Starting from explicit seed shapes, the planner follows request outputs and
reflected children until structural reachability stops growing. Each request
needs every required input. Cancellation is supplied by the host; optional request
filters can be omitted.

Requests with no required domain inputs cannot start discovery automatically.
This includes unit inputs and requests containing only cancellation or optional
fields. Their complete request input shape must be supplied explicitly to the
planner, so an operation that consults ambient configuration or defaults cannot
silently become a discovery root. The current CLI seeds organization and
authentication shapes; it does not offer arbitrary request seeds.

The returned graph keeps one deterministic derivation for reachable targets and
candidate producers with missing inputs for blocked targets. It excludes unrelated
branches.

Collection elements, optional values, and enum variants have conditional edges.
For example, a build definition can expose its process's YAML filename only when
that optional property is present. A reachable type does not promise a nonempty
service response, successful conversion, API permission, or executable request
count. The graph also does not establish that independently discovered IDs belong
to the same organization, project, or tenant. A future live runner must retain
that provenance when binding inputs.

The existing `DiscoverySimulation` complements this structural view by exercising
registered Arbitrary response constructors and conversions offline. Simulation
samples values; planning inspects shapes. Neither is a live inventory dump.

## Current coverage gaps

The registered project inventory unlocks project arguments, service endpoints,
build definition references, and full build definitions. Tenant authentication
unlocks resource, role assignment, user, group, and service principal inventories.
`RoleAssignment` represents Azure RBAC; `UnifiedRoleAssignment` represents Entra
directory roles.

Repository listing currently exists as a free SDK function, without a registered
repository-list request. `AzureDevOpsRepo` therefore has no discoverable inventory
route. The `AzureDevOpsBuildRepository` record embedded in full build definitions
is a separate, shallow model that can describe non-Azure repository providers.

Repository file retrieval and recursive YAML checkout/template analysis do not
yet have registered traversal operations. The planner cannot establish pipeline
service connection usage from those files. It also does not consume `source hunt`
fetch URLs or produce the eventual Power BI cloud inventory payload. Those data
relationships remain work for the live dump and analysis features.

The existing Terraform-oriented `dump-everything` command remains separate from
`dump plan`.
