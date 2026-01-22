use crate::prelude::AzureDevOpsLicenseType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsLicenseRule {
    pub licensing_source: AzureDevOpsLicenseRuleSource,
    pub account_license_type: AzureDevOpsLicenseType,
    pub msdn_license_type: AzureDevOpsLicenseRuleMsdnLicenseType,
    pub git_hub_license_type: AzureDevOpsLicenseRuleGitHubLicenseType,
    pub license_display_name: String,
    pub status: AzureDevOpsLicenseRuleStatus,
    pub status_message: String,
    pub assignment_source: AzureDevOpsLicenseRuleAssignmentSource,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseRuleSource {
    Account,
    Msdn,
    #[serde(untagged)]
    Other(String),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseRuleMsdnLicenseType {
    None,
    Eligible,
    #[serde(untagged)]
    Other(String),
}
#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseRuleGitHubLicenseType {
    None,
    #[serde(untagged)]
    Other(String),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseRuleStatus {
    Active,
    #[serde(untagged)]
    Other(String),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseRuleAssignmentSource {
    Unknown,
    GroupRule,
    #[serde(untagged)]
    Other(String),
}

#[cfg(test)]
mod test {
    use crate::prelude::AzureDevOpsLicenseRule;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json = r#"
        {
            "licensingSource": "account",
            "accountLicenseType": "express",
            "msdnLicenseType": "none",
            "gitHubLicenseType": "none",
            "licenseDisplayName": "Basic",
            "status": "active",
            "statusMessage": "",
            "assignmentSource": "unknown"
        }"#;
        let rule: AzureDevOpsLicenseRule = serde_json::from_str(json)?;
        println!("Deserialized rule: {:#?}", rule);

        Ok(())
    }
}

/*
[
  {
    "licensingSource": "account",
    "accountLicenseType": "express",
    "msdnLicenseType": "none",
    "gitHubLicenseType": "none",
    "licenseDisplayName": "Basic",
    "status": "active",
    "statusMessage": "",
    "assignmentSource": "unknown"
  },
  {
    "licensingSource": "account",
    "accountLicenseType": "stakeholder",
    "msdnLicenseType": "none",
    "gitHubLicenseType": "none",
    "licenseDisplayName": "Stakeholder",
    "status": "active",
    "statusMessage": "",
    "assignmentSource": "unknown"
  },
  {
    "licensingSource": "account",
    "accountLicenseType": "advanced",
    "msdnLicenseType": "none",
    "gitHubLicenseType": "none",
    "licenseDisplayName": "Basic + Test Plans",
    "status": "active",
    "statusMessage": "",
    "assignmentSource": "unknown"
  },
  {
    "licensingSource": "msdn",
    "accountLicenseType": "none",
    "msdnLicenseType": "eligible",
    "gitHubLicenseType": "none",
    "licenseDisplayName": "Visual Studio Subscriber",
    "status": "active",
    "statusMessage": "",
    "assignmentSource": "unknown"
  },
  {
    "licensingSource": "account",
    "accountLicenseType": "stakeholder",
    "msdnLicenseType": "none",
    "gitHubLicenseType": "none",
    "licenseDisplayName": "Stakeholder",
    "status": "active",
    "statusMessage": "",
    "assignmentSource": "unknown"
  }
]
*/
