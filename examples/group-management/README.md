# Group management example

This consumer CLI describes desired membership and ownership for **existing**
Microsoft Entra groups. The current milestone implements local validation and
read-only planning. It never creates or deletes groups, renames them, or applies
membership or ownership changes. There is no `apply` command yet.

The command tree and reconciliation policy belong to this example. Cloud
Terrastodon supplies application startup, authentication, cancellation, typed
directory values, SDK requests, and output. The local `GroupRelationship`
vocabulary identifies independently reconciled lists in policy and diagnostics;
member and owner reads use their respective SDK request types. This package joins
the repository workspace, inherits its dependencies, and shares the root
`Cargo.lock`. The [root manifest](../../Cargo.toml) defines dependency versions
and sources. Its Facet types use the same Teamy package identity as Cloud
Terrastodon; the direct `facet` alias supports Figue's attribute macros. This
example is not published to crates.io.

## Desired state

See [desired.example.json](desired.example.json) for a document containing only
synthetic IDs and `example.invalid` annotations. Do not use those IDs against a
directory. The top-level value is an array of objects with required `tenantId`,
`groupId`, `displayName`, `membersMode`, `members`, `ownersMode`, and `owners`
properties. A group's identity is its tenant ID plus group object ID; its display
name is a human-readable expectation, not a rename instruction.

Group `displayName` values remain JSON strings. The shared `EntraGroupDisplayName`
type requires 1–256 characters; this example additionally rejects whitespace-only
annotations. Comparisons preserve spelling, case, and whitespace.

`members` and `owners` are managed independently:

- Set `membersMode` or `ownersMode` to `ignore` and supply an empty associated
  array to leave that relationship unmanaged. Nonempty ignored lists are rejected
  so the document cannot silently disregard listed principals.
- Set the mode to `additive` to ensure listed entries exist without removing
  others.
- Set the mode to `exact` to propose removals for unlisted entries. An empty exact
  list deliberately describes no relationships.

Every relationship requires an explicit mode and array; omission and `null` are
not accepted for those fields. Duplicate tenant/group identities and duplicate
principal references are rejected locally.

Each member or owner accepts `id`, `userPrincipalName`, or both. Object IDs are
authoritative. Supplying a stale UPN alongside an ID produces a warning when
the annotation no longer describes that object; a UPN resolving to a different
object is a conflict. UPN-only entries are resolved to object IDs during planning.
UPNs use the SDK's shared `UserPrincipalName` type, which preserves their spelling
and validates their basic `prefix@suffix` syntax. Guest `#EXT#` names remain
supported; the type does not verify a tenant's domains or provision accounts.
An optional `displayName` annotates ID references, including nested member groups
and service principals. Owners must be users or service principals. This milestone
deliberately treats devices and organizational contacts as unsupported: a desired
reference or an observed object of either kind blocks changes to that relationship.
Limited-information objects remain visible with unknown names rather than being
dropped from the inventory. These relationships are direct:
a nested group's transitive users do not become direct memberships of the
containing group.

A managed identity owner is identified by its service-principal **object ID**
in that tenant, rather than its application/client ID.

The current live adapter supports non-mail-enabled security groups. Dynamic
groups, on-premises-synchronized groups, and role-assignable groups are blocked;
missing role-assignability information also blocks proposals rather than assuming
the group is eligible.

The planner requires complete current relationship inventories before comparing
an exact list. Microsoft Graph v1.0 has documented service-principal omissions
in relationship listings, so this adapter marks both inventories incomplete and
blocks all live exact-mode comparisons. Validation can still check an exact-mode
document locally. Additive plans warn about incomplete inventory; an unobserved
service principal also blocks that relationship because the adapter cannot
establish whether it is already present. Other additive proposals still require
a fresh existence check before a future apply. The sample includes exact mode
and a service-principal reference to demonstrate these contracts; it is not an
example of an automatically actionable live plan.

## Validate and plan

Run these commands from the repository root. Validation uses the local file and
does not request directory credentials or service data, even when inherited
workload credential configuration is incomplete or malformed:

```powershell
cargo +1.96.0 run --locked -p cloud-terrastodon-group-management -- validate --desired examples/group-management/desired.example.json
```

Planning reads directory data **when you run it**. Supply your own desired-state
file and an explicitly chosen authentication source:

```powershell
group-management plan --desired groups.json --auth-source workload-identity --out plan.json
group-management plan --desired groups.json --auth-source azure-cli --output-format json
```

