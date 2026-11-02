use crate::AsyncValueFn;
use crate::CancellationBindingFn;
use crate::CancellationScope;
use crate::FunctionExecutor;
use crate::FunctionInvocation;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::FunctionDescriptor;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::RegistrationSite;
use cloud_terrastodon_registry::RuntimeFromBoxedFn;
use cloud_terrastodon_registry::SyncMutFn;
use cloud_terrastodon_registry::SyncRefFn;
use cloud_terrastodon_registry::SyncValueFn;
use cloud_terrastodon_registry::describe_function;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;

/// An executor-dependent registration and its callback contract.
///
/// When called with runtime-borrowed input, an operation must keep that input
/// and its shallow clones within the invocation future or its returned value.
/// It must not let borrowed data escape into detached tasks, blocking jobs,
/// globals, or other independently retained state. Awaiting a spawned task is
/// not sufficient: cancelling the parent can detach that task. Clone the
/// underlying data into genuinely owned storage before such an escape.
///
/// These are obligations of the caller using the registry's unsafe borrowed
/// pointer bridge, not lifetime guarantees supplied by type-erased metadata.
#[derive(Clone, Copy)]
pub struct RuntimeFunction {
    // Constructors keep the receiver mode and asynchronous metadata consistent
    // with the callback; callers cannot mutate either side independently.
    descriptor: FunctionDescriptor,
    executor: FunctionExecutor,
    cancellation_binding: Option<CancellationBindingFn>,
}

impl std::ops::Deref for RuntimeFunction {
    type Target = FunctionDescriptor;

    fn deref(&self) -> &Self::Target {
        self.descriptor()
    }
}

impl RuntimeFunction {
    pub const fn descriptor(&self) -> &FunctionDescriptor {
        &self.descriptor
    }

    pub const fn executor(&self) -> FunctionExecutor {
        self.executor
    }

    const fn assert_request_effects(kind: FunctionKind, effects: &[Effect]) {
        assert!(
            !matches!(kind, FunctionKind::AsyncInvoke) || !effects.is_empty(),
            "executable requests must explicitly declare nonempty effects"
        );
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn async_value(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        effects: &'static [Effect],
        invoke: AsyncValueFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self::assert_request_effects(kind, effects);
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByValue,
                kind,
                label,
                trait_contract,
                effects,
                output_to_runtime,
                registration_site,
                asynchronous: true,
            },
            executor: FunctionExecutor::AsyncValue(invoke),
            cancellation_binding: None,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_value(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        effects: &'static [Effect],
        invoke: SyncValueFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self::assert_request_effects(kind, effects);
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByValue,
                kind,
                label,
                trait_contract,
                effects,
                output_to_runtime,
                registration_site,
                asynchronous: false,
            },
            executor: FunctionExecutor::SyncValue(invoke),
            cancellation_binding: None,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_ref(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        effects: &'static [Effect],
        invoke: SyncRefFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self::assert_request_effects(kind, effects);
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByRef,
                kind,
                label,
                trait_contract,
                effects,
                output_to_runtime,
                registration_site,
                asynchronous: false,
            },
            executor: FunctionExecutor::SyncRef(invoke),
            cancellation_binding: None,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Registry functions are assembled from static macro metadata"
    )]
    pub const fn sync_mut(
        input_shape: &'static Shape,
        output_shape: &'static Shape,
        kind: FunctionKind,
        label: &'static str,
        trait_contract: FunctionTraitContract,
        effects: &'static [Effect],
        invoke: SyncMutFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self::assert_request_effects(kind, effects);
        Self {
            descriptor: FunctionDescriptor {
                input_shape,
                output_shape,
                receiver_mode: ReceiverMode::ByMut,
                kind,
                label,
                trait_contract,
                effects,
                output_to_runtime,
                registration_site,
                asynchronous: false,
            },
            executor: FunctionExecutor::SyncMut(invoke),
            cancellation_binding: None,
        }
    }

    pub const fn with_cancellation_binding(mut self, binding: CancellationBindingFn) -> Self {
        assert!(
            matches!(self.executor, FunctionExecutor::AsyncValue(_))
                && matches!(self.descriptor.kind, FunctionKind::AsyncInvoke),
            "cancellation binding belongs to an asynchronous executable request"
        );
        self.cancellation_binding = Some(binding);
        self
    }

    pub fn invoke_value_boxed(
        &self,
        mut input: Box<dyn Any + Send>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<FunctionInvocation> {
        if let Some(bind) = self.cancellation_binding {
            bind(input.as_mut(), cancellation)?;
        }
        match self.executor {
            FunctionExecutor::AsyncValue(invoke) => Ok(FunctionInvocation::Pending(invoke(input))),
            FunctionExecutor::SyncValue(invoke) => Ok(FunctionInvocation::Ready(invoke(input)?)),
            FunctionExecutor::SyncRef(_) => {
                eyre::bail!("{} requires ByRef", describe_function(self));
            }
            FunctionExecutor::SyncMut(_) => {
                eyre::bail!("{} requires ByMut", describe_function(self));
            }
        }
    }

    pub fn invoke_ref_boxed(&self, input: &(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            FunctionExecutor::SyncRef(invoke) => invoke(input),
            _ => {
                eyre::bail!("{} does not support ByRef", describe_function(self));
            }
        }
    }

    pub fn invoke_mut_boxed(
        &self,
        input: &mut (dyn Any + Send),
    ) -> eyre::Result<Box<dyn Any + Send>> {
        match self.executor {
            FunctionExecutor::SyncMut(invoke) => invoke(input),
            _ => {
                eyre::bail!("{} does not support ByMut", describe_function(self));
            }
        }
    }
}

