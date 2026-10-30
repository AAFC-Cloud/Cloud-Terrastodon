use crate::Function;
use crate::KNOWN_ARBITRARY_VALUES;
use crate::KNOWN_FUNCTIONS;
use facet::Shape;
use std::iter::Chain;
use std::slice::Iter;
use std::sync::LazyLock;

/// An immutable view of both executable and pure value registrations.
///
/// Original executable entries retain their addresses, including associated
/// registration handles. Low-level constructors receive one persistent adapter
/// each; their descriptors remain the authoritative registration.
#[derive(Clone, Copy)]
pub struct KnownFunctions;

impl KnownFunctions {
    pub fn iter(self) -> Chain<Iter<'static, Function>, Iter<'static, Function>> {
        static VALUE_CONSTRUCTORS: LazyLock<Box<[Function]>> = LazyLock::new(|| {
            KNOWN_ARBITRARY_VALUES
                .iter()
                .map(Function::from_arbitrary_registration)
                .collect()
        });
        KNOWN_FUNCTIONS.iter().chain(VALUE_CONSTRUCTORS.iter())
    }
}

impl IntoIterator for KnownFunctions {
    type Item = &'static Function;
    type IntoIter = Chain<Iter<'static, Function>, Iter<'static, Function>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Borrow the complete linked catalog without copying existing registrations.
///
/// Adapters for lower-level arbitrary descriptors are constructed once, on the
/// first traversal. Constructing the catalog never invokes a generator.
pub const fn known_functions() -> KnownFunctions {
    KnownFunctions
}

pub fn functions_from(shape: &'static Shape) -> Vec<&'static Function> {
    known_functions()
        .iter()
        .filter(|f| f.input_shape.is_shape(shape))
        .collect()
}

pub fn functions_to(shape: &'static Shape) -> Vec<&'static Function> {
    known_functions()
        .iter()
        .filter(|f| f.production_kind(shape).is_some())
        .collect()
}

pub fn functions_from_to(
    input_shape: &'static Shape,
    output_shape: &'static Shape,
) -> Vec<&'static Function> {
    known_functions()
        .iter()
        .filter(|f| f.input_shape.is_shape(input_shape))
        .filter(|f| f.production_kind(output_shape).is_some())
        .collect()
}
