# Cloud Terrastodon development conventions

Apply these conventions to new and refactored workspace production code. Existing
code that does not follow them is not a precedent for adding more exceptions.
Keep changes within the requested scope rather than cleaning up unrelated code.
Standalone examples may remain self-contained, as described in
[REUSABLE_APPLICATIONS.md](docs/REUSABLE_APPLICATIONS.md).

## Types need a semantic purpose

- Introduce a type for a concept, contract, or responsibility that people
  recognize. Give that concept a name and a central home for its invariants,
  implementations, wire representation, or lifecycle.
- Good reasons include domain values such as IDs and names with their own valid
  value constraints; request bodies, response records, and their nested shapes;
  and executable SDK requests that own an operation and its execution contract.
- Typed input contracts are also a responsibility. A CLI argument type can own
  parsing, normalization, validation, and conversion to a domain value while
  keeping the domain value's API representation strict.
- Execution contexts must represent meaningful state or behavior, such as
  authentication or cancellation. Resource coordinators are appropriate when
  they centralize ownership and exclusive access to shared resources, as
  `TerminalCoordinator` does for the terminal.
- Convenience-only `Options` structs whose sole purpose is to collect fields
  passed between functions or layers are forbidden. Pass the required fields
  directly. Renaming a parameter bag to `Context`, `Config`, `Scope`, or another
  name does not give it a semantic purpose.
- Repeated parameter subsets, numerous call sites, or shorter signatures do not
  justify a new type. If 50 of 100 methods need the same three fields alongside
  other parameters, pass those fields explicitly. Avoid broad parameter bags
  that make consumers receive fields they do not use; adding more fields to a
  reused `Options` object compounds this problem.
- Put an operation's filters and limits directly on its request type and pass
  the fields needed by each helper explicitly. Do not add an `Options` layer
  solely to repackage them. Argument-count lint pressure is not a reason to
  create a parameter bag; a focused lint exception can explain the explicit
  operation parameters.

## One type per file

- Give each production struct, enum, and trait its own file, including request,
  argument, report, and independently named private helper types.
  A CLI group's paired argument struct and command enum are an exception, as
  described below.
- Name the file after its type in `snake_case`, retaining the domain prefix used
  by neighboring files: for example, `AzureDevOpsBuildListRequest` belongs in
  `azure_devops_build_list_request.rs`. CLI types follow the command-based
  filename convention below rather than converting the type name verbatim.
- Keep the type's implementations, constructors, conversions, registry
  registrations, closely related helper functions, and focused tests with it.
  Test-only fixture types can remain in the owning test module.
- Give a CLI argument type with a distinct selection or parsing contract its
  own file, separate from the domain value it produces. Keep a CLI group's
  argument struct and corresponding command enum together in its CLI file;
  other related types still belong in separate files.
- Keep module files focused on declarations and re-exports. Give standalone
  operations and shared helpers small, cohesive modules when they have no owning
  type. File length is a signal to review responsibilities, not a numeric limit.
- Split mixed responsibilities rather than moving a large feature into another
  large helper file. Extract abstractions for actual shared behavior; do not
  introduce a generic framework just to reduce repetition or line counts.

## Domain types and invariants

- Prefer `pub` fields for domain records, requests, responses, and CLI schemas.
  Do not make ordinary fields private or `pub(crate)` merely to expose them
  through getters. Independently validated domain values can be public fields
  when replacing one valid value with another preserves the containing type's
  invariants.
- Restrict field visibility when direct mutation would bypass validation, break
  a concrete relationship between fields, or interfere with resource ownership.
  Explain that responsibility beside the type or field. For example, keep a
  validated newtype's raw string private; an organization URL's name must also
  stay consistent with its validated base URL.
- Domain objects should use domain types for IDs, names, paths, and other
  properties with meaningful identity, vocabulary, or validation rules. Avoid
  interchangeable `String` or integer fields for distinct domain concepts.
- Reuse existing domain types before adding new ones. Use established semantic
  types such as `chrono::DateTime<Utc>` for timestamps. Free-form descriptions
  and genuinely opaque values can remain strings.
- Represent domain URL properties with domain URL types, such as
  `AzureDevOpsWorkItemUrl`, rather than interchangeable `url::Url` fields.
  For URLs with a known resource route, store the typed organization, project,
  resource ID, and other meaningful route components; assemble `url::Url` only
  for parsing or at the transport boundary. A wrapper around a stored `Url`,
  even with a separately extracted ID, does not capture that domain identity.
  Encapsulate parsing, validated construction, route/identity invariants, URL
  encoding, and string wire conversions in the owning type; use `url::Url`
  internally and at generic transport boundaries. Preserve supported service
  URL forms and do not invent host or path restrictions for opaque URL properties.
  Keep straightforward route parsing and assembly beside each owning type
  rather than introducing a feature-wide URL builder/validator. Parse the
  components into their domain types instead of merely validating an opaque
  URL and retaining it. Opaque image, attachment, and external-link URLs can
  retain a generic URL internally when no documented resource identity exists.
