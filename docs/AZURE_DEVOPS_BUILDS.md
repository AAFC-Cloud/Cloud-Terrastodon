# Azure DevOps builds and definition folders

The Build commands use the Azure DevOps Build REST API. They cover build runs,
pipeline definitions, and the folders containing those definitions.

```powershell
ct az devops build definition list
ct az devops build definition list --project abc --name 'CI*' --path '\Team'
ct az devops build list --project abc
ct az devops build list --project abc --no-cache
ct az devops build list --project abc --definition 42 --status completed --result failed --branch refs/heads/main --limit 25
ct az devops build definition folder list --project abc
ct az devops build definition folder list --project abc --path '\Team'
ct az devops build definition folder prune --project abc --dry-run
ct az devops build definition folder prune --project abc
ct az devops build definition folder prune --project abc --path '\Team' --dry-run
```

Listings accept `--org` and `--project`; omitted values use the configured Azure
DevOps defaults. Pruning requires an explicit `--project`. `--tenant` selects a
tracked tenant or alias for delegated authentication, following the other Azure
DevOps commands. Both `ct az devops` and `ct azdo` reach the same command surface.

Build, definition, and folder listings cache GET responses through the shared
REST cache, including continuation pages. Each list command accepts `--no-cache`
to invalidate that listing operation's cached pages for the selected project
before fetching fresh results. This clears its filter and page variants while
keeping other projects' entries.
`--skip-cache` and `--clean` are aliases for `--no-cache` with the same refresh
behavior. Service identity, project selector, filters, and limits distinguish
cache entries.

Project IDs and project names are distinct cache scopes unless resolved to a
shared identity. Project-name scopes use the selector's ASCII case-insensitive
matching. Modern and legacy cloud organization URL forms share invalidation
within that project scope; Server scopes retain the server, port, instance, and
collection identity.

Cache scope paths retain visible organization and project identities. For
example, the definition-list root for synthetic names is:

```text
az/devops/org/synthetic/project/synthetic%20project/build/definition/list/
```

Build and folder listings use `build/list/` and `build/definition/folder/list/`
under the same project scope. DELETE artifacts use
`build/definition/folder/delete/{escaped-path}`. Spaces, `%`, and filesystem-special
characters are escaped so names containing spaces remain distinct from names
containing underscores. Ordinary domain names, Unicode text, and canonical
project UUIDs remain readable; an explicit GUID-shaped project name carries a
`name=` marker to distinguish it from an ID selector. Only exceptionally long
components use a bounded readable prefix and hash suffix.

Server scope paths use
`az/devops/server/{scheme}/{authority}/{base-prefix-segments}/org/{collection}/project/{project}/build/`.
The escaped parts preserve host, port, instance prefix, and collection identity.
Full request URL and filter fingerprints remain beneath the readable operation
root, followed by separate page entries.

Definition listing follows every continuation page. Build listing defaults to
the latest 100 runs in descending queue-time order; `--limit` sets a positive total
limit across pages and rejects zero during parsing. `--top` remains an alias for
`--limit`. Repeat `--definition` to select several definition IDs. Status and
result filters use the same domain
types and spellings as API records: `inProgress`, `completed`, `cancelling`,
`postponed`, `notStarted`, `none`, or `all` for status; `succeeded`,
`partiallySucceeded`, `failed`, `canceled`, or `none` for result. Unknown values
are preserved and sent unchanged; Azure DevOps validates filter support.

Leaves return `CliOutput`, so the global `--output-format text|json|facet-pretty|auto`
works throughout this surface. Interactive text shows concise colored rows;
redirected output defaults to JSON. List JSON contains arrays of typed API
records, while prune JSON contains `dry_run`, `candidates`, `deleted`, and
`skipped` fields.

