use std::fmt;

use cloud_terrastodon_registry::Function;

use super::invocation_mode::InvocationMode;

/// Static registry metadata describing an action the UI may request for an
/// owned Ready root. It contains no arena value or UI callback.
#[derive(Clone, Copy)]
pub(crate) enum RootActionSnapshot {
    Invoke {
        function: &'static Function,
        mode: InvocationMode,
    },
    InvokeArbitrary {
        request_function: &'static Function,
        constructor: &'static Function,
    },
}

impl RootActionSnapshot {
    pub(crate) fn id(self) -> String {
        match self {
            Self::Invoke { function, mode } => {
                format!("invoke:{mode:?}:{}:{}", function.origin, function.label)
            }
            Self::InvokeArbitrary {
                request_function,
                constructor,
            } => format!(
                "invoke-arbitrary:{}:{}:{}:{}",
                request_function.origin,
                request_function.label,
                constructor.origin,
                constructor.label
            ),
        }
    }

    pub(crate) fn label(self) -> String {
        match self {
            Self::Invoke {
                function,
                mode: InvocationMode::Consume,
            } => format!(
                "invoke {} -> {}",
                function.label,
                cloud_terrastodon_registry::describe_shape(function.output_shape)
            ),
            Self::Invoke {
                function,
                mode: InvocationMode::Retain,
            } => format!(
                "clone and invoke {} -> {}",
                function.label,
                cloud_terrastodon_registry::describe_shape(function.output_shape)
            ),
            Self::InvokeArbitrary {
                request_function, ..
            } => format!(
                "invoke arbitrary {} -> {}",
                request_function.label,
                cloud_terrastodon_registry::describe_shape(request_function.output_shape)
            ),
        }
    }

    pub(crate) const fn invocation(self) -> Option<(&'static Function, InvocationMode)> {
        match self {
            Self::Invoke { function, mode } => Some((function, mode)),
            Self::InvokeArbitrary { .. } => None,
        }
    }

    pub(crate) const fn arbitrary_invocation(
        self,
    ) -> Option<(&'static Function, &'static Function)> {
        match self {
            Self::InvokeArbitrary {
                request_function,
                constructor,
            } => Some((request_function, constructor)),
            Self::Invoke { .. } => None,
        }
    }
}

impl fmt::Debug for RootActionSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RootActionSnapshot")
            .field("id", &self.id())
            .field("label", &self.label())
            .finish()
    }
}

impl PartialEq for RootActionSnapshot {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (
                Self::Invoke {
                    function: left,
                    mode: left_mode,
                },
                Self::Invoke {
                    function: right,
                    mode: right_mode,
                },
            ) => std::ptr::eq(left, right) && left_mode == right_mode,
            (
                Self::InvokeArbitrary {
                    request_function: left_request,
                    constructor: left_constructor,
                },
                Self::InvokeArbitrary {
                    request_function: right_request,
                    constructor: right_constructor,
                },
            ) => {
                std::ptr::eq(left_request, right_request)
                    && std::ptr::eq(left_constructor, right_constructor)
            }
            _ => false,
        }
    }
}

impl Eq for RootActionSnapshot {}
