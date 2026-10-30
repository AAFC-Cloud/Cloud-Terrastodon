# Cache publication and dependency coordination

**Status:** Phase 1 decision document. The algorithm below is selected for G5 and
G6 of the [executor/cache migration plan](executor-cache-migration.md). It is not
implemented or runtime-validated. The validation scenarios describe future proof
obligations; they are not test results.

**Decision date:** 2026-10-30.

## Guarantee and ownership

One explicitly constructed cache handle owns memory values, initializer locks,
artifact access, invalidation generations, and dependency tracking. Its clones
share that state. Independent handles and processes do not yet coordinate, even
when configured with the same disk root. Cross-process locking, crash recovery,
and durable invalidation agreement remain deferred to migration phase 8.

Successful reusable reads return shared owned snapshots. A warm memory hit neither
deserializes nor deep-copies the canonical result. Invalidating a snapshot does
not revoke an `Arc` already returned to a caller. An old in-flight attempt may
finish for its existing caller, but cannot publish a reusable result after its
own or a dependency's generation changes.

Initializers belong to their calling futures. There is no cache actor, detached
fetch task, or background task that keeps an initializer alive after its caller
is cancelled. Existing required-completion rules still apply after an outcome
exists. A surviving admitted waiter can acquire the same entry mutex and retry;
it does not inherit the cancelled caller's scope.

Caller ownership and cancellation intent are separate. A normal linked child
scope can share its ancestor's cancellation token; requesting cancellation on
that child can therefore cancel other callers under the same root. Independent
initializer-cancellation scenarios must use independently cancellable caller
ownership/signals or drop that caller's future. Root cancellation ends all
affected callers and is not a waiter-retry scenario. Preserve these distinctions
when applying the pinned cancellation library's scope helpers.

## Identities and state

The G3 identity contract distinguishes three concepts:

| Concept | Identity and use |
| --- | --- |
| Logical scope | A validated component hierarchy used for prefix invalidation across namespaces, variants, and authentication partitions |
| Ownership claim | `(executor namespace, logical operation key)` bound to one canonical operation/decoder owner and output type for this handle |
| Concrete entry | Ownership address plus semantic request fingerprint and authentication partition; identifies one decoded value, initializer mutex, and physical artifact entry |

Check the ownership claim before looking up a variant, reading disk, executing,
or waiting. A mismatch is an error even when the requested variant is different
or the output Rust type happens to match. Namespace, fingerprint, and partition
encoding must not alias on disk. Owner identity, output `TypeId`, and caller
location must not be appended to the address to turn a collision into a miss.

The handle maintains the following state behind one short synchronous mutex:

| State | Responsibility |
| --- | --- |
| Ownership claims | Retain the accepted canonical owner, in-process output type, and diagnostic names/locations |
| Coordination registry | Map concrete entries to weak references to stable coordination objects containing asynchronous initializer mutexes |
| Decoded values | Retain type-erased `Arc` values with their complete freshness certificates in a separately reclaimable map |
| Prefix generations | Record invalidations of logical component prefixes, regardless of namespace |
| Exact-entry generations | Record rejection/invalidation of one concrete artifact entry |
| Quarantine | Prevent unsafe disk reuse after an incomplete invalidation or ambiguous publication |
| Active attempts | Track initial stamps, semantic dependencies, admission, and non-reusability for bounded execution attempts |
| Wait graph | Reference-count directed parent-entry to child-entry edges while child requests are awaited |

A generation has a nonwrapping identity within the handle. If its counter cannot
advance, return an error and retain quarantine rather than reuse an earlier
identity. The absence of a prefix record means generation zero. Every entry
certificate includes all logical ancestors, including prefixes that currently
have generation zero; creating a new prefix generation must invalidate an older
certificate that previously observed its absence.

Do not substitute one global invalidation generation. Invalidating one project
must preserve unrelated projects. A dependency on a different branch is tracked
explicitly.

The explicitly bound execution context carries the current active-attempt
identity into child execution and dependency registration. Do not discover the
parent through a global cache, task-local default, or unrelated root context.
Child cancellation callbacks still use their own child scopes while sharing
the same cache owner and appropriate dependency ancestry.

## Locks and linearization

