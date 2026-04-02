---
name: arbitrary-usage
description: 'Apply the repo-preferred Arbitrary patterns in cloud-terrastodon. Use this for derive vs #[arbitrary(default)] vs newtype vs manual impl decisions, and for converting brittle tests to arbitrary-sample-plus-targeted-mutation patterns.'
argument-hint: 'Type, field, or test area to update, for example: Entra application registration model, constrained name newtype, show-command matching tests'
user-invocable: true
---

# Arbitrary Usage

Use this skill when adding or updating `arbitrary::Arbitrary` support in `cloud-terrastodon`.

This workflow captures the repo pattern reinforced by recent work on:
- Entra application registrations
- Entra service principals
- Azure App Service show-command tests

## What This Produces
- `Arbitrary` implementations that fit the repo's preferred style
- Tests that generate broad arbitrary samples and then mutate only the fields needed for the assertion
- Smaller, cleaner test helpers with less hard-coded JSON
- Better coverage of type surface area without fighting awkward fields like `serde_json::Value`

## When To Use
- Add `Arbitrary` support to a new type in `crates/azure_types`
- Fix tests that currently depend on large hand-written JSON payloads
- Make matching or filtering tests more realistic without making them brittle
- Decide whether a type should derive `Arbitrary`, use `#[arbitrary(default)]`, introduce a newtype, or use a manual impl

## Inputs To Gather First
Before editing, identify:
- Which type or tests are being changed
- Whether this is a transport-model problem, a constrained-value problem, or a test-shape problem
- Which exact fields block `#[derive(Arbitrary)]`, if any
- Whether the test is a behavior test or a deserialization-focused test

## Quick Decision Tree
Use this order:
1. Try `#[derive(Arbitrary)]`
2. If only a few awkward JSON or pass-through fields fail, add `#[arbitrary(default)]` to those fields
3. If the problem is a domain rule on one field, introduce or reuse a focused newtype
4. If the type is a constrained name newtype, prefer a custom `Arbitrary` impl that generates only valid values
5. Use a manual `Arbitrary` impl for a larger parent type only when the above options still produce unrealistic or invalid data

Default rules:
- Prefer `#[derive(Arbitrary)]` in most cases
- Prefer `#[arbitrary(default)]` for spillover JSON like `HashMap<String, serde_json::Value>` or `Option<serde_json::Value>`
- Prefer introducing a newtype when only one or two fields need special handling
- Prefer hard-coded JSON only when the real payload shape is what the test is asserting
- Prefer arbitrary sample generation plus targeted mutation for behavior tests

## Procedure

### 1. Start With The Smallest Plausible Change
Try the simplest shape that matches the production type.

Preferred first attempt:
- Add `use arbitrary::Arbitrary;`
- Add `#[derive(Arbitrary)]`
- Reuse existing repo patterns before inventing a custom generator

Good fit for derive:
- Plain structs and enums whose fields already implement `Arbitrary`
- Types already using repo-friendly newtypes
- Test-only helper types with little validation logic

### 2. Identify Exactly What Blocks Derive
If derive fails, inspect the specific fields that caused it.

Common blockers in this repo:
- `serde_json::Value`
- `HashMap<String, serde_json::Value>`
- Flattened JSON spillover properties
- Constrained string wrappers that cannot accept arbitrary raw strings

Default reaction:
- Do not jump straight to a manual impl for the whole parent type
- First ask whether only a few fields are the problem

### 3. Use `#[arbitrary(default)]` For Awkward Fields
When a field exists for tolerance, payload preservation, or pass-through behavior and does not matter to most tests, default it.

Common examples:
- `#[serde(flatten)]` additional-properties maps
- Optional JSON blobs
- Arrays of `serde_json::Value`
- Loose payload fragments that are not used by the behavior under test

