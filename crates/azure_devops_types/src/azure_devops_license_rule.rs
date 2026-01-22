use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsLicenseRule {
    pub licensing_source: String,
    pub account_license_type: String,
    pub msdn_license_type: String,
    pub git_hub_license_type: String,
    pub license_display_name: String,
    pub status: String,
    pub status_message: String,
    pub assignment_source: String,
}
