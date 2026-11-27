use cloud_terrastodon_azure_devops::AzureDevOpsAccountLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseRuleAssignmentSource;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseRuleStatus;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsMsdnLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use facet::Facet;
use std::borrow::Cow;

/// Display values and the stable entitlement ID for a row in the Users table.
#[derive(Facet)]
pub(crate) struct AzureDevOpsOrganizationUserRow {
    pub id: AzureDevOpsUserId,
    pub name: String,
    pub sign_in: String,
    pub license: String,
    pub assignment_source: String,
    pub status: String,
    pub action: &'static str,
}

impl AzureDevOpsOrganizationUserRow {
    pub fn new(entitlement: &AzureDevOpsUserLicenseEntitlement, demo_mode: bool) -> Self {
        let user = &entitlement.user;
        let name = user.display_name.as_str();
        let access_level = &entitlement.access_level;
        Self {
            id: entitlement.id,
            name: if demo_mode && name.contains('@') {
                "Redacted user".to_owned()
            } else {
                name.to_owned()
            },
            sign_in: if demo_mode {
                "redacted@redacted.ca".to_owned()
            } else {
                user.mail_address
                    .as_deref()
                    .filter(|address| !address.is_empty())
                    .unwrap_or(&user.principal_name)
                    .to_owned()
            },
            license: if access_level.license_display_name.is_empty() {
                license_label(&entitlement.license()).into_owned()
            } else {
                access_level.license_display_name.clone()
            },
            assignment_source: match &access_level.assignment_source {
                AzureDevOpsLicenseRuleAssignmentSource::GroupRule => "Group rule".to_owned(),
                AzureDevOpsLicenseRuleAssignmentSource::Unknown => "Unknown".to_owned(),
                AzureDevOpsLicenseRuleAssignmentSource::None => "None".to_owned(),
                AzureDevOpsLicenseRuleAssignmentSource::Other(source) if source == "direct" => {
                    "Direct".to_owned()
                }
                AzureDevOpsLicenseRuleAssignmentSource::Other(source) => source.clone(),
            },
            status: match &access_level.status {
                AzureDevOpsLicenseRuleStatus::Active => "Active".to_owned(),
                AzureDevOpsLicenseRuleStatus::Other(status) => status.clone(),
            },
            action: "Change access level",
        }
    }
}

pub(crate) fn license_label(license: &AzureDevOpsLicenseType) -> Cow<'static, str> {
    Cow::Borrowed(match license {
        AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::None) => "None",
        AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Express) => "Basic",
        AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Advanced) => {
            "Basic + Test Plans"
        }
        AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Stakeholder) => {
            "Stakeholder"
        }
        AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Eligible) => {
            "Visual Studio Subscriber"
        }
        AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Enterprise) => {
            "Visual Studio Enterprise"
        }
        AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Professional) => {
            "Visual Studio Professional"
        }
        license => return Cow::Owned(license.to_string()),
    })
}
