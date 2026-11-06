use crate::ComputePublisherVmImageOfferId;

/// Identity projection from [Compute image catalogue list-offers](https://learn.microsoft.com/rest/api/compute/virtual-machine-images/list-offers).
///
/// This intentionally decodes only the typed ID. Catalogue metadata such as
/// location, display name, tags, and expanded image properties is outside this
/// operation's output contract, so unknown-field rejection does not apply.
#[derive(facet::Facet)]
pub struct ComputePublisherVmImageOfferReference {
    pub id: ComputePublisherVmImageOfferId,
}
