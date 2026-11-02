# Check Azure audit cache races

> I'm thinking about our azure audit surface and how some independent audit activities will involve the same data and how we can separate out the audit logic to new files, just brainstorming, and the ways for sharing data, my understanding is that the Cloud Terrastodon Command C:\Users\teamy\source\repos\cloud-terrastodon\crates\command\src\command.rs stuff has in-memory caching but C:\Users\teamy\source\repos\cloud-terrastodon\crates\azure\src\role_definitions_and_assignments.rs for example uses ...
>
> so it looks like if we make it so that our audit activities are independent, they should be able to fetch the in-memory cache for the request they are performing. If we convert them to be independent and two audits require `fetch_all_role_definitions_and_assignments` then is there a race condition where they are both going to invoke the request and store it in the cache rather than the first one acquiring a lock on the cache key?
>
> User context
>
> Attachments:
> - audit_azure.rs: `./crates/azure_cli/src/audit_azure.rs`
> - reflow_new.rs: `./crates/hcl/src/reflow/reflow_new.rs`
> - role_definitions.rs: `./crates/azure/src/role_definitions.rs`
> - resource_graph.rs: `./crates/azure/src/resource_graph.rs`
> - rest_request.rs: `./crates/rest/src/rest_request.rs`
> - rest_request.rs: `./crates/rest/src/rest_request.rs`
> - cacheable_work.rs: `./crates/command/src/cacheable_work.rs`
> - cacheable_work.rs: `./crates/command/src/cacheable_work.rs`
> - cacheable_work.rs: `./crates/command/src/cacheable_work.rs`

<details><summary>4 previous messages</summary>

> I’ll trace how those requests use the memory and disk caches, then check whether concurrent audits share an in-flight fetch. I’ll keep this to analysis and design discussion.
>
> <details><summary>Ran 8 commands</summary>
>
> - Ran `Get-Content crates/command/src/cacheable_work.rs; Get-Content crates/command/src/command.rs; Get-Content crates/command/src/artifact_cache.rs`
> - Ran `Get-Content crates/azure/src/role_definitions_and_assignments.rs; Get-Content crates/azure_cli/src/audit_azure.rs; Get-Content crates/azure/src/role_definitions.rs`
> - Ran `git status --short; rg --files -g AGENTS.md -g '*cache*' crates/rest crates/command; Get-Content crates/command/tests/cacheable_work_tests.rs`
> - Ran `Get-Content crates/azure/src/role_definitions_and_assignments.rs; Get-Content crates/azure/src/role_definitions.rs -TotalCount 112; Get-Content crates/azure_cli/src/audit_azure.rs -TotalCount 170`
> - Ran `rg -n 'static |async fn|fn |lock|MEMORY|memory|rename|write\(' crates/command/src/artifact_cache.rs; rg -n 'cached|cache|lock\(' crates/command/src/command.rs; rg -n 'run_cached|cache|receive' crates/rest/src/rest_request.rs`
> - Ran `Get-Content crates/azure/src/resource_graph.rs | Select-Object -Skip 260 -First 135; Get-Content crates/hcl/src/reflow/reflow_new.rs | Select-Object -Skip 40 -First 60; Get-Content crates/azure/src/role_assignments.rs -TotalCount 125`
> - Ran `Get-Content crates/command/src/artifact_cache.rs | Select-Object -First 215; Get-Content crates/command/src/artifact_cache.rs | Select-Object -Skip 275 -First 92`
> - Ran `Get-Content crates/azure/src/resource_graph.rs -TotalCount 250; Get-Content crates/rest/src/rest_request.rs | Select-Object -Skip 375 -First 50; rg -n 'impl_cacheable_into_future|async fn run|cache' crates/command/src/cacheable_command.rs; rg -n 'try_new|struct ' crates/azure_types/src/role_definitions_and_assignments.rs`
>
> </details>
>
> Yes—the REST path reaches the shared memory cache, but the lock only protects individual map lookups and inserts. It doesn’t reserve a missing entry while the request runs, so two audits can both fetch the same page. The cache also holds serialized responses; each audit still decodes them and constructs its own combined RBAC value.
>
> <details><summary>Ran a command</summary>
>
> - Ran `rg -n 'trait CacheableCommand|macro_rules! impl_cacheable_into_future' crates/command/src; Get-Content crates/azure_types/src/role_definitions_and_assignments.rs -TotalCount 100; Get-Content crates/command/src/command.rs | Select-Object -Skip 610 -First 37; rg -n 'pub fn debug_inputs|fn debug_inputs|auth|tenant' crates/rest/src/rest_request.rs; rg -n 'let cached_output|let fresh_raw|put_memory|write_output_with|from_slice::<Raw>|work_fingerprint' crates/command/src/cacheable_work.rs; rg -n 'try_join!|try_new|async fn run' crates/azure/src/role_definitions_and_assignments.rs`
>
> </details>
</details>

**Yes. Two concurrent audits can both miss the cache, execute the same request, and then write their results.** The current cache shares completed responses; it doesn’t coordinate requests already in progress.

Your understanding is otherwise right: the role-definition requests reach the same memory-cache infrastructure, even though they use REST rather than `CommandBuilder`.

The path is:

