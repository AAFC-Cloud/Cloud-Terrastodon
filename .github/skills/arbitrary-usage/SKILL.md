---
name: arbitrary-usage
description: 'Apply the repository conventions for adding or updating arbitrary::Arbitrary support in cloud-terrastodon Rust models and tests. Use this when choosing derive, facet defaults, ArbitraryJson, Facet proxies, constrained newtypes, manual generators, or arbitrary-sample-plus-targeted-mutation tests.'
argument-hint: 'Type, field, or test area to update, for example: Entra application registration model, constrained name newtype, show-command matching test'
user-invocable: true
---

# Arbitrary Usage

Use this skill when adding or updating `arbitrary::Arbitrary` support in
`cloud-terrastodon`.

This skill targets the repository's `facet`/`facet-json` model layer. Do not
introduce new Serde derives, attributes, or `serde_json::Value` fields.

## Core conventions

- Prefer `#[derive(Arbitrary, facet::Facet)]` for shared JSON model types.
- Use `facet_json::from_str`, `facet_json::to_string`, and
  `facet_json::to_string_pretty` for JSON I/O and round-trip tests.
- Use `ArbitraryJson` for opaque or pass-through JSON. It is a Facet proxy
  around `facet_json::RawJson<'static>` and has its own `Arbitrary` impl.
- Use `#[facet(rename_all = "camelCase")]`, `#[facet(rename = "...")]`,
  `#[facet(flatten)]`, `#[facet(default)]`, and repository Facet proxies for
  wire-format behavior.
- Register public model types with the repository registry when neighboring
  modules do so:

  ```rust
  cloud_terrastodon_registry::register_thing!(Example);
  cloud_terrastodon_registry::register_arbitrary!(Example);
  cloud_terrastodon_registry::register_arbitrary!(Vec<Example>);
  ```

  Register only the forms that are appropriate for the type and follow the
  surrounding module's pattern.

## Migration mapping

When updating code that still uses the old model layer, make these focused
conversions:

| Legacy pattern | Repository pattern |
| --- | --- |
| `serde_json::Value` | `ArbitraryJson` for opaque JSON fields |
| `#[serde(flatten)]` | `#[facet(flatten)]` on a map such as `HashMap<String, ArbitraryJson>` |
| `#[serde(rename = "...")]` | `#[facet(rename = "...")]` |
| `#[serde(rename_all = "camelCase")]` | `#[facet(rename_all = "camelCase")]` |
| `Serialize`/`Deserialize` JSON calls | `facet::Facet` plus `facet_json` |
| `#[arbitrary(default)]` used for JSON decoding | `#[facet(default)]` when the wire field may be absent and `Default` is valid |

Do not mechanically replace every old default attribute. `#[facet(default)]`
controls Facet decoding/encoding; it is not a substitute for making a field
arbitrary. Prefer `ArbitraryJson`, an existing proxy, or a focused newtype when
the generation problem is separate from the wire-format problem.

## Decision order

Use the first option that preserves realistic values and the production wire
contract:

1. Derive `Arbitrary` and `facet::Facet` together.
2. Replace opaque JSON fields with `ArbitraryJson`; use `#[facet(flatten)]` for
   additional-property maps.
3. Add or reuse a Facet proxy for a representation mismatch, such as a
   string-backed newtype, nullable map/vector, or alternate wire shape.
4. Introduce a focused newtype when a field has a domain constraint such as a
   name grammar, length limit, or stable identifier format.
5. Implement `Arbitrary` manually only when validity or cross-field
   coordination cannot be expressed by the previous options.

Do not write a manual generator for a large transport struct merely because
one or two fields are awkward.

## Handling JSON-shaped fields

Use `ArbitraryJson` in shared models when the payload is intentionally loose,
preserved, or not relevant to the behavior under test:

```rust
use crate::ArbitraryJson;
use arbitrary::Arbitrary;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct Example {
    pub display_name: String,
    pub metadata: Option<ArbitraryJson>,
    #[facet(flatten)]
    pub additional_properties: HashMap<String, ArbitraryJson>,
}
```

`ArbitraryJson` currently generates a small valid set of values (`null`, an
empty object, an empty array, or a JSON string). That is intentional for
generic samples; shape the field explicitly when a test needs a particular
object or payload.

Use `ArbitraryJson::object()` when an empty JSON object is the semantically
correct test value. Use `facet_json::RawJson` directly only in low-level code
that needs raw JSON and does not need the shared `ArbitraryJson` behavior.

For fields whose wire representation differs from their Rust representation,
inspect and reuse a proxy from `crates/azure_types/src/facet_proxies.rs` before
writing a new one. Common repository patterns include:

```rust
#[facet(default, proxy = crate::VecDefaultNullProxy<String>)]
pub values: Vec<String>,

#[facet(default, proxy = crate::HashMapDefaultNullProxy<SomeType>)]
pub keyed_values: HashMap<String, SomeType>,
```

Keep `#[facet(default)]` on a field only when missing/null input is a valid
wire representation and the resulting default is meaningful.

## Constrained newtypes and manual implementations

Prefer a newtype when validity belongs to the domain, not just to a test. A
newtype keeps parent structs derive-friendly and prevents callers from
repeating validation or repair logic.

For a constrained newtype:

- Implement `Arbitrary` so every generated value is valid; do not generate
  arbitrary strings and patch them in every caller.
- Derive or implement `facet::Facet` using a proxy that matches the wire
  representation, commonly `#[facet(proxy = String)]`.
- Cover useful valid variation: lengths, prefixes, suffixes, and allowed
  characters.
- Register the newtype if it is a public model used by the repository's
  registry.

