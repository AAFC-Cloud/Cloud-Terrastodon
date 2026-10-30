use crate::InputDependency;
use crate::KnownShapeInfo;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use crate::RuntimeToBoxedFn;
use crate::RuntimeValue;
use crate::describe_shape;
use crate::input_dependencies;
use facet::Shape;
use linkme::distributed_slice;
use std::any::Any;
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub struct Thing {
    pub shape: &'static Shape,
    pub from_runtime: RuntimeFromBoxedFn,
    pub to_runtime: RuntimeToBoxedFn,
    pub registration_site: RegistrationSite,
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
        }
    }

    pub fn input_dependencies(&self) -> Vec<InputDependency> {
        input_dependencies(self.shape)
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
pub static KNOWN_THINGS: [Thing];

pub fn known_thing_for_shape(shape: &'static Shape) -> Option<&'static Thing> {
    KNOWN_THINGS
        .iter()
        .find(|thing| thing.shape.is_shape(shape))
}
pub fn known_things() -> Vec<&'static Thing> {
    KNOWN_THINGS.iter().collect()
}

pub fn known_shapes() -> Vec<KnownShapeInfo> {
    let mut by_label = BTreeMap::<String, &'static Thing>::new();
    for thing in KNOWN_THINGS {
        by_label.entry(describe_shape(thing.shape)).or_insert(thing);
    }
    by_label
        .into_iter()
        .map(|(label, thing)| KnownShapeInfo { thing, label })
        .collect()
}
#[macro_export]
macro_rules! register_thing {
    ($thing_ty:ty) => {
        const _: () = {
            #[$crate::distributed_slice($crate::KNOWN_THINGS)]
            static REGISTERED_THING: $crate::Thing = $crate::Thing::value(
                <$thing_ty as facet::Facet<'static>>::SHAPE,
                $crate::runtime_from_boxed::<$thing_ty>,
                $crate::runtime_into_boxed::<$thing_ty>,
                $crate::RegistrationSite::new(file!(), line!()),
            );
        };
    };
}
