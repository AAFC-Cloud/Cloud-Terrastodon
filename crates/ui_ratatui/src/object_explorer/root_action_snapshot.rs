use super::invocation_mode::InvocationMode;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::describe_shape;
use std::fmt;
use std::fmt::Write;

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
                format!(
                    "invoke:{mode:?}:{}:{}",
                    function.trait_contract, function.label
                )
            }
            Self::InvokeArbitrary {
                request_function,
                constructor,
            } => format!(
                "invoke-arbitrary:{}:{}:{}:{}",
                request_function.trait_contract,
                request_function.label,
                constructor.trait_contract,
                constructor.label
            ),
        }
    }

    pub(crate) fn label(self) -> String {
        let (function, mut label) = match self {
            Self::Invoke { function, mode } => (
                function,
                format!(
                    "Self({}) -> {}",
                    match mode {
                        InvocationMode::Consume => "Drop",
                        InvocationMode::Retain => "Clone",
                    },
                    describe_shape(function.output_shape)
                ),
            ),
            Self::InvokeArbitrary {
                request_function, ..
            } => (
                request_function,
                format!(
                    "Self -> {}(Arbitrary)",
                    describe_shape(request_function.output_shape)
                ),
            ),
        };
        // Ordinary request invocation is already conveyed by the signature.
        // Keep named operations and trait contracts visible: multiple projections
        // or conversions can share the same input and output shapes.
        match (function.trait_contract, function.label) {
            (FunctionTraitContract::IntoFuture, "invoke") => {}
            (FunctionTraitContract::From, "from")
            | (FunctionTraitContract::TryFrom, "try_from")
            | (FunctionTraitContract::Arbitrary, "arbitrary") => {
                let _ = write!(label, " [{}]", function.trait_contract);
            }
            (contract, name) => {
                let _ = write!(label, " [{name} ({contract})]");
            }
        }
        if let Self::InvokeArbitrary { constructor, .. } = self
            && (constructor.trait_contract != FunctionTraitContract::Arbitrary
                || constructor.label != "arbitrary")
        {
            let _ = write!(
                label,
                " [constructor: {} ({})]",
                constructor.label, constructor.trait_contract
            );
        }
        label
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

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::RegistrationSite;
    use cloud_terrastodon_registry::runtime_from_boxed;
    use facet::Facet;
    use std::any::Any;
    use std::collections::HashSet;

    fn unused_conversion(_: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        panic!("displaying an action must not execute it")
    }

    const fn conversion(label: &'static str, contract: FunctionTraitContract) -> Function {
        Function::sync_value(
            String::SHAPE,
            String::SHAPE,
            FunctionKind::Conversion,
            label,
            contract,
            &[],
            unused_conversion,
            runtime_from_boxed::<String>,
            RegistrationSite::new(file!(), line!()),
        )
    }

    #[test]
    fn operations_sharing_shapes_still_have_distinct_named_action_labels() {
        static FIRST: Function = conversion("summarize", FunctionTraitContract::Other("Compact"));
        static SECOND: Function = conversion("summarize", FunctionTraitContract::Other("Detailed"));
        static THIRD: Function = conversion("format", FunctionTraitContract::Other("Compact"));
        let labels: HashSet<_> = [&FIRST, &SECOND, &THIRD]
            .into_iter()
            .map(|function| {
                RootActionSnapshot::Invoke {
                    function,
                    mode: InvocationMode::Retain,
                }
                .label()
            })
            .collect();
        assert_eq!(labels.len(), 3);
        assert!(
            labels
                .iter()
                .all(|label| label.starts_with("Self(Clone) -> String"))
        );
    }
}