Without `--out`, the plan is emitted to stdout using the shared output format.
With `--out`, the same plan is also saved as versioned JSON. The file records
resolved identities, observations, proposed operations, and warnings; it does
not contain credentials. Treat it as directory information when publishing
pipeline artifacts. Saving is atomic and refuses to overwrite an existing plan;
choose a new artifact path for each review. JSON streams to a caller-owned
buffered temporary file. Its outer cancellation-aware writer checks bounded
writes before the buffer accepts output; the operation also checks before syncing
or publishing the complete file. The buffer is explicitly flushed, with unfinished
buffered output discarded on errors. Cancelled work drops the unpublished
temporary file. Allocation, user proxy code, and operating-system calls already
in progress cannot be interrupted, and computation that does not write continues
until the next checkpoint.
This first milestone does not execute a saved plan.

Plan schema version 2 stores `warnings` and `blockers` as structured objects.
Each has a camelCase `code` identifying the diagnostic and typed fields carrying
the relevant IDs, expected and observed values, or relationship. Principal lookup
blockers include the requested reference and a nested typed `error`. Human-readable
messages come from the Rust diagnostic types' `Display` implementations; consumers
can match variants or inspect `code` without parsing those messages. Diagnostics
are sorted by their structured fields and exact duplicates are removed.

Plans with blockers are still emitted and saved when requested, then the process
exits unsuccessfully so a pipeline cannot treat them as ready to continue.

Every selected tenant must pass authentication preflight before group inventory
begins. Environment-backed workload identity authentication covers the tenant
configured for that identity; the tool fails when a document requires another
tenant rather than silently skipping it or switching to interactive login.
Tenant-scoped pipeline jobs can select their tenant explicitly:

```powershell
group-management plan --desired groups.json --only-tenant 11111111-1111-4111-8111-111111111111 --auth-source workload-identity --out tenant-plan.json
```

The resulting plan reports that selection. A tenant ID in desired state selects
the target directory; it does not grant the identity permissions in that tenant.
Configure the pipeline service connection and Microsoft Graph permissions before
running the read-only planner.

Shared `App` startup provides the global flags and builtin help/version handling.
The entrypoint uses `parse_from` followed by `run_parsed` so local validation can
skip credential-environment configuration while retaining common logging and
cancellation. Validation selects `AuthSource::None`, which resolves to
`AuthContext::None` and disables credential discovery and authenticated requests.
The root dispatcher forwards one `CliOutput`; `main` emits it once.
SDK reads receive the handler's `CancellationScope`, with checkpoints between
principal-resolution and inventory operations. Automatic cancellation is checked
at poll boundaries; synchronous local validation and comparison are not preempted.
The local validation command does not bind an authentication context to a tenant
or acquire a token.

Tests include Figue argument-consistency checks and 128 successful roundtrips per
`validate` and `plan` command leaf. These generate and parse argument values
without invoking commands, reading desired-state files, or resolving authentication.

## Layout and next milestones

`src/main.rs` is a thin entrypoint. `src/cli/group_management_cli.rs` owns the root
arguments and command enum; leaf commands live under `src/cli/command/` in their
own `group_management_*_cli.rs` files. Domain values, document validation, plan
records, and planning operations live outside CLI modules, with each named type
in its own file. Future commands should preserve that structure instead of
growing a single-file example.

`DesireDocument` owns validated group declarations and their source provenance.
Both file loading and programmatic construction validate local policy; callers
inspect declarations through a shared slice. Selecting one tenant consumes the
document and retains its original source digest for the resulting plan.
The CLI loads the document with `DesireDocument::try_from_path`, then calls
`document.try_plan(...)` to obtain fresh directory observations. That operation
uses `DesireDocumentAchievementPlan::try_new` to compare the document with group
snapshots and resolved principals, producing proposed changes and diagnostics.
The plan's type file keeps its declaration and small query methods; construction
and persistence use separate inherent `impl` blocks in its `try_new.rs` and
`save.rs` child modules. Call `plan.save(path, cancellation)` to persist an artifact.

Applying plans will require a separate milestone: drift revalidation, explicit
confirmation, durable mutation journaling, required-completion cancellation
scopes, and cache invalidation. Group creation and deletion remain outside the
initial contract. Consumer repositories can own their desired-state documents
and plan/approval/apply pipelines that consume the generic executable.

Windows and Linux release packaging follows after those contracts are exercised.
For a WSL build, use a Linux-filesystem checkout synchronized to the source being
verified. A local Git remote transfers committed objects, so uncommitted Windows
changes require an explicit source transfer before they can be built there.
