# Authentication

Cloud Terrastodon uses one shared authentication boundary for Graph, Azure
Resource Manager (ARM), and Azure DevOps REST requests. The resource audience
is selected by the request; callers do not pass tokens between services.

For the next-release crate/import cutover and the scope of the executor work,
see [authentication selections and execution](auth-execution-migration.md).

## Selecting an authentication source

Use the global option on any command:

```text
ct --auth-source auto <command>
```

The supported values are:

- `auto` (the default): use workload identity when configured and otherwise use
  the application-owned Azure CLI source. It never starts a new browser login;
  use `ct az tenant login <tenant>` or select `browser` explicitly for that.
- `none`: disable credential discovery and refuse authenticated requests that
  use the invocation's `AuthContext`. Use it for local-only operations such as
  group-management validation.
- `workload-identity`: require the pipeline workload-identity inputs and use a
  short-lived Entra bearer token.
- `browser`: use the delegated browser/PKCE flow for shared REST requests to
  Graph, ARM, and Azure DevOps, plus PIM operations. Browser-backed REST
  requests use the tenant from the stored browser session when one is
  available; otherwise callers must provide an explicit tenant so the correct
  Entra authority can be selected.
- `azure-cli`: use an explicitly selected tenant's login in Cloud Terrastodon's
  persistent Azure CLI store. A separate `az login` does not populate this store.
- `pat`: explicitly use the Azure DevOps PAT compatibility source. This is not
  a workload-identity flow and is not a pipeline requirement.

`--debug` controls diagnostics only; it does not select authentication.

Selecting `none` creates `AuthContext::None`, an explicitly disabled policy without
inspecting workload-identity environment variables or discovering stored browser
sessions. AuthContext-governed Graph, ARM, and Azure DevOps operations reject it,
including REST requests supplied an explicit bearer token. Default-tenant lookup
and tenant login also reject it instead of consulting a signed-in Azure CLI
account. Consumer CLIs can select this policy before `App::run_parsed` for
local-only subcommands.

`AuthContext::Unresolved` is the inert placeholder used by default and generated
request construction. It means no invocation policy has been attached yet;
it is distinct from explicitly disabling authentication with `AuthContext::None`.
The explicit `None` variant carries no credential configuration or token caches.

This policy governs operations using the supplied `AuthContext`; it does not
change independent credential APIs invoked directly without that context or
Gitea's separate authentication.

## Per-tenant authentication defaults

Tracked tenants can have their own default source. Set it while adding a
tenant:

```text
ct az tenant add <tenant-id> --auth-source browser
```

Or update an existing tenant, using its id or alias:

```text
ct az tenant set-auth-source <tenant-or-alias> browser
```

The tenant setting is used when the global source is `auto`; an explicit global
`--auth-source` takes precedence. Set the tenant source to `auto` to restore
automatic selection.

In an automatically detected headless context (`CI`, Azure DevOps build
variables, or non-terminal standard input), Cloud Terrastodon rejects Azure CLI
authentication even when a stored login exists. It does not launch device code
or browser authentication when WIF configuration is missing. Use a WIF service
connection for pipelines.

## Explicit tenant and Azure CLI sessions

Every bearer request carries a concrete tenant, including requests authenticated
through Azure CLI. Azure DevOps commands accept `--tenant <tenant-id-or-alias>`;
omitting it requires a tenant already carried by workload identity or a selected
browser session. Azure CLI's active tenant is not an authentication fallback.
PAT authentication is tenant-independent. Organization and project selection
remain separate from authentication.

Establish the tenant's CLI login explicitly:

```text
ct az tenant login <tenant-id-or-alias> --auth-source azure-cli
ct az devops project list --org <organization-url> --tenant <tenant-id-or-alias> --auth-source azure-cli
```

Cloud Terrastodon stores this login under its configuration directory in
`azure-cli-sessions`, using an isolated `AZURE_CONFIG_DIR`. The store persists
between invocations. Token acquisition does not start a login; if the selected
session is missing or unusable, run the explicit tenant-login command. The
default tenant-login flow remains browser/PKCE; choose `azure-cli` explicitly
for this store.

Each login creates a new generation. An authentication context and its clones
retain their tenant's selected generation, so a later login does not replace
the identity used by work already in progress. Credential subprocess output
bypasses the general command artifacts. The logout command revokes the stored
CLI generations as well as clearing the other sources described below.

## Workload identity environment variables

Both environment naming conventions are accepted. Use one complete set, and do
not set conflicting values:

| Azure SDK-style | Azure DevOps task-style | Meaning |
| --- | --- | --- |
| `AZURE_CLIENT_ID` | `servicePrincipalId` | Entra application/client ID |
| `AZURE_TENANT_ID` | `tenantId` | Entra tenant ID |
| `AZURE_FEDERATED_TOKEN` | `idToken` | OIDC client assertion |

The assertion is exchanged for a resource-specific bearer token. The current
resource audiences are Graph, ARM, and Azure DevOps; assertions and access
tokens are never included in logs, cache keys, or command summaries.

The pipeline identity is the service connection/app registration. It is not a
signed-in user. ARM access is governed by that identity's RBAC assignments,
Graph access by its Entra application permissions, and Azure DevOps access by
the service principal's explicit organization membership, licensing, and
organization permissions. Local delegated flows continue to act as the user.

## Delegated browser boundary

Browser authentication is an interactive, delegated-user workflow. Access
tokens are resource-specific: a Graph token cannot be reused for ARM or Azure
DevOps. PIM remains delegated and interactive; the first pipeline milestone
does not activate PIM or turn a service principal into a user. A pipeline that
needs PIM must use a separately designed and authorized flow.

