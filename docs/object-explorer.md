# Reflected object explorer

The Ratatui object browser is a thin presentation layer over the
`object_explorer` engine in `crates/ui_ratatui`. The engine is currently
crate-private while its API settles; it has no Ratatui or Crossterm dependency
and can be exercised headlessly.

## Excalidraw rendering and discovery planning

```powershell
cargo run -- ratatui --excalidraw target/objects.excalidraw
```

The terminal owns selection, object construction, and invocation. Excalidraw is
a disposable diagram of those objects. **Ctrl-O** replaces the drawing with the
focused slot, including an incomplete builder. **Ctrl-U** replaces it with every
object matching the current tab's breadcrumbs, including objects outside the
terminal viewport. Fields are shown as individual text elements; diagrams are
not grouped into one canvas object.

Omit `--excalidraw` to create a uniquely named `object-explorer-….excalidraw` file
in the working directory on the first export. The status line reports its path;
subsequent exports reuse it. No file is created just by opening the explorer.
The parent directory of an explicit path must exist. Each export replaces the
previous drawing, including canvas edits. Save or revert pending editor edits
before rendering again; the VS Code fork reports conflicting unsaved edits.
Atomic replacement uses the existing cooperative `.sync-lock` protocol and
waits for an active writer. Ctrl-C closes the explorer and drains publication
already in progress. While an export collects its consistent snapshot, browser
commands wait; Esc or Ctrl-C can close the session.

To configure a discovery plan without invoking cloud APIs:

1. Create a **DiscoveryPlanBuildRequest** slot using the normal shape picker.
   `DiscoveryPlan` is the generated graph; its nodes and edges are output,
   rather than the place to configure targets.
2. Leave `functions` at its default to use the linked registry, or choose
   **create new owned List<RegisteredFunction>** to select a smaller catalog.
3. On `seeds`, choose **create new owned List<RegisteredShape>** and select the
   input shapes to assume available, for example `AzureDevOpsOrganizationUrl`
   and `AzureDevOpsAuthContext`. This supplies type assumptions, not credentials
   or actual objects. Ctrl-Enter with nothing checked supplies an empty list.
4. On `targets`, choose **create new owned List<RegisteredShape>** (the reflected
   `Vec<RegisteredShape>`). Search for
   target types such as `AzureAppServiceSiteConfig` or
   `AzureDevOpsBuildDefinition`. Enter includes the highlighted result and
   checked results; Tab toggles checks, and Ctrl-Enter applies only the checks.
5. Invoke the completed request to produce a **DiscoveryPlan**. This operation
   only inspects the supplied shapes and functions. It does not execute the proposed requests,
   resolve credentials, or call services.
6. Focus the resulting plan slot and press **Ctrl-O**. Its graph includes type
   fields and shows missing inputs.

The separate **DiscoveryPlanRequest** infers seed shapes from actual ready
objects in the explorer. Its `inputs` field defaults to the unfiltered pool;
supply a tab's projected `breadcrumbs` value to restrict it. Registered types
and incomplete builders do not count as ready inputs. Use this request when
planning from objects already acquired, rather than explicit shape assumptions.

To export just the discovery graph through the normal registry workflow, create
a **ProduceExcalidrawDumpPlanRequest** slot. It has two user-supplied fields:

```text
plan: DiscoveryPlan
filename: PathBuf
```

On `plan`, select an existing plan and choose **Clone** to retain it or **Move**
to transfer it into the request. Alternatively choose **create DiscoveryPlanBuildRequest
for DiscoveryPlan (edit, then ...)** to open a request slot in its own tab for configuration.
The export's `plan` field remains unset. Configure seeds and targets, explicitly
invoke the builder request, then return to the export and select the resulting
plan. **Invoke default** is a separate action that immediately runs a producer
with default inputs; it does not pause for editing.

On `filename`, create an owned
`PathBuf` and enter a path such as `target/plan.excalidraw` in the scalar editor.
Invoke the completed request to write the drawing and receive its output path
as a `PathBuf` slot. The caller supplies the cancellation scope automatically.