Use an asynchronous **mutex** as the initial cache-wide artifact I/O/publication
gate. It serializes artifact reads, reusable publication, and durable invalidation
within the shared handle. A stable asynchronous mutex on each concrete entry
coordinates its initializer. The synchronous state mutex protects only in-memory
bookkeeping.

The lock rules are:

1. Obtain or create the entry coordination `Arc` under the state mutex, then
   release state before awaiting initializer ownership.
2. A holder of an initializer mutex may acquire the I/O gate and then state.
3. An I/O-gate holder may acquire state, but must release state before any await.
4. Never acquire or await an initializer mutex while holding the I/O gate.
5. Never hold the I/O gate across HTTP/process execution, canonical decoding,
   child requests, caller validators/projections, or observer callbacks.
6. Invalidation acquires the I/O gate and state; it never acquires descendant
   initializer mutexes.

The gate may cover asynchronous filesystem work. That initial serialization is
intentional. Unique attempt staging can happen outside it because staging is not
visible as reusable storage. Replacing the gate with a more concurrent mechanism
is a later optimization with the same correctness obligations.

A memory hit linearizes while holding state: check ownership and the complete
certificate, check the current caller's expiry bound, check dependency-cycle
admission, and register the returned certificate in the current parent before
cloning the value and releasing state. There is no gap in which a parent receives
a child value without also recording the generations that made it acceptable.

Invalidation linearizes when, under gate and state, it advances the target
generation, evicts direct values, and installs quarantine. Publication linearizes
when it validates the complete certificate and inserts the typed result under
state while retaining the I/O gate. No invalidation can interleave between that
publication's generation checks and its durable commit.

Already returned snapshots and calls admitted before invalidation retain their
ordinary caller ownership. They do not gain permission to republish obsolete
results. A newly admitted call after successful invalidation cannot reuse a
certificate from before that invalidation.

## Initialization and typed reuse

The shared lifecycle follows this procedure:

1. Validate eligibility and the namespace/logical ownership claim. Obtain the
   stable coordination object and attempt a memory hit as described above.
2. On a miss, register the parent-to-child wait edge, when there is a parent, and
   check it for a cycle. Release state and await the entry mutex cancellably.
3. After acquiring it, check memory again. A waiter uses a completed result only
   if that result meets its own ownership, generation, and expiry requirements.
4. Begin an active initializer attempt and capture its own ancestor and exact
   generations before disk loading or execution. Keep these original stamps
   throughout the attempt, including retries.
5. If disk reuse is permitted, acquire the I/O gate, check quarantine and the
   commit record, and load one complete immutable artifact set. Release the
   gate before canonical decoding. Revalidate freshness before returning or
   publishing the decoded result; invalidation may have happened during decode.
6. On an ordinary miss, execute the raw operation using the calling executor's
   bounded scope. Capture its observation time, perform canonical decoding,
   and collect any semantic child certificates. No global cache guard is held.
7. Finalize the dependency ledger. Required children must have completed; an
   attempt with outstanding semantic child work cannot publish. Freeze the
   certificate so a late child cannot silently change a published value.
8. Stage artifacts, then use the gated commit procedure below. Register the
   final returned certificate in the parent under state as part of completing
   the child invocation, whether the result was published or only returned to
   its existing caller. An obsolete or otherwise nonreusable child makes the
   parent's attempt nonreusable.
9. Release initializer ownership and the wait-edge guard. Failure/cancellation
   publishes no successful value. A waiter can subsequently retry under the
   same mutex; shared error results are not promised.

Failure, cancellation, or a dropped semantic child also marks every active parent
attempt nonreusable, even if the parent catches the error and returns an empty or
partial fallback. Install completion guards when admitting children so a drop
cannot skip this bookkeeping. A successful canonical domain value, such as an
explicitly modeled absence, can carry a normal freshness certificate; an error
without a certificate cannot silently become a reusable successful aggregate.

Canonical validation belongs to the operation's decoder. Caller-specific
validation and projections run after obtaining the shared result for every
caller. Their rejection does not invalidate a canonical value that remains
valid under its owning contract.

## Freshness certificates and dependencies

A certificate contains the concrete entry identity, its original prefix and
exact-entry generations, its observation time/deadline, and the transitive
certificates of semantic child reads. Flatten the dependency records for the
initial implementation. Preserve entry identities as well as generations so
dependency cycles can be detected without executing another initializer.