Use a manual parent implementation only when generation requires coordinated
fields, substantial invariants, or a valid value space that derive cannot
express. Keep it local and explain the constraint in a short comment.

## Choosing the test shape

First classify what the test proves.

Use a realistic JSON fixture parsed with `facet_json::from_str` when testing:

- Graph or ARM wire compatibility
- field renames, camel-case names, flattening, null, or missing fields
- alternate representations or a deserialization regression
- an exact payload shape that is itself the behavior under test

Use an arbitrary sample plus targeted mutation when testing matching,
filtering, branching, or formatting behavior. Do not depend on arbitrary data
accidentally containing the id, URI, hostname, IP, or name the assertion needs.

Preferred deterministic helper:

```rust
fn sample_thing() -> Thing {
    let data = (0u8..=255).cycle().take(4096).collect::<Vec<_>>();
    let mut unstructured = arbitrary::Unstructured::new(&data);
    Thing::arbitrary(&mut unstructured)
        .expect("sample thing should be generated from arbitrary")
}
```

Then mutate only the fields relevant to the assertion:

```rust
#[test]
fn matches_by_name() {
    let mut thing = sample_thing();
    thing.name = "expected-name".to_string();

    assert!(matches_thing(&thing, "expected-name"));
}
```

Keep sample helpers generic. Put scenario-specific values in each test so the
generated object still exercises the surrounding structure.

## Facet-only types

Not every JSON type needs `Arbitrary`. For small production-only response
types, derive `facet::Facet` and parse them with `facet_json`. For example,
OAuth response structs such as `TokenResponse` and `DeviceCodeResponse` may be
Facet-only when they are only used to decode HTTP responses. Add `Arbitrary`
when tests or registry discovery need generated instances.

Round-trip pattern:

```rust
let json = facet_json::to_string(&value)?;
let decoded = facet_json::from_str::<Example>(&json)?;
assert_eq!(decoded, value);
```

## Procedure

1. Inspect the target type, its module exports, adjacent migrated models, and
   any existing Facet proxy before editing.
2. Classify the change as a model-generation, constrained-value, wire-format,
   or behavior-test problem.
3. Apply the smallest decision-order option that solves the actual problem.
4. For behavior tests, generate first and mutate the target fields second.
5. Keep real wire-format fixtures and `facet_json` tests when decoding is the
   behavior being protected.
6. Run the smallest relevant test or compile command, then broaden validation
   when the shared model or CLI surface changed.

Useful commands:

```powershell
cargo test -p cloud_terrastodon_azure_types --no-run
cargo test -p cloud_terrastodon_entrypoint --no-run
cargo test -p cloud_terrastodon_entrypoint
.\check-all.ps1
```

## Anti-patterns

- Do not add Serde derives, attributes, or `serde_json::Value` to migrated
  models.
- Do not use `#[arbitrary(default)]` as a replacement for Facet wire defaults.
- Do not hand-write a large JSON fixture for a simple matching test.
- Do not expect arbitrary output to contain the exact value a test must match.
- Do not default a field that is central to the behavior under test.
- Do not write a whole-parent manual `Arbitrary` impl when a field-level
  `ArbitraryJson`, proxy, or newtype solves the issue.
- Do not generate invalid constrained names and repair them in every caller.
- Do not force arbitrary generation into a test whose purpose is wire-format
  compatibility.

## Completion criteria

A change is complete when:

- the model uses `Arbitrary` and `facet::Facet` derives where appropriate;
- opaque JSON uses `ArbitraryJson`, and Facet attributes express wire behavior;
- constrained values are generated validly by a focused type or local manual
  implementation;
- behavior tests use arbitrary samples with explicit target-field mutation;
- wire-format tests use realistic payloads with `facet_json`;
- public model types follow neighboring registry-registration patterns; and
- the relevant crate compiles and its focused tests pass.

## Repository examples

Use these files as nearby references; inspect their current contents before
copying a pattern:

- `crates/azure_types/src/arbitrary_json.rs`: Facet proxy plus manual
  `Arbitrary` for opaque JSON.
- `crates/azure_types/src/entra_application_registration.rs`: a large Graph
  transport model deriving both traits and using flattened
  `HashMap<String, ArbitraryJson>` properties.
- `crates/azure_types/src/entra_service_principal.rs`: the same pattern across
  a related Graph model and nested credential structs.
- `crates/azure_types/src/azure_app_service_resource.rs`: Facet defaults,
  proxies, flattened properties, and round-trip tests.
- `crates/azure_types/src/address_prefix.rs`: constrained values with a
  custom `Arbitrary` implementation and a string Facet proxy.
- `crates/entrypoint/src/cli/command/azure/entra/application_registration/azure_entra_application_registration_show_cli.rs`:
  arbitrary sample plus explicit identifier-URI mutation.
- `crates/entrypoint/src/cli/command/azure/entra/service_principal/azure_entra_service_principal_show.rs`:
  arbitrary sample plus explicit service-principal-name mutation.
- `crates/entrypoint/src/cli/command/azure/app_service/azure_app_service_show.rs`:
  arbitrary sample plus explicit hostname/private-endpoint mutation.
- `crates/credentials/src/pim_graph_access_token.rs`: Facet-only OAuth
  response structs parsed with `facet_json`.
- `crates/credentials/src/pim_config.rs`: a small model deriving both traits
  with a `facet_json` round-trip test.

## Example prompts

- `Use the arbitrary-usage skill for Entra application registration Arbitrary support.`
- `Apply arbitrary-usage to convert this show-command matching test away from a hand-written JSON fixture.`
- `Use arbitrary-usage to choose between ArbitraryJson, a Facet proxy, and a manual Arbitrary impl for this field.`
