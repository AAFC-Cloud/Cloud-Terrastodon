#[derive(Debug, PartialEq, Clone, Default, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputePublisherVmImageOfferSkuVersionProperties {
    #[facet(default)]
    pub image_deprecation_status: Option<ImageDeprecationStatus>,
    #[facet(default)]
    pub hyper_v_generation: Option<String>,
    #[facet(default)]
    pub architecture: Option<String>,
    #[facet(default)]
    pub replica_type: Option<String>,
    #[facet(default)]
    pub replica_count: Option<u32>,
    #[facet(default)]
    pub go_live_date: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ImageDeprecationStatus {
    #[facet(default)]
    pub image_state: Option<String>,
    #[facet(default)]
    pub scheduled_deprecation_time: Option<String>,
}