On child completion or a warm hit, merge its complete certificate into the active
parent under state. Matching observations can be deduplicated. Conflicting
generations must not overwrite earlier observations: record non-reusability.
When matching observations have different caller bounds, retain the earlier
deadline; deduplication must not discard the stricter dependency certificate.
The same applies when a previously consumed dependency expires during assembly.
This ledger is monotonic for the lifetime of the attempt.

For example, `principals` can depend on `users`, `groups`, and
`service_principals` even when those keys are siblings. A cached higher-level
result that consumes `principals` also receives the transitive stamps. Invalidating
`users` prevents both levels from being reused or published. A reverse-dependency
index is not required initially: every hit and publication checks the flattened
certificate, and stale retained values are evicted when encountered.

Graph continuation recovery may invalidate the operation's own logical subtree
and retry its pages. The retry observes newer child generations, but the outer
attempt retains its original own stamp. It may return the successful recovered
result to its caller; it cannot publish it as an unaffected original attempt.
A later invocation starts a new attempt with current stamps.

## Memoization admission and semantic effects

Memoization eligibility is decided **before parent cache lookup**. The owning
operation must declare that its result and behavior can be reproduced by its
serialized request inputs and reusable read dependencies. Its effect contract
includes transitive semantic obligations, rather than only its immediate HTTP
method. A workflow that may need a mutation, solicitation, zero-duration child,
or deliberately uncached data read must bypass parent memoization for those
request inputs. If this cannot be determined from the explicit request contract,
the workflow is nonmemoized.

It is insufficient to discover these effects only when executing a cache miss:
a parent hit would bypass that discovery and suppress required work. Runtime
child admission is a second enforcement boundary. Before executing a semantic
child under an admitted reusable parent, reject an incompatible effect or
nonreuse requirement diagnostically and mark the parent attempt nonreusable,
even if its implementation catches the error. Do not silently execute a side
effect and subsequently call its parent safely memoized.

Credential/token acquisition is explicit execution infrastructure. Its token
expiry does not become an inventory dependency and refreshing a token does not
invalidate inventory data. Only the executor's credential capability selects
that role; arbitrary request code cannot label semantic work as infrastructure
to evade dependency or effect admission. Infrastructure still participates in
execution/cancellation ownership and applicable wait-cycle checks.

Mutation, zero-duration, and deliberately uncached operations execute for every
invocation. They retain required artifacts under unique attempts, never share a
successful execution result to replace a second execution, and perform affected
read invalidation through the same cache owner.

## Wait and dependency cycles

Before a parent waits for a missing child entry, insert a directed edge from the
parent's concrete entry to the child's. Reject an edge if its target already
reaches its source. Direct recursion is the simplest instance. Independent
tasks holding `A` and `B` cannot each wait indefinitely for the other's mutex.

Keep the edge until the complete child invocation returns, including child
execution after it acquires its mutex. An ancestor is still waiting while that
child starts grandchildren. Store edge reference counts so two parallel child
awaits do not remove each other's edge. An RAII guard removes its edge under the
short state mutex on success, error, cancellation, or future drop. No async
cleanup or detached task is required for graph bookkeeping.

Uncached and infrastructure callbacks retain the ancestry of any initializer
whose future awaits them. When such a callback requests a cached entry, its wait
edge originates at that nearest still-active initializer. A callback that owns
no initializer mutex does not need an artificial lock vertex; it must not erase
the ancestor relationship and hide a cycle.

Also check completed certificates. If rebuilding `B` consumes a warm `A` whose
transitive certificate already contains `B`, reject that dependency cycle even
though no mutex wait would occur. The identities here are concrete entries;
an SDK aggregate and its transport child can legitimately share a logical key
while occupying different namespaces.

## Expiry and caller freshness bounds

Record one observation timestamp for a raw result. Appending timestamps on refill
is replaced by a new complete artifact set and commit record. Disk loading,
decoding, aggregate assembly, and warm hits do not restart a child's age.

Within the process, compare monotonic deadlines. The persisted raw artifact
records the corresponding wall-clock observation and expiry needed after
restart. When loading, compute the remaining lifetime once and correlate it
with a monotonic deadline; do not repeatedly translate an old observation into
an increasing lifetime. Missing/untrustworthy timestamps, future observation
times, malformed records, and expired entries are misses. Unbounded validity
uses an explicit absence of deadline and does not perform overflowing addition.

