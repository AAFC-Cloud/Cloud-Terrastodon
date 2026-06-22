use crate::AzureLocationName;
use crate::ComputePublisherVmImageOfferSkuVersionId;
use crate::ComputePublisherVmImageOfferSkuVersionName;
use crate::ComputePublisherVmImageOfferSkuVersionProperties;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuVersion {
    pub id: ComputePublisherVmImageOfferSkuVersionId,
    pub name: ComputePublisherVmImageOfferSkuVersionName,
    pub location: AzureLocationName,
    pub properties: ComputePublisherVmImageOfferSkuVersionProperties,
}
