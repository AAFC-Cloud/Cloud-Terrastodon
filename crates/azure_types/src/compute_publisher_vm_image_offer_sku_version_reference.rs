use crate::ComputePublisherVmImageOfferSkuVersionId;
use facet::Facet;

/// Identity projection from [Compute image catalogue list](https://learn.microsoft.com/rest/api/compute/virtual-machine-images/list).
///
/// This intentionally decodes only the typed ID. Catalogue metadata such as
/// location, display name, tags, and expanded image properties is outside this
/// operation's output contract, so unknown-field rejection does not apply.
#[derive(Facet)]
pub struct ComputePublisherVmImageOfferSkuVersionReference {
    pub id: ComputePublisherVmImageOfferSkuVersionId,
}
