use crate::ArbitraryBytes;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;

/// An erased synchronous constructor used to integrate pure value catalogs.
pub type ArbitraryInvocationFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;

/// An offline arbitrary constructor that does not require an execution runtime.
///
/// The descriptor is the authoritative registration. Higher-level catalogs may
/// expose an executable view while retaining a reference to this descriptor.
#[derive(Clone, Copy)]
pub struct ArbitraryRegistration {
    pub output_shape: &'static Shape,
    pub invoke_boxed: ArbitraryInvocationFn,
    pub output_to_runtime: RuntimeFromBoxedFn,
    pub registration_site: RegistrationSite,
}

impl ArbitraryRegistration {
    pub const fn new(
        output_shape: &'static Shape,
        invoke_boxed: ArbitraryInvocationFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            output_shape,
            invoke_boxed,
            output_to_runtime,
            registration_site,
        }
    }

    /// Generate one value, consuming only the supplied offline entropy.
    pub fn generate(&self, bytes: &mut ArbitraryBytes) -> eyre::Result<Box<dyn Any + Send>> {
        (self.invoke_boxed)(bytes)
    }
}

#[distributed_slice]
pub static KNOWN_ARBITRARY_VALUES: [ArbitraryRegistration];

/// Register a pure offline constructor beside its value type.
///
/// Associated handles identify this exact descriptor; no executable registry
/// or authentication dependency is needed by the defining crate.
#[macro_export]
macro_rules! register_arbitrary {
    ($thing_ty:ty) => {
        const _: () = {
            $crate::register_arbitrary!(static REGISTERED_ARBITRARY: $thing_ty);
        };
    };
    ($thing_ty:ty, associated = $name:ident $(,)?) => {
        const _: () = {
            $crate::register_arbitrary!(static REGISTERED_ARBITRARY: $thing_ty);
            impl $thing_ty {
                /// The exact pure arbitrary registration for this type.
                #[allow(dead_code, reason = "Registrations may be consumed solely through the linked catalog")]
                pub const $name: &'static $crate::ArbitraryRegistration = &REGISTERED_ARBITRARY;
            }
        };
    };
    ($visibility:vis static $name:ident: $thing_ty:ty $(,)?) => {
        #[$crate::distributed_slice($crate::KNOWN_ARBITRARY_VALUES)]
        $visibility static $name: $crate::ArbitraryRegistration = $crate::ArbitraryRegistration::new(
            <$thing_ty as facet::Facet<'static>>::SHAPE,
            $crate::invoke_arbitrary_mut::<$thing_ty>,
            $crate::runtime_from_boxed::<$thing_ty>,
            $crate::RegistrationSite::new(file!(), line!()),
        );
    };
}