This export uses the same request graph as `dump plan --diagram`, including
required inputs, missing inputs and schema fields. It atomically replaces the
destination through the cooperative `.sync-lock` protocol; the parent directory
must exist. The operation writes a local diagram without executing the plan or
calling cloud APIs. **Ctrl-O** and **Ctrl-U** remain the shortcuts for rendering
object slots and tab contents to the explorer's diagram destination.

The plan describes possible type coverage; it does not yet retain executable
value bindings or run the discovery. Existing **ProduceJsonRequest** exports
objects selected by a `Breadcrumbs` query for subsequent Power BI ingestion.
The Ctrl-O/Ctrl-U object snapshots observe at most 256 fields per object and
explicitly mark any omission; this display bound does not change JSON export or
truncate tab membership.

On a completed owned enum slot, focus `variant` and press Enter to change it.
Cancelling or selecting the current variant preserves the value. Selecting a
different variant replaces the previous payload; if it needs fields, the same
slot returns to Building. A value borrowed by another object cannot be changed
until those borrows are released. Projected fields remain read-only views.

The earlier `dump plan --interactive` command also uses terminal-owned target
selection now. Its canvas contains the generated graph, with the former search
and selected-target controls retired. The `ct excalidraw interactive` control
playground remains available separately.

## Auditing registry operations

`ct registry entry list` lists registered operations, including request
invocations, constructors, conversions, and projections. Text output includes
their declared effects, input/output types, and registration source locations.
It inspects linked metadata without running operations or discovering cloud
credentials. Empty effects are permitted for pure operations and displayed as
`none`. A request with missing effects displays `UNCLASSIFIED` so it stands out
during an audit. Declarations still need review against their implementations.

```powershell
ct registry entry list --requests-only
ct registry entry list --requests-only --effect write
ct registry entry list --requests-only --effect solicit
ct registry entry list --requests-only --filter AzureDevOps

$entries = ct registry entry list --requests-only --output-format json | ConvertFrom-Json
$entries | Where-Object { $_.effects -contains 'write' } | Sort-Object input |
    Format-Table input, effects, registration_file, registration_line
```

`--effect` matches membership, so `--effect write` includes operations declaring
both `Read` and `Write`. `Solicit` declares that an operation asks a person for
input, selection, or confirmation; pickers that also query services declare
`[Read, Solicit]`. `--filter` performs a case-insensitive substring search
over input/output type names, labels, trait contracts, and source filenames. JSON retains
those fields plus the operation kind and receiver mode for further filtering.

## Data model

- An `Arena` is the instance-owned source of truth for every owned value.
  Tabs, requests, responses, strings, and lists are all ordinary arena roots.
- A `SlotId` identifies exactly one arena root. It never identifies a field,
  list element, projection, or borrow.
- A `ValueAddress` is a `SlotId` plus reflected field/index/key path. For
  example, `slot 5[0].permissionObjects[4]` addresses a value inside slot 5
  without allocating another slot.
- A `CardAddress` identifies either a reflected `ValueAddress` card or the
  new-object placeholder. A `CardRowKey` identifies a semantic row. Neither
  is a flattened display ordinal.
- A `Tab` is ordinary data containing exactly a name and `Breadcrumbs`.
  `OpenTabs` stores only non-owning tab `SlotId` values and per-tab UI state
  is kept separately.

Reflection is observational. Visiting a request, Tab, or Breadcrumbs value does
not invoke it or evaluate a query.

## Queries and cards

`Breadcrumbs` is a small, composable query program over the complete stream
of Ready arena roots and their reflected descendants. Projection, shape,
address-kind, value-filter, field-projection, and Pop operations preserve
encounter order. Pop coalesces only adjacent duplicate parents, so retained
state remains bounded.

