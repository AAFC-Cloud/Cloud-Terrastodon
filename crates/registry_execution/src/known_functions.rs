use crate::Function;
use crate::KnownFunctionsIter;
use facet::Shape;

/// An immutable view of runtime and pure operation registrations.
///
/// Iteration wraps references to the original registrations. No adapter table,
/// lookup, metadata copy or operation execution is needed to obtain a handle.
#[derive(Clone, Copy)]
pub struct KnownFunctions;

impl KnownFunctions {
    pub fn iter(self) -> KnownFunctionsIter {
        KnownFunctionsIter::new()
    }
}

impl IntoIterator for KnownFunctions {
    type Item = Function;
    type IntoIter = KnownFunctionsIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Traverse all linked registrations without allocating or invoking operations.
pub const fn known_functions() -> KnownFunctions {
    KnownFunctions
}

pub fn functions_from(shape: &'static Shape) -> Vec<Function> {
    known_functions()
        .iter()
        .filter(|function| function.input_shape.is_shape(shape))
        .collect()
}

pub fn functions_to(shape: &'static Shape) -> Vec<Function> {
    known_functions()
        .iter()
        .filter(|function| function.production_kind(shape).is_some())
        .collect()
}

pub fn functions_from_to(
    input_shape: &'static Shape,
    output_shape: &'static Shape,
) -> Vec<Function> {
    known_functions()
        .iter()
        .filter(|function| function.input_shape.is_shape(input_shape))
        .filter(|function| function.production_kind(output_shape).is_some())
        .collect()
}
