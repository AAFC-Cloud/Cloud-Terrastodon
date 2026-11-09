use crate::ArbitraryRegistration;
use crate::RegistrationSite;
use crate::RelatedValueRegistrations;
use arbitrary::Arbitrary;
use facet::Facet;

/// Fallback arbitrary constructors for values without shared ownership support.
#[doc(hidden)]
pub trait OwnedArbitraryRegistrations {
    const ARBITRARY: [ArbitraryRegistration; 1];
}

impl<T> OwnedArbitraryRegistrations for RelatedValueRegistrations<T>
where
    T: Facet<'static> + for<'a> Arbitrary<'a> + Send + 'static,
{
    const ARBITRARY: [ArbitraryRegistration; 1] = [ArbitraryRegistration::of::<Vec<T>>(
        RegistrationSite::new(file!(), line!()),
    )];
}
