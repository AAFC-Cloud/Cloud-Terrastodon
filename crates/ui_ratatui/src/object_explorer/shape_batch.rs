use facet::Shape;

/// Ready-value type evidence from one bounded query step; no object data is copied.
#[derive(Debug)]
pub(crate) struct ShapeBatch {
    pub(crate) shapes: Vec<&'static Shape>,
    pub(crate) inspected: usize,
    pub(crate) complete: bool,
}