- Implement valid, offline `Arbitrary` generation on the domain URL type and
  register it alongside the type. This lets records derive `Arbitrary`, including
  for `Option<DomainUrl>`, without repeated field-level generator overrides or
  feature-specific URL generator modules.
- Keep raw storage private when mutation would bypass a validated newtype's
  invariants, and use fallible construction. Validate parsing and deserialization
  through the same invariant boundary; do not expose mutable access or unchecked
  conversions that let callers invalidate a value.
- Use a validating Facet proxy when a scalar wire representation is needed.
  `#[facet(transparent)]` alone does not ensure a constructor is called during
  deserialization. Preserve the API's string or numeric JSON representation.
- Ensure `Arbitrary` produces valid domain values. Keep generation separate from
  live credentials, defaults, configuration, caches, and service state.
- Keep exact domain values separate from user selectors and filter patterns.
  For example, a project ID/name selector requires resolution, a tenant
  default/ID/alias selector has distinct alternatives, and a wildcard name
  filter is different from an exact definition name.
- Prefer typed CLI fields for structured domain input. Use the domain type
  directly when its parsing contract matches the accepted CLI syntax. When CLI
  input accepts shorthand or alternatives that strict API decoding must reject,
  use a validated argument type to own that input contract and conversion.
  For example, a build-folder argument accepts `Team/CI` and produces the
  canonical `\Team\CI` path. Parse and validate during argument construction,
  before authentication, defaults, or service calls; avoid raw `String` fields
  followed by manual domain parsing in command handlers. Keep API decoding strict.
- An argument type may store an already validated domain value and expose an
  infallible conversion that simply unwraps it. Delegating normalization and
  validation to domain constructors or helpers does not make the argument type
  redundant: its accepted input contract is the responsibility. Avoid wrappers
  that repeat the same parser without a distinct input, selection, validation,
  or conversion contract.
- Model known nested API objects with typed references rather than
  `ArbitraryJson`. Reuse typed ID/name properties in a shallow reference instead
  of requiring fields that only exist on the full object. Reserve arbitrary JSON
  for genuinely open property bags or deliberately opaque portions of a schema.
- Preserve missing versus known values. Use `Option<T>` for optional API fields;
  never replace an unknown identity or path with a default that implies knowledge.
  Choose optionality from documented or observed omission; do not make documented
  response properties optional solely as a speculative compatibility precaution.
- Prefer typed response vocabularies. When the service may introduce new values,
  preserve unknown values explicitly rather than breaking response decoding.
  Request filter enums may have a different, closed vocabulary or wire spelling.
- Reuse a domain vocabulary for request filters when the API defines the same
  type and values for requests and responses. A property's role as a filter does
  not by itself justify a duplicate enum. Separate types need a distinct contract;
  if CLI shorthand is desired, keep that parsing contract separate from API values.

## Official documentation and API contracts

- Public service/domain types and SDK operations need useful doc comments. Link
  directly to the corresponding official platform schema or API documentation
  (Microsoft Learn for Azure and Azure DevOps), including the API version and
  schema anchor where useful.
- When a newtype has no dedicated documentation page, link to the documented
  property it represents, such as the `id` field of its parent API object.
- Verify endpoint paths, versions, pagination, filters, and destructive behavior
  against official documentation or the platform's official source before
  implementing them. Explain material behavior and limitations in the comments.
- Add `#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]` to REST response
  structs and nested response records. Debug decoding should surface unmodeled
  fields so observable service behavior is reviewed rather than silently dropped.
  Release decoding remains tolerant of new fields. Keep genuinely open property
  bags explicit instead of adding catch-all fields to bypass these checks.
- Enforce verified platform invariants. Clearly distinguish additional library
  safety constraints from platform guarantees. Do not invent naming or length
  limits or borrow restrictions from a different Azure DevOps subsystem.

## Responsibilities and crate boundaries

- Put reusable API records and domain values in the corresponding `*_types`
  crate. Keep authentication and network execution out of those types.
- Put service requests, transport coordination, pagination, and domain workflows
  in the SDK crate. Follow existing request/`IntoFuture` patterns and reuse shared
  authentication and REST execution helpers.
