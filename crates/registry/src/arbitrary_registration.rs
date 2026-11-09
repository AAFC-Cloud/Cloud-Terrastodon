use crate::ArbitraryBytes;
use crate::FunctionDescriptor;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::ReceiverMode;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use arbitrary::Arbitrary;
use facet::Facet;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::LazyLock;

/// An erased synchronous constructor used to integrate pure value catalogs.
pub type ArbitraryInvocationFn = fn(&mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>;

/// An offline arbitrary constructor that does not require an execution runtime.
///
/// The descriptor is the authoritative registration. Higher-level catalogs may
/// expose an executable view while retaining a reference to this descriptor.
#[derive(Clone, Copy)]
pub struct ArbitraryRegistration {
    // Keep the mutable entropy callback paired with its fixed constructor metadata.
    descriptor: FunctionDescriptor,
    invoke_boxed: ArbitraryInvocationFn,
    collections: &'static [ArbitraryRegistration],
}

impl Deref for ArbitraryRegistration {
    type Target = FunctionDescriptor;

    fn deref(&self) -> &Self::Target {
        &self.descriptor
    }
}

impl ArbitraryRegistration {
    /// Borrow the metadata without changing the constructor's invocation contract.
    pub const fn descriptor(&self) -> &FunctionDescriptor {
        &self.descriptor
    }

    /// Copy the synchronous callback for an execution-layer adapter.
    pub const fn invocation(&self) -> ArbitraryInvocationFn {
        self.invoke_boxed
    }

    pub const fn new(
        output_shape: &'static Shape,
        invoke_boxed: ArbitraryInvocationFn,
        output_to_runtime: RuntimeFromBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            descriptor: FunctionDescriptor {
                input_shape: ArbitraryBytes::SHAPE,
                output_shape,
                receiver_mode: ReceiverMode::ByMut,
                kind: FunctionKind::Constructor,
                label: "arbitrary",
                trait_contract: FunctionTraitContract::Arbitrary,
                effects: &[],
                asynchronous: false,
                output_to_runtime,
                registration_site,
            },
            invoke_boxed,
            collections: &[],
        }
    }

    #[doc(hidden)]
    pub const fn of<T>(registration_site: RegistrationSite) -> Self
    where
        T: Facet<'static> + for<'a> Arbitrary<'a> + Send + 'static,
    {
        Self::new(
            T::SHAPE,
            crate::invoke_arbitrary_mut::<T>,
            crate::runtime_from_boxed::<T>,
            registration_site,
        )
    }

    #[doc(hidden)]
    pub const fn with_collections(mut self, collections: &'static [ArbitraryRegistration]) -> Self {
        self.collections = collections;
        self
    }

    #[doc(hidden)]
    pub const fn collection_registration_sites<const N: usize>(
        mut collections: [ArbitraryRegistration; N],
        registration_site: RegistrationSite,
    ) -> [ArbitraryRegistration; N] {
        let mut index = 0;
        while index < N {
            collections[index].descriptor.registration_site = registration_site;
            index += 1;
        }
        collections
    }

    /// Generate one value, consuming only the supplied offline entropy.
    pub fn generate(&self, bytes: &mut ArbitraryBytes) -> eyre::Result<Box<dyn Any + Send>> {
        (self.invoke_boxed)(bytes)
    }
}

#[distributed_slice]
/// Explicit arbitrary constructor registration points. Use
/// [`known_arbitrary_values`] for the complete catalog, including automatically
/// supplied collection constructors.
pub static ARBITRARY_REGISTRATION_POINTS: [ArbitraryRegistration];

/// All explicit offline constructors, retaining their original static handles,
/// plus automatic collection forms not already supplied for that output shape.
/// Distinct explicit constructors remain distinct even when their shapes match.
pub fn known_arbitrary_values() -> &'static [&'static ArbitraryRegistration] {
    static ARBITRARY_CATALOG: LazyLock<Vec<&'static ArbitraryRegistration>> = LazyLock::new(|| {
        let mut values = ARBITRARY_REGISTRATION_POINTS.iter().collect::<Vec<_>>();
        let mut seen = values
            .iter()
            .map(|value| value.output_shape)
            .collect::<HashSet<_>>();
        values.extend(
            ARBITRARY_REGISTRATION_POINTS
                .iter()
                .flat_map(|value| value.collections)
                .filter(|value| seen.insert(value.output_shape)),
        );
        values
    });
    &ARBITRARY_CATALOG
}

/// Register a pure offline constructor beside its value type.
///
/// Associated handles identify this exact descriptor; no executable registry
/// or authentication dependency is needed by the defining crate.
/// The same finite collection family as [`crate::register_thing!`] receives
/// arbitrary constructors without adding a `Clone` or `Sync` requirement to `T`.
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
        #[$crate::distributed_slice($crate::ARBITRARY_REGISTRATION_POINTS)]
        $visibility static $name: $crate::ArbitraryRegistration = {
            #[allow(unused_imports)]
            use $crate::OwnedArbitraryRegistrations as _;

            $crate::ArbitraryRegistration::of::<$thing_ty>(
                $crate::RegistrationSite::new(file!(), line!()),
            ).with_collections(&$crate::ArbitraryRegistration::collection_registration_sites(
                $crate::RelatedValueRegistrations::<$thing_ty>::ARBITRARY,
                $crate::RegistrationSite::new(file!(), line!()),
            ))
        };
    };
}