```text
fetch_all_role_definitions_and_assignments
  ├─ fetch_all_role_definitions
  └─ fetch_all_role_assignments
       ↓
  ResourceGraphHelper — paginated REST requests
       ↓
  RestRequest → run_cached_work → artifact_cache
```

The combined request [fetches both collections and constructs the RBAC value](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/role_definitions_and_assignments.rs:72). Caching happens underneath that, per Resource Graph page.

In [run_cached_work](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/command/src/cacheable_work.rs:105), cache lookup and execution are separate steps. The [memory-cache mutex](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/command/src/artifact_cache.rs:181) protects the map lookup, then releases. Nothing reserves a missing entry:

```text
Audit A: lookup → miss → fetch ─────────→ decode → write
Audit B: lookup → miss → fetch ─────────→ decode → write
```

That’s a logical race, often called a **cache stampede**, rather than an unsafe Rust data race. Writes also target individual files in the same directory without atomic publication of the whole entry, so overlapping activity can affect artifact consistency.

For the independent-audit design you’re describing, **per-request “single-flight” coordination fits well**: callers requesting identical work wait for one fetch to populate the cache.

A straightforward implementation would:

1. Check the cache.
2. On a miss, acquire an asynchronous lock for that request identity.
3. **Check the cache again after acquiring the lock.**
4. If still missing, execute, decode, and publish the successful result before releasing the lock.

The second check is what makes the waiting audit reuse the first audit’s result. Different request identities should continue concurrently. This belongs in shared execution infrastructure; `run_cached_work` would cover this REST path, while [CommandBuilder has a separate execution path](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/command/src/command.rs:622) that would need equivalent treatment.

There are two distinct kinds of sharing to consider:

| Sharing | What it saves |
|---|---|
| Cached responses with single-flight coordination | Duplicate network requests |
| A shared typed RBAC value for one audit run | Network requests, repeated decoding, and rebuilding the collections |

Today the memory cache stores serialized `CommandOutput`, **not an `Arc<RoleDefinitionsAndAssignments>`**. Even a warm hit gets decoded again, and each combined request rebuilds and validates its maps.

For splitting the audit logic into files, I’d start with checks that accept exactly the typed data they need. The existing audit runner can fetch once and pass references to several checks; standalone audit entrypoints can fetch those same inputs themselves. That gives you independently testable checks immediately.

If you prefer each concurrent audit to fetch its own dependencies, single-flight coordination makes that practical without a large shared audit context. Its contract should explicitly handle failures, cancellation, and invalidation, and exclude zero-duration mutation artifacts from result reuse. It would prevent duplicate successful fetches within one process; sharing one typed value remains useful when every check must inspect exactly the same collected data.

No files changed or service calls made.

> \>
>
> 1. Check the cache.
> 2. On a miss, acquire an asynchronous lock for that request identity.
> 3. **Check the cache again after acquiring the lock.**
> 4. If still missing, execute, decode, and publish the successful result before releasing the lock.
>
> The second check is what makes the waiting audit reuse the first audit’s result.
>
> Would this be an appropriate place to use RwLock?
>
> Can we reshape things such that Command and RestRequest cache/execute through a shared C:\Users\teamy\source\repos\cloud-terrastodon\crates\command\src\cacheable_work.rs or something so that we can implement this in a single spot?
>
> we can probably make an enum ExecutorKind { Command, Rest, Other(String) } to make it more strongly typed.
>
> The run_cached_work fn would perhaps better be an IntoFuture impl for CachedWorkSpec, so we could .await the spec directly.
>
> You're right that we want to avoid the memory cache requiring duplicate decoding. This could get strange unless we are explicit in correlating the cache key to a type T where we can make the app bail if the same cache key tries to decode to two different T.&#x20;
>
> Ideally, each audit definition can call `fetch_all_principals(auth_context, cancellation)` without having some shared World parameter, .....
>
> Explicit is better than implicit.
>
> Right now, all our fetch_all\_\* functions do not accept a parameter for the cache.
>
> If we consider
>
> pub struct PrincipalListRequest<'a> {\
> &#x20;   \#[facet(skip_serializing)]\
> &#x20;   pub cancellation: CancellationScope,\
> &#x20;   pub auth_context: Cow<'a, AzureTenantAuthContext>,\
> }\
> pub fn fetch_all_principals<'a>(\
> &#x20;   auth_context: &'a AzureTenantAuthContext,\
> &#x20;   cancellation: &CancellationScope,\
> ) -> PrincipalListRequest<'a> {\
> &#x20;   PrincipalListRequest {\
> &#x20;       cancellation: cancellation.clone(),\
> &#x20;       auth_context: Cow::Borrowed(auth_context),\
> &#x20;   }\
> }\
>
> then at the deepest level there's an implicit dependency on a `artifact_cache::get_cached_output` call
>
> which checks
>
> if let Some(output) = memory_cache()\
> &#x20;       .lock()\
> &#x20;       .expect("memory cache poisoned")\
> &#x20;       .get(&memory_key)\
> &#x20;       .cloned()\
> &#x20;   {\
> &#x20;       debug!("Loaded cached output from memory");\
> &#x20;       return Ok(Some(output));\
> &#x20;   }\
>
> before going out to disk
>
> load_file(&cache_dir, METADATA_FILE)
>
> load_file(&cache_dir, CONTEXT_FILE)
>
> load_file(&cache_dir, TIMESTAMP_FILE)
>
> load_file(&cache_dir, STATUS_FILE)
>
> load_file(&cache_dir, STDOUT_FILE)\
> load_file(&cache_dir, STDERR_FILE)
>
> It's strange enough having our&#x20;
>
> pub struct PrincipalListRequest<'a> {\
> &#x20;   \#[facet(skip_serializing)]\
> &#x20;   pub cancellation: CancellationScope,\
> \
> &#x20;   pub auth_context: Cow<'a, AzureTenantAuthContext>,\
> }\
>
> request be semi-serializable, containing the auth context is awkward and adding a reference to a cache here similar to cancellation where the cli top-level creates a cache handle that we pass down sounds like the most sensible thing, but there is also value in having our requests be serializable to text and a handle doesn't help with that. The auth context is basically (AuthSource, Resolved context info), so if we split the request so it looked more like
>
> CloudTerrastodonExecutor { cancellation, auth_context, cache_handle }.execute(PrincipalListRequest{ auth_source: AuthSource::CLI, tenant_id}) then that would leave the in-memory command/rest/auth caches living in the executor, the request would be easily serializeable to text and it would still preserve the auth source and tenant id in the request without the token caches and stuff being part of the request object. Having the executor be the context that we pass around, ideally cheaply cloneable, would help us keep things ergonomic, explicit, and would improve the hygiene of our request objects.
>
> So it seems like we have a few things afoot:
>
> - Command and RestRequest to share cacheable work implementation so we have a single locaiton to follow up with implementing cache key locking to ensure single-in-flight-request
> - Extraction of AuthContext and CancellationToken/CancellationScope from request objects, ensuring auth source and tenant id remain inside the request object
> - Creation of a Cloud Terrastodon Executor type to hold the cancellation,auth context, work cache to keep things explicit instead of implicit; helping us avoid "global variables"
>
> How does this proposal sound?
>
> User context
>
> Attachments:
> - cacheable_work.rs: `./crates/command/src/cacheable_work.rs`

