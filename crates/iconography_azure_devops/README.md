# Azure DevOps iconography

Embedded SVGs with a concrete type for each icon and no build-time downloads.

```toml
[dependencies]
cloud_terrastodon_iconography = "0.37.0"
cloud_terrastodon_iconography_azure_devops = { version = "0.37.0", default-features = false, features = ["boards"] }
```

```rust
use cloud_terrastodon_iconography::SvgIcon;
use cloud_terrastodon_iconography_azure_devops::AzureDevOpsPlanIcon;

let svg = AzureDevOpsPlanIcon.svg();
assert!(svg.contains("<svg"));
```

The public methods belong to `SvgIcon`. Each method returns
`Cow<'static, str>`; packaged icons borrow their static embedded SVG and metadata.

No categories are enabled by default. `boards` enables `AzureDevOpsBoardsIcon`
and `AzureDevOpsPlanIcon`; `repos` enables `AzureDevOpsReposIcon`; `all` enables
every category. Disabled icon types and their SVGs are absent from the compiled API.

Enable `registry` to expose enabled icon types through the shared registry.
`cloud_terrastodon_iconography::registered_svg_functions()` discovers their
pure `SvgIcon::svg`, `SvgIcon::name`, and `SvgIcon::category` projections. Match
projections by their input shape; no collection enum or manual icon list is needed.
A registry consumer must link the collection, for example with
`use cloud_terrastodon_iconography_azure_devops as _;`.

Features are additive across the dependency graph. They select compiled assets,
while the package still contains all source SVGs. See [ASSETS.md](ASSETS.md)
for artwork provenance and terms.
