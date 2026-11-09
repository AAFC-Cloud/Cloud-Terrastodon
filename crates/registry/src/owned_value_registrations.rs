use crate::RegistrationSite;
use crate::RelatedValueRegistrations;
use crate::Thing;
use facet::Facet;

/// Fallback for owned values that can cross threads but cannot be shared.
#[doc(hidden)]
pub trait OwnedValueRegistrations {
    const THINGS: [Thing; 1];
}

impl<T: Facet<'static> + Send + 'static> OwnedValueRegistrations for RelatedValueRegistrations<T> {
    const THINGS: [Thing; 1] = [Thing::of::<Vec<T>>(RegistrationSite::new(file!(), line!()))];
}