Build folder paths use the `AzureDevOpsBuildFolderPath` newtype throughout API
models, SDK requests, and prune reports. Its string proxy validates API records
while keeping JSON paths as strings. Full paths use backslash separators and
start with `\`; root is exactly `\`. CLI `--path` input also accepts relative
paths and forward slashes through `AzureDevOpsBuildFolderPathArgument`. Its
parser normalizes and validates the input, and commands receive a typed value
that converts infallibly to the domain path. API decoding remains strict.

The path type rejects blank and dot segments, repeated or trailing separators,
controls, and wildcards as concrete-path safety constraints. Spelling and case
are preserved. Microsoft's [Folder schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#folder)
documents a full-path string but no complete naming grammar or maximum length;
the type adds no length restriction derived from TFVC or Azure Boards.

Build and definition IDs, exact definition names, build numbers, folder paths,
and list limits use domain types with validation at construction and decoding.
Name patterns used by `--name` remain separate from exact definition names.
Response status, result, queue status, definition type and quality have known
values while preserving unknown server values. Nested projects use shallow
references with typed project IDs and names. Build, definition, project, and
team-image URLs use their own domain types with validated string proxies and
offline `Arbitrary` implementations; records need no URL generator overrides.
REST URL values hold typed organization, project, and resource identities and
assemble their transport URLs locally. Parsing preserves service hosts,
collection prefixes, ports, and query strings; rendering canonicalizes route
spelling and path encoding. Server collections have their own name type rather
than inheriting cloud organization naming restrictions. Team-image URLs remain
opaque HTTP(S) URLs without a guessed image route. Timestamps use `DateTime<Utc>`.
JSON retains scalar IDs, names, paths, URLs, and vocabulary strings.

Modeled build properties are required, including timestamps and retention flags.
Folder creator identity references are optional because the service has been
observed omitting them; last-change identity references remain required. The
build REST response records, nested references, and list envelope reject unknown
fields when debug assertions are enabled, making unmodeled metadata visible during
request use. Release builds continue accepting additional fields.

## Empty-folder pruning

Azure DevOps' [folder-delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1)
also deletes contained definitions and their builds. Pruning is therefore based
on definitions, including definitions that have never run, rather than build
history. It invalidates cached listings before fetching fresh inventories,
follows all definition pages, protects occupied folders and their ancestors,
includes returned draft references, and
preserves the root folder. Unresolvable draft paths stop pruning. Paths are
compared without case distinctions and at folder boundaries; `\Team` does not
match `\TeamOther`. Ambiguous or malformed inventory stops pruning.

Candidates are processed deepest first. Each deletion is preceded by another
fresh inventory; a folder with a newly added definition or remaining child
folder is skipped. An inventory or deletion error stops the operation before
further deletion; the error identifies folders already deleted. `--dry-run`
only obtains the inventory and reports the planned candidates. Both the initial
inventory and every pre-delete recheck invalidate and refresh their cached lists.

A successful folder deletion constructs the affected folder, definition, and
build list requests and invalidates each operation's cache subtree for the
selected project, including all filters and pages. This accounts for cascading
deletion while preserving unrelated projects' caches. A project supplied by name
does not invalidate entries populated through its ID, or vice versa, without
resolving that relationship.

The DELETE request attaches its own zero-validity cache key (`Duration::ZERO`)
to the shared REST execution layer. Responses remain available as inspection
artifacts, and a previous response never substitutes for another execution.

The API has no atomic "delete only if empty" condition. Pruning requires
credentials that can see every definition and coordination with concurrent
pipeline/folder edits; a fresh preflight cannot eliminate changes between its
check and the delete request. A dry run is a preview, not a reservation.

## API and offline validation

The implementation uses [Definitions List](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1)
and [Builds List](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1)
at version `7.1`; [Folders List](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1)
and folder deletion use `7.1-preview.2`. Projects, path selectors, filters, and
opaque continuation tokens are encoded with `Url`. Pruning refreshes cached
inventories before using them.

Production types have their own files, except each CLI group's paired argument
struct and command enum share its `_cli.rs` file. CLI argument schemas, command
enums, and default selection stay in `entrypoint`. Each listing command declares
its organization, project, and tenant flags directly and resolves them through
the existing field-level functions. Each SDK request constructs its URL and
executes its REST calls in `IntoFuture`, with pagination directly in each list
request. Each list request assembles its readable organization or collection,
project, and operation cache root locally, reuses `pathing`'s shared Windows
filename sanitization, and hashes the complete GET URL, including the initial
total build limit, beneath that root before adding separate page entries.
Cloud URL aliases share the project-scoped invalidation root while
full request URLs distinguish cached responses. A mutation's own invalidation
contract addresses its inspection artifact; after success, its `IntoFuture`
procedure constructs and invalidates affected list requests without
reconstructing their cache keys.
Pruning awaits those lists for fresh inventories and uses the folder
delete request after its checks. Each request passes its authentication context to
`RestRequest::azure_devops_auth_context`; the setter preserves the selected source
and bearer tenant. Requests obtain the decoded page and continuation header through
`RestRequest::receive_with_ms_continuation_token` and follow pages locally. The
REST crate's `MicrosoftContinuationToken` rejects blank tokens at construction
and preserves all other token text unchanged; list requests retain their
operation-specific repeated-token checks.
API records and validated properties stay in `azure_devops_types`. Command dispatch
forwards `CliOutput` to the single top-level emitter, and leaf text renderers read
the same typed values used for JSON and Facet Pretty output.

Offline tests use typed synthetic records to check domain validation and pruning
policy: root and occupied-folder protection, scope boundaries, candidate ordering,
fresh-inventory skip decisions, ambiguous identities, and draft references. The
root-delete guard is checked without configured authentication. These tests do
not contact Azure DevOps or claim to verify its request/response contract;
pagination and asynchronous request sequencing are not covered by fake HTTP
scripts. Production requests use the same execution path in test builds.