The selected conservative policy never extends an existing certificate's
deadline. A caller requesting a shorter validity obtains a certificate bounded
by the earlier of the existing expiry and `observation + caller validity`.
The narrower certificate belongs to that return/dependency observation and does
not mutate another caller's certificate. A longer caller validity cannot extend
the stored deadline. A fresh execution establishes a new observation and expiry.
Zero validity always disables reuse, including joining another execution's result.

An aggregate's effective deadline is the earliest of its own bound and all
semantic child deadlines. Preserve the observations that produced these bounds;
do not derive each child's age from the aggregate's completion time. A fresh
aggregate construction is not evidence that its children were freshly fetched.
A workflow requiring a fresh input inventory must explicitly refresh/invalidate
the corresponding input requests.

Validate deadlines on memory lookup, disk reuse, and publication, including
after a durable commit. If a deadline elapses during commit, no typed reusable
value is published, and the persisted deadline makes the committed artifact
ineligible for reuse. Existing callers can still receive their completed snapshot.

## Artifact layout, staging, and commit

Each namespace/partition/fingerprint has a distinct physical artifact entry.
The cache layer owns this mapping; a request's logical invalidation prefix is
not reconstructed by assuming one particular namespace directory. Existing
command fingerprints retain executable, arguments, environment, working
directory, stdin, and adjacent input files.

Create a unique attempt directory for execution inputs and output artifacts.
Command adjacent files must not be written into another invocation's reusable
directory before execution. Once complete, an artifact set is immutable. A
small commit record selects the complete set currently eligible for reuse and
contains its semantic fingerprint and freshness information.

Stage files outside the I/O gate. Under the gate, validate the attempt's entire
certificate under state, release state, and update the commit record only after
the complete immutable artifact set exists. Then recheck expiry and publish the
typed value under state. Every shared-handle reader also holds the gate while
reading the commit record and selected files, so it observes one complete set.
Readers never discover reusable data by enumerating unfinished attempt directories.

Generation checks and reusable commit occur within the same gate acquisition.
Prefix and exact invalidation use that same gate. If invalidation wins first,
the old candidate cannot commit; if publication wins first, invalidation removes
its eligibility before completing. No reusable artifact is created merely
because its stage was written successfully.

Before the first visible commit-record mutation, install exact-entry publication
quarantine under state while retaining the gate. Clear that guard only after the
complete commit has succeeded. Error paths and explicit future drop leave the
guard in place, preventing a later reader from selecting an ambiguous record.
Required completion covers cooperative cancellation, not arbitrary future drop;
quarantine must not depend on an error handler getting another poll.

The commit record may carry format metadata needed to read the artifact store,
but persisted operation/output type names remain diagnostic. No persisted
`TypeId`, Rust schema token, or caller location controls payload acceptance.
Read reusable raw disk data using the current canonical decoder. Legacy entries
without trustworthy variant/authentication isolation or freshness metadata are
misses; preserve them for diagnostic inspection instead of upgrading their trust
by guessing missing information.

This protocol promises coherence among users of one handle while it is running.
It does not claim that a commit record provides cross-process locking, durable
generation agreement, atomicity across an operating-system crash, or recovery
from interrupted writes by unrelated readers. Those require separate phase 8
design and platform evidence.

## Aggregate persistence

The initial implementation memoizes assembled SDK results only in memory.
Persist raw leaf responses and command output where the operation's contract
permits it. Memory-only aggregates can retain diagnostic attempt artifacts, but
those artifacts are never selected by a reusable commit record.

A new handle rebuilds an aggregate by invoking its children, which may decode
valid raw disk artifacts using their current canonical decoders. It never restores
an assembled aggregate without authoritative child provenance. A purported leaf
that acquires semantic dependencies cannot publish a reusable durable aggregate
through this leaf contract. Persisted aggregate reuse is a later explicit design
change, independent of the decision to keep disk Rust type metadata diagnostic.

## Invalidation, rejection, and quarantine

Logical invalidation and canonical decode rejection have different scopes:

| Operation | Generation and durable scope |
| --- | --- |
| Invalidate logical scope | Advance its prefix generation; remove eligibility for every descendant logical entry across namespaces, variants, and partitions |
| Reject a cached canonical payload | Advance the exact concrete-entry generation; invalidate only that entry's reusable artifact selection |
| Caller-specific validation failure | Return that caller's error without invalidating the canonical value |