/// Register an executable request with shape-discovered caller cancellation.
///
/// A single direct cancellation-token field is replaced before invocation,
/// including opaque fields. Tokenless inputs are unchanged; multiple matching
/// fields produce an error rather than selecting an arbitrary token.
///
/// Executable requests must explicitly declare at least one effect. Declare
/// `Write` for mutations, including workflows which also read or verify data.
/// Declare `Solicit` when the operation asks a person for input or confirmation.
/// Pure conversions and arbitrary generators use their separate registrations.
///
/// The input must be a locally defined request type. The macro also exposes
/// `Request::REGISTERED_INTO_FUTURE`, a handle to this exact distributed-slice
/// entry. It does not copy the metadata or search the registry. Lifetime-bearing
/// requests expose the constant on their registered `'static` specialization.
#[macro_export]
macro_rules! register_into_future {
    ($thing_ty:ty => $output_ty:ty) => {
        compile_error!("register_into_future! requires explicit, nonempty effects containing Read, Write, Solicit, or a combination");
    };
    ($thing_ty:ty => $output_ty:ty, effects = []) => {
        compile_error!("executable requests cannot have empty effects; declare Read, Write, Solicit, or a combination");
    };
    ($thing_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_RUNTIME_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::RuntimeFunction = $crate::RuntimeFunction::async_value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                cloud_terrastodon_registry::FunctionKind::AsyncInvoke,
                "invoke",
                cloud_terrastodon_registry::FunctionTraitContract::IntoFuture,
                &[$(cloud_terrastodon_registry::Effect::$effect),*],
                $crate::invoke_result_future::<$thing_ty, $output_ty, _>,
                cloud_terrastodon_registry::runtime_from_boxed::<$output_ty>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            ).with_cancellation_binding($crate::bind_request_cancellation::<$thing_ty>);

            #[cloud_terrastodon_registry::distributed_slice(cloud_terrastodon_registry::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &cloud_terrastodon_registry::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();


            impl $thing_ty {
                /// The exact registered `IntoFuture` operation for this request.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const REGISTERED_INTO_FUTURE: $crate::Function = $crate::Function::Runtime(&REGISTERED_FUNCTION);
            }
        };
    };
}

