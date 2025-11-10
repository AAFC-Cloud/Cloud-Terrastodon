use crate::prelude::SubscriptionId;
use crate::tenants::TenantId;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug)]
pub struct TenantLicenseCollection(pub Vec<TenantLicense>);
impl TenantLicenseCollection {
    pub fn has_aad_premium_p2(&self) -> bool {
        self.0
            .iter()
            .filter(|license| license.capability_status == TenantLicenseCapabilityStatus::Enabled)
            .flat_map(|license| license.service_plans.iter())
            .any(|plan| plan.service_plan_name == "AAD_PREMIUM_P2")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantLicense {
    pub account_id: TenantId,
    pub account_name: String,
    pub applies_to: TenantLicenseAppliesTo,
    pub capability_status: TenantLicenseCapabilityStatus,
    pub consumed_units: u32,
    /// {tenant_id}-{uuid}
    pub id: String,
    pub prepaid_units: TenantLicensePrepaidUnits,
    pub service_plans: Vec<TenantLicenseServicePlan>,
    pub sku_id: Uuid,
    pub sku_part_number: String,
    pub subscription_ids: Vec<SubscriptionId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TenantLicenseAppliesTo {
    User,
    Company,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TenantLicenseCapabilityStatus {
    Enabled,
    LockedOut,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantLicensePrepaidUnits {
    pub enabled: u32,
    pub locked_out: u32,
    pub suspended: u32,
    pub warning: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantLicenseServicePlan {
    pub applies_to: TenantLicenseAppliesTo,
    pub provisioning_status: TenantLicenseServicePlanProvisioningStatus,
    pub service_plan_id: Uuid,
    /// E.g., "AAD_PREMIUM_P2"
    pub service_plan_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TenantLicenseServicePlanProvisioningStatus {
    Success,
    Disabled,
}