Object-pool browsing also includes lifecycle cards for incomplete, pending, and
consumed roots. Shape choices and filters use their known types even when they
have no completed value; field-value filters and exports still require Ready
values. Add **hide consumed slots** from **+Add Breadcrumb** to omit
consumed roots in that tab while keeping incomplete builders and pending work
visible. This filters the view without deleting slots; remove the breadcrumb
with Delete to show consumed slots again.

`QueryCursor` evaluates that program cooperatively. Each call receives a
`WorkBudget`; a long or no-match scan returns Pending when the budget is
spent. Its address cache is fixed-capacity. The Ratatui controller requests a
`CardWindow` sized from the current viewport, and rendering receives only
bounded snapshots—never the Arena, a Facet `Peek`, or all projection paths.

Changing the number of projections before a selected card therefore cannot
change selection identity. If the selected address itself becomes stale, the
controller re-anchors explicitly instead of silently treating the old ordinal
as another card.

## Browser navigation

Moving Up past the first semantic card row focuses the breadcrumb bar.
Left/Right selects an existing operation or +Add Breadcrumb; Enter edits the
selected operation and Delete removes it. Down returns to card rows.

Shift+; and +Add Breadcrumb open the same fuzzy PickerTui, with filter shapes
first. A new shape filter starts with no marked choices. Editing an existing
shape or field-projection breadcrumb reopens its contextual picker with
persisted choices marked, using only the query prefix before that operation.

Activating a reflected child beneath a restrictive query opens an exact
projection tab and leaves the source tab unchanged. Exact projections seed
the query cursor at their ValueAddress; they do not linearly scan unrelated
earlier roots.

Creating an object from a projected or filtered tab opens a separate projection
tab for the new root, including while its shape is unset. The source query stays
unchanged. If an existing viewport anchor disappears, the browser returns to the
first remaining match. Completed missing-address scans stay complete until the
operation or revision changes, rather than restarting on every refresh.

## Construction, movement, and borrowing

`ValueBuilder` owns incomplete construction state for one reserved root. A
field binding is explicit: Default, Inline, CloneFrom, MoveFrom, BorrowFrom, or
PendingProducer. Once all required fields can be materialized, only that
builder is finalized immediately into the Ready root.

Generic value discovery scans compatible roots and reflected descendants under
a work budget. A candidate retains its `ValueAddress` and owner provenance,
such as:

`slot 3.breadcrumbs — field breadcrumbs of slot 3 (Tab)`

The engine derives legal operations from ownership:

- an independent root may be moved or cloned;
- a projected field/element may be cloned but is not independently movable;
- a compatible Cow field may borrow, move, or clone a pointee when validation
  permits it.

Clone choices require reflected clone support, including recursive list cloning
for projected fields such as `Tab.breadcrumbs.operations`. Previewing a choice
does not execute `Clone`. Lifetime-generic request types must explicitly expose
their existing Rust implementation with `#[facet(traits(Clone))]`: the pinned
Facet derive does not infer it for those types. Cloning preserves borrowed
`Cow` inputs and their leases; it does not promote them to owned values.

The browser uses an inline viewport on stderr, with tracing output retained in
normal terminal scrollback above it. Scroll up in the terminal to read full
errors, including their fields, source locations, and span context. Logs keep
the tracing subscriber's styling and filters; `--debug` is not needed to see
errors. Nested pickers temporarily take the terminal, then buffered logs are
printed when the browser resumes. Closing also flushes cleanup diagnostics.
Invocation failures preserve the full error report, including its original
capture location and causes. JSON serialization also records its caller before
crossing the engine's message channel, so the log includes the failing export
or read location alongside the UI's `error!` emission location.

Card text wraps at the available width; selection markers and scrollbars account
for the wrapped lines. The status bar keeps short feedback and directs failed
actions to the full log above. Rejected actions leave the explorer open.
The engine retains ownership of failed-slot and rollback behavior; lost engine
connections and terminal failures still end the session.

`BorrowGraph` and transferable `BorrowLease` values protect the complete
source root while a reflected borrow exists. Builders, pending invocations,
ready borrowers, cancellation, cloning, moving, and deletion all transfer or
release those leases explicitly. No synthetic “view slot” represents a borrow.