<details><summary>4 previous messages</summary>

> The proposal separates request data from runtime resources in a useful way. I’ll check the command, REST, authentication, and registry contracts to see where a shared executor fits—and where typed caching or cancellation needs a more precise boundary.
>
> <details><summary>Ran 5 commands, searched the web</summary>
>
> - Ran `git status --short; Get-Content crates/azure/src/principals.rs -TotalCount 220; Get-Content crates/credentials/src/auth_context.rs; Get-Content crates/credentials/src/azure_tenant_auth_context.rs; Get-Content crates/credentials/src/auth_source.rs`
>
> - Ran `rg --files -g AGENTS.md; Get-Content crates/azure_devops/src/azure_devops_projects.rs -TotalCount 200; Get-Content crates/command/src/cacheable_work.rs -TotalCount 255; rg -n 'AuthContext|CancellationScope|execute|IntoFuture' crates/registry/src`
>
> - Ran `rg -n 'auth_context|CancellationScope|CancellationToken|run_parsed|automatic_cancellation' crates/app/src crates/entrypoint/src/lib.rs; rg -n 'struct |static |Arc<|Mutex|RwLock|OnceLock' crates/credentials/src/*.rs; Get-Content crates/command/Cargo.toml; Get-Content crates/credentials/Cargo.toml`
>
> Searched the web for `https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html`
>
> - Ran `Get-Content crates/credentials/src/auth_source.rs -TotalCount 110; Get-Content crates/azure_devops/src/azure_devops_projects.rs -TotalCount 175; Get-Content crates/registry/src/bind_request_cancellation.rs -TotalCount 65; Get-Content crates/registry/src/lib.rs | Select-Object -Skip 1070 -First 50`
>
> - Ran `rg --files -g AGENTS.md; rg -n 'static |Arc<|Mutex|RwLock|OnceLock' crates/credentials/src -g '*.rs'; Get-Content crates/command/src/cachable_command.rs -TotalCount 115; Get-Content crates/rest/src/rest_request.rs | Select-Object -Skip 225 -First 22; rg -n 'struct EntraPrincipalCollection|derive|Deref|fn new' crates/azure_types/src/entra_principal_collection.rs; Get-Content crates/registry/Cargo.toml -TotalCount 80; Get-Content crates/command/src/cache_key.rs -TotalCount 145`
>
> </details>
>
> Two details affect the design. First, a key-to-type check won’t make arbitrary decoder closures interchangeable: two validators can return the same type but enforce different rules. The cache needs a fixed decoded-output contract, with caller-specific validation outside it. Second, the registry currently executes requests through `IntoFuture` and binds cancellation by inspecting request fields, so it will need an explicit executor-aware invocation path too.
>
> <details><summary>Ran a command, searched the web</summary>
>
> Searched the web for `'cancelled' in https://docs.rs/tokio/latest/tokio/sync/struct.OnceCell.html`
>
> - Ran `Get-Content AGENTS.md -TotalCount 30; Get-Content crates/credentials/src/azure_devops_auth_context.rs -TotalCount 105; rg -n 'name = "tokio"|version = "1\.5' Cargo.lock | Select-Object -First 15; rg -n 'async fn get_cached_output|TIMESTAMP_FILE|from_rfc|valid_for|put_memory_cache_entry' crates/command/src/artifact_cache.rs; Get-Content crates/command/src/artifact_cache.rs | Select-Object -Skip 235 -First 40; Get-Content crates/credentials/src/azure_tenant_auth_context.rs | Select-Object -Skip 45 -First 28`
>
> </details>
</details>

