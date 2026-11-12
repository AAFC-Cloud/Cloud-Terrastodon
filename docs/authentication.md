# Authentication

Cloud Terrastodon accounts identify a principal within a tenant or service
namespace. Each account has a stable `CloudTerrastodonAccountId`, a scoped alias,
registered credential mechanisms, and a preferred source. Requests retain the
resolved account ID, concrete source, and Entra tenant where applicable; the
executor's shared provider holds credentials and token caches.

Account aliases such as `admin` and `user` are scoped to an Entra tenant, an Azure
DevOps organization/Server collection for PATs, or a Gitea instance. Entra accounts
can acquire the Graph, ARM, Azure DevOps, Key Vault, and Container Registry
audiences supported by their provider. An organization or project remains request
scope, independently of the tenant that issues a bearer token.

## Registering and selecting accounts

Register an acquisition source, then explicitly provision its credential session:

```text
ct account add azure-cli --name admin --tenant <tenant-or-alias>
ct account login <account-id>
ct az devops project list --org <organization> --tenant <tenant-or-alias> --account admin
```

The `add` result includes the stable account ID. Authenticated command leaves own
`--account`, `--auth-source`, and their tenant selector; these are not global
arguments. `--account` accepts an ID or an alias within the command's explicit
tenant/service namespace. No account is selected merely because it is first or
is the only account available. The alias `default` is reserved, ignoring case.

```text
ct account list
ct account show <account-id>
ct account default set <account-id>
ct account default clear <account-id>
```

Defaults are deliberate, namespace-specific choices. An explicit `--account`
overrides the default; a missing, incompatible, retired, or removed account
fails without fallback. Changing a default does not change an already resolved
request. Defaults do not supply Azure DevOps `--org` or `--project`.

`--auth-source auto` uses the selected account's preferred registered mechanism;
it does not inspect environment variables, discover credentials, or pick an
ambient Azure CLI account. Explicit `azure-cli`, `browser`, `workload-identity`,
or `pat` selects that registered mechanism. An unavailable preferred or explicitly
selected source fails; another source is never tried implicitly.

`--auth-source none` rejects authenticated service operations before account or
tenant-default lookup. Local commands do not need authentication arguments.
Account provisioning and cleanup have their own explicit command surfaces.

Interactive Azure DevOps menu actions prompt for the organization, then the
authentication scope, account, and source. A blank tenant selects that
organization's PAT namespace; an explicit tenant ID or alias selects Entra.
Blank account/source choices use only that scope's configured account default
and its preferred source, with no probing or fallback between PAT and Entra.

Tenant tracking keeps aliases and an explicitly configured CT tenant default:

```text
ct az tenant add <tenant-id> --alias example
ct az tenant default example
```

Bearer requests always contain a concrete tenant, including Azure CLI requests.
An explicit `default` tenant selector resolves the CT setting; it never reads
Azure CLI's active subscription or tenant. Azure DevOps bearer commands require
`--tenant`; PAT commands omit it and resolve accounts in their organization
namespace.

## Several mechanisms for one account

An Entra user account can register both Azure CLI and delegated browser
authentication without changing its account ID:

```text
ct account credential add-browser <account-id> --tenant <tenant-or-alias> --client-id <application-client-id>
ct account login <account-id> --source browser
ct account credential prefer <account-id> --source browser
ct az resource-group list --tenant <tenant-or-alias> --account admin --auth-source azure-cli
```

`account credential add-azure-cli <account-id> --tenant <tenant-or-alias>` adds
the CLI mechanism to an existing browser account. Registration checks the
account's tenant. Login verifies the authenticated Graph `/me` object ID against
the principal already recorded for the account; a different user is rejected and
the candidate session is cleaned up. An unverified JWT claim is not sufficient
identity proof. Each mechanism has its own credential generation and token cache.

Changing the preferred source does not provision it. Login is still required
before use. Workload identities represent application principals and cannot be
added to an interactive user account. PAT and Gitea accounts retain one source;
their service namespace remains independent of Entra bearer authentication.

## Source-specific provisioning

### Azure CLI

`account add azure-cli` records the tenant. `account login` establishes a login
in an account-owned, generation-specific `AZURE_CONFIG_DIR` beneath the CT
configuration directory. Two accounts in the same tenant have separate stores.
A separate ambient `az login` does not populate them.

Token acquisition is noninteractive and verifies the captured CLI identity.
It cannot run account-changing service commands or select another login.
Explicitly provisioned sessions can be reused noninteractively in scripts and
CI. Requests never trigger login, including when that session has expired or is
missing. Workload identity remains available for application-based pipelines.

### Delegated browser

```text
ct account add browser --name admin --tenant <tenant-or-alias> --client-id <application-client-id>
ct account login <account-id>
```

The explicit login performs authorization code + PKCE with a dynamic localhost
callback port. The account owns its refresh credential; resource-specific access
tokens remain in memory. Windows stores the refresh credential in Credential
Manager. Unix uses an account-owned file with owner-only permissions.

Ordinary requests only redeem the captured refresh session. They never launch
another browser login or consult a tenant-wide PIM client setting. Refresh
rotation preserves the selected account; failed refresh does not switch users.
PIM activation uses the already selected delegated browser/Azure CLI account,
rather than acquiring a separate user identity.

`ct az pim setup` remains an app-registration discovery/permission-validation
tool. Supply the resulting client ID explicitly when registering a browser
account. Setup reports that registration command without writing tenant-wide client settings.

### Workload identity

