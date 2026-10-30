# Executor and cache public contracts

**Decision record:** G1–G4 for [the migration plan](executor-cache-migration.md).
These are implementation contracts, not a claim that the migration is implemented.
The [coordination contract](executor-cache-coordination.md) supplies G5/G6, including
publication, dependencies, invalidation, and persistence. The
[migration inventory](executor-cache-migration-inventory.md) records affected callers.
The owning plan records compilation evidence for the separate offline contract
sketch; the signatures below describe the selected API.

## G1 — package responsibilities and dependency direction

The concrete `CloudTerrastodonExecutor` belongs in a new
`cloud_terrastodon_execution` crate. It owns capabilities through interfaces and
does not import credential implementations. The credentials crate supplies
`CredentialAuthenticationProvider`, which implements the execution crate's
`AuthenticationProvider` interface. Application startup constructs the provider
and injects it into the executor. A separate higher-level executor crate and a
second wrapper around the executor are unnecessary.

The following table lists the new edges and relevant existing edges. An arrow
means the left package depends on the right package; other existing workspace
dependencies remain as recorded in their manifests.

| Package | Required dependency direction | Responsibility |
| --- | --- | --- |
| `registry_values` | Existing pinned external reflection/registration dependencies only | Value reflection, owned erased values, and pure arbitrary registration |
| `auth_types` | `registry_values` | Credential-selection vocabulary and tenant identity; no discovery or token acquisition |
| `cache` | `registry_values`, `pathing` | Explicit cache owner, typed result ownership, artifacts, generations, and dependencies |
| `execution` | `cache`, `auth_types` | Executor, executable request contract, bounded execution, typed credential capability |
| `registry` | `registry_values`, `execution` | Executable invocation, effects, discovery, and adapters for pure value registrations |
| `command` | `execution`, `cache`, `registry` | Subprocess adapter and its execution policy |
| `credentials` | `execution`, `auth_types`, `command`, `registry` | Credential provider implementations and invocation-owned token caches |
| `rest` | `execution`, `cache`, `registry`; existing dependencies until their callers migrate | HTTP adapter, typed response decoding, and response artifacts |
| SDK and CLI crates | The lower interfaces/adapters they use | Operation procedures and application policy, respectively |
| `app` | `execution`; `credentials` only behind its auth feature | Bounded invocation ownership and resource construction |

The full package names use the `cloud_terrastodon_` prefix. Neither `cache` nor
`execution` depends on `command`, `credentials`, the high-level `registry`, or
`azure_types`. This prevents both credential/subprocess and registry/executor
cycles. New packages reuse the existing pinned dependencies; this decision does
not authorize dependency upgrades.

### Pure registry extraction

The current value registry cannot simply be imported by `auth_types`: executable
registry invocation will itself depend on `execution`, which needs the auth
vocabulary. Extract these responsibilities into `registry_values`:

- `RuntimeValue`, its owning-allocation operations and pointer bridge;
  `BorrowedPointerKind` and its distributed slice.
- `Thing`, `RegistrationSite`, the value-registration slice, boxed conversion
  helpers, and reflection helpers used by those types.
- `register_thing!` and `register_borrowed_pointer_kind!`.
- `ArbitraryBytes`, a pure `ArbitraryRegistration` descriptor, its distributed
  slice, and a low-level `register_arbitrary!` macro.

`ArbitraryRegistration` owns an output shape, a synchronous generator accepting
`&mut ArbitraryBytes`, its boxed-output-to-runtime conversion, and its registration
site. It has no executor, cancellation scope, authentication, or async invocation.
Executable `Function`, executable callback variants, effects admission, and
discovery stay in `registry`.

The high-level registry exposes the low value types by re-export and adapts low
arbitrary descriptors into its constructor catalog. An adapted entry retains
provenance to its authoritative descriptor. It is not a second distributed
registration. The current high-level `register_arbitrary!` and its existing
associated `&Function` handles remain available during migration. New low-level
types use the low-level macro and its descriptor handles; do not silently change
existing associated constants to a different type. Catalog traversal must include
both sources without duplicate registration or executing generators during
planning. This is an explicit integration task, not an assumption about the
current `KNOWN_FUNCTIONS` slice.

Task 2.0 implements the merged catalog as `KnownFunctions`, returned by
`known_functions()`. Its iterator chains original executable registrations and
persistent pure-descriptor adapters. It preserves existing associated handles
and their addresses; it no longer promises a contiguous slice. The raw
`KNOWN_FUNCTIONS` distributed slice contains only legacy registrations, so
discovery and lookup use the merged view.

