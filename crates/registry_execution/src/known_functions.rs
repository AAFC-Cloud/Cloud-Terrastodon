use crate::Function;
use crate::KnownFunctionsIter;
use facet::Shape;

/// An immutable view of runtime and pure operation registrations.
///
/// Iteration retains references to the original static registrations. The first
/// iteration indexes arbitrary collection metadata once; it never constructs a
/// value or invokes an operation. Direct associated handles need no lookup.
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

/// View all linked registrations. Iteration lazily initializes the shared
/// arbitrary metadata index, without invoking any registered operation.
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
