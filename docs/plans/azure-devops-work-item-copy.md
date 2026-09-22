# Azure DevOps work item reads, creation, and deep copy

Status: implemented. See [CLI usage](../AZURE_DEVOPS_WORK_ITEMS.md).

The previous surface exposed saved queries directly under `az devops query` and
raw REST calls, without typed item reads or writes. Queries now live under
`az devops work-item query`, alongside item, field, relation, and type commands.
All requests preserve Cloud Terrastodon's authentication context and tenant
selection, including browser sessions independent of Azure CLI's login.

## API contracts

Requests use REST API version 7.1. Relevant official documentation:

| Operation | Contract |
| --- | --- |
| Show item | [GET workitems/{id}](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/get-work-item?view=azure-devops-rest-7.1), with fields, asOf, and $expand |
| List explicit items | [POST workitemsbatch](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/get-work-items-batch?view=azure-devops-rest-7.1), at most 200 IDs per request; errorPolicy=fail |
| Create item | [POST workitems/${type}](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/create?view=azure-devops-rest-7.1), JSON Patch |
| Update fields and links | [PATCH workitems/{id}](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/update?view=azure-devops-rest-7.1), JSON Patch |
| Field definitions | [Fields](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/fields/list?view=azure-devops-rest-7.1) |
| Project types | [Work item types](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-item-types/list?view=azure-devops-rest-7.1) |
| Type constraints | [Type fields](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-item-types-field/list?view=azure-devops-rest-7.1), expanded constraints |
| Relation definitions | [Relation types](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-item-relation-types/list?view=azure-devops-rest-7.1), direction and opposite-end reference name |
| Query inspection | [Get query](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/queries/get?view=azure-devops-rest-7.1), by UUID or path |
| Query creation | [POST queries/{folder}](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/queries/create?view=azure-devops-rest-7.1), JSON containing name and wiql |
| Query execution | [WIQL](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1), references or link results without item hydration |

JSON Patch preserves explicit null separately from an omitted value, escapes
field names as JSON Pointer segments, and uses `application/json-patch+json`.
Project and custom type names are URL path segments. Reads and writes are
uncached to avoid stale fields and relation indices.

`--if-rev` inserts a `test /rev` precondition. Relation update/removal first reads
the source, resolves exactly one type/target match, and protects that revision
and index. Target contents are never fetched by relation commands. Editable link
attributes are comment and lock state; server-generated metadata is not replayed.

Writes expose validateOnly and suppressNotifications. Validation responses can
contain temporary IDs, so generic create/update responses retain arbitrary JSON.
Copy requires a persisted item response with a positive ID.

## Copy algorithm

1. Require the source project and choose one `asOf` snapshot timestamp. Historical
   responses have revision URLs; validate and normalize those separately from
   canonical relation target URLs.
2. Read the root and follow only Child relations with `--deep`. Check every
   frontier against `--allowed-ids` before any request. Other targets are only
   examined as references.
3. Validate a closed, single-project tree: reject missing items, cycles, duplicate
   edges, multiple parents, and conflicting Parent/Child references.
4. Fetch field definitions, fields for the observed types, and relation types.
   Keep supported writable fields and JSON types/HTML. Exclude generated IDs,
   audit fields, board/rank values, and history. Reduce identities to unique name
   or ID inputs. Required server-derived fields are not missing user inputs.
5. Build the complete plan before writing. Report omitted fields, comments,
   attachments, shallow-copy children, and external links. Root parent and
   external relations require their respective retention flags.
6. Create nodes parent-first, recording old-to-new ID/URL mappings. Each child's
   create patch links to its newly created parent.
7. Add internal non-hierarchy links after all destinations exist, remapping both
   endpoints and deduplicating opposite ends using relation-type metadata.
8. Read new items and verify parents and internal links before completion.

Copy supports the source project. Attachment binaries, comments/revision history,
generated fields, and embedded work item URLs in rich text are not migrated.
Server process rules remain authoritative; metadata cannot prove every create
will succeed, so an error can leave a partial copy.

## Recovery

The journal holds the plan, mappings, link progress, and completion status. It is
atomically replaced and synchronized before/after each remote write. A companion
file holds an OS lock throughout execution, preventing concurrent resume while
the journal is replaced. The OS releases the lock on process exit.

A new run never overwrites a journal. Resume checks scope, ordering, unique
mappings, and progress, skips acknowledged writes, and repeats read-only
verification. An `in_flight` entry indicates a request may have committed without
its response being recorded. Replay is refused until an operator reconciles the
operation and corrects the journal. There is no automatic rollback.

## Implementation and tests

- [Work item model](../../crates/azure_devops_types/src/azure_devops_work_item.rs)
- [Work item ID](../../crates/azure_devops_types/src/azure_devops_work_item_id.rs)
- [JSON Patch operation](../../crates/azure_devops_types/src/azure_devops_json_patch_operation.rs)
- [Item and metadata requests](../../crates/azure_devops/src/work_item_requests.rs)
- [Shared field/link builders](../../crates/azure_devops/src/work_item_patch.rs)
- [Query list request](../../crates/azure_devops/src/azure_devops_work_item_query_list_request.rs)
- [Query get request](../../crates/azure_devops/src/azure_devops_work_item_query_get_request.rs)
- [Query invoke request](../../crates/azure_devops/src/azure_devops_work_item_query_invoke_request.rs)
- [Query create request](../../crates/azure_devops/src/azure_devops_work_item_query_create_request.rs)
- [Copy planner and executor](../../crates/azure_devops/src/work_item_copy.rs)
- [CLI](../../crates/entrypoint/src/cli/command/azure_devops/work_item/azure_devops_work_item_cli.rs)
- [SDK tests](../../crates/azure_devops/src/work_item_tests.rs)
- [CLI tests](../../crates/entrypoint/src/cli/command/azure_devops/work_item/azure_devops_work_item_cli_tests.rs)

Offline tests generate identifiers and scope names. They build requests without
sending writes and execute copies with an in-memory writer and temporary local
journals. The opt-in live test reads only runtime-supplied IDs and schema metadata:
no queries, traversal, service identifiers in fixtures, or response logging.
Its returned errors are sanitized.