For invalidation, acquire gate, then state. Advance the relevant generation,
evict directly affected typed values, and install a uniquely identified
quarantine record before releasing state. Under the gate, invalidate the matching
durable commit records, including records not currently represented in memory.
After durable success, remove only quarantine records whose exact covered
invalidation has been resolved. Overlapping failures remain effective. A
successful full-prefix invalidation can resolve contained failed invalidations;
an exact-entry operation cannot clear an ancestor's quarantine.

Epoch validation immediately makes dependent aggregates stale even when their
values remain retained elsewhere in the map. New descendants observe the new
prefix generation. No invalidation needs to acquire or enumerate descendant
initializer mutexes, so Graph self-invalidation cannot perform a read-to-write
lock upgrade against its own execution.

If durable invalidation fails, keep quarantine and return the original I/O error
with context. If invalidation is owed completion work, use the existing bounded
completion policy. An unexpected drop after the state transition still leaves
quarantine in place. A subsequent memory miss cannot reload old disk artifacts
under the newly incremented generation.

**Quarantine is a disk-admission rule, not an assertion that every current memory
value is stale.** Pre-invalidation values fail their generation checks. Fresh
work started under current generations may return and publish a valid memory
value even while durable invalidation remains quarantined. Such a memory hit
must pass its complete certificate and caller deadline as usual. Disk reuse
under the quarantined scope remains disabled until recovery succeeds.

Fresh work in a prefix quarantined after invalidation retains nonreusable
artifacts and may share memory results; it does not publish reusable disk records
until that prefix invalidation succeeds. A successful complete fresh commit can
resolve an exact-entry quarantine caused only by a previous ambiguous publication,
provided no ancestor quarantine remains. Retain reason/coverage distinctions so
a write cannot accidentally clear an unresolved invalidation.

A rejected disk payload is not automatically reexecuted. Preserve the original
decoder error, retain the rejected artifacts, and attach any invalidation or
artifact-retention failure as context. This exact-entry rejection does not
invalidate an independently cached later page. The caller can explicitly retry
after receiving the failure.

## Publication failure and retention

Staging or reusable disk publication failure does not turn a successfully decoded
result into a failed service operation. Report the storage failure through the
existing artifact diagnostics, and permit memory publication if its certificate
and deadlines remain valid. An ambiguous or partially completed commit creates
an exact-entry disk quarantine before releasing the gate. Memory eviction must
not expose an older commit record whose freshness is no longer established.

Old-generation successful outcomes and rejected outcomes retain their required
execution artifacts as nonreusable attempts. Never replace a complete immutable
artifact set by writing new files into it. A failure to retain failure artifacts
remains attached to the original execution/decoder error. If required storage
cannot be written, report that failure rather than claim retention succeeded.

Artifact retention is an explicit operation policy independent of cache reuse
and semantic dependency classification. Credential infrastructure selects
sensitive-output retention before execution: tokens and other secret-bearing
raw stdout/bodies/errors stay out of success and failure artifacts and out of the
work cache. Preserve only redacted diagnostic metadata; provider token caches
remain the sole intended runtime owners. Generic staging must honor this policy
before writing any bytes, including for uncached credential commands.

Decoded-value retention is independently configurable and reclaimable. Eviction
does not invalidate an `Arc` held by a caller or remove a live initializer lock.
The coordination registry can remove an idle weak entry only when it cannot
upgrade; every active owner, waiter, or pending acquisition retains an `Arc`.
Lookup and reclamation share the state mutex so two coordination objects cannot
become live for the same concrete entry.

Owner claims and generation tombstones remain for the handle's lifetime in the
initial implementation. This preserves collision detection after value eviction
and prevents old certificates from regaining validity. The design bounds or
reclaims decoded values and idle coordination objects; it does not claim all
metadata maps have a fixed memory bound. Strictly bounded long-lived metadata
would require reference-aware reclamation and new proof before introduction.

Complete immutable artifact sets and diagnostic attempts remain inspectable
until explicit cleaning; there is no implicit background retention worker.
Cleaning must respect the same handle's publication protocol and avoid removing
active attempts. Stronger retention/cleaning coordination across processes is
deferred with the other independent-owner guarantees.

