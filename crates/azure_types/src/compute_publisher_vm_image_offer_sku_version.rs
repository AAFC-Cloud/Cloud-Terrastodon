use crate::AzureLocationName;
use crate::ComputePublisherVmImageOfferSkuVersionId;
use crate::ComputePublisherVmImageOfferSkuVersionName;
use crate::ComputePublisherVmImageOfferSkuVersionProperties;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuVersion {
    pub id: ComputePublisherVmImageOfferSkuVersionId,
    pub name: ComputePublisherVmImageOfferSkuVersionName,
    pub location: AzureLocationName,
    pub properties: ComputePublisherVmImageOfferSkuVersionProperties,
}
