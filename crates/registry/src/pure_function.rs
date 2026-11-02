use crate::FunctionDescriptor;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::PureFunctionExecutor;
use crate::ReceiverMode;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use crate::SyncMutFn;
use crate::SyncRefFn;
use crate::SyncValueFn;
use crate::describe_function;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;
use std::ops::Deref;

/// A pure synchronous operation and its authoritative metadata.
///
/// Registered constructors, conversions, and projections must consume only supplied values.
/// Operations requiring credentials, service state, or application effects must
/// register with the execution layer instead.
#[derive(Clone, Copy)]
pub struct PureFunction {
    // Keep the synchronous callback consistent with its receiver and effect-free metadata.
    descriptor: FunctionDescriptor,
    executor: PureFunctionExecutor,
}

impl Deref for PureFunction {
    type Target = FunctionDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}

impl PureFunction {
    /// Borrow the authoritative metadata without changing its invocation contract.
    pub const fn descriptor(&self) -> &FunctionDescriptor {
        &self.descriptor
    }

    /// Copy the synchronous callback contract for an execution-layer adapter.
    pub const fn executor(&self) -> PureFunctionExecutor {
        self.executor
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registrations are assembled from static macro metadata"
    )]
    pub const fn sync_value(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        invoke: SyncValueFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        assert!(
            !matches!(kind, FunctionKind::AsyncInvoke),
            "pure registrations cannot be executable requests"
        );
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByValue,
                kind,
                label,
                trait_contract,
                effects: &[],
                asynchronous: false,
                output_to_runtime,
                registration_site,
            },
            executor: PureFunctionExecutor::SyncValue(invoke),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registrations are assembled from static macro metadata"
    )]
    pub const fn sync_ref(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        invoke: SyncRefFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        assert!(
            !matches!(kind, FunctionKind::AsyncInvoke),
            "pure registrations cannot be executable requests"
        );
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByRef,
                kind,
                label,
                trait_contract,
                effects: &[],
                asynchronous: false,
                output_to_runtime,
                registration_site,
            },
            executor: PureFunctionExecutor::SyncRef(invoke),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registrations are assembled from static macro metadata"
    )]
    pub const fn sync_mut(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        invoke: SyncMutFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        assert!(
            !matches!(kind, FunctionKind::AsyncInvoke),
            "pure registrations cannot be executable requests"
        );
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByMut,
                kind,
                label,
                trait_contract,
                effects: &[],
                asynchronous: false,
                output_to_runtime,
                registration_site,
            },
            executor: PureFunctionExecutor::SyncMut(invoke),
        }
    }

    pub fn invoke_value_boxed(
        &self,
        input: Box<dyn Any + Send>,
    ) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            PureFunctionExecutor::SyncValue(invoke) => invoke(input),
            _ => Err(eyre::eyre!(
                "{} does not support ByValue",
                describe_function(self)
            )),
        }
    }

    pub fn invoke_ref_boxed(&self, input: &(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            PureFunctionExecutor::SyncRef(invoke) => invoke(input),
            _ => Err(eyre::eyre!(
                "{} does not support ByRef",
                describe_function(self)
            )),
        }
    }

    pub fn invoke_mut_boxed(
        &self,
        input: &mut (dyn Any + Send),
    ) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            PureFunctionExecutor::SyncMut(invoke) => invoke(input),
            _ => Err(eyre::eyre!(
                "{} does not support ByMut",
                describe_function(self)
            )),
        }
    }
}

#[distributed_slice]
pub static KNOWN_PURE_FUNCTIONS: [PureFunction];

/// Linked pure functions; arbitrary generators retain their separate typed registration.
pub fn known_pure_functions() -> &'static [PureFunction] {
    &KNOWN_PURE_FUNCTIONS
}

pub fn functions_from(shape: &'static Shape) -> Vec<&'static PureFunction> {
    known_pure_functions()
        .iter()
        .filter(|function| function.input_shape.is_shape(shape))
        .collect()
}

pub fn functions_to(shape: &'static Shape) -> Vec<&'static PureFunction> {
    known_pure_functions()
        .iter()
        .filter(|function| function.production_kind(shape).is_some())
        .collect()
}

pub fn functions_from_to(
    input: &'static Shape,
    output: &'static Shape,
) -> Vec<&'static PureFunction> {
    known_pure_functions()
        .iter()
        .filter(|function| {
            function.input_shape.is_shape(input) && function.production_kind(output).is_some()
        })
        .collect()
}

/// Register a `From` conversion, optionally exposing its distributed-slice entry
/// through `register_from!(Input => Output, associated = REGISTERED_INTO_OUTPUT)`.
/// This adds an inherent constant to the locally defined input type. Give each
/// conversion a distinct constant name when an input has multiple conversions.
///
/// For externally owned input types, use the free-static form:
/// `register_from!(pub static NAME: Input => Output)`. Both handles reference the
/// exact linked entry without searching the registry or duplicating metadata.
/// Only conversions without runtime effects belong in this registry.
#[macro_export]
macro_rules! register_from {
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        compile_error!("effectful conversions must use cloud_terrastodon_registry_execution::register_from!");
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident, effects = [$($effect:ident),+ $(,)?] $(,)?) => {
        compile_error!("effectful conversions must use cloud_terrastodon_registry_execution::register_from!");
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        compile_error!("effectful conversions must use cloud_terrastodon_registry_execution::register_from!");
    };
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = []) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = []);
        };
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident $(,)?) => {
        $crate::register_from!($input_ty => $output_ty, associated = $name, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident, effects = [] $(,)?) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = []);

            impl $input_ty {
                /// The exact registered `From` conversion for this input type.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const $name: &'static $crate::PureFunction = &REGISTERED_FUNCTION;
            }
        };
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty) => {
        $crate::register_from!($visibility static $name: $input_ty => $output_ty, effects = []);
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty, effects = []) => {
        #[$crate::distributed_slice($crate::KNOWN_PURE_FUNCTIONS)]
        $visibility static $name: $crate::PureFunction = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::From<$input_ty>>::from(*input);
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            $crate::PureFunction::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "from",
                $crate::FunctionTraitContract::From,
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            )
        };
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &$crate::FunctionDescriptor = $name.descriptor();
        };
    };
}