Pattern:

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Arbitrary)]
pub struct Example {
    pub id: MyId,
    #[arbitrary(default)]
    pub info: Option<serde_json::Value>,
    #[serde(flatten)]
    #[arbitrary(default)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}
```

Use `#[arbitrary(default)]` when:
- The field is not central to most tests
- The default value is semantically acceptable
- The field exists mainly for forward compatibility or payload preservation

Do not default a field when:
- The field is central to the behavior under test
- A default value would hide important logic branches
- The field encodes a real project invariant

### 4. Introduce Newtypes Before Going Manual
If a field has project-specific validity rules, consider a newtype instead of a custom impl on the whole parent type.

This is often the right choice for:
- Name types with length limits
- Name types with restricted characters
- Name types with start or end character rules
- IDs with a stable format that should not stay as `String`

Why:
- It localizes the generation problem
- It keeps parent structs derive-friendly
- It makes production code safer, not just tests cleaner

Special rule for constrained name newtypes:
- Once the constrained name newtype exists, prefer a custom `Arbitrary` impl on that newtype.
- Do not generate invalid names and hope tests overwrite them later.
- Do not rely on retry-heavy generation from arbitrary raw strings when a direct valid generator is practical.

### 5. Manually Implement `Arbitrary` Only When Needed
Use a manual impl as the fallback, not the default.

Justified cases:
- The type has heavy validity constraints that derive cannot express
- Defaulting key fields would make the generated values too unrealistic
- The type needs coordination across multiple fields
- A constrained newtype must generate only values that satisfy naming rules

When writing a manual impl for a constrained name newtype:
- Treat valid production values as a hard requirement, not a best effort goal
- Aim to cover as much of the valid input surface as practical
- Exercise different valid lengths, prefixes, suffixes, and character combinations
- Avoid collapsing to one or two trivial examples
- Prefer generating valid values, not arbitrary strings followed by retry loops everywhere

### 6. Classify Tests Before Rewriting Them
Choose the test strategy based on what the test is actually proving.

Use arbitrary sample plus targeted mutation when the test is about:
- Matching or filtering behavior
- Branching on one or two fields
- Search keys like ids, names, URIs, hostnames, or IPs

Keep hard-coded JSON when the test is about:
- serde rename behavior
- null-handling behavior
- real payload compatibility
- alternate wire representations
- regression protection against actual Graph or ARM payloads

Do not force arbitrary generation into tests that are really validating deserialization.

### 7. Treat Behavior Tests As Two-Phase: Generate, Then Shape
For behavior tests, do not depend on the arbitrary sample to already contain the exact field values you want.

Preferred pattern:
1. Generate a broad arbitrary sample
2. Mutate the field or fields the test cares about
3. Assert only the behavior under test

Pattern:

```rust
fn sample_thing() -> Thing {
    let data = (0u8..=255).cycle().take(4096).collect::<Vec<_>>();
    let mut unstructured = Unstructured::new(&data);
    Thing::arbitrary(&mut unstructured)
        .expect("sample thing should be generated from arbitrary")
}

#[test]
fn matches_by_name() {
    let mut thing = sample_thing();
    thing.name = "expected-name".to_string();

    assert!(matches_thing(&thing, "expected-name"));
}
```

This is preferred because:
- The sample still exercises the rest of the structure
- The test becomes deterministic
- The assertion stays focused on one branch of behavior

### 8. Validate At The Right Level
After adding or changing `Arbitrary`:
- Run the smallest relevant test first
- Then run the most relevant crate-level compile or test command
- If command shapes changed, validate the entrypoint crate and the actual CLI help surface when useful

Useful commands in this repo:

```powershell
cargo test -p cloud_terrastodon_azure_types --no-run
cargo test -p cloud_terrastodon_entrypoint --no-run
cargo test -p cloud_terrastodon_entrypoint
.\check-all.ps1
```

## Decision Summary

### Choose `derive`
Use `#[derive(Arbitrary)]` when:
- Most fields already support `Arbitrary`
- Problem fields can be defaulted safely
- The type is mostly a transport container

### Choose `#[arbitrary(default)]`
Use it when:
- The field is incidental to most behavior tests
- It is a payload-preservation field
- It is loose JSON that does not deserve a custom generator yet

### Choose A Newtype
Introduce or reuse a newtype when:
- The field has naming or format rules in production code
- The same validation concern will recur in multiple places
- Strong typing improves both runtime safety and test generation

Skip a newtype when:
- The field is purely incidental and never reused
- The extra type would add complexity without improving correctness

### Choose A Manual Impl
Use a manual `Arbitrary` impl when:
- The type is a constrained name newtype
- The type must generate only valid constrained values
- Defaulting too many fields would gut the value of generation
- The parent type would otherwise become mostly empty noise

## Anti-Patterns
- Do not rely on arbitrary output to accidentally contain the exact id, URI, hostname, IP, or name a test needs.
- Do not hand-write a giant JSON payload for a simple matching test when an arbitrary sample plus one or two mutations would do.
- Do not implement manual `Arbitrary` for a large transport struct when only one or two fields are problematic.
- Do not generate invalid constrained names and patch them up later in every caller.
- Do not hide important behavior behind `#[arbitrary(default)]` on fields that are central to the logic being tested.

## Completion Criteria
A change to Arbitrary usage is complete when all of the following are true:
- The type uses derive by default unless there is a concrete reason not to
- Problematic fields are handled with `#[arbitrary(default)]` or focused helper newtypes before considering a manual impl
- Constrained name newtypes have a custom `Arbitrary` implementation that generates only valid values
- Any other manual impl is localized and justified by real constraints
- Behavior tests generate an arbitrary sample and then mutate target fields instead of depending on huge JSON fixtures
- Deserialization tests still use realistic payloads where that is the real behavior under test
- The relevant crate compiles and tests pass

## Examples
- Add `Arbitrary` to an Entra application registration type by deriving it and defaulting JSON spillover fields
- Update a service principal show-command test to generate an arbitrary sample and then set `service_principal_names` explicitly
- Create a constrained name newtype with a manual `Arbitrary` impl that always emits values valid under the naming rules instead of generating random invalid strings

## Case Studies
Use these as concrete repo examples, but keep the inline lesson in mind even if the files evolve later.

### 1. Derive Plus `#[arbitrary(default)]` For JSON-Heavy Transport Types
Path:
`crates/azure_types/src/application_registration.rs`

What this demonstrates:
- A mostly transport-shaped Graph type can still derive `Arbitrary`
- `serde_json::Value` fields and flattened spillover maps should usually be handled with `#[arbitrary(default)]`
- You do not need a manual `Arbitrary` impl for the whole type just because a handful of pass-through JSON fields are awkward

Promoted lesson:
- Default the noisy JSON-preservation fields so the rest of the type can stay derive-friendly

### 2. Apply The Same Pattern Consistently Across Similar Graph Types
Path:
`crates/azure_types/src/service_principal.rs`

What this demonstrates:
- Another large Graph transport type can use the same approach as application registrations
- Nested credential structs can still derive `Arbitrary` while only the `serde_json::Value` fields need defaults
- Consistency across related types reduces one-off generator code and keeps tests easier to read

Promoted lesson:
- If two transport models have the same problem shape, solve them the same way unless there is a concrete reason not to

### 3. Arbitrary Sample Helpers Should Be Small And Boring
Path:
`crates/entrypoint/src/cli/command/azure/entra/application_registration/azure_entra_application_registration_show.rs`

What this demonstrates:
- The sample helper should just generate a broad arbitrary instance from `Unstructured`
- The tests then mutate only the target field, such as `identifier_uris` or `unique_name`
- The sample helper should not be doing the test setup work for every scenario

Promoted lesson:
- Keep `sample_*` helpers generic; specialize in each test, not in the helper

### 4. Matching Tests Must Mutate The Field They Intend To Match On
Path:
`crates/entrypoint/src/cli/command/azure/entra/service_principal/azure_entra_service_principal_show.rs`

What this demonstrates:
- After switching a helper from hard-coded JSON to arbitrary generation, tests can fail if they assume the generated sample already contains the target value
- The fix is to mutate the exact field under test, such as `service_principal_names` or `app_id`
- This keeps the rest of the structure broad and realistic while making the assertion deterministic

Promoted lesson:
- For behavior tests, generate first and then shape the exact match target explicitly

### 5. The Pattern We Want To Copy For Show-Command Matching Tests
Path:
`crates/entrypoint/src/cli/command/azure/app_service/azure_app_service_show.rs`

What this demonstrates:
- A good matching test helper generates a broad arbitrary sample once
- Each test then overwrites the exact fields relevant to the scenario, like hostnames or private endpoint data
- This avoids giant JSON fixtures while still exercising the surrounding object shape

Promoted lesson:
- Prefer the two-phase test structure: arbitrary sample first, targeted mutation second

## Notes For This Repo
- `serde_json::Value` is usually a signal to try `#[arbitrary(default)]` first
- Flattened `additional_properties` maps should usually default rather than forcing a manual impl for the entire parent type
- Arbitrary-generated test helpers should still be deterministic enough for the assertion by mutating the field under test
- If only one or two fields are difficult, solve those fields specifically instead of hand-writing an Arbitrary impl for a large transport struct
- Prefer production-quality type improvements like newtypes over test-only workarounds when the field truly has domain constraints

## Example Prompts
- `Use the arbitrary-usage skill for Entra application registration Arbitrary support.`
- `Apply arbitrary-usage to convert this show-command matching test away from a hand-written JSON fixture.`
- `Use arbitrary-usage to decide whether this Azure name wrapper should derive Arbitrary or implement it manually.`