**Yes—I think this is a sound direction.** It gives requests a clear meaning as serializable descriptions of work, and gives the executor responsibility for the resources and lifecycle needed to perform that work. That is a substantive responsibility under your conventions.

I’d shape it around the following boundaries.

1. **Use a per-key async `Mutex` for initialization; an `RwLock` is optional for the cache’s lookup table.**

   An `RwLock` permits multiple readers or one writer, but changing the existing map’s mutex to an `RwLock` would leave the miss–fetch–insert race intact. Readers can still independently discover a miss. [Tokio’s documentation](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html) describes those access guarantees.

   I’d start with a briefly held map lock to find or create an entry, then an asynchronous mutex on that entry to coordinate fetching. Release the map lock before awaiting anything. After acquiring the entry lock, check again whether another caller completed the work.

   Successful hits should clone an `Arc<T>` and release their guard immediately. There’s little reason to keep a read lock while an audit examines immutable data.

2. **Unify Command and REST around the execution lifecycle, while preserving their execution-specific behavior.**

   Both can use one implementation of:

   ```text
   identify work
     → check typed memory result
     → acquire ownership of a miss
     → recheck
     → load disk artifacts or execute
     → decode and validate the canonical result
     → retain artifacts and publish the result
   ```

   Command owns subprocess execution and retries; REST owns HTTP execution. The shared layer owns coordination, caching, and outcome retention.

   There are two concrete migration traps:

   - Command currently [reenters its cache/execution boundary during retries](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/command/src/command.rs:433). Putting a non-reentrant key lock around that would deadlock. Retries need to stay inside the uncached execution owned by the outer operation.
   - Command’s fingerprint includes executable, arguments, environment, working directory, stdin, and adjacent files. Consolidation must preserve those inputs rather than replacing them with the current generic fingerprint’s narrower inputs.

   Your `ExecutorKind` enum makes sense as execution metadata. I’d consider an explicit `InProcess` variant alongside `Command`, `Rest`, and `Other(String)`. Cacheability and cancellation policy still need their own contracts: the execution kind alone cannot determine them.

3. **Keep serializable requests separate from awaitable, execution-bound work.**

   The public usage could look like this:

   ```rust
   let request = PrincipalListRequest {
       auth_source: AuthSource::AzureCli,
       tenant_id,
   };

   let principals: Arc<EntraPrincipalCollection> =
       executor.execute(request).await?;
   ```

   Here:

   | Value | Responsibility |
   |---|---|
   | `PrincipalListRequest` | Describes the operation, scope, authentication selection, and filters |
   | `CloudTerrastodonExecutor` | Owns shared authentication resources, work cache, and execution coordination |
   | The value returned by `execute` | Binds the request to runtime resources and the current cancellation scope |

   **`IntoFuture` fits that last value well.** `CachedWorkSpec` could implement it once bound to those resources. Its closures already make it an internal execution object; it needn’t be serializable.

   A bare request would no longer be independently awaitable, because it deliberately lacks the resources required to execute. The explicit binding step expresses that dependency.

   This also gives `fetch_all_principals` a straightforward migration: accept the executor plus the request’s scope/authentication selection, or construct the request and call `execute` directly. Each audit can obtain its own dependencies through the same executor.

