use facet::Facet;

/// A Facet value that can be decoded from a command's JSON stdout.
pub trait FromCommandOutput: Facet<'static> + Send + 'static {}
impl<T> FromCommandOutput for T where T: Facet<'static> + Send + 'static {}