### Owning files

Each production type gets its own file. The initial homes are:

| Home | Types/responsibilities |
| --- | --- |
| `crates/registry_values/src/` | `runtime_value.rs`, `thing.rs`, `registration_site.rs`, `borrowed_pointer_kind.rs`, `arbitrary_bytes.rs`, `arbitrary_registration.rs`; cohesive function-only reflection/registration modules |
| `crates/auth_types/src/` | `auth_source.rs`, `azure_bearer_auth_source.rs`, `azure_tenant_id.rs`, `azure_credential_resource.rs`, `authentication_selection.rs`, `azure_devops_auth_selection.rs`, `authorization.rs` |
| `crates/cache/src/` | `work_cache.rs`, `cache_scope.rs`, `cache_key.rs`, `executor_kind.rs`, `cache_operation_owner.rs`, `auth_partition.rs`, `cached_work_spec.rs`, and the coordination types selected in G5/G6 |
| `crates/execution/src/` | `cloud_terrastodon_executor.rs`, `executable_request.rs`, `request_execution.rs`, `authentication_provider.rs`, `disabled_authentication_provider.rs`, and invocation-observation types |
| `crates/credentials/src/` | `credential_authentication_provider.rs`; existing credential/session/token implementations |
| `crates/registry/src/` | Executable function metadata, lifetime-aware invocation, and executable registration adapters |

Move the existing `AzureTenantId` definition to its lower home and re-export it
from `azure_types`. Likewise move/re-export the pure `AuthSource` definition from
credentials. Do not introduce a second tenant-ID type with conversions between
otherwise identical values. Keep parsing, Facet representation, arbitrary
generation, and registrations beside the owning low-level definitions.

## G2 — request execution, scopes, and lifetime bounds

Requests contain operation data and serializable credential selection. They do
not contain authentication contexts, token caches, cache handles, observers,
`CancellationToken`, or `CancellationScope`. A bound execution value supplies
those resources and is awaitable.

Both public spellings dispatch through the same implementation:

```rust
executor.execute(request).await?;
request.execute(&executor).await?;
```

The selected trait shape is:

```rust
pub type ExecutionFuture<'a, T> =
    Pin<Box<dyn Future<Output = eyre::Result<T>> + Send + 'a>>;

pub trait ExecutableRequest: Sized + Send {
    type Output: Send;

    fn run<'a>(
        self,
        executor: CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a;

    fn execute<'a>(
        self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> RequestExecution<'a, Self>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        executor.execute(self)
    }
}
```

`RequestExecution<'a, R>` owns `R` and borrows the executor. Its `IntoFuture`
output is `eyre::Result<R::Output>` and its future is
`ExecutionFuture<'a, R::Output>`. On execution it clones the resource handles,
opens a linked child through the current scope's `cancellable_scope`, binds the
factory's child scope into the child executor, and calls `request.run(child)`.
Nested requests receive that child executor. No hidden current executor or
task-local resource fallback is introduced.

The executor is cheaply cloneable. Its private runtime fields protect ownership
of the provider, cache, bounded scope, and dependency lineage; these are concrete
resource/lifecycle responsibilities. Startup constructs it inside the existing
App handler's bounded scope. Executor construction does not manufacture a root
token, an unbound placeholder scope, or an independent request lifetime. Child
scope rebinding is internal; public `cancellable_scope`, `with_async_scope`,
`deferred`, and `finish_required` helpers pass correctly bound child executors to
their callbacks. Explicitly dropping a future still can abandon work; the executor
does not promise protection beyond the underlying cancellation contract.

The pinned cancellation API accepts factories of the form
`FnOnce(CancellationScope) -> F`, where `F: IntoFuture<Output = Result<T, E>>`
and `E: From<CancellationError>`. It does not impose `'static` callbacks or
outputs. The execution wrapper must preserve these lifetimes. It must also
preserve the distinction between protected new work and already-owed cleanup.
The initiating caller owns a shared read attempt; cancelling it cancels that
attempt, releases initializer ownership, and permits a surviving waiter to retry.
No detached cache-fill task is introduced.

