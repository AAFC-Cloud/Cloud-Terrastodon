# Cloud Terrastodon iconography

The shared `SvgIcon` contract lets offline consumers use Azure, Azure DevOps, and
downstream icon implementations through the same interface. Its SVG, display
name, and category methods return `Cow<'static, str>`: embedded assets borrow
static text without allocating, while dynamic implementations return owned text.
The returned value can outlive the icon instance.

The default library has no dependencies. Enable `registry` to expose
`register_svg_icon!` and `registered_svg_functions()` through the existing Cloud
Terrastodon function registry. Registration includes the type, its offline
`Arbitrary` constructor, and three pure shared-reference projections returning
`Cow<'static, str>`. Their explicit contracts are `SvgIcon::svg`,
`SvgIcon::name`, and `SvgIcon::category`. The iterator returns all three kinds;
filter the SVG contract to enumerate one content projection per registered icon.

Registered types implement `Facet`, `Arbitrary`, and `SvgIcon`. Callers using the
registration macro need the same direct Facet, Arbitrary, Eyre, and Linkme
dependencies used by other Cloud Terrastodon registrations. Asset collections
derive `Default` too, so the existing registry can construct unit icons.
Registration covers enabled types in linked collections; it does not discover
unregistered Rust trait implementations.

Asset collections live in `cloud_terrastodon_iconography_azure` and
`cloud_terrastodon_iconography_azure_devops`. Category features select the
available icon types and embedded SVGs. Source artwork retains its own terms;
see each collection's `ASSETS.md`. Importing and code generation belong to the
separate maintenance CLI; this library downloads no assets or configuration.
