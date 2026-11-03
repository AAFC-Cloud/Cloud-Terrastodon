# Authentication selections and execution

This describes the workspace changes for the next release.
For source selection, WIF configuration, browser/PIM setup, and logout, see
the [authentication guide](authentication.md).

## Crates and imports

`cloud_terrastodon_credentials` has been renamed to `cloud_terrastodon_auth`.
Serializable authentication choices belong to `cloud_terrastodon_auth_types`:

| Responsibility | Crate and types |
| --- | --- |
| Startup preference | `auth_types::AuthSource` |
| Resolved request selection | `auth_types::AuthenticationSelection`, `AzureDevOpsAuthSelection`, and `AzureBearerAuthSource` |
| Credential audience | `auth_types::AzureCredentialResource`, replacing `AzureRestResource` |
| Credential acquisition and invocation-owned state | `auth::AuthContext`, `AzureTenantAuthContext`, and `AzureDevOpsAuthContext` |
| Bound execution and injected authentication contract | `execution::CloudTerrastodonExecutor`, `ExecutableRequest`, and `AuthenticationProvider` |
| Tenant identity | `azure_types::AzureTenantId` |

The facade exposes `cloud_terrastodon::auth` with feature `auth`, and
`cloud_terrastodon::auth_types` with feature `auth-types`. Update direct crate
dependencies and imports too; the former crate, feature, and facade names have
no compatibility exports. `app-auth` continues to enable authentication in the
application runner.

Request selections contain the source and tenant, not tokens or credential
caches. The runtime admits those selections against its owned authentication
state. Acquired `auth_types::Authorization` values are sensitive and have no
serialization implementation.

## Binding execution resources

`cloud_terrastodon_execution` supplies the executor and request contract. The
facade exposes them through `cloud_terrastodon::execution` with feature
`execution`. Inside an app handler, `context.executor()` binds its current
cancellation scope, selected authentication, and invocation-owned cache. New
requests implementing `ExecutableRequest` support either
`executor.execute(request).await` or `request.execute(&executor).await`.

Repeated app bindings share cache ownership and the resolved credential session.
Binding does not discover credentials, acquire tokens, or read cache files.
Without app authentication support, the executor uses a disabled provider that
permits anonymous work. The existing public `context.auth` and
`context.cancellation` fields remain available while SDK consumers migrate;
new bindings use their current values, and existing executor handles keep their
original binding.

The executor depends on the provider interface, while
`auth::CredentialAuthenticationProvider` supplies the implementation from an
already-selected `AuthContext`. Admission checks source/tenant policy before
cache reuse; acquiring credentials is separate sensitive infrastructure.
Registry invocation and production Command/REST adapter migration remain later
steps in the [executor/cache plan](plans/executor-cache-migration.md).

## Choosing a source and tenant

`--auth-source auto` prefers configured workload identity and otherwise selects
Azure CLI. A tracked tenant's preference can refine an automatic selection;
an explicit `--auth-source` takes precedence. `--auth-source none` disables
credential discovery and authenticated requests for local operations.

Bearer authentication always has a concrete tenant, including Azure CLI.
Azure DevOps commands accept a tenant UUID or tracked alias through `--tenant`;
omitting it requires an already tenant-bound context, such as configured
workload identity. Azure CLI's active tenant is not a bearer-authentication
fallback. Organization and project selection are independent of this tenant.

PAT authentication has no tenant requirement. The runtime supplies the stored
credential; scoped commands still require their organization and project:

```powershell
ct az devops project list --org example-org --auth-source pat
```

## Cloud Terrastodon's Azure CLI login store

Establish a login for the selected tenant explicitly:

```powershell
ct az tenant login example-tenant --auth-source azure-cli
ct az devops project list --org example-org --tenant example-tenant --auth-source azure-cli
```

Replace `example-tenant` with a tracked alias or tenant UUID. `--auth-source` is
a global option and can also appear before `az`. The default tenant-login flow
uses browser/PKCE; `--auth-source azure-cli` selects CLI login explicitly.

The login is stored beneath Cloud Terrastodon's configuration directory in
`azure-cli-sessions`. It persists between invocations and uses its own
`AZURE_CONFIG_DIR`; a separate `az login` does not populate this store.
Token acquisition uses the selected stored login and never starts an implicit
login. If no usable session exists, run the explicit tenant-login command.
The existing headless policy still rejects Azure CLI authentication; pipelines
can use workload identity.

Each explicit login creates a new generation. An authentication context binds
to one generation per tenant, and clones share that binding. A later login does
not replace the identity used by work already in progress. Credential
subprocess output bypasses general command artifacts. The logout command also
revokes Cloud Terrastodon's stored CLI generations.

## Migration scope

Policy remediation and the storage-backend picker execute typed ARM REST
requests with the selected authentication context. The auth crate no longer
provides a generic CLI service runner or a service-command allowlist. Azure CLI
remains an optional credential source through the isolated login/token lifecycle.

Remediation creates or updates a job at the selected resource-group or
management-group scope using [Policy Insights](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01).
The operation retains its mutation response and execution artifacts; submission
does not wait for all policy-driven deployments to finish.

The backend picker shares the SDK's container-names request and invalidates its
cached pages on reload. It uses the management-plane
[Blob Containers List API](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01),
with the existing ARM credential audience. This inventories container resources;
it does not read blobs or establish permission/network access to Terraform state
data. That differs from the former Blob-service CLI listing and is an intentional
selection-workflow contract. Resource IDs supply the account, resource group,
and subscription explicitly.

Legacy SDK helpers that execute Azure CLI without an authentication-context
parameter still use their existing execution path. Their request migrations
remain in phases 5 and 6 of the [executor/cache plan](plans/executor-cache-migration.md).
The executor binding is available for migrated requests; the full Command/REST
cache cutover remains subsequent work. Existing SDK requests are not silently
switched to the new cache owner by adding that binding.