```text
ct account add workload-identity --name automation --tenant <tenant-or-alias> --client-id <application-client-id> --assertion-file <path>
ct account login <account-id>
```

The registration records an explicit absolute assertion-file location and
tenant/client identity. Login exchanges the configured assertion to verify that
identity before activating the registration. Session acquisition
reads the selected assertion once and exchanges it for each required audience;
SDK requests do not rediscover environment inputs. Pipelines own obtaining and
refreshing the assertion file before a new invocation.

The Entra application/service principal is the account's remote identity.
Its RBAC, Graph permissions, Azure DevOps membership, licensing, and permissions
must be configured independently. Selecting workload identity does not create
delegated user rights.

### Azure DevOps PAT

```text
ct account add azure-devops --name admin --org <organization-or-server-collection>
ct account login <account-id> --pat-stdin
ct az devops project list --org <organization-or-server-collection> --account admin --auth-source pat
```

Pipe the PAT into the login command. Tokens are not command-line arguments and
interactive echoed standard input is rejected. Each PAT is stored independently
under its account and generation, never in list/show output or request JSON.

PAT requests must remain within the registered organization/collection.
Admission checks the target before result reuse or credential transmission,
including supported API-area hosts. REST does not automatically follow redirects
that could bypass that target check. A PAT does not require an Entra tenant.

### Gitea

```text
ct account add gitea --name user --instance <instance-url> --login <tea-login-name> --configuration <tea-config-file>
ct account login <account-id>
```

Supply a tea configuration containing exactly the selected login. Explicit
account login validates its name and instance using tea's local login metadata
and captures the configuration as a credential. It does not query the service.
Later commands use a private temporary configuration snapshot through
`XDG_CONFIG_HOME`, select the captured login explicitly, and clear overriding
tea/Gitea environment settings. Changes to the original external file cannot
switch an admitted request's identity.

The last retained snapshot owner removes its temporary credential file. Gitea
result reuse remains disabled under the existing external-provider policy.

## Identity renewal, logout, and removal

```text
ct account logout <account-id>
ct account logout <account-id> --source browser
ct account remove <account-id>
```

Logout without `--source` retires every registered source; supplying it retires
only that mechanism. Removal retires all sources. They do not log out ambient
Azure CLI accounts, revoke credentials remotely, or clear browser cookies.
Removal also deletes the registration and any default referring to it; reusing
its alias produces a new account ID.

Automatic browser/CLI token renewal retains an admitted identity. Explicit login
refuses to overwrite an active source; log it out first. A later Entra login
must verify the same tenant-local user or application principal before publishing
a replacement session. PAT and Gitea replacements still require a new account
because their flows do not establish principal continuity. An old request ID
cannot silently select a different principal.

First-login/import generations are durably marked as provisioning before
credential writes. They cannot authorize requests. Retrying an interrupted
first attempt cleans that generation; retired generations retain their identifier
so cleanup errors can be retried.

Embedded applications first call `executor.retire_account(account_id, source)`;
`None` retires all sources. Owners holding the provider directly can use
`CredentialAuthenticationProvider::retire` with their shared cache and scope,
or `retire_source` for one mechanism. The account logout/remove CLI commands
already perform this step before durable cleanup.
It closes new admission, cancels optional work, drains admitted deferred work,
and invalidates affected authentication partitions before and after draining.
Only then should the owner remove durable credentials through the account store.
Already returned `Arc` values cannot be retracted. Separate running processes
are outside this cache/lease coordination contract; durable retirement prevents
new invocations from reopening the retired stored generation.

## Explicit legacy import

```text
ct account import-legacy <new-account-id>
```

First register the intended source and scope. `--source` chooses a registered
mechanism; omission uses the preferred source. Import can adopt the previous
CT-owned Azure CLI tenant store, the matching old CT browser session, or the
legacy Azure DevOps PAT location. Ordinary startup never imports them. Browser
imports require the registered tenant and client to match. Gitea and workload
identity use their explicit acquisition configuration through `account login`.
Entra imports verify the authenticated principal before activation. The earlier
single-credential draft account catalogue is not silently relocated or upgraded;
an incompatible catalogue fails decoding.

The former `ct az tenant login`, `logout`, and `set-auth-source` commands have
been removed. Tenant aliases remain separate from account aliases.

## Application-supplied providers

Applications can implement `AuthenticationProvider` and install it through
`App::authentication_provider` or `CloudTerrastodonExecutor::new`. That replaces
credential acquisition and admission for the existing authentication selections.
It does not register additional services in the built-in account catalogue:
account namespaces, credential registrations, and source selections are closed
Azure/Entra, Azure DevOps, and Gitea enums, and token audiences are Azure-specific.

`GiteaCommandSession` is currently part of the execution contract because both
the authentication implementation and Gitea SDK use its retained tea configuration.
Moving it into the SDK would introduce a dependency cycle. Supporting independent
service adapters would require separating common account identity, defaults,
admission, retirement, and cache partitioning from each service's registration
schema and credential/session capabilities. The current provider injection API
does not yet provide that extension mechanism.

## Verification

Offline Cargo tests cover account selection, immutable snapshots, partition
isolation, destination validation, and cooperative retirement with synthetic
data. They do not acquire production credentials. Operator-selected service
verification belongs to `ct test run <registry-pattern>`; `--clean` requests
normal cache invalidation before execution. Without it, successful cached reads
are not evidence of current live API compatibility.

See [authentication selections and execution](auth-execution-migration.md) for
the SDK contract and crate responsibilities.