An ordinary `cancellable_scope` creates a child ownership boundary while sharing
the parent's cancellation intent. Calling `request_cancel` on that scope is not
an isolated cancellation of one sibling. Tests of a surviving waiter must use
independent caller signals established by their owners, or explicitly drop the
initiating caller's future; they must not assume an ordinary child scope has an
independent token. Pinned cancellation version 0.7 provides
`cancellable_child_scope` and `with_async_child_scope` for independent child
signals behind its `inheritance` feature. If that facility is adopted, enable
the existing pinned feature explicitly. Root cancellation cancels all linked
callers and does not leave a waiter authorized to retry.

Only APIs retaining shared decoded values impose `T: Send + Sync + 'static`.
They return `Arc<T>`. Ordinary executable outputs remain `Send` and may borrow
data with a valid invocation lifetime. Cache initializer futures need not be
`'static`: the caller owns and awaits them. Fixed canonical decoding can use owned
inputs on a blocking worker, while per-caller callbacks remain outside that
worker and outside the cached operation.

### Canonical execution versus caller policy

An operation owns one canonical decoder and one reusable result contract.
`CachedWorkSpec` represents that executable/cache contract and binds to the
executor before awaiting. It is not a serializable request DTO: execution
closures and runtime ownership naturally belong to the bound work.
The low cache specification receives generic execution/decoding callbacks and
explicit cache/scope inputs through an execution-layer adapter; it does not
import `CloudTerrastodonExecutor` or a credential provider back into `cache`.
Authentication admission supplies an `AuthPartition` before that adapter asks
the cache to reuse an entry.

An arbitrary validator/mapper is caller policy. It runs for every caller after
obtaining the canonical result, including hits and joiners, and its output is
not automatically memoized. Different callbacks returning the same Rust type
must not be treated as equivalent canonical decoders. A caller's validator
failure does not invalidate a canonical result accepted by the operation's own
decoder. Canonical decoding failure retains raw artifacts and follows the shared
cache rejection policy.

Borrowed observers and validators may also be non-`Send`. They execute in a
per-caller wrapper around the `Send` request future, not inside `ExecutableRequest::run`
or a retained cache entry. Consequently that wrapper itself need not be `Send`.
Callers that spawn it are responsible for supplying `Send` callbacks. A projection
may return an owned caller-specific value or a value borrowing caller-owned data;
it does not acquire cache bounds merely because its input was an `Arc<T>`.

### Registry invocation

The registry's executable callback borrows the low-level executor directly:

```rust
pub type InvocationFuture<'a> =
    ExecutionFuture<'a, Box<dyn Any + Send>>;

pub type AsyncExecutionFn = for<'a> fn(
    Box<dyn Any + Send>,
    &'a CloudTerrastodonExecutor,
) -> InvocationFuture<'a>;
```

`FunctionInvocation<'a>` carries this pending future or an already-produced boxed
value. Executable dispatch accepts `&'a CloudTerrastodonExecutor` and returns
`Result<FunctionInvocation<'a>>`. Its generic adapter downcasts the request,
awaits `executor.execute(request)`, and boxes the result. The erased registry
boundary requires `R: ExecutableRequest + Any + Send + 'static` and
`R::Output: Facet<'static> + Any + Send + 'static`, but does not require `Sync`.
Those ownership requirements belong to boxed reflection, not every direct request.

Keep the current async-value callback as a temporary separate executor variant
for legacy requests/conversions. The new executable registration uses the new
callback and does not reflectively inject a cancellation field. Pure conversions
and arbitrary constructors remain callable without authentication or live
execution. UI workers that spawn an invocation own an executor clone and create
and await the borrowed invocation inside that worker.

## G3 — identity, ownership, and credential isolation

`ExecutorKind` has `Command`, `Rest`, `InProcess`, and `Other(String)` variants.
The cache centrally encodes their namespaces. An `Other` namespace cannot alias
a built-in kind or another distinct `Other` value after filesystem encoding.
Callers supply readable logical scope/operation keys; they do not prepend their
own transport namespace.

The ownership address is `(ExecutorKind, logical_operation_key)`. Before any
variant lookup, disk read, wait, or initialization, the cache claims/checks its
canonical operation owner and decoded output `TypeId`. The owner is a declared
operation/decoder contract with a stable in-process marker type. It is not the
caller's source location or the generic `RestRequest` adapter. Borrowed request
implementations may name an owned marker or their owned request specialization;
they do not need to make borrowed input `'static` just to identify the operation.