## Validation scenarios for implementation

Use synthetic operations, isolated temporary storage, explicit barriers/counters,
and controlled clocks. These scenarios are planned tests, not completed checks.

| Scenario | Required observable result |
| --- | --- |
| Concurrent identical reads | One successful raw execution and canonical decode; callers share allocation |
| Different entries and variants | Raw execution proceeds concurrently; physical inputs/artifacts never alias |
| Owner mismatch before variant lookup | Different owner or output contract fails before disk, execution, or lock waiting, including matching output types and different fingerprints |
| Initializer/waiter cancellation | Independently cancelled/dropped waiter leaves initializer intact; independently cancelled/dropped initializer releases ownership for another admitted waiter; shared-root cancellation ends all affected callers |
| Invalidation before commit | Original fill may return to its caller but cannot publish reusable memory/disk output |
| Commit before invalidation | Invalidation removes that commit's eligibility before returning |
| Invalidation during decode or stage | Candidate revalidation rejects obsolete publication |
| Descendant created during invalidation | It observes current ancestry; an older attempt cannot evade the prefix by creating a new child path |
| Warm child hit followed by invalidation | Parent already recorded the child's old certificate and cannot publish |
| Sibling/transitive dependency invalidation | Parent and grandparent fail freshness checks while unrelated scopes remain reusable |
| Graph self-invalidation and retry | No lock upgrade deadlock; retry does not renew the outer attempt's original stamp |
| Exact rejected page | Only that concrete entry is rejected; independent pages stay usable and dependent aggregates become stale |
| Two-task wait cycle | The cycle is rejected diagnostically; cancellation/error leaves no orphaned graph edge |
| Warm dependency cycle | Rebuilding a child cannot consume a cached ancestor that transitively depends on it |
| Multiple identical wait edges | One completed/cancelled waiter does not remove another's edge |
| Semantic effect eligibility | A possible required effect/uncached obligation disables parent lookup before a hit; runtime violation is rejected before child effects |
| Credential refresh | Token lifetime/refresh does not expire an otherwise valid inventory certificate |
| Shorter/longer caller validity | Shorter bound is honored; longer bound and disk reload do not extend existing deadlines |
| Child expiry during assembly/commit | Parent is not reusable; no completion timestamp renews child freshness |
| Failed durable invalidation then eviction | Old disk data remains blocked; fresh valid memory may be used; unrelated scopes remain usable |
| Ambiguous commit then eviction | Exact-entry quarantine prevents fallback to an untrusted old selection |
| Future dropped during commit mutation | Preinstalled publication quarantine survives drop and blocks ambiguous disk reload after memory eviction |
| Failed/cancelled semantic child caught by parent | A returned fallback is not memoized without a successful canonical child certificate |
| Sensitive credential outcome | Neither success nor failure artifacts/work-cache values contain raw synthetic token output; redacted diagnostics remain usable |
| Overlapping quarantine recovery | Exact success does not clear a failed ancestor invalidation |
| Value/coordination eviction during fill | The live initializer mutex remains unique; existing `Arc` snapshots remain readable |
| New handle | Aggregate rebuilds through children; current raw decoder is authoritative and diagnostic type text is not a gate |
| Partial artifact set/legacy entry | No incomplete set becomes reusable; unsafe legacy entries are misses |

## Inspected source relationships

- [Current artifact cache](../../crates/command/src/artifact_cache.rs): global
  serialized memory results, fingerprinted memory identity, and separate file
  writes motivate typed ownership, physical variants, and coherent publication.
- [Current cache key](../../crates/command/src/cache_key.rs): descendant disk
  scanning and memory invalidation motivate logical generations and quarantine.
- [Current shared work lifecycle](../../crates/command/src/cacheable_work.rs):
  post-execution decoding and artifact retention remain required responsibilities.
- [Graph helper](../../crates/azure/src/microsoft_graph.rs): continuation recovery
  invalidates its own subtree and retries, so hierarchy locks cannot span execution.
- [Principal inventory](../../crates/azure/src/principals.rs): separate child
  inventories motivate transitive dependency stamps and memory-only aggregates.

Only repository source and planning documents were inspected to make these
decisions. No service calls, credentials, production cache contents, or production
data are inputs to this document. Implementation and the scenarios above remain
pending.
