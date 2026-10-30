use crate::ArbitraryBytes;
use crate::ArbitraryRegistration;
use crate::AsyncValueFn;
use crate::CancellationBindingFn;
use crate::CancellationScope;
use crate::Effect;
use crate::FunctionExecutor;
use crate::FunctionInvocation;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::ProductionKind;
use crate::ReceiverMode;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use crate::SyncMutFn;
use crate::SyncRefFn;
use crate::SyncValueFn;
use crate::describe_shape;
use crate::list_element_shape;
use facet::Facet;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;

/// A reflected operation exposed to the Object Explorer.
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
pub struct Function {
    pub input_shape: &'static Shape,
    pub output_shape: &'static Shape,
    pub receiver_mode: ReceiverMode,
    pub kind: FunctionKind,
    /// Human-facing operation text; this is not a discovery classification.
    pub label: &'static str,
    /// The registration contract used to classify the operation.
    pub trait_contract: FunctionTraitContract,
    pub effects: &'static [Effect],
    pub executor: FunctionExecutor,
    pub output_to_runtime: RuntimeFromBoxedFn,
    pub registration_site: RegistrationSite,
    /// Runtime binding performed before an executable request starts.
    pub cancellation_binding: Option<CancellationBindingFn>,
    /// Authoritative low-level descriptor when this is a pure constructor adapter.
    pub arbitrary_registration: Option<&'static ArbitraryRegistration>,
}

impl Function {
    /// Adapt a pure constructor without registering or running it again.
    pub const fn from_arbitrary_registration(registration: &'static ArbitraryRegistration) -> Self {
        let mut function = Self::sync_mut(
            ArbitraryBytes::SHAPE,
            registration.output_shape,
            FunctionKind::Constructor,
            "arbitrary",
            FunctionTraitContract::Arbitrary,
            &[],
            registration.invoke_boxed,
            registration.output_to_runtime,
            registration.registration_site,
        );
        function.arbitrary_registration = Some(registration);
        function
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
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByValue,
            kind,
            label,
            trait_contract,
            effects,
            executor: FunctionExecutor::AsyncValue(invoke),
            output_to_runtime,
            registration_site,
            cancellation_binding: None,
            arbitrary_registration: None,
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
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByValue,
            kind,
            label,
            trait_contract,
            effects,
            executor: FunctionExecutor::SyncValue(invoke),
            output_to_runtime,
            registration_site,
            cancellation_binding: None,
            arbitrary_registration: None,
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
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByRef,
            kind,
            label,
            trait_contract,
            effects,
            executor: FunctionExecutor::SyncRef(invoke),
            output_to_runtime,
            registration_site,
            cancellation_binding: None,
            arbitrary_registration: None,
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
            input_shape,
            output_shape,
            receiver_mode: ReceiverMode::ByMut,
            kind,
            label,
            trait_contract,
            effects,
            executor: FunctionExecutor::SyncMut(invoke),
            output_to_runtime,
            registration_site,
            cancellation_binding: None,
            arbitrary_registration: None,
        }
    }

    pub const fn with_cancellation_binding(mut self, binding: CancellationBindingFn) -> Self {
        self.cancellation_binding = Some(binding);
        self
    }

    pub fn production_kind(&self, requested_shape: &'static Shape) -> Option<ProductionKind> {
        if self.output_shape.is_shape(requested_shape) {
            return Some(ProductionKind::Exact);
        }
        list_element_shape(self.output_shape)
            .is_some_and(|item| item.is_shape(requested_shape))
            .then_some(ProductionKind::ListElement)
    }

    pub fn supports_slot_kind(&self, slot_is_owned: bool) -> bool {
        self.receiver_mode != ReceiverMode::ByMut || slot_is_owned
    }

    pub fn is_async(&self) -> bool {
        matches!(self.executor, FunctionExecutor::AsyncValue(_))
    }

    /// Whether this operation declares a read and no writes.
    ///
    /// An empty effect list does not establish that an executable request is a
    /// read. A read may also solicit input; unattended callers must exclude
    /// operations for which [`Self::requires_user_input`] returns true. Scope
    /// and authentication admission remain the caller's responsibility.
    pub fn is_read_only(&self) -> bool {
        self.effects.contains(&Effect::Read) && !self.effects.contains(&Effect::Write)
    }

    /// Whether this operation declares that it solicits a person's response.
    pub fn requires_user_input(&self) -> bool {
        self.effects.contains(&Effect::Solicit)
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

pub fn describe_function(function: &Function) -> String {
    format!(
        "{} {} -> {} ({})",
        function.label,
        describe_shape(function.input_shape),
        describe_shape(function.output_shape),
        function.trait_contract,
    )
}

/// Register an offline `Arbitrary` constructor.
///
/// Use `register_arbitrary!(Type, associated = REGISTERED_ARBITRARY)` for a
/// locally defined type to expose a direct reference to its linked constructor.
/// Use `register_arbitrary!(pub static NAME: Type)` for collection or externally
/// owned types. The anonymous form remains available; none of these handles
/// duplicates the registration or searches the distributed slice.
#[macro_export]
macro_rules! register_arbitrary {
    ($thing_ty:ty) => {
        const _: () = {
            $crate::register_arbitrary!(static REGISTERED_FUNCTION: $thing_ty);
        };
    };
    ($thing_ty:ty, associated = $name:ident $(,)?) => {
        const _: () = {
            $crate::register_arbitrary!(static REGISTERED_FUNCTION: $thing_ty);

            impl $thing_ty {
                /// The exact registered `Arbitrary` constructor for this type.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const $name: &'static $crate::Function = &REGISTERED_FUNCTION;
            }
        };
    };
    ($visibility:vis static $name:ident: $thing_ty:ty $(,)?) => {
        #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
        $visibility static $name: $crate::Function = $crate::Function::sync_mut(
            <$crate::ArbitraryBytes as facet::Facet<'static>>::SHAPE,
            <$thing_ty as facet::Facet<'static>>::SHAPE,
            $crate::FunctionKind::Constructor,
            "arbitrary",
            $crate::FunctionTraitContract::Arbitrary,
            &[],
            $crate::invoke_arbitrary_mut::<$thing_ty>,
            $crate::runtime_from_boxed::<$thing_ty>,
            $crate::RegistrationSite::new(file!(), line!()),
        );
    };
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
/// `Request::REGISTERED_INTO_FUTURE`, a reference to this exact distributed-slice
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
            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::async_value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::AsyncInvoke,
                "invoke",
                $crate::FunctionTraitContract::IntoFuture,
                &[$($crate::Effect::$effect),*],
                $crate::invoke_result_future::<$thing_ty, $output_ty, _>,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            ).with_cancellation_binding($crate::bind_request_cancellation::<$thing_ty>);

            impl $thing_ty {
                /// The exact registered `IntoFuture` operation for this request.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const REGISTERED_INTO_FUTURE: &'static $crate::Function = &REGISTERED_FUNCTION;
            }
        };
    };
}

