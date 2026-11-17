//! Shared contracts for embedded and dynamically constructed SVG icons.
//!
//! Without features this library has no dependencies. The optional `registry`
//! feature exposes pure icon projections through the existing Cloud Terrastodon
//! registry. It performs no network access.

#[cfg(feature = "registry")]
mod register_svg_icon;
#[cfg(feature = "registry")]
mod registered_svg_functions;
mod svg_icon;
#[cfg(feature = "registry")]
mod svg_icon_projection;

#[cfg(all(test, feature = "registry"))]
mod svg_icon_tests;

#[cfg(feature = "registry")]
#[doc(hidden)]
pub use cloud_terrastodon_registry as __registry;
#[cfg(feature = "registry")]
pub use registered_svg_functions::*;
pub use svg_icon::*;
#[cfg(feature = "registry")]
#[doc(hidden)]
pub use svg_icon_projection::*;
