use crate::Function;
use crate::RuntimeValue;

/// One representative of a type reached by an offline discovery simulation.
///
/// Depth counts request, conversion, and reflected-child steps from the supplied
/// seeds. A reflected child retains the producer of its containing value.
pub struct DiscoveryValue {
    pub value: RuntimeValue,
    pub depth: usize,
    pub producer: Option<&'static Function>,
}
