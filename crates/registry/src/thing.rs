use crate::InputDependency;
use crate::KnownShapeInfo;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use crate::RuntimeToBoxedFn;
use crate::RuntimeValue;
use crate::describe_shape;
use crate::input_dependencies;
use facet::Facet;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Thing {
    pub shape: &'static Shape,
    pub from_runtime: RuntimeFromBoxedFn,
    pub to_runtime: RuntimeToBoxedFn,
    pub registration_site: RegistrationSite,
    // Only the explicitly registered value owns these finite companion entries.
    collections: &'static [Thing],
}

impl Thing {
    pub const fn value(
        shape: &'static Shape,
        from_runtime: RuntimeFromBoxedFn,
        to_runtime: RuntimeToBoxedFn,
        registration_site: RegistrationSite,
    ) -> Self {
        Self {
            shape,
            from_runtime,
            to_runtime,
            registration_site,
            collections: &[],
        }
    }

    #[doc(hidden)]
    pub const fn of<T: Facet<'static> + Send + 'static>(
        registration_site: RegistrationSite,
    ) -> Self {
        Self::value(
            T::SHAPE,
            crate::runtime_from_boxed::<T>,
            crate::runtime_into_boxed::<T>,
            registration_site,
        )
    }

    #[doc(hidden)]
    pub const fn with_collections(mut self, collections: &'static [Thing]) -> Self {
        self.collections = collections;
        self
    }

    #[doc(hidden)]
    pub const fn collection_registration_sites<const N: usize>(
        mut collections: [Thing; N],
        registration_site: RegistrationSite,
    ) -> [Thing; N] {
        let mut index = 0;
        while index < N {
            collections[index].registration_site = registration_site;
            index += 1;
        }
        collections
    }

    pub fn input_dependencies(&self) -> Vec<InputDependency> {
        input_dependencies(self.shape)
    }

    /// Collection forms supplied by this explicit registration point.
    ///
    /// This preserves their provenance without guessing from type names. A
    /// companion has no further forms: collection expansion is deliberately
    /// finite. Registering a value alone does not promise an arbitrary generator.
    pub const fn collection_forms(&self) -> &'static [Thing] {
        self.collections
    }

    pub fn runtime_from_boxed(&self, value: Box<dyn Any + Send>) -> eyre::Result<RuntimeValue> {
        (self.from_runtime)(value)
    }

    pub fn runtime_into_boxed(&self, value: RuntimeValue) -> eyre::Result<Box<dyn Any + Send>> {
        (self.to_runtime)(value)
    }
}

impl core::fmt::Display for Thing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", describe_shape(self.shape))
    }
}

#[distributed_slice]
/// Explicit value registration points. Use [`known_things`] to include automatic
/// collection forms and deduplicate aliases by their reflected type identity.
pub static THING_REGISTRATION_POINTS: [Thing];

// Explicit registrations take precedence over automatically supplied forms.
// Index only metadata; looking up a type must never construct a value.
static THING_CATALOG: LazyLock<Vec<&'static Thing>> = LazyLock::new(|| {
    let mut seen = HashSet::new();
    THING_REGISTRATION_POINTS
        .iter()
        .chain(
            THING_REGISTRATION_POINTS
                .iter()
                .flat_map(|thing| thing.collections),
        )
        .filter(|thing| seen.insert(thing.shape))
        .collect()
});

pub fn known_thing_for_shape(shape: &'static Shape) -> Option<&'static Thing> {
    THING_CATALOG
        .iter()
        .copied()
        .find(|thing| thing.shape.is_shape(shape))
}
/// The complete value catalog, including automatic collection forms and
/// deduplicating registrations for the same reflected type.
pub fn known_things() -> Vec<&'static Thing> {
    THING_CATALOG.clone()
}

pub fn known_shapes() -> Vec<KnownShapeInfo> {
    let mut by_label = BTreeMap::<String, &'static Thing>::new();
    for &thing in THING_CATALOG.iter() {
        by_label.entry(describe_shape(thing.shape)).or_insert(thing);
    }
    by_label
        .into_iter()
        .map(|(label, thing)| KnownShapeInfo { thing, label })
        .collect()
}
/// Register a value and its ordinary collection forms.
///
/// Every value supplies `Vec<T>`. Values supporting shared ownership also supply
/// `Arc<T>`, `Vec<Arc<T>>`, `Arc<Vec<T>>`, and `Arc<Vec<Arc<T>>>`. Expansion is
/// finite: the companion forms do not recursively register more collections.
/// Neither `Clone` nor `Sync` is required for the original owned value.
#[macro_export]
macro_rules! register_thing {
    ($thing_ty:ty) => {
        const _: () = {
            #[allow(unused_imports)]
            use $crate::OwnedValueRegistrations as _;

            #[$crate::distributed_slice($crate::THING_REGISTRATION_POINTS)]
            static REGISTERED_THING: $crate::Thing =
                $crate::Thing::of::<$thing_ty>($crate::RegistrationSite::new(file!(), line!()))
                    .with_collections(&$crate::Thing::collection_registration_sites(
                        $crate::RelatedValueRegistrations::<$thing_ty>::THINGS,
                        $crate::RegistrationSite::new(file!(), line!()),
                    ));
        };
    };
}
