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

- `auto` (the default): use a complete workload-identity environment in a
  headless context; otherwise preserve the local compatibility behavior.
- `workload-identity`: require the pipeline workload-identity inputs and use a
  short-lived Entra bearer token.
- `browser`: reserve the delegated browser/PKCE flow for interactive user
  operations such as PIM.
- `azure-cli`: explicitly use the locally signed-in Azure CLI as a compatibility
  source.
- `pat`: explicitly use the Azure DevOps PAT compatibility source. This is not
  a workload-identity flow and is not a pipeline requirement.

`--debug` controls diagnostics only; it does not select authentication.

In an automatically detected headless context (`CI`, Azure DevOps build
variables, or non-terminal standard input), Cloud Terrastodon will not launch
`az login`, device code, or browser authentication because a WIF configuration
is missing. Select a source explicitly only when that behavior is intentional.

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
signed-in user, and its ARM role assignments and application permissions are
used for pipeline authorization. Local delegated flows continue to act as the
user.

## Delegated PIM boundary

PIM remains an interactive, delegated-user workflow. The first pipeline
milestone does not activate PIM or turn a service principal into a user. A
pipeline that needs PIM must use a separately designed and authorized flow.

## Azure CLI and PAT compatibility

Azure CLI and PAT-backed paths remain for local compatibility and follow-on
commands. They are not required by the WIF audit targets. The first migration
removes direct Azure CLI request execution from the five Azure DevOps request
families used by `ct audit azure-devops`; other CLI-backed commands are tracked
in the local secretless-authentication plan.

## Pipeline prerequisites

Before live validation, the pipeline owner must verify the single app
registration/service connection's federated credential, tenant, ARM scope,
Graph permissions, Azure DevOps organization access, and licensing. Those
tenant and service-connection checks are intentionally not performed by local
build/test work.
