# Runtime verification

`cargo test` covers offline behavior: domain policy, synthetic execution/cache
coordination, loopback HTTP, and controlled temporary files/processes. Operator
verification against a selected service belongs to `ct test`.

```text
ct test list
ct test list "*resource-group*"
ct test run resource-group-list --tenant example --account admin
ct test run resource-group-list --tenant example --account admin --clean
ct test run azure-devops-projects-list --org example --tenant example --account admin
```

`run` takes a positional registry-name pattern, not a subcommand for each case.
An exact name selects that registration. `*` matches any sequence and `?` one
character across the full name; quote patterns to avoid shell expansion.
No matching cases is an error. Listing inspects registration metadata without
resolving accounts or executing requests, and includes effects and input fields.

Service execution requires a registered account. An explicit `--account` or a
deliberately configured account default selects it within the supplied tenant or
service scope. `--auth-source none` rejects authenticated cases before account
lookup. See [authentication](authentication.md) for provisioning and selection.
Authentication arguments belong to `test run`; `test list` has none. `auto`
chooses the selected account's preferred registered source, while an explicit
source selects that mechanism without fallback. Resolved request data contains
the concrete account/source/tenant selection, and all cases use the invocation's
shared executor rather than replacing its authentication provider.
The examples above contact the selected service when normal execution requires
it; they are operator actions, not development-validation commands.

## Inputs and effect admission

`--tenant`, `--org`, `--project`, and `--instance` supply only corresponding fields
declared by each selected request. Other operation fields come from `--input`, a
JSON object keyed by registered case name. Each value is an object of explicit
request fields. Use `test list` to inspect names/fields and the request's typed
schema for their representations. Unknown, missing, and conflicting fields fail
instead of being generated or discovered. Resource identities are never chosen
arbitrarily. Account aliases/defaults resolve to immutable account IDs before
the request executes.

The default allowed effect is `Read`. Use `--allow-effect` to explicitly allow
each effect needed by the selected checks. Every declared effect must be allowed;
a request declaring both Read and Write is skipped when only Read is allowed.
Solicitation also requires explicit admission. Effect declarations describe
behavior; they are not a security sandbox. Mutating cases require deliberate
targets and their documented completion/cleanup behavior.

## Freshness and evidence

`--clean` is the only freshness switch. It invokes the selected request's normal
`.with_invalidation(clean)` contract, including the underlying reads owned by a
composed request. Without it, the runner warns on stderr that eligible cached
results may be reused. Cache misses can still contact services.

A decoded memory hit skips decoding. An eligible raw disk hit uses the current
decoder. Failed-decode artifacts are diagnostic evidence and are not reusable
successful results; a retry can fetch again. Account partitions can also prevent
cross-invocation reuse. There is no separate replay, decoding-only, or offline
mode, and omitting `--clean` does not guarantee any of those behaviors.

The typed report records pass/fail/skip/cancelled outcomes, whether the build has
debug response checks, output coverage, and cache observations. Root collection
counts distinguish an empty collection from observed items; other shapes
are explicitly unmeasured. Counts do not establish that every nested optional
schema appeared. Cache initialization can mean disk decoding or composition,
not necessarily a network exchange. Reports do not include service inventories
or credentials, and cached success does not establish current API compatibility.

Cases execute serially. Ordinary failures are reported while later eligible
cases can still run. Cancellation stops further admission, drains required
completion, and emits the partial report before returning failure. No matches,
zero completed cases, failed cases, and cancellation are non-success outcomes.
Normal global output-format selection applies to the report.

Registration is separate from unattended discovery. Simple execution/decoding
checks reuse the production request directly; checks with additional assertions
own a verification request. Neither approach duplicates service transport logic.
See the [migration inventory](plans/runtime-test-cutover-inventory.md) for the
classification of previous live Cargo tests.
