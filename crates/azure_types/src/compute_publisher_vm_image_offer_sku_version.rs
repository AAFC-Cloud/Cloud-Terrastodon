use crate::ComputePublisherVmImageOfferSkuVersionId;
use crate::ComputePublisherVmImageOfferSkuVersionName;
use crate::ComputePublisherVmImageOfferSkuVersionProperties;
use crate::LocationName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuVersion {
    pub id: ComputePublisherVmImageOfferSkuVersionId,
    pub name: ComputePublisherVmImageOfferSkuVersionName,
    pub location: LocationName,
    pub properties: ComputePublisherVmImageOfferSkuVersionProperties,
}