A different owner or output type for the same ownership address is an error,
even if its auth partition or request fingerprint differs. Check the contract
before selecting those variants. Do not add expected owner, output `TypeId`, or
caller location to the address to turn misuse into a second successful entry.
Keep type names and caller locations for diagnostics. Disk type metadata remains
diagnostic; raw disk payloads use the current operation's canonical decoder.

The concrete reusable entry additionally includes the admitted auth partition
and complete request fingerprint. Memory coordination and physical artifacts use
that same concrete identity. An SDK aggregate occupies `InProcess` while its
REST operation occupies `Rest`, even when both use the same logical key. This
handles the existing location-list wrapper/transport case without weakening
same-namespace ownership checks. Logical prefix invalidation traverses all
namespaces and variants in its selected scope. Physical layout and publication
are specified in G5/G6.

Process fingerprints retain executable, arguments, environment, working
directory, stdin, and adjacent inputs. REST fingerprints retain all semantic
request inputs. Credential material is never a printable key or artifact input.
Authentication selection and the provider's admitted identity distinguish
credential contexts without using a current access token as request identity.

### Serializable authentication selection

`AuthSource` retains application selection preferences, including `Auto` and
`None`. Executable bearer request data uses `AzureBearerAuthSource`, a selected
source vocabulary containing `AzureCli`, `WorkloadIdentity`, and `Browser`.
This is a distinct validated contract: a request cannot encode an unresolved
auto preference, disabled source, or PAT as a bearer source.

Principal and ARM requests keep explicit `auth_source` and `tenant_id` fields.
`AzureDevOpsAuthSelection` has a tenant-bearing bearer case and a
`PersonalAccessToken` case. There is no active-CLI-tenant case. PAT is explicitly
tenant-independent. Every Azure CLI bearer token subprocess receives the
request's tenant; account defaults never complete missing request data.

The provider interface uses `AuthenticationSelection` to represent anonymous
work, explicit tenant-bound bearer selection, or Azure DevOps PAT selection,
and an `AzureCredentialResource` audience. `Authorization` owns sensitive runtime
authorization data with redacted diagnostics. It is not part of request wire
data or a cache identity.

```rust
pub trait AuthenticationProvider: Send + Sync {
    fn admit(
        &self,
        selection: &AuthenticationSelection,
    ) -> eyre::Result<AuthPartition>;

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization>;
}
```

Admission is synchronous policy checking against already-owned provider state.
It checks disabled authentication, permitted source, explicit tenant, and
configured tenant agreement before any reusable cache hit. It does not fetch a
token or consult live account defaults. Credential acquisition runs only when
execution needs it. The current executor lets credential implementations retain
the correct child cancellation relationship when running subprocesses or HTTP
exchanges. Such work is explicitly credential infrastructure, not a semantic
inventory dependency. Infrastructure classification cannot hide a domain write
or user solicitation.

### Auth partitions and their lifetime

For workload identity, already-configured authority, tenant, and client identity
establish a stable partition. This enables persistent reuse for that configured
principal without acquiring a token solely to determine a cache hit. Source
policy and tenant agreement still apply before reuse.

Current browser session metadata identifies tenant and client, but not the
signed-in user. Azure CLI and PAT selection likewise do not establish the
principal from request data alone. These sources use a fresh random
provider-session identifier. Provider/executor clones share it; independently
created providers do not, even when they share a work-cache handle. Consequently
CLI, browser, and PAT disk results are not reusable across provider sessions in
the initial contract. This deliberately sacrifices that disk reuse rather than
allowing one account to reuse another account's inventory. An unpartitioned
legacy artifact cannot establish identity and is a miss.

A provider represents a credential session, not a live reference to whatever
external account happens to be selected later. Every acquisition, including a
new audience or tenant and a refresh, must preserve its established identity. A
changed or unverifiable identity requires a new provider/partition or an error,
not silent reuse under the old partition.

Authenticated Azure CLI service subprocesses must use the same provider-bound
session that supplied admission and the cache partition. A token-only check is
insufficient: an ordinary service command can otherwise read a different account
through mutable external CLI state. The selected contract requires an isolated
provider-owned CLI session with explicit tenant/account binding. Do not fall back
to an ambient CLI configuration directory. If a stable session cannot be
established, reject authenticated subprocess execution before it starts. Copying
configuration files alone is not proof of a stable credential snapshot. Task 3.1
must verify the actual provisioning mechanism against official platform behavior
and synthetic isolation tests before enabling this migrated path; this document
does not assert that a particular copy/login mechanism already meets it. External
CLI account switches cannot redefine an existing executor's identity.

