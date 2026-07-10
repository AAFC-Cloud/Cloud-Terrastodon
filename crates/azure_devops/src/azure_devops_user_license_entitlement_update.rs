use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseRule;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops_types::LastAccessedDate;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

/// <https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1>
#[must_use = "This is an unsent request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub user: AzureDevOpsUserArgument<'a>,
    pub license_kind: AzureDevOpsLicenseType,
}

pub fn update_azure_devops_user_license_entitlement<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    user: impl Into<AzureDevOpsUserArgument<'a>>,
    license_kind: AzureDevOpsLicenseType,
) -> AzureDevOpsUserLicenseEntitlementUpdateRequest<'a> {
    AzureDevOpsUserLicenseEntitlementUpdateRequest {
        org_url: Cow::Borrowed(org_url),
        user: user.into(),
        license_kind,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            user: AzureDevOpsUserArgument::arbitrary(u)?.into_owned(),
            license_kind: AzureDevOpsLicenseType::arbitrary(u)?,
        })
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
                &self.user.to_string(),
            ]),
            valid_for: Duration::ZERO, // this is an update operation, so no caching
        }
    }
    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            user = %self.user,
            license_kind = ?self.license_kind,
            "Updating license entitlement for user",
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "user",
            "update",
            "--user",
            &self.user.to_string(),
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
                AzureDevOpsLicenseType::None => "none",
            },
        ]);
        cmd.cache(self.cache_key());
        Ok(cmd.run().await?)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementUpdateRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsUserLicenseEntitlementUpdateRequest<'static> => AzureDevOpsLicenseEntitlementUpdateResponse, effects = [Write]);

#[derive(facet::Facet, Debug)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsLicenseEntitlementUpdateResponse {
    pub access_level: AzureDevOpsLicenseRule,
    pub date_created: DateTime<Utc>,
    pub extensions: Vec<RawJson<'static>>,
    pub group_assignments: Vec<RawJson<'static>>,
    pub id: AzureDevOpsUserId,
    pub last_accessed_date: LastAccessedDate,
    pub project_entitlements: Vec<RawJson<'static>>,
    pub user: RawJson<'static>,
}
use arbitrary::Arbitrary;
