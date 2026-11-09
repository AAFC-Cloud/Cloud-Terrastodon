use crate::ArbitraryRegistration;
use crate::RegistrationSite;
use crate::Thing;
use arbitrary::Arbitrary;
use facet::Facet;
use std::marker::PhantomData;
use std::sync::Arc;

/// Macro support for the finite collection family of an explicitly registered value.
///
/// The inherent constants supply shared forms only when `T: Sync`. Otherwise
/// the registration macros resolve the owned-only fallback trait constants.
/// Each macro replaces these template sites with its own source location.
#[doc(hidden)]
pub struct RelatedValueRegistrations<T>(PhantomData<fn() -> T>);

impl<T: Facet<'static> + Send + Sync + 'static> RelatedValueRegistrations<T> {
    pub const THINGS: [Thing; 5] = [
        Thing::of::<Vec<T>>(RegistrationSite::new(file!(), line!())),
        Thing::of::<Arc<T>>(RegistrationSite::new(file!(), line!())),
        Thing::of::<Vec<Arc<T>>>(RegistrationSite::new(file!(), line!())),
        Thing::of::<Arc<Vec<T>>>(RegistrationSite::new(file!(), line!())),
        Thing::of::<Arc<Vec<Arc<T>>>>(RegistrationSite::new(file!(), line!())),
    ];
}

impl<T> RelatedValueRegistrations<T>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + Send + Sync + 'static,
{
    pub const ARBITRARY: [ArbitraryRegistration; 5] = [
        ArbitraryRegistration::of::<Vec<T>>(RegistrationSite::new(file!(), line!())),
        ArbitraryRegistration::of::<Arc<T>>(RegistrationSite::new(file!(), line!())),
        ArbitraryRegistration::of::<Vec<Arc<T>>>(RegistrationSite::new(file!(), line!())),
        ArbitraryRegistration::of::<Arc<Vec<T>>>(RegistrationSite::new(file!(), line!())),
        ArbitraryRegistration::of::<Arc<Vec<Arc<T>>>>(RegistrationSite::new(file!(), line!())),
    ];
}