4. **Associate each cacheable operation with one canonical output type.**

   I agree with rejecting attempts to use one logical cache identity for incompatible types. Record the expected type when reserving the entry, including while initialization is in progress, and return a diagnostic containing both type names on mismatch.

   Don’t merely add `TypeId` to the key: that would silently create two entries instead of detecting the collision. Use `TypeId` for the in-process check; persistent artifacts need a stable contract/schema version. Rust’s [`TypeId` documentation](https://doc.rust-lang.org/std/any/struct.TypeId.html) cautions that its representation and hashes are unstable across releases.

   **Type agreement alone is insufficient, though.** Existing [REST validators and mappers](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/rest/src/rest_request.rs:524) can contain arbitrary behavior. Two closures returning the same `T` might apply different validation or capture different inputs. Returning the first cached `T` would skip the second caller’s behavior.

   I’d make canonical decoding and invariant validation part of the operation’s contract, cache that result, then run caller-specific validation or projections afterward.

   For memoized reads, returning `Arc<T>` avoids both repeated decoding and deep cloning. Shareable results would require `Send + Sync`, beyond today’s `Send` requirement.

   Also, **memoize the composed SDK operation when its assembled result matters**. Caching decoded REST pages alone would still leave [`PrincipalListRequest`](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/principals.rs:65) rebuilding the combined principal collection on every call. The same cache machinery can coordinate both page operations and aggregate operations, under distinct identities.

5. **Share runtime resources cheaply, while keeping cancellation bound to the current operation.**

   Executor clones can share authentication state and caches through `Arc`. An executor used inside a child cancellation callback should carry that callback’s child scope. Cloning the executor must preserve the existing bounded scope ownership; it cannot extend a deferral beyond its owner.

   For shared initialization, I’d start with an explicit policy: the initiating caller owns the fetch; cancelling a waiter only ends its wait; if the initializer is cancelled, another waiter may attempt initialization. Keeping a fetch alive independently of its initiating caller requires executor-owned task lifecycle and draining.

   Authentication also needs an explicit agreement between request and executor. The request selects the source and tenant; the executor supplies compatible credentials or reports a mismatch. Preserve the existing tenant validation. Azure DevOps additionally needs to retain its distinct bearer, CLI-active-tenant, and tenant-independent PAT cases.

   Cache identity must account for the authentication partition: source plus tenant does not distinguish two accounts using Azure CLI. This applies to persisted artifacts as well as memory.

6. **Make invalidation and registry invocation part of the design.**

   An executor-owned cache implies executor-aware invalidation. A bare `CacheKey::invalidate()` cannot reliably invalidate runtime state without knowing its owner. Invalidation must cover typed aggregates and underlying pages, and prevent an older in-flight fill from republishing invalidated data. Memory entries also need validity tracking; current memory hits return before disk timestamp checks.

   Zero-duration mutation artifacts must continue to execute on every invocation. Sharing their results would change behavior.

   The registry currently [invokes `IntoFuture` directly](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/registry/src/lib.rs:1079) and injects cancellation by inspecting request fields. It needs an invocation contract accepting request data and execution resources separately.

   Finally, crate placement matters: `credentials` already depends on `command`. Putting an executor that directly owns `AuthContext` inside `command` would create a dependency cycle. The shared work engine can remain lower-level, with the authentication-aware executor above it.

I’d implement this in stages: establish the explicit work-cache owner and shared Command/REST lifecycle; introduce executor-bound request execution, including registry support; then add canonical typed-result caching and per-key coordination. Principal collection would make a useful first end-to-end example because it exercises both shared page reads and a shared aggregate result.

No files changed.

> So point number one, the per-key mutex, and then the per-entry, like the hierarchical mutex strategy, one to get the first one, and then the second one is for the mutex corresponding to that cache key. That makes sense. And then unifying command and REST around the lifecycle, that makes sense. And then having an in-process variant makes sense. For number three, that makes sense. I think it would be nice to be able to call executor.execute and then pass the request, versus, and like we should also be able to call request.execute and pass in the executor. So when we call fetch all principles, that would accept the tenant ID and the auth source, and then the executor would hold the auth context and the cancellation. So when we think about the principal list request, and then that calls fetch all entry users, our current implementation for fetch all entry users, we have the CachableCommand trait, and then we call run, and then right now it implements into Future on itself, and it's able to grab the cancellation from self, when really what we'd need to be doing is adjusting it so that the entry user list request.rs file, when it's implementing some trait or whatever for this user list request, when it's calling run, that run command is going to accept, in addition to self, the executor it's running with, and that is where we'll be able to grab the auth context and the cancellation information, or rather the... So I guess that would mean the Microsoft Graph helper would, when we call fetch all, that would also take the executor. So instead of Microsoft Graph helper's new constructor accepting those, the new constructor would just accept the URL and the cache key, and then the auth context and cancellation would be part of the fetch all. But that leaves out the tenant ID and the auth source. So the Microsoft Graph helper would also accept the tenant ID and the auth source as part of its construction. Because right now, anywhere the auth context is used, it's kind of implicitly burying the tenant ID, which we want to make explicit in the request object, the one that's, like, serializable to text, which I guess the graph helper is not quite that, but it could be, and then the cancellation is more about the actual execution, same with the, like, the auth source. That should also be part of the constructor. So the auth source and the tenant ID are things that we need to pull out when we see the auth context, and then the cancellation can just be moved to the executor being passed into the fetch all command. And then associating each Cachable operation with one canonical output type, that is only important for the in-memory cache, because for the disk representation we don't need to explicitly state what the type is. Like, we can have it there as diagnostic info rather than being part of the control flow. So the type ID and the lack of guarantees that that has around it in regards to, like, if we were to serialize it, that would just be for manual inspection. It wouldn't be something that the code actually uses, because it's just the actual output of JSON or whatever that the commands are then deserializing. It's when we have it in the in-memory cache, we want to avoid it being done such that the same cache key is used by two different entry points. So really, if we want to go all the way, we could say that the in-memory cache is locked to kind of like the file line and column or whatever as the entry point. So there should only be one entry point to a given cache key in terms of execution and accessing the in-memory cached values. And that would solve the type ID problems, where we wouldn't even be using the type ID. We'd be using the file and line number and column number. But that approach might get tricky because the... well, I'm sure we can think it through. It's just one option. The main thing is that we want to avoid paying the cost of deserialization for subsequent requests, because right now the current cache is skipping the disk read, but it's not skipping the deserialization for duplicate requests. An interesting consequence of this is that the in-memory cache and the way the commands are being returned and stuff is, I guess we're cloning the value, or, well, the command output, which contains the information that's written to disk that we are skipping the reading from disk back, is a text representation. So currently, when we are deserializing that on each invocation, we're returning the owned value from the deserialization. When we migrate to having the in-memory cache holding the value, that would mean that our command and REST executions would be returning an Arc, so a reference to the deserialized value rather than an owned copy of the deserialized value, so that we can avoid copying. So in the case of the Azure DevOps ones, if I look at the Azure DevOps project, then our Azure DevOps project list request contains the auth context, which is an Azure DevOps auth context, which can be none, bearer, Azure CLI, or personal access token. So we would want an Azure DevOps auth source enum that is separate from the auth context, because the auth source, Azure DevOps auth source enum, would just tell us, okay, is it none, bearer, Azure CLI, or personal access token? And then the actual details of that, the resolution and the in-memory caching of that, is inside the executor. So the executor holds the auth context, and then the request, for example the Azure DevOps project list request, would hold the org URL, and the auth source would be, if it is bearer, then the auth source, I guess, would be able to hold the tenant ID inside the auth source bearer variant, and then for the Azure CLI variant, that would also be able to hold the Azure auth source, which could be, like workload identity or browser or whatever. And we can kind of work out the details on that. Because I guess even though Azure DevOps organizations are sometimes associated with a tenant ID, it would be... The tenant ID is only relevant in terms of acquiring the correct credential for Azure DevOps, not because the requests themselves are associated to the tenant ID. And that's kind of the same thing for the Azure side, is the tenant ID is part of resolving which credential is correct, and it's useful for the text serialization of our requests. But when we actually fetch, like a resource group or whatever, the URLs and stuff doesn't contain. You are correct that cache invalidation is now tied to the executor, because invalidating the in-memory cache is just as important as invalidating the disk cache. So in validation, we also have to lock that cache key, where the cache keys are hierarchical. So if we are invalidating the parent, then all of the children need to also be kind of locked. So exclusivity guarantees need to accommodate the fact that it's a hierarchical cache, because we have pages and stuff. So we could create a Cloud Trestledon cache crate to avoid circular dependencies between credentials and command and stuff if that is necessary.

<details><summary>4 previous messages</summary>

> I’ll check the Graph helper and cache invalidation paths against this design. The main points to settle are operation identity, authentication selection, and how parent invalidation coordinates with active child requests.
>
> <details><summary>Ran 3 commands, searched the web</summary>
>
> - Ran `rg --files crates/azure/src | rg '(graph|entra_user)'; Get-Content crates/azure/src/entra_users.rs -TotalCount 150; Get-Content crates/azure/src/microsoft_graph_helper.rs -TotalCount 260; Get-Content crates/credentials/src/azure_devops_auth_context.rs -TotalCount 100`
>
> - Ran `git status --short; Get-Content crates/command/src/cache_key.rs -TotalCount 145; Get-Content crates/azure/src/msgraph.rs -TotalCount 250; rg -n 'MicrosoftGraphHelper|GraphHelper|fetch_all' crates/azure/src -g '*graph*'; rg -n 'CancellationScope|CancellationToken|with_.*scope|cancellable_scope' ../teamy-cancellation/src/lib.rs | Select-Object -First 24`
>
> Searched the web for `https://docs.rs/tokio/1.52.3/tokio/sync/struct.RwLock.html#method.read`
>
> - Ran `Get-Content crates/azure/src/entra_user_list_request.rs -TotalCount 145; Get-Content crates/azure/src/microsoft_graph.rs -TotalCount 180; Get-Content crates/credentials/src/auth_source.rs -TotalCount 33`
>
> </details>
>
> The Graph helper already invalidates its page subtree and retries after a continuation failure. That makes holding a subtree read lock through the whole fetch risky: the fetch could block trying to invalidate its own cache. I’m separating ownership of the in-flight computation from coordination of cache reads, writes, and invalidation.
>
> <details><summary>Searched the web</summary>
>
> Searched the web for `https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html#method.read`
>
> </details>
</details>

Yes—the request/executor split you’re describing is coherent. I agree with your correction about disk caching: **type information can remain diagnostic on disk**, with the current decoder deciding whether the stored response is acceptable. The stronger ownership/type contract is needed for reusing already decoded values in memory.

I’d refine the design in these ways.

1. **Support both execution spellings through one implementation.**

   These should be equivalent:

   ```rust
   executor.execute(request).await?;
   request.execute(&executor).await?;
   ```

   The request-side method can be a default trait method forwarding to the executor. The operation implementation then receives both `self` and the execution environment:

   ```rust
   // Illustrative operation contract.
   async fn run(self, executor: &CloudTerrastodonExecutor)
       -> Result<Self::Output>;
   ```

   `run` produces the owned result; the shared execution machinery handles caching and returns an `Arc` for memoized operations.

   Your proposed constructor follows naturally:

   ```rust
   fetch_all_principals(tenant_id, auth_source)
       .execute(&executor)
       .await?;
   ```

   Inside that operation, the Entra user, group, and service-principal requests receive explicit authentication selection and execute using the supplied executor. Independently cancellable children receive executors bound to their child scopes while sharing the runtime resources.

2. **The Graph helper can follow exactly that separation.**

   Its construction would describe the work:

   ```rust
   MicrosoftGraphHelper::new(url, cache_key, tenant_id, auth_source)
       .fetch_all(&executor)
       .await?;
   ```

   URL, headers, tenant, and source belong to the operation description. Credential resolution, token caches, and cancellation come from execution.

   There is another runtime field in [the current user request](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/entra_user_list_request.rs:24): its progress callback. That should move to execution-bound state too, so the request can be fully serializable. We would also define what progress observers receive when another caller performs the fetch or the result is already cached.

3. **A separate Azure DevOps authentication-selection enum makes sense.**

   I would give its variants precise meanings:

   | Selection | Serializable information |
   |---|---|
   | Bearer | Tenant ID and bearer credential source: CLI, browser, or workload identity |
   | Azure CLI active tenant | Explicitly requests the existing CLI behavior without a supplied tenant |
   | Personal access token | Selects PAT authentication; the executor supplies the credential |

   Browser and workload identity would belong in the bearer selection. A variant named `AzureCli` should specifically mean CLI behavior.

   The existing [`AzureDevOpsAuthContext::None`](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/credentials/src/azure_devops_auth_context.rs:16) means incomplete or synthetic state. We should decide whether the new request enum needs that non-executable state rather than automatically copying it.

   Agreed that the authentication tenant and resource address are separate concepts. Tenant still needs to participate in cache identity where it distinguishes the requested data, even when it does not appear in the URL—as the current Graph user cache already does.

4. **Bind the memory entry to an operation, and retain call locations for diagnostics.**

   The rule I’d use is:

   > One operation contract owns a cache entry; any number of callers may invoke that operation.

   File/line/column would be useful in an error identifying where conflicting access originated, but it cannot establish output compatibility:

   - Two audits should be able to invoke the same operation from different source locations.
   - One generic helper location can execute with multiple output types.
   - One location can run closures with different captured validation rules.

   `Location::caller()` also depends on the chain of `#[track_caller]` functions, so its meaning changes with the wrapper structure. [Rust documents that behavior here.](https://doc.rust-lang.org/std/panic/struct.Location.html#method.caller)

   I’d use the operation’s identity and associated output contract, with a checked typed downcast inside the heterogeneous memory cache. That can use `TypeId` internally without serializing it at all. A mismatch becomes an error reporting the key, operation, types, and relevant locations.

   Disk artifacts can continue storing raw output plus diagnostic type names.

5. **`Arc` sharing is right, but collection assembly needs attention.**

   Subsequent calls can return `Arc::clone(&cached_result)` without deserializing or copying the collection.

   One consequence appears in [Graph pagination](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/microsoft_graph.rs:155): it currently moves each page’s items into the combined vector. If the cache retains an `Arc` to each decoded page, those items can no longer simply be moved out.

   We therefore need to choose where sharing happens: final collections, individual records, retained pages, or some combination. A reasonable initial approach is to share the completed SDK collection and accept any necessary assembly copy once per fill. Repeated audit requests then share that completed result. We shouldn’t promise that putting `Arc` around every intermediate value automatically eliminates all copying.

6. **Hierarchical invalidation needs its own coordination contract.**

   You’re right that invalidating a parent must cover all descendants, including pages being created while invalidation runs. Enumerating existing children and locking each one would miss newly introduced entries.

   However, holding ancestor read locks throughout network execution introduces a deadlock:

   ```text
   Parent fetch holds ancestor read lock
     → invalidation queues for ancestor write lock
     → parent awaits a page fetch
     → page waits for ancestor read lock behind the writer
   ```

   Tokio’s writer-preferring `RwLock` can produce that dependency cycle. [Its documentation explicitly warns about recursive read acquisition with a waiting writer.](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html#method.read)

   There is also an existing self-invalidation case: [Graph pagination invalidates its own subtree before retrying](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/microsoft_graph.rs:83).

   I’d therefore separate:

   - **Entry ownership:** a stable per-key mutex coordinates the in-flight computation.
   - **Cache access/publication:** short-lived coordination protects reading and publishing entries against subtree invalidation.
   - **Freshness:** an invalidation generation identifies work started before a subtree was invalidated.

   Invalidating a prefix advances its generation, removes descendant memory values, and invalidates disk entries under exclusive cache coordination. A fetch started under an older generation cannot subsequently publish into either cache.

   **This defines invalidation as excluding stale cache reuse and publication.** Already running callers may still finish with their result, and already returned `Arc`s remain valid snapshots. Waiting for every active descendant operation to finish would be a stronger, separate contract.

   An `RwLock` is useful for that short-lived cache I/O barrier. A cache-wide barrier is a simple starting implementation; prefix-specific barriers can preserve more concurrency later. Neither should span network execution or nested requests.

   Invalidation must also retain live per-key mutexes. Removing a coordination object while its initializer runs would allow a second mutex—and a second initializer—for the same key.

A `cloud_terrastodon_cache` crate is a sensible home for keys, typed entries, artifact storage, and this coordination. It should remain independent of credentials, commands, and REST. The shared execution core can use it, while the authentication-aware executor sits above that core. We still need to preserve that dependency direction: moving caching alone would not fix a cycle if `command` then depended on an executor that itself depended on `credentials`.

No files changed.

> >
> >
> > | Selection               | Serializable information                                                   |
> > | ----------------------- | -------------------------------------------------------------------------- |
> > | Bearer                  | Tenant ID and bearer credential source: CLI, browser, or workload identity |
> > | Azure CLI active tenant | Explicitly requests the existing CLI behavior without a supplied tenant    |
> > | Personal access token   | Selects PAT authentication; the executor supplies the credential           |
>
> The azure CLI credential must still have an explicit tenant id specified as part of our request. We do not want to depend on the default auth/tenant/devops setup in the azure cli.
>
> > One consequence appears in [Graph pagination (line 155)](/C:/Users/teamy/source/repos/cloud-terrastodon/crates/azure/src/microsoft_graph.rs:155): it currently moves each page’s items into the combined vector. If the cache retains an `Arc` to each decoded page, those items can no longer simply be moved out. &#x20;
>
> We can update the azure devops thing to return a Vec\<Arc\<T>> if that makes sense, probably easier than adding lifetimes to everything if we were going to be explicit about the borrows being tied to the in memory cache's lifetime.
>
> > Hierarchical invalidation needs its own coordination contract.
>
> &#x20;You are correct. It makes sense to have a generation or similar mechanism so that we can invalidate things while there may still be inflight requests, or at least coordinate such invalidation so that things do not deadlock.

Agreed. These settle three parts of the design.

**Every bearer-authenticated request must specify its tenant**, including requests using Azure CLI credentials. Azure DevOps authentication selection becomes:

| Selection | Request data |
|---|---|
| Bearer | Explicit tenant ID and source: Azure CLI, browser, or workload identity |
| Personal access token | Explicit PAT selection; executor supplies the credential |

There would be no active-tenant variant. CLI credential acquisition must use the request’s tenant, and organization/project selection remains explicit rather than coming from CLI defaults.

**`Vec<Arc<T>>` fits collection assembly well.** Pages and aggregate collections can share individual records without borrowing from the cache. For example:

```rust
// The operation produces:
Vec<Arc<AzureDevOpsProject>>

// The executor retains and returns:
Arc<Vec<Arc<AzureDevOpsProject>>>
```

The inner `Arc`s let pagination and aggregate operations reuse records without copying them. The outer `Arc` lets subsequent callers reuse the entire collection without cloning its vector of pointers. Neither requires cache-bound lifetimes, and returned values remain usable after eviction or invalidation.

We should apply that shape where records are shared across collections; ordinary scalar results can remain `Arc<T>`.

**Generation-based invalidation is the agreed coordination model.** An operation captures the relevant generation before loading or computing its result. Publication checks that generation while coordinated with invalidation:

- Parent invalidation advances the generation affecting its descendants.
- Older work may finish for its existing caller, but cannot repopulate memory or disk.
- Subsequent requests observe the new generation.
- The per-key coordination object remains alive while work or waiters reference it.
- Previously returned `Arc`s remain valid snapshots.

That gives us explicit request identity, shared owned results, and invalidation that can proceed while network requests are in flight.

> Please review our design so far.
> We should curate a plan for the migration.
> C:\Users\teamy\source\repos\skills\.agents\skills\resumable-implementation-plans\SKILL.md
>
> It can live in C:\Users\teamy\source\repos\cloud-terrastodon\docs\plans
>
> The plan should not mention my username or other PII

<details><summary>5 previous messages</summary>

> I’m using the resumable implementation plans skill to review the design and write a migration plan in `docs/plans`. I’ll use repository-relative paths and keep personal information out of the plan.
>
> <details><summary>Ran a command</summary>
>
> <details><summary>Explored a file</summary>
>
> - Read `~/source/repos/skills/.agents/skills/resumable-implementation-plans/SKILL.md`
>
> </details>
>
> - Ran `git status --short; rg --files docs/plans; rg --files -g AGENTS.md; Get-Content Cargo.toml -TotalCount 180`
>
> </details>
>
> For a shared read, what should happen if its initiating caller is cancelled while other callers are waiting?
> - Cancel that attempt; a waiter may retry (recommended)
> - Keep the fetch alive under executor ownership until it completes
>
> Should the initial migration guarantee cache coordination only among executors sharing one cache handle, or also between separate processes using the same disk cache?
> - Shared cache handle first; cross-process coordination is a later phase (recommended)
> - Include cross-process coordination in this migration
>
> <details><summary>Ran 3 commands</summary>
>
> - Ran `Get-Content AGENTS.md; Get-Content docs/plans/secretless-authentication.md -TotalCount 125; Get-Content docs/plans/azure-devops-work-item-copy.md -TotalCount 95`
> - Ran `rg --files .github docs scripts | rg '(workflow|release|build|test|REUSABLE|CANCELL|README|plan)'; Get-Content rust-toolchain.toml; Get-Content rustfmt.toml; Get-Content .cargo/config.toml; rg -n 'name = |cloud_terrastodon_(command|credentials|registry|rest)' crates/{command,credentials,registry,rest,app,entrypoint}/Cargo.toml`
> - Ran `rg --files crates/command/tests crates/rest/tests crates/registry | Select-Object -First 65; rg -n 'cargo (test|check|fmt|clippy)|release|publish' .github docs/REUSABLE_APPLICATIONS.md README.md -g '*.md' -g '*.yml' -g '*.yaml'`
>
> </details>
>
> **For a shared read, what should happen if its initiating caller is cancelled while other callers are waiting?**
>
> Cancel that attempt; a waiter may retry (recommended)
>
> **Should the initial migration guarantee cache coordination only among executors sharing one cache handle, or also between separate processes using the same disk cache?**
>
> Shared cache handle first; cross-process coordination is a later phase (recommended)
</details>
