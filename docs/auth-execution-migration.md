# Authentication selections and execution

The workspace uses `cloud_terrastodon_auth` for runtime credentials and
`cloud_terrastodon_auth_types` for serializable authentication intent. The old
`credentials` crate/import names and context-bearing SDK request constructors
have no compatibility exports. See the [authentication guide](authentication.md)
for the concrete account CLI and source-specific provisioning.

## Ownership and request data

| Responsibility | Owning surface |
| --- | --- |
| CLI source preference | `auth_types::AuthSource` |
| Shared authenticated-leaf parsing and metadata resolution | `auth_cli::AzureAuthenticationArgs`, `AzureDevOpsAuthenticationArgs` |
| Stable CT account identity and source configuration | `CloudTerrastodonAccountId`, account profile/namespace/credential types |
| Serializable request authentication | `AuthenticationSelection`, `AzureBearerAuthenticationSelection`, `AzureDevOpsAuthSelection` |
| Credential sessions and account catalogue | `auth::CloudTerrastodonAccountStore`, `CredentialAuthenticationProvider` |
| Execution, cancellation, cache, and injected provider | `execution::CloudTerrastodonExecutor` |
| Service identities | `AzureTenantId` in `azure_types`, organization/collection URLs in `azure_devops_types` |

A request carries its stable account ID, concrete source and authority where
needed, operation scope, and input. It has no credential context, token cache,
cancellation scope, or credential-generation handle. Aliases and defaults are
resolved at the application boundary before request serialization, approval, or
execution.

Authenticated leaves flatten the relevant auth argument schema. Its resolver
reads tenant/account metadata and returns a concrete selection; it neither
acquires credentials nor creates a differently bound executor. Omitted account
arguments resolve only an explicitly configured namespace default. Source `auto`
resolves the account's preferred registered mechanism. These conveniences never
appear in a serialized request. Source `none` rejects service operations before
configuration lookup.

For Azure DevOps, a supplied tenant selects the Entra account namespace and a
bearer mechanism. Without a tenant, selection uses the organization/collection
namespace and PAT authentication. Explicit bearer sources require `--tenant`
(which can be `default`); PAT and tenant cannot be combined. Organization/project
scope stays on the leaf schema and request independently of authentication.

## Executing requests

Prefer constructing a request and calling:

```rust,ignore
request.execute(&executor).with_invalidation(clean).await
```

`executor.execute(request)` remains equivalent. Meaningful workflows compose
request objects directly; forwarding functions that only construct them are
removed. Requests declare their canonical output and error types through
`ExecutableRequest`, retaining domain errors such as selection errors.

Cached decoded results use `Arc` ownership. Collection consumers borrow or
clone elements deliberately rather than forcing the SDK to copy entire cached
responses. Raw command/HTTP artifacts and decoded in-memory reuse share the
execution/cache lifecycle.

The application installs one `CredentialAuthenticationProvider::configured()`
or `from_store(store)` through `App::authentication_provider`. Construction does
not open the catalogue or acquire credentials. The same provider and work cache
serve every request; credential sessions are acquired lazily by stable account
ID and concrete source. A request never depends on a CLI calling a binding
setter first. Metadata resolution and credential acquisition validate the
selected namespace and mechanism rather than changing to another account.

One Entra user account can have independently provisioned CLI and browser
mechanisms after login verifies the same principal. Source preferences and
defaults do not change requests already resolved to concrete selections. Cache
partitions include the admitted session identity, while recorded dependencies
connect composed work to its actual child requests.
Azure CLI, browser, PAT, and workload-identity partitions remain isolated to the
invocation: a stored credential generation does not enable response reuse across
separate CT runs. Within an invocation, requests for the same account/source
share its loaded session, token cache, and eligible work results.

Account admission happens before result reuse. Service-bound PAT accounts also
validate the request organization/collection before semantic or REST cache
lookup. Entra tenant identity remains separate from Azure DevOps organization
scope. Gitea validates its configured instance and supplies an immutable private
tea session snapshot.

## Leases and cancellation

Authenticated requests retain a provider lease for their complete execution,
including response decoding, artifacts, and affected-read invalidation. The
shared admission coordinator covers uses of the same captured credential session.
`ExecutableRequest::bind_authentication` is an internal admission/lease hook;
it does not select or mutate an executor's current account.

Account retirement closes new admission, requests cancellation of optional work
through independent linked child signals, and drains required completion.
Retirement never cancels unrelated parent work. An already admitted deferred
mutation keeps its original credential binding through its remaining work;
starting a new operation from the unleased provider fails.

Direct authorization, Command/REST execution, and the executor's cached/uncached
work APIs also retain account leases and observe retirement. Direct work binding
claims canonical ownership synchronously; first polling acquires its account lease
until the future finishes or is dropped. Unpolled work cannot delay logout or
return a warm result after retirement. Its initializer uses the account-linked
child scope. Existing cancellation policies remain authoritative: cancellable reads
may exit early, while deferred units retain response/decoding/artifact completion.
Coordinators must continue polling those children.

Retirement invalidates authentication partitions before and after draining.
Parent dependency generations advance even across account boundaries. Already
returned values remain readable. This guarantees coordination among owners
sharing the provider/cache, not arbitrary separate processes.

Operations needing only cancellation still take a cancellation scope.
Local formatting, filesystem work, and ordinary uncached subprocesses do not
need an executor merely to obtain cancellation.

## Registration and offline tests

Register executable requests through
`registry_execution::register_executable_request!`. Typed registry handles
retain the same request execution path and declared effects. Runtime test
registrations select real requests by pattern through `ct test run`; they are
separate from effect metadata and ordinary Cargo tests.

`execution::AnonymousAuthenticationProvider` admits anonymous work and rejects
credential use. With the `test-support` feature, use
`UnreachableAuthenticationProvider` when any auth call is a test failure, or
`TestAuthenticationProvider` for synthetic admission, counters, and credentials.
No fixture should resolve real account configuration or contact production
services.

## Existing REST behavior retained

Policy remediation uses typed ARM requests and retains mutation responses and
artifacts. Submission does not wait for every remediation deployment to finish.
See [Policy Insights](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01).

The storage backend picker inventories containers through the management-plane
[Blob Containers List API](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01).
It does not read Terraform state blobs or establish data-plane permission.
Azure CLI remains an optional account credential source, with isolated login and
token acquisition, rather than an SDK service-command bridge.
