# Iconography maintenance CLI

This unpublished standalone application updates the Azure and Azure DevOps icon
crates. Icon consumers depend on `cloud_terrastodon_iconography` and the selected
asset crates; they do not depend on this tool or its networking/codegen libraries.

Run commands from the Cloud Terrastodon workspace root. Use
`--workspace PATH` after a subcommand to target a different worktree explicitly.

```powershell
cargo +1.96.0 run --offline -p cloud_terrastodon_iconography_cli -- check
cargo +1.96.0 run --offline -p cloud_terrastodon_iconography_cli -- generate
cargo +1.96.0 run --offline -p cloud_terrastodon_iconography_cli -- update --source azure
cargo +1.96.0 run --offline -p cloud_terrastodon_iconography_cli -- update --source azure-devops
cargo +1.96.0 run --offline -p cloud_terrastodon_iconography_cli --example gallery
```

Cargo's `--offline` controls dependency resolution. The `update` subcommand
explicitly accesses public upstream icon sources; `generate`, `check`, and the
gallery use local files and do not contact services or resolve credentials.

- `update` refreshes existing catalogue entries for one selected family. It
  downloads and validates the selected assets before replacing local files, then
  regenerates both asset APIs and their category features. Unchanged URLs and
  hashes retain their existing retrieval timestamps.
- `generate` validates local SVG integrity and produces deterministic Rust and
  Cargo feature definitions from `catalogue.json`.
- `check` validates the same inputs and fails when generated files differ. It
  does not modify files or fetch missing inputs.

## Initial catalogue

Azure's `resources` category contains Resource group and Subscription. Azure
DevOps's `boards` category contains Boards and Plan; `repos` contains Repos.
Categories are library-owned names, independent of upstream bundle folders.
Both asset crates default to no categories; `all` enables every category.
Azure's optional `resource-types` integration is enabled separately.

File replacements are individually atomic and share cancellation protection.
If a filesystem error interrupts an update after some replacements, rerun the
same source update to reconcile the SVGs and catalogue. Offline generation can
repair generated-output drift after the local SVGs match their recorded hashes.

The gallery enables `resources`, `boards`, and registry integration. It discovers
concrete icon types through `registered_svg_functions()`, selects each
`SvgIcon::svg` projection, and pairs its name and category projections by input
shape. Default construction creates the registered icon units without service or
configuration access. The gallery sorts these results and writes a self-contained
HTML file with embedded SVG image data, so identically named SVG definitions
cannot interfere with one another. The disabled `repos` category stays absent.

Each packaged icon has its own family-prefixed type, such as
`AzureResourceGroupIcon` or `AzureDevOpsPlanIcon`. Their public methods come from
`SvgIcon` and return `Cow<'static, str>`; packaged assets and metadata are borrowed
from static storage. Registry discovery uses the linked implementations and needs
no hand-maintained collection list. The example explicitly links both collection
crates with `use ... as _;` so their registrations reach the executable.

## Inputs and provenance

`catalogue.json` is the generation input. Each entry records its family, stable
Rust type stem (the `variant` field), display name, category, asset filename,
source and discovery URLs, SHA-256, UTC retrieval time, terms reference, and
resource-type aliases. Generation expands the stem into a family-prefixed unit
type ending in `Icon`, keeps each implementation in its own file, and gates its
module and re-export by category.

Initial source research is retained in `sources-research.json`, including
the original Portal extraction layer and the differences between extracted SVGs
and raw Portal bundle contents. Downloads preserve the selected source's bytes.

The Azure updater follows the public
[maskati/azure-icons](https://github.com/maskati/azure-icons) inventory: it resolves
`main` to a Git revision, verifies each selected logical filename against that
revision's README, then fetches revision-pinned SVG URLs. This slice uses that
existing extractor; implementing an independent full Portal extractor remains
separate work.

The DevOps updater resolves Boards and Repos from the public
[status page](https://status.dev.azure.com/_history) asset map. Plan uses the
recorded, user-supplied CDN URL because a public manifest for its logical name
has not been verified. A missing or moved Plan asset fails explicitly instead of
guessing a new filename suffix. Boards and Plan currently have identical bytes;
their separate logical identities and source records are retained.

## Artwork terms

The repository's Rust code license does not assign a license to Microsoft artwork.
The catalogue's terms URLs identify the sources reviewed; they are not an
assertion of a general redistribution grant. No publication is performed by
these commands. Review the applicable artwork terms before distributing assets.

Microsoft's [copyright guidance](https://www.microsoft.com/en-us/legal/intellectualproperty/copyright/permissions)
and [trademark guidance](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks)
remain separate from the Rust code license. The
[Azure Architecture collection](https://learn.microsoft.com/en-us/azure/architecture/icons/)
has its own terms; those terms are not assumed to cover Portal-extracted or
DevOps CDN files.

## Updating the catalogue

Add a selected asset with a stable PascalCase type stem, one owning category, and
a single SVG filename inside the matching crate's `svg` directory. Import and
record the real source bytes and SHA-256 before offline generation. Updating
refreshes known entries; this first slice does not claim complete enumeration of
Portal or Azure DevOps artwork.

Validation rejects duplicate type stems, logical names, files, and conflicting
resource mappings.
It checks SVG structure and self-contained references before generation.
Generated Rust and feature declarations are checked in alongside the SVG files.