- Cache GET and other nonmutating service requests through the existing cache
  layers. Give paginated responses separate entries beneath an invalidatable
  request key, and include the service identity, operation, filters, and limits
  in cache identity. Use filesystem-safe keys rather than raw URL or filter text.
- Keep scope keys human-inspectable: use named hierarchy segments and readable
  organization, project, and resource identities, rather than opaque hashes for
  those identities. Escape spaces, `%`, and filesystem-special characters to
  avoid collisions introduced by cache-path normalization. Retain readable
  Unicode where safe, and distinguish ambiguous ID/name selectors explicitly.
  Preserve Server scheme, authority, instance prefix, and collection identity.
  URL/filter fingerprints may distinguish variants below the readable scope;
  exceptionally long components may use a bounded readable prefix and hash suffix.
- Keep cache-key construction beside the owning request. Do not extract a
  feature-wide cache-key builder merely to share path fragments or hide an
  operation's cache identity. Keep request-specific URL fingerprints and page
  entries in the request's execution procedure.
- Reuse `pathing::sanitize_windows_path_component` for readable filesystem-safe
  string components. Shared filename sanitization belongs in `pathing`; requests
  still own the hierarchy and semantic identity of their cache keys.
- Expose the existing cache-invalidation contract on cached requests so callers
  and CLI `--no-cache` flags can refresh results. Workflows that require fresh
  inventories must explicitly invalidate before reading, including repeated
  preflight checks before a mutation.
- Scope invalidation to the request's selected project or resource. Clear its
  filtered and paginated variants without invalidating unrelated projects or
  resources. Do not widen to an entire organization merely because unresolved
  ID/name selectors may refer to the same resource; preserve distinct selector
  scopes unless resolution establishes their shared identity. Name scopes should
  follow the selector's existing case-matching behavior.
- Mutating requests own a `CacheKey` with `valid_for: Duration::ZERO` and attach
  it to the existing execution-cache layer. This retains inspectable request and
  response artifacts while preventing either disk or memory results from
  substituting for execution.
- Successful mutations must invalidate affected reads, including filtered and
  paginated variants and cascading effects within the selected scope. Construct
  the affected read requests and call `.invalidate()` on them after success,
  visibly in the mutation's `IntoFuture` procedure. A mutation's own
  `CacheInvalidatable` contract invalidates its artifact key; do not hide affected
  read-request invalidation in a custom implementation of that contract. A
  mutation or workflow should not need to know another request's cache layout or
  reconstruct its keys.
- Keep a request's execution procedure visible in its `IntoFuture` body:
  construct the URL and query parameters, apply authentication, execute REST
  calls, follow pages, and interpret the response there. A longer procedural
  body is appropriate when those steps all belong to the same operation.
- Do not introduce a feature `Client` intermediary solely to hold organization,
  project, or auth fields and hide operations already represented by request
  types. Compose workflows by awaiting those request types directly. A client
  abstraction needs a separate responsibility, such as shared transport state
  or resource ownership, beyond forwarding request parameters.
- Start with straightforward pagination in each request. Extract a helper only
  after concrete implementations reveal repeated behavior and a clear pattern
  for ownership and error handling. Do not prescribe a generic paginator or
  framework in advance just because several endpoints return pages.
- Keep CLI parsing, default/scope selection, dispatch, and text presentation in
  `entrypoint`. Put reusable application startup/output facilities in `app`.
- Preserve source errors when adding context: prefer `wrap_err` or
  `wrap_err_with` over replacing an error with a new message. When an API returns
  an error without diagnostic information, such as `Err(())`, explain its actual
  failure condition rather than using a vague label or printing `()`.
- Keep registrations beside the owning type. Register reusable values, request
  operations, arbitrary generators, and collection outputs as required by the
  registry's consumers. Declare accurate read/write effects for operations.

## CLI naming and output

- Put command implementations under `crates/entrypoint/src/cli/command/`.
  Follow neighboring command-group directories and their module/re-export
  layout. Leaves may live directly in a group directory or in an operation
  subdirectory; do not impose a new directory layout on an existing group.
- CLI schema and command implementation files use `snake_case` and end in
  `_cli.rs`. Include the full canonical command hierarchy in the basename,
  retaining prefixes such as `azure_` and `azure_devops_`; omit the executable
  name and do not use aliases such as `az`, `azdo`, or `rg` in filenames.
- Name argument structs after that hierarchy in PascalCase with an `Args`
  suffix. The filename replaces `Args` with `_cli`, for example:
  `AzureSubscriptionListArgs` in `azure_subscription_list_cli.rs`, and
  `AzureDevOpsBuildDefinitionFolderPruneArgs` in
  `azure_devops_build_definition_folder_prune_cli.rs`.
