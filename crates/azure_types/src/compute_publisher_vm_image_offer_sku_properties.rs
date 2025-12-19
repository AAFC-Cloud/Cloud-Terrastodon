use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuProperties {
    #[serde(default)]
    pub automatic_os_upgrade_properties: Option<AutomaticOsUpgradeProperties>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutomaticOsUpgradeProperties {
    #[serde(rename = "automaticOSUpgradeSupported")]
    pub automatic_os_upgrade_supported: bool,
}
