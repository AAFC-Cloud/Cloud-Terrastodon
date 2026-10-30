use crate::RuntimeValue;
use crate::describe_shape;
use std::fmt;

/// Formats a reflected value using its advertised display operation.
pub(crate) struct RuntimeDisplayValue<'a>(pub &'a RuntimeValue);

impl fmt::Display for RuntimeDisplayValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { self.0.shape().call_display(self.0.peek().data(), formatter) }
            .unwrap_or_else(|| write!(formatter, "⟨{}⟩", describe_shape(self.0.shape())))
    }
}