- For a command group, keep `<Hierarchy>Args` and its corresponding
  `<Hierarchy>Command` enum together in `<hierarchy>_cli.rs`, along with their
  implementations and registrations. For example, `AzureDevOpsBuildArgs` and
  `AzureDevOpsBuildCommand` both belong in `azure_devops_build_cli.rs`.
  When consolidating a split pair, retain `<hierarchy>_cli.rs` and remove the
  separate `<hierarchy>_command_cli.rs`. Preserve established dispatcher
  filenames, such as `azure_command_cli.rs`, when extending existing groups.
- Shared CLI schema types need a semantic purpose and retain the hierarchy prefix
  and role, for example `azure_cognitive_services_account_argument_cli.rs` for an
  account ID/name/pattern selector. Keep each type in its own file. Put flags such
  as `org`, `project`, and `tenant` directly on each command's argument struct;
  do not extract a shared scope struct solely to reuse field declarations.
- Keep `mod.rs` focused on child modules and re-exports. Avoid generic command
  implementation filenames such as `list.rs`, `show.rs`, or `prune.rs` and avoid
  adding new command types or implementation bodies to `mod.rs`. Descriptively
  named function-only helpers such as `output.rs` or `error_summary.rs` remain
  appropriate; the `_cli.rs` convention applies to CLI schema/command files.

- Output-producing leaves return `Result<CliOutput>`; parent dispatchers forward
  that output and the top-level handler selects the format and emits it once.
- Prefer `CliOutput::facet` for default presentation or
  `CliOutput::facet_with_text` for custom text with shared JSON/Facet Pretty
  behavior. Implement `CliOutputValue` when all formats need custom behavior.
- Render all formats from the same typed value. Keep hyperlinks conditional on
  terminal output, and avoid mixing human diagnostics or ANSI styling into JSON.
- Apply text styles directly in the renderer with
  `color_eyre::owo_colors::OwoColorize`, using
  `if_supports_color(Stream::Stdout, |text| ...)` to let the library handle
  `NO_COLOR`, `FORCE_COLOR`, and stream detection. This requires owo-colors'
  `supports-colors` feature. Plain styling methods such as `.green()` always emit
  ANSI; they do not apply that policy by themselves.
  Do not gate colors solely on a terminal boolean, since explicit color controls
  can disable terminal colors or force colors in redirected text. Terminal
  detection still controls automatic format selection and hyperlinks.
- Keep terminal-status detection at the output boundary for automatic format
  selection. Do not ferry that boolean through `CliOutputValue` or custom text
  callbacks; color guards inspect their stream, and hyperlink rendering can check
  stdout locally when it needs terminal capabilities.
- Avoid helpers that merely hide a styling expression behind names such as
  `heading`, `name`, or `detail`. Keep the style beside the text it describes;
  shared helpers remain appropriate for substantive behavior such as sanitizing
  service-provided text before terminal presentation.

## Tests and verification

- Focus tests on behavior this code owns: domain invariants, conversions, policy
  decisions, and meaningful failure handling. Prefer small typed fixtures for
  pure policy tests.
- Fabricated service responses can exercise local procedure, but cannot establish
  remote API compatibility. Avoid brittle assertions that merely repeat fixed
  endpoints, parameter forwarding, or assumed response shapes. Test coverage must
  justify the fixture machinery and maintenance it requires.
- Do not introduce feature-specific fake transports, test-only execution
  substitutes, or production intermediaries merely to preserve such tests.
  When service-boundary tests are warranted, prefer an established injection
  boundary and keep the request procedure in its production implementation.
- Do not add tests that merely prove Figue can parse declared flags or dispatch
  declared variants. Test application behavior and integration when those are the
  actual subject of the change; retain useful existing schema/registry checks.
- Do not add hardcoded JSON deserialization fixtures or tests that merely prove
  Facet or Figue can deserialize declared fields. Such tests do not establish
  API compatibility. Test owned validation, parsing, and policy directly with
  typed values. Service response shapes are verified through actual request use
  within the user's authorized data-access scope; they do not justify accessing
  production data for development.
- Do not access production data for development validation, copy real data into
  fixtures, or print it in tests. Inspect existing tests before running them:
  some contact services or resolve credentials/configuration. Cargo `--offline`
  prevents dependency downloads, not network access by test code.
- Run focused checks appropriate to the change and respect `rustfmt.toml` and
  the repository's toolchain/dependency pins. Do not add implementation-mirroring
  tests for documentation-only or routine reversible edits.
- Preserve user edits and staging. Do not stage, commit, or change unrelated
  dependencies and lockfiles as part of a refactor unless requested.
