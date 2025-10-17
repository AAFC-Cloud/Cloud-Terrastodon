#[expect(clippy::module_inception)]
mod location;
mod location_name;

pub use location::*;
pub use location_name::*;