## Invocation and coherent export

The engine is the single linear writer of Arena state. Independent async
requests may run concurrently outside it; completed outputs become visible
only when their ingestion command reaches the engine.

`ProduceJsonRequest` has ordinary reflected fields:

```text
breadcrumbs: Breadcrumbs
filename: String
```

It does not take a Tab. To export a tab query, the generic picker discovers the
Tab's reflected Breadcrumbs field and clones it into the request. This is the
same owner-aware field-selection path used by every type.

The ordinary registry `IntoFuture` implementation reacquires a scoped
`ArenaQueryContext` inside its async body. An explicit
`ProduceJsonRequest::run(context)` path supports headless callers. Export
opens a bounded query session, establishes a linear read barrier, serializes
one coherent JSON array in work/byte-bounded batches, and writes with async
backpressure. Mutations and completed-result ingestion queue until the export
ends or is cancelled; background futures may continue running. There is no
MVCC snapshot and no materialized `Vec<ArbitraryJson>`.

## Extending or testing without Ratatui

`cloud_terrastodon_registry::DiscoverySimulation` explores registered type
coverage entirely offline. Supply explicit seed `RuntimeValue`s, an audited
operation catalog, a deterministic random seed, round/value budgets, and a
caller-owned cancellation scope. Its result includes discovered values, their
depth and producing functions, simulated requests, convergence, and diagnostics.

For a known request type, `Request::REGISTERED_INTO_FUTURE` refers directly to
the entry emitted by `register_into_future!`. The function-like macro adds an
inherent associated constant on the registered request specialization; no derive
macro or registry search is involved. The reference aliases the exact linked
static so operation identity is preserved. Requests must be defined in the crate
that invokes the macro, as required for an inherent implementation.

For a locally defined value type, use
`register_arbitrary!(Type, associated = REGISTERED_ARBITRARY)` and access
`Type::REGISTERED_ARBITRARY`. Collection and externally owned value types can use
`register_arbitrary!(pub static NAME: Type)` instead. Both forms expose the exact
linked constructor, so tests and selected catalogs need no registry search.

For a conversion whose input type is defined in the registering crate, use
`register_from!(Input => Output, associated = REGISTERED_INTO_OUTPUT)` and include
`Input::REGISTERED_INTO_OUTPUT` in a catalog. The caller names the associated
constant so a type can expose several conversions. Like request handles, these
references alias the exact linked static without searching the registry.

Registrations between externally owned types can use
`register_from!(pub static NAME: Input => Output)` and include `&NAME` in a
catalog; this form requires no inherent implementation. The anonymous
registration form also remains available. Explicit test catalogs list their
selected request/conversion references directly and explicitly include their
response constructors. Pass `known_functions()` for the full linked catalog.

`Function::trait_contract` declares a `FunctionTraitContract`: `IntoFuture`, `From`,
`TryFrom`, `Arbitrary`, or `Other(&'static str)` for custom registrations. Static
custom names preserve constant registration and copyable metadata. Discovery
compares enum variants; `Other("Arbitrary")` is not the built-in generator
contract. `Function::label` remains free-form display text for UI verbs and
custom operation names, rather than a policy selector. Registry metadata JSON
exposes the typed contract in the `trait_contract` field.

The simulator never invokes executable request futures. An admitted request
must declare `Read` without `Write` or `Solicit`; its response comes from the
exact `Arbitrary` constructor supplied in the catalog. Missing constructors are
reported in diagnostics; the simulator never falls back to the global function
registry. The simulator follows reflected descendants, executes pure synchronous
conversions/projections, and assembles downstream requests from discovered domain
values. Optional inputs can remain `None`.
Missing identities and selectors are not filled by arbitrary request generators,
string proxies, configured defaults, or unrelated constructors. Removing a
conversion edge therefore exposes a real gap in the discovery chain.

