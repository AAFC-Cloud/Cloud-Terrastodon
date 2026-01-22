use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsLicenseRule;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops_types::prelude::LastAccessedDate;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1>
#[must_use = "This is an unsent request, you must .await it"]
pub struct AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub user_id: AzureDevOpsUserId,
    pub license_kind: AzureDevOpsLicenseType,
}

pub fn update_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user_id: AzureDevOpsUserId,
    license_kind: AzureDevOpsLicenseType,
) -> AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    AzureDevOpsUserLicenseEntitlementUpdateRequest {
        org_url,
        user_id,
        license_kind,
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    type Output = AzureDevOpsLicenseEntitlementUpdateResponse;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "az",
                "devops",
                self.org_url.organization_name.as_ref(),
                "license",
                "entitlement",
                "update-user",
                &self.user_id.to_string(),
            ]),
            valid_for: Duration::ZERO, // this is an update operation, so no caching
        }
    }
    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            user_id = %self.user_id,
            license_kind = ?self.license_kind,
            "Updating license entitlement for user",
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "user",
            "update",
            "--user",
            &self.user_id.to_string(),
            "--organization",
            &self.org_url.to_string(),
            "--license-type",
            match &self.license_kind {
                AzureDevOpsLicenseType::AccountExpress => "express",
                AzureDevOpsLicenseType::AccountStakeholder => "stakeholder",
                AzureDevOpsLicenseType::AccountAdvanced => "advanced",
                AzureDevOpsLicenseType::MsdnEligible => "professional",
                AzureDevOpsLicenseType::MsdnEnterprise => "professional",
                AzureDevOpsLicenseType::MsdnProfessional => "professional",
                AzureDevOpsLicenseType::Other(s) => s,
            },
        ]);
        cmd.cache(self.cache_key());
        Ok(cmd.run().await?)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'a>, 'a);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsLicenseEntitlementUpdateResponse {
    pub access_level: AzureDevOpsLicenseRule,
    pub date_created: DateTime<Utc>,
    pub extensions: Vec<Value>,
    pub group_assignments: Vec<Value>,
    pub id: AzureDevOpsUserId,
    pub last_accessed_date: LastAccessedDate,
    pub project_entitlements: Vec<Value>,
    pub user: Value,
}