No identity partition includes raw tokens, assertions, refresh tokens, PATs,
or secret-derived printable keys. A valid hit performs no token acquisition.
The auth-disabled provider permits anonymous/local work and rejects requests
requiring credentials before reuse. Local startup tests still need isolated
storage: disabled authentication does not prevent reading an existing disk cache.

Credential infrastructure also declares sensitive-output retention before work
starts. Token acquisition results stay in the provider's runtime token cache,
not the reusable work cache or raw artifact files. Successful or failed token
stdout, HTTP bodies, credential inputs, and token-bearing error text must not be
persisted or logged; retain only explicitly redacted diagnostic metadata. The
new shared lifecycle must not start archiving successful uncached token commands
that currently leave no success artifact. This is distinct from whether a child
participates in inventory dependency freshness.

## G4 — shared records, observation, and effect admission

Canonical inventory results use an outer `Arc` so hits and joiners share the
same allocation. Selected record ownership is:

| Inventory | Shared shape | Assembly/conversion consequence |
| --- | --- | --- |
| Graph category lists | `Arc<Vec<Arc<EntraUser>>>`, corresponding group/service-principal forms | Aggregate construction copies record handles |
| Principal inventory | `Arc<EntraPrincipalCollection>`; `EntraPrincipal` variants hold shared category records | Existing ID/name access remains borrowed; consuming conversions return shared records or explicitly clone owned records |
| Azure DevOps projects | `Arc<Vec<Arc<AzureDevOpsProject>>>` | Pages and final ordered list share project records |
| Combined RBAC | `Arc<RoleDefinitionsAndAssignments>` | The first validated assembly is shared; internal role/assignment record sharing follows the invariant-preserving conversion work in its migration slice |

These are targeted ownership changes, not a requirement to replace every nested
domain field with `Arc`. Facet/Arbitrary support, external JSON/text behavior,
registry collection projection, and CLI presentation must be verified through
the focused application integration checks. No generated service response
fixture is evidence of remote API compatibility.

Cached snapshots are shared immutable results. An explicit owned edit uses an
owned conversion or copy-on-write; mutation must not change the retained snapshot
visible to another caller. Do not expose mutable access through the cache. Preserve
domain validation when constructing a new edited value. A result remains usable
after cache eviction because its caller owns an `Arc`.

Invocation observers are runtime behavior, never request wire data or cache
policy. Every caller receives its own lifecycle notification: execution admitted,
hit or initialization/waiting status as applicable, and completion/failure. The
initializer can additionally report operation progress. Joiners receive waiting
and completion rather than fabricated page progress; memory hits do not pretend
to re-fetch pages. Observer callbacks are not retained in shared entries and are
not silently skipped on hits. A local wrapper may receive events from the `Send`
execution future while invoking a borrowed, non-`Send` callback in the caller's
task. It must not create detached fetch work to achieve that separation.

The operation's cache policy and declared effects are known before parent cache
lookup. A reusable operation may not conceal a domain mutation, solicitation,
or required per-invocation noncacheable child behind a warm parent result. Such
work belongs outside the memoized portion, or the parent is noncacheable.
Dependency discovery while filling an entry is an additional check, not a way
to learn this policy only after a hit has already skipped the body. Credential
refresh is explicitly infrastructure and does not make token expiry the semantic
expiry of an inventory; its authentication admission still precedes reuse.
The full generation/dependency policy is in G6.

## Compatibility and verification obligations

Introduce low contracts and explicitly bound adapters before moving caller
families. Preserve old import paths with re-exports where they retain the same
type. Preserve legacy registry registration/associated handles until their
consumers migrate. Temporary execution adapters require an explicit executor;
none may select a global cache, manufacture a scope, invent a tenant, or resolve
an implicit authentication provider.

Direct request `.await`, runtime-bearing request fields, owned versus shared
inventory results, and registry invocation signatures are deliberate Rust API
changes. The migration inventory must cover each consumer. Remove the old
execution path, scope injection, and implicit state in phase 7.1; adapters are
not a permanent alternative API.

Compile the contracts with borrowed request/output examples, a `Send` but
non-`Sync` uncached output, and borrowed non-`Send` caller callbacks. Verify boxed
registry invocation separately from direct invocation and confirm the new
dependency DAG in manifests. The owning plan records actual commands/results.
Further runtime proofs use synthetic values and isolated storage only: no
credential discovery, live service calls, production data, or existing production
cache reads are authorized for development validation.
