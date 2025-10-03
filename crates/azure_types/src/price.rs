use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::location::LocationName;
/// https://learn.microsoft.com/en-us/rest/api/cost-management/retail-prices/azure-retail-prices
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub arm_region_name: LocationName,
    pub arm_sku_name: String,
    pub currency_code: String,
    pub effective_start_date: DateTime<Utc>,
    pub is_primary_meter_region: bool,
    pub location: String,
    pub meter_id: Uuid,
    pub meter_name: String,
    pub product_id: String,
    pub product_name: String,
    pub retail_price: f32,
    pub service_family: String,
    pub service_id: String,
    pub service_name: String,
    pub sku_id: String,
    pub sku_name: String,
    pub tier_minimum_units: f32,
    #[serde(rename = "type")]
    pub kind: String,
    pub unit_of_measure: String,
    pub unit_price: f32,
}