The browser provider caches access and refresh tokens only inside the explicit
`AuthContext` for the current invocation. It does not write browser tokens to
the command cache or logs. `ct az tenant login <tenant-or-alias>` performs the
interactive login and persists only the refresh token: on Linux it is written
to a user-only (`0600`) file, while Windows uses Credential Manager. Subsequent
commands can select the browser source explicitly or through a tenant default.
The initial browser consent requests the delegated Graph, ARM, and Azure DevOps scopes
together, while each access-token exchange remains resource-specific. A browser
context stays bound to its admitted tenant and client. To switch tenants, run
`ct az tenant login <tenant-or-alias> --auth-source browser`, then start a new
invocation with that tenant. If refresh requires reauthentication, explicitly
log in and start a new context; the existing context never prompts for another
account.
PIM's existing Windows Credential Manager path remains compatible. To use browser
authentication by default for a tracked tenant, configure its per-tenant
authentication default.

The callback listener binds to a dynamically chosen `localhost` port. In WSL,
VS Code can forward that port to the host browser when automatic port
forwarding is enabled; if it does not, the command prints the authorization URL
and the forwarded port can be opened manually.

The browser flow needs the public client's app registration ID before its first
login. Bootstrap a known registration without Azure CLI or Graph authentication:

```text
ct az pim setup --tenant <tenant-or-alias> --client-id <application-client-id>
ct --auth-source browser az tenant login <tenant-or-alias>
```

Supplying `--client-id` only persists the tenant-to-client-ID mapping; it does
not verify the registration or its delegated permissions. Omitting it preserves
the authenticated discovery and permission-validation setup path.

## Logging out

```text
cloud_terrastodon.exe az tenant logout
```

This logs out across all tenants and organizations, regardless of `--auth-source`.
It removes the stored browser session and, on Windows, all Cloud Terrastodon
browser and legacy PIM refresh tokens from Credential Manager. It revokes Cloud
Terrastodon's application-owned CLI generations and clears their credentials,
retaining the revoked directories so existing contexts fail on their next use.
It also runs
[`az logout`](https://learn.microsoft.com/en-us/cli/azure/reference-index#az-logout)
for the active account,
[`az account clear`](https://learn.microsoft.com/en-us/cli/azure/account#az-account-clear)
to log out the remaining cached users and service principals, and
[`az devops logout`](https://learn.microsoft.com/en-us/cli/azure/devops#az-devops-logout)
without an organization filter. Every source is attempted even if
another fails; any failure is reported after cleanup. Azure CLI and its Azure
DevOps extension must be installed for those logout steps to succeed.

Logout preserves tracked tenants, aliases, and PIM client registrations. It does
not revoke tokens remotely or clear browser cookies. Credentials supplied through
environment variables, including PATs and workload identity assertions or token
files, remain available to future processes until removed from the calling shell
or pipeline. Logout bypasses credential discovery so a broken authentication
configuration does not prevent cleanup.

## Azure CLI and PAT compatibility

Azure CLI and PAT-backed paths remain for local compatibility and follow-on
commands. They are not required by the WIF audit targets. The first migration
removes direct Azure CLI request execution from the five Azure DevOps request
families used by `ct audit azure-devops`; the audit command also passes its
entrypoint context through every Graph, ARM Resource Graph, and Azure DevOps
request (including parallel and paginated work). Other CLI-backed commands are
tracked in the local secretless-authentication plan. The typed `ct az devops
project list` path now receives the same invocation context on every paginated
request. Policy remediation and storage-backend selection use typed ARM REST
requests with that same selected authentication context. The backend picker
lists management-plane container metadata; listing does not establish access to
the blobs themselves. The CLI service-command bridge has been removed, while
the owned CLI login store remains available for token acquisition. See the
[migration scope](auth-execution-migration.md#migration-scope) for the remaining
legacy paths and the inventory behavior change.

In a workload-identity pipeline, pass `--tenant` to an audit command when the
tenant cannot be inferred from the service-connection environment, and pass
`--org` explicitly for Azure DevOps. Project-scoped commands also require
`--project`; neither selector falls back to configured Azure DevOps defaults.
This avoids depending on a signed-in Azure CLI account for tenant or organization
metadata.
For raw `ct rest` ARM requests, an explicit `--tenant` wins; otherwise the
tenant carried by workload identity or the stored browser session is used
before the legacy tracked-subscription fallback.

## Linux browser-flow acceptance checklist

Run this checklist from the WSL terminal after building Cloud Terrastodon. Use
an organization URL appropriate to your environment; do not put credentials in
the command line.

1. With no application-owned Azure CLI session, run
   `ct az devops project list --org <organization-url> --tenant <tenant-or-alias> --auth-source azure-cli`.
   The command should fail quickly at authentication and must not launch
   device-code or `az login`.
2. Run `ct az tenant login <tenant-or-alias> --auth-source browser`. The command
   should open (or print) the Entra authorization URL and show the dynamically
   selected localhost callback port.
3. If the host browser cannot reach WSL directly, forward that port in VS Code
   and open the printed URL. After the callback completes, rerun the project
   list command with `--auth-source browser` and the same explicit tenant; the
   stored browser refresh token should obtain the Azure DevOps bearer token
   without Azure CLI.

## Pipeline prerequisites

Before live validation, the pipeline owner must verify the single app
registration/service connection's federated credential, tenant, ARM scope,
Graph permissions, Azure DevOps organization access, and licensing. Those
tenant and service-connection checks are intentionally not performed by local
build/test work.
