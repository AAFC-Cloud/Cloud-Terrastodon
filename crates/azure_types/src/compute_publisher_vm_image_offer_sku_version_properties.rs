use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuVersionProperties {
    #[serde(default)]
    pub image_deprecation_status: Option<ImageDeprecationStatus>,
    #[serde(default)]
    pub hyper_v_generation: Option<String>,
    #[serde(default)]
    pub architecture: Option<String>,
    #[serde(default)]
    pub replica_type: Option<String>,
    #[serde(default)]
    pub replica_count: Option<u32>,
    #[serde(default)]
    pub go_live_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageDeprecationStatus {
    #[serde(default)]
    pub image_state: Option<String>,
    #[serde(default)]
    pub scheduled_deprecation_time: Option<String>,
}