#[macro_export]
macro_rules! register_try_from {
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        compile_error!("effectful conversions must use cloud_terrastodon_registry_execution::register_try_from!");
    };
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_try_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = []) => {
        const _: () = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::TryFrom<$input_ty>>::try_from(*input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_PURE_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::PureFunction = $crate::PureFunction::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "try_from",
                $crate::FunctionTraitContract::TryFrom,
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
            #[$crate::distributed_slice($crate::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &$crate::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();
        };
    };
}
#[macro_export]
macro_rules! register_fn_ref {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),+ $(,)?]) => {
        compile_error!("effectful operations must use cloud_terrastodon_registry_execution::register_fn_ref!");
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_ref!($input_ty => $output_ty, kind = $kind, label = $label, trait_contract = $trait_contract, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = []) => {
        const _: () = {
            fn invoke(input: &(dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_ref::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("shared-reference invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_PURE_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::PureFunction = $crate::PureFunction::sync_ref(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
            #[$crate::distributed_slice($crate::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &$crate::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();
        };
    };
}

#[macro_export]
macro_rules! register_fn_mut {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),+ $(,)?]) => {
        compile_error!("effectful operations must use cloud_terrastodon_registry_execution::register_fn_mut!");
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_mut!($input_ty => $output_ty, kind = $kind, label = $label, trait_contract = $trait_contract, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = []) => {
        const _: () = {
            fn invoke(input: &mut (dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_mut::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("mutable invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_PURE_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::PureFunction = $crate::PureFunction::sync_mut(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
            #[$crate::distributed_slice($crate::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &$crate::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use facet::Facet;

    #[derive(Facet)]
    struct Number(i32);

    #[derive(Facet)]
    struct Nonnegative(i32);

    #[derive(Debug)]
    struct NegativeNumber(i32);

    impl std::fmt::Display for NegativeNumber {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "{} is negative", self.0)
        }
    }

    impl std::error::Error for NegativeNumber {}

    impl From<Number> for i32 {
        fn from(value: Number) -> Self {
            value.0
        }
    }

    impl TryFrom<Number> for Nonnegative {
        type Error = NegativeNumber;

        fn try_from(value: Number) -> Result<Self, Self::Error> {
            if value.0 < 0 {
                Err(NegativeNumber(value.0))
            } else {
                Ok(Self(value.0))
            }
        }
    }

    crate::register_from!(Number => i32, associated = REGISTERED_INTO_I32);
    crate::register_try_from!(Number => Nonnegative);

    #[test]
    fn typed_conversion_handle_and_metadata_share_one_registration() {
        let registered = Number::REGISTERED_INTO_I32;
        let discovered = functions_from_to(Number::SHAPE, i32::SHAPE);
        assert_eq!(discovered.len(), 1);
        assert!(std::ptr::eq(registered, discovered[0]));
        assert_eq!(
            crate::known_function_descriptors()
                .filter(|descriptor| std::ptr::eq(*descriptor, &registered.descriptor))
                .count(),
            1
        );
        let output = registered.invoke_value_boxed(Box::new(Number(42))).unwrap();
        assert_eq!(*output.downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn erased_conversion_rejects_wrong_input_and_preserves_source_errors() {
        let conversions = functions_from_to(Number::SHAPE, Nonnegative::SHAPE);
        assert_eq!(conversions.len(), 1);
        let conversion = conversions[0];
        let wrong_input = conversion.invoke_value_boxed(Box::new(42_i32)).unwrap_err();
        assert!(
            wrong_input
                .to_string()
                .contains(std::any::type_name::<Number>())
        );
        let rejected = conversion
            .invoke_value_boxed(Box::new(Number(-7)))
            .unwrap_err();
        assert_eq!(rejected.downcast_ref::<NegativeNumber>().unwrap().0, -7);
        let output = conversion.invoke_value_boxed(Box::new(Number(7))).unwrap();
        assert_eq!(output.downcast::<Nonnegative>().unwrap().0, 7);
        assert!(conversion.invoke_ref_boxed(&Number(7)).is_err());
    }
}
