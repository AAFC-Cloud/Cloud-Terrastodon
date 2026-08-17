# Reflected object explorer

The Ratatui object browser is a thin presentation layer over the
`object_explorer` engine in `crates/ui_ratatui`. The engine is currently
crate-private while its API settles; it has no Ratatui or Crossterm dependency
and can be exercised headlessly.

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
