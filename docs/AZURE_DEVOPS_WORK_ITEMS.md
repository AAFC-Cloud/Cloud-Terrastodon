# Azure DevOps work items

Use `ct az devops work-item`. Saved queries moved from `az devops query` to
`az devops work-item query`.

| Surface under `az devops work-item` | Commands |
| --- | --- |
| Items | `list`, `show`, `create`, `update`, `copy` |
| Field values | `field list`, `show`, `set`, `remove` |
| Field definitions | `field definition list`, `show` |
| Links | `relation list`, `show`, `create`, `update`, `remove` |
| Link definitions | `relation type list`, `show` |
| Project types | `type list`, `show` |
| Fields for a type | `type field list`, `show` |
| Queries | `query list`, `show`, `invoke`, `create` |

Every leaf command accepts `--org`, `--project`, and `--tenant`. With
`--auth-source browser`, commands use Cloud Terrastodon's stored browser session,
independently of Azure CLI's login. Organization defaults to the configured value.
Creation, copying, saved-query management, and type metadata require a project.

Examples use PowerShell variables for your chosen scope, type, field, and IDs.
`$itemIds` is a comma-separated string of permitted IDs.

```powershell
$scope = @('--org', $organization, '--project', $project, '--tenant', $tenant)
ct --auth-source browser az devops work-item show $itemId @scope
ct --auth-source browser az devops work-item list --ids $itemIds @scope
ct --auth-source browser az devops work-item field show $itemId System.Title @scope
ct --auth-source browser az devops work-item field definition list @scope
ct --auth-source browser az devops work-item type field list --type $typeName @scope
ct --auth-source browser az devops work-item relation list $itemId @scope
ct --auth-source browser az devops work-item query invoke --wiql '@query.wiql' @scope
```

Results are JSON. `list` fetches explicit IDs in batches of at most 200. `show`
and `list` accept `--fields` (comma-separated references), `--expand`, and
`--as-of`. Query invocation returns references without reading item contents.
Metadata `show` accepts a name. Type-field commands also require `--type`.

## Create and edit

```powershell
ct --auth-source browser az devops work-item create --type $typeName --fields '@fields.json' --parent $parentId @scope
ct --auth-source browser az devops work-item field set $itemId System.Title --value '@title.json' @scope
ct --auth-source browser az devops work-item field remove $itemId $fieldName @scope
ct --auth-source browser az devops work-item update $itemId --patch '@patch.json' --if-rev $revision @scope
ct --auth-source browser az devops work-item relation create $itemId --type System.LinkTypes.Related --target $targetId @scope
ct --auth-source browser az devops work-item relation update $itemId --type System.LinkTypes.Related --target $targetId --comment 'Updated comment' @scope
ct --auth-source browser az devops work-item relation remove $itemId --type System.LinkTypes.Related --target $targetId @scope
ct --auth-source browser az devops work-item query create --folder $queryFolder --name $queryName --wiql '@query.wiql' @scope
```

JSON inputs accept inline JSON or `@file`, preserving value types. For example,
`fields.json` contains `{"System.Title":"New item","System.Description":"<p>Description</p>"}`;
`title.json` contains a JSON string including quotes. Null differs from removing
a field. Creation also accepts `--title` and `--relations`, an array of
`{rel,url,attributes}`. Supply the title once.

Relation selection uses a type reference name plus target ID or URL. Update
changes `comment` and `isLocked` attributes (also accepted as `--attributes` JSON).
Changing the target or type requires removing and creating the link.

Item and relation writes accept `--validate-only` and `--suppress-notifications`.
Existing-item writes accept `--if-rev`; relation writes automatically protect the
fresh revision used to find the link. Saved-query creation has no validation mode.

## Deep copy

```powershell
# Preview without writes, restricted to an explicit set of source IDs.
ct --auth-source browser az devops work-item copy $itemId --deep --allowed-ids $itemIds --dry-run @scope
# Create the copy and retain its progress journal.
ct --auth-source browser az devops work-item copy $itemId --deep --allowed-ids $itemIds --journal ./copy-progress.json @scope
# Resume the saved plan; omit discovery options such as --deep.
ct --auth-source browser az devops work-item copy $itemId --resume --journal ./copy-progress.json @scope
```

Parents are created before children; internal links are remapped after all new
targets exist. The resulting hierarchy is verified. No external link target is
fetched during discovery. A Child reference outside `--allowed-ids` fails planning
before reading it or writing anything. Without this option, discovery may read
any descendant in the organization.

`--title` changes only the root's title. `--keep-parent` retains its existing
parent; `--keep-external-relations` retains external links except attachments.
Both default off. Without `--deep`, only the selected item is copied.

Copy supports the source project and preserves supported writable fields and HTML.
It omits attachments, comments/history, and generated board/rank/audit values.
Embedded URLs in rich text remain as stored. The plan lists omissions. Process
rules may reject a later create; copy is not a transaction.

Journals contain source fields and destination mappings. A companion `.lock` file
prevents concurrent runs on one journal. If a write may have committed without
its response being recorded, resume refuses automatic replay until that operation
is reconciled. Completed operations are not automatically rolled back.

## Tests

```powershell
cargo test -p cloud_terrastodon_azure_devops work_item_tests --lib --offline
cargo test -p cloud_terrastodon_entrypoint azure_devops_work_item_cli_tests --lib --offline
```

These focused tests never mutate service data. An ignored read-only smoke test
accepts `CT_TEST_AZDO_ORG`, `CT_TEST_AZDO_PROJECT`, `CT_TEST_AZDO_TENANT_ID` (tenant
UUID), and `CT_TEST_AZDO_IDS` (comma-separated explicit IDs) from the environment.
It uses an existing browser session, reads only those items and schema metadata,
and reports no response contents or scope values:

```powershell
cargo test -p cloud_terrastodon_azure_devops --lib --offline work_item_tests::live_work_item_deserialization -- --ignored --exact
```

Optionally set `CT_TEST_AZDO_COPY_ROOT` to validate a deep-copy plan from that
same snapshot. The explicit IDs must form the complete subtree; this check makes
no further work item requests and performs no writes.

See the [implementation design](plans/azure-devops-work-item-copy.md) for official
API contracts and recovery details.