Each eligible registration is attempted once after its input becomes available.
Registrations with missing inputs are reconsidered as the pool grows; failed
attempts are not retried in later rounds. Completion is tracked by registration
identity, so repeated catalog references do not repeat work and separate entries
with the same signature remain distinct. Response constructors have their own
bounded sampling retries. Traversal follows the supplied catalog order; reproduce
a run with the same order and random seed.

Values are indexed by Facet shape in `HashMap<&'static Shape, Vec<DiscoveryValue>>`
buckets, currently retaining one representative per shape. Lookups can select a
shape directly; map iteration order is unspecified, while discovery follows
catalog order and depth-first reflection order. `max_values` caps the total
stored values across all buckets, including cloned descendants, rather than the
number of shape keys. A duplicate shape is discarded and uses no slot; container
elements and heap bytes already owned by a retained value are not separately
capped. Reaching the pool limit stops exploration without
claiming convergence. Convergence describes that sampled type pool, rather than
exhaustive payload or live resource coverage. It does not establish
organization/tenant correlations for a production dump scheduler.

For a target-directed view without generating values or executing conversions,
[`ct dump plan`](dump-planning.md) computes structural reachability from explicit
seed shapes and exports the request dependencies as JSON or an editable
Excalidraw diagram. The planner and simulator share the linked registry but
answer different questions: possible type coverage versus sampled value flow.

`Function::is_read_only()` classifies data mutation; unattended
admission additionally excludes `Function::requires_user_input()`. Authentication
context still controls whether credential acquisition can interact with a user.
Operation effects and pure generator/conversion
contracts must be audited; the simulator cannot infer side effects from code.

Executable request registrations require explicit nonempty effects at
compilation, and the entrypoint's linked registry check covers custom async
registrations. Pure constructors/conversions can retain empty effects. Cache
artifacts and diagnostics are operational infrastructure; deliberate domain,
configuration, and export-file changes declare `Write`.

The Azure DevOps integration test seeds an arbitrary project-list request and
follows fabricated projects through the registered project-to-selector
conversion into a build-folder request and fabricated folder records. Its
negative control removes the conversion and verifies that folder discovery
stops, despite the request's available arbitrary constructor.
Another test traverses the full linked catalog and prints sampled type names,
depths, and convergence totals when run with `--nocapture`.

```pwsh
cargo test --offline -p cloud_terrastodon_registry --lib discovery_simulation_tests
cargo test --offline -p cloud_terrastodon_azure_devops --test discovery_simulation
cargo test --offline -p cloud_terrastodon_azure_devops --test discovery_simulation full_linked_catalog -- --nocapture
cargo test --offline -p cloud_terrastodon_entrypoint --lib entrypoint::tests::async_requests_declare_effects -- --exact
```

Within the UI crate, a headless test follows this pattern:

1. create an `ExplorerEngine` and bounded `ArenaQueryContext::channel`;
2. run `engine.run(inbox)` concurrently with the test client;
3. use the context's engine handle or `ObjectBrowserController` to insert
   values, build fields, invoke functions, and request bounded windows;
4. identify nested values with `ValueAddress`, never a display index;
5. drop sessions and handles so the engine run loop can finish; and
6. assert addresses, lifecycle states, work counters, cache/batch bounds, and
   borrow edges rather than elapsed time.

Another UI should retain only command handles, logical selection, local picker
text/selection, and bounded snapshots. It must not retain `RuntimeValue`,
`Peek`, a second root store, or evaluated tab results.

The engine remains in `ui_ratatui` for now because its API is crate-private
and still shares registry-oriented composition with the browser. Extracting it
today would create public API churn without reducing dependencies; the
terminal-free architecture test preserves a mechanical future extraction
path.

Useful validation commands:

```pwsh
cargo test -p cloud_terrastodon_ui_ratatui --lib project_admin_query_composes_and_exports_without_azure
cargo test -p cloud_terrastodon_ui_ratatui --lib million_value
cargo test -p cloud_terrastodon_ui_ratatui --lib
```
