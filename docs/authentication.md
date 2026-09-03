# Authentication

Cloud Terrastodon uses one shared authentication boundary for Graph, Azure
Resource Manager (ARM), and Azure DevOps REST requests. The resource audience
is selected by the request; callers do not pass tokens between services.

## Selecting an authentication source

Use the global option on any command:

```text
ct --auth-source auto <command>
```

The supported values are:

- `auto` (the default): use workload identity when configured and otherwise use
  the Azure CLI compatibility source. It never starts a new browser login;
  use `ct az tenant login <tenant>` or select `browser` explicitly for that.
- `workload-identity`: require the pipeline workload-identity inputs and use a
  short-lived Entra bearer token.
- `browser`: use the delegated browser/PKCE flow for shared REST requests to
  Graph, ARM, and Azure DevOps, plus PIM operations. Browser-backed REST
  requests use the tenant from the stored browser session when one is
  available; otherwise callers must provide an explicit tenant so the correct
  Entra authority can be selected.
- `azure-cli`: use the locally signed-in Azure CLI as a compatibility source.
- `pat`: explicitly use the Azure DevOps PAT compatibility source. This is not
  a workload-identity flow and is not a pipeline requirement.

`--debug` controls diagnostics only; it does not select authentication.

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
variables, or non-terminal standard input), Cloud Terrastodon will not launch
`az login`, device code, or browser authentication because a WIF configuration
is missing. A missing CLI session therefore fails immediately; use a WIF
service connection for pipelines.

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
together, while each access-token exchange remains resource-specific. Commands
that need a different tenant can pass `--tenant <tenant-or-alias>`. PIM's
existing Windows Credential Manager path remains compatible. To use browser
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

## Azure CLI and PAT compatibility

Azure CLI and PAT-backed paths remain for local compatibility and follow-on
commands. They are not required by the WIF audit targets. The first migration
removes direct Azure CLI request execution from the five Azure DevOps request
families used by `ct audit azure-devops`; the audit command also passes its
entrypoint context through every Graph, ARM Resource Graph, and Azure DevOps
request (including parallel and paginated work). Other CLI-backed commands are
tracked in the local secretless-authentication plan. The typed `ct az devops
project list` path now receives the same invocation context on every paginated
request.

In a workload-identity pipeline, pass `--tenant` to an audit command when the
tenant cannot be inferred from the service-connection environment, and pass
`--org` for Azure DevOps when no organization default is configured. This avoids
depending on a signed-in Azure CLI account for tenant or organization metadata.
For raw `ct rest` ARM requests, an explicit `--tenant` wins; otherwise the
tenant carried by workload identity or the stored browser session is used
before the legacy tracked-subscription fallback.

## Linux browser-flow acceptance checklist

Run this checklist from the WSL terminal after building Cloud Terrastodon. Use
an organization URL appropriate to your environment; do not put credentials in
the command line.

1. With no Azure CLI session and no stored browser session, run
   `ct az devops project list --org <organization-url>`. The command should
   fail quickly at authentication and must not launch device-code or `az login`.
2. Run `ct az tenant login agr`. The command should open (or print) the Entra
   authorization URL and show the dynamically selected localhost callback port.
3. If the host browser cannot reach WSL directly, forward that port in VS Code
   and open the printed URL. After the callback completes, rerun the project
   list command; the stored browser refresh token should obtain the Azure DevOps
   bearer token without Azure CLI.

## Pipeline prerequisites

Before live validation, the pipeline owner must verify the single app
registration/service connection's federated credential, tenant, ARM scope,
Graph permissions, Azure DevOps organization access, and licensing. Those
tenant and service-connection checks are intentionally not performed by local
build/test work.