/// Register an effectful `From` operation, optionally exposing its distributed-slice entry
/// through `register_from!(Input => Output, associated = REGISTERED_INTO_OUTPUT, effects = [Read])`.
/// This adds an inherent constant to the locally defined input type. Give each
/// conversion a distinct constant name when an input has multiple conversions.
///
/// For externally owned input types, use the free-static form:
/// `register_from!(pub static NAME: Input => Output, effects = [Read])`. Both handles reference the
/// exact linked entry without searching the registry or duplicating metadata.
/// A named static owns a `RuntimeFunction`; wrap its reference with `Function::Runtime`.
#[macro_export]
macro_rules! register_from {
    ($input_ty:ty => $output_ty:ty) => {
        compile_error!("Pure conversions belong to cloud_terrastodon_registry::register_from");
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = [$($effect),*]);
        };
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident $(,)?) => {
        compile_error!("Pure conversions belong to cloud_terrastodon_registry::register_from");
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident, effects = [$($effect:ident),+ $(,)?] $(,)?) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = [$($effect),*]);

            impl $input_ty {
                /// The exact registered `From` conversion for this input type.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const $name: $crate::Function = $crate::Function::Runtime(&REGISTERED_FUNCTION);
            }
        };
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty) => {
        compile_error!("Pure conversions belong to cloud_terrastodon_registry::register_from");
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        #[$crate::distributed_slice($crate::KNOWN_RUNTIME_FUNCTIONS)]
        $visibility static $name: $crate::RuntimeFunction = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::From<$input_ty>>::from(*input);
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            $crate::RuntimeFunction::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                cloud_terrastodon_registry::FunctionKind::Conversion,
                "from",
                cloud_terrastodon_registry::FunctionTraitContract::From,
                &[$(cloud_terrastodon_registry::Effect::$effect),*],
                invoke,
                cloud_terrastodon_registry::runtime_from_boxed::<$output_ty>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            )
        };
        const _: () = {

            #[cloud_terrastodon_registry::distributed_slice(cloud_terrastodon_registry::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &cloud_terrastodon_registry::FunctionDescriptor = $name.descriptor();
        };
    };
}

#[macro_export]
macro_rules! register_try_from {
    ($input_ty:ty => $output_ty:ty) => {
        compile_error!("Pure conversions belong to cloud_terrastodon_registry::register_try_from");
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),+ $(,)?]) => {
        const _: () = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::TryFrom<$input_ty>>::try_from(*input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_RUNTIME_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::RuntimeFunction = $crate::RuntimeFunction::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                cloud_terrastodon_registry::FunctionKind::Conversion,
                "try_from",
                cloud_terrastodon_registry::FunctionTraitContract::TryFrom,
                &[$(cloud_terrastodon_registry::Effect::$effect),*],
                invoke,
                cloud_terrastodon_registry::runtime_from_boxed::<$output_ty>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            );

            #[cloud_terrastodon_registry::distributed_slice(cloud_terrastodon_registry::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &cloud_terrastodon_registry::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();

        };
    };
}
#[macro_export]
macro_rules! register_fn_ref {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        compile_error!("Pure operations belong to cloud_terrastodon_registry::register_fn_ref");
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),+ $(,)?]) => {
        const _: () = {
            fn invoke(input: &(dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_ref::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("shared-reference invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_RUNTIME_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::RuntimeFunction = $crate::RuntimeFunction::sync_ref(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                &[$(cloud_terrastodon_registry::Effect::$effect),*],
                invoke,
                cloud_terrastodon_registry::runtime_from_boxed::<$output_ty>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            );

            #[cloud_terrastodon_registry::distributed_slice(cloud_terrastodon_registry::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &cloud_terrastodon_registry::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();

        };
    };
}

#[macro_export]
macro_rules! register_fn_mut {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        compile_error!("Pure operations belong to cloud_terrastodon_registry::register_fn_mut");
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),+ $(,)?]) => {
        const _: () = {
            fn invoke(input: &mut (dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_mut::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("mutable invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_RUNTIME_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::RuntimeFunction = $crate::RuntimeFunction::sync_mut(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                &[$(cloud_terrastodon_registry::Effect::$effect),*],
                invoke,
                cloud_terrastodon_registry::runtime_from_boxed::<$output_ty>,
                cloud_terrastodon_registry::RegistrationSite::new(file!(), line!()),
            );

            #[cloud_terrastodon_registry::distributed_slice(cloud_terrastodon_registry::KNOWN_FUNCTION_DESCRIPTORS)]
            static REGISTERED_DESCRIPTOR: &cloud_terrastodon_registry::FunctionDescriptor = REGISTERED_FUNCTION.descriptor();

        };
    };
}

/// Executor-dependent registrations. Use `known_functions()` to include
/// context-free operation handles in the complete catalogue.
#[distributed_slice]
pub static KNOWN_RUNTIME_FUNCTIONS: [RuntimeFunction];