/// Register a `From` conversion, optionally exposing its distributed-slice entry
/// through `register_from!(Input => Output, associated = REGISTERED_INTO_OUTPUT)`.
/// This adds an inherent constant to the locally defined input type. Give each
/// conversion a distinct constant name when an input has multiple conversions.
///
/// For externally owned input types, use the free-static form:
/// `register_from!(pub static NAME: Input => Output)`. Both handles reference the
/// exact linked entry without searching the registry or duplicating metadata.
#[macro_export]
macro_rules! register_from {
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = [$($effect),*]);
        };
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident $(,)?) => {
        $crate::register_from!($input_ty => $output_ty, associated = $name, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, associated = $name:ident, effects = [$($effect:ident),* $(,)?] $(,)?) => {
        const _: () = {
            $crate::register_from!(static REGISTERED_FUNCTION: $input_ty => $output_ty, effects = [$($effect),*]);

            impl $input_ty {
                /// The exact registered `From` conversion for this input type.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the distributed slice")]
                pub const $name: &'static $crate::Function = &REGISTERED_FUNCTION;
            }
        };
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty) => {
        $crate::register_from!($visibility static $name: $input_ty => $output_ty, effects = []);
    };
    ($visibility:vis static $name:ident: $input_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
        $visibility static $name: $crate::Function = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::From<$input_ty>>::from(*input);
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            $crate::Function::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "from",
                $crate::FunctionTraitContract::From,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            )
        };
    };
}

#[macro_export]
macro_rules! register_try_from {
    ($input_ty:ty => $output_ty:ty) => {
        $crate::register_try_from!($input_ty => $output_ty, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: Box<dyn std::any::Any + Send>) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast::<$input_ty>().map_err(|_| {
                    eyre::eyre!("conversion expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = <$output_ty as std::convert::TryFrom<$input_ty>>::try_from(*input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_value(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $crate::FunctionKind::Conversion,
                "try_from",
                $crate::FunctionTraitContract::TryFrom,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}
#[macro_export]
macro_rules! register_fn_ref {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_ref!($input_ty => $output_ty, kind = $kind, label = $label, trait_contract = $trait_contract, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: &(dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_ref::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("shared-reference invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_ref(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

#[macro_export]
macro_rules! register_fn_mut {
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path $(,)?) => {
        $crate::register_fn_mut!($input_ty => $output_ty, kind = $kind, label = $label, trait_contract = $trait_contract, invoke = $invoke, effects = []);
    };
    ($input_ty:ty => $output_ty:ty, kind = $kind:expr, label = $label:expr, trait_contract = $trait_contract:expr, invoke = $invoke:path, effects = [$($effect:ident),* $(,)?]) => {
        const _: () = {
            fn invoke(input: &mut (dyn std::any::Any + Send)) -> eyre::Result<Box<dyn std::any::Any + Send>> {
                let input = input.downcast_mut::<$input_ty>().ok_or_else(|| {
                    eyre::eyre!("mutable invocation expected input type {}", std::any::type_name::<$input_ty>())
                })?;
                let output: $output_ty = $invoke(input)?;
                Ok(Box::new(output) as Box<dyn std::any::Any + Send>)
            }

            #[$crate::distributed_slice($crate::KNOWN_FUNCTIONS)]
            static REGISTERED_FUNCTION: $crate::Function = $crate::Function::sync_mut(
                <$input_ty as facet::Facet<'static>>::SHAPE,
                <$output_ty as facet::Facet<'static>>::SHAPE,
                $kind,
                $label,
                $trait_contract,
                &[$($crate::Effect::$effect),*],
                invoke,
                $crate::runtime_from_boxed::<$output_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}

/// Legacy executable and pure function registrations. Use `known_functions()`
/// to traverse the complete catalog, including lower-level value constructors.
#[distributed_slice]
pub static KNOWN_FUNCTIONS: [Function];
