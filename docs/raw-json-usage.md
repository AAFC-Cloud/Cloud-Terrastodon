# JSON value boundaries

Use a concrete domain record for a modeled service object. Use
`cloud_terrastodon_azure_types::ArbitraryJson` for deliberately opaque domain
values and property bags. It preserves JSON values on the wire and supplies the
shared `Arbitrary` implementation; consumers should not build JSON strings merely
to satisfy arbitrary generation.

`ArbitraryJson::try_from_facet` and `try_from_peek` centralize serialization of
typed values and reflected fields. `from_text` encodes a string scalar; it does
not interpret that string as a JSON document.

## Clippy reminder

The workspace configures Clippy's
[`disallowed_types`](https://rust-lang.github.io/rust-clippy/master/index.html#disallowed_types)
lint for `facet_json::RawJson`. The root package and standalone group-management
example have the same warning in their separate lint tables. No automatic fix is
configured: the appropriate replacement may be a concrete type, and some crates
cannot depend on `azure_types`.

Intentional raw boundaries use a narrowly scoped
`#[allow(clippy::disallowed_types, reason = "...")]`. Imports, types, and
implementations are independently checked. Allowing an import does not exempt
new fields that use the imported name. Do not suppress the lint for an entire
crate or all tests.

Facet derives emit codec items beside the annotated type. A codec module may
therefore need an explained module-level allowance to cover those generated
items; a type-level allowance alone does not cover them. Keep unrelated domain
records outside that module.

## Audited uses

The 2026-11-05 audit replaced retained opaque values in the license-update
response, other Azure/DevOps records, Resource Graph cells, resource properties,
policy payloads, group-member responses, and interactive query results. Existing
partial service models retain their wire shapes and optionality; this storage
refactor does not claim to complete their schemas.

| Remaining raw boundary | Responsibility |
| --- | --- |
| `azure_types::arbitrary_json` | Implements the shared JSON wrapper and its wire conversions. |
| Azure value proxies and Entra object/principal discriminators | Inspect nulls, sentinels, scalar alternatives, or discriminators before choosing a concrete value. |
| Azure policy-rule and DevOps service-authorization codecs | Select a typed alternative from a JSON object. Retained fallback values use `ArbitraryJson`. |
| REST bodies, cache-body proxies, and headers | Retain response artifacts or distinguish supported wire forms before domain decoding. |
| Graph batch bodies and principal lookup | Decode a response after its status or request identity identifies the expected shape. |
| HCL plan/provider metadata | Keep opaque provider values in `hcl_types`; importing `azure_types` would create a dependency cycle because `azure_types` already depends on `hcl_types`. Terraform presentation follows that existing wire representation. |
| HCL provider-version parsing | Read version-map keys without interpreting the provider-defined payloads. |
| Gitea login default flag | Decode the CLI's boolean/string/number alternatives without adding an Azure dependency. |
| Excalidraw scene editing | Preserve unmodeled document fields while editing selected fields, without adding an Azure dependency. |
| Focused wire/transport tests | Inspect or construct the exact representations exercised by those boundaries. |

The HCL dependency constraint is explicit; relocating the shared JSON type would
be a separate crate-boundary change. It is not a reason to add new raw JSON fields
to domain crates that already have access to `ArbitraryJson`.

All verification uses synthetic values and compile/lint checks. The audit does
not require reading credentials, production caches, or live service data.
