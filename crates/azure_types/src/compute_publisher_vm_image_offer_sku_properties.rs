use facet::Facet;
#[derive(Debug, PartialEq, Clone, Default, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuProperties {
    #[facet(default)]
    pub automatic_os_upgrade_properties: Option<AutomaticOsUpgradeProperties>,
}

#[derive(Debug, PartialEq, Clone, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AutomaticOsUpgradeProperties {
    #[facet(rename = "automaticOSUpgradeSupported")]
    pub automatic_os_upgrade_supported: bool,
}
