use crate::AzureDevOpsAccountLicenseType;
use crate::AzureDevOpsLicenseType;
use crate::AzureDevOpsMsdnLicenseType;

/// An access-level change with a matching licensing source and license family.
/// See [AccessLevel 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#accesslevel).
#[derive(Debug, Clone, facet::Facet)]
#[facet(tag = "licensingSource", rename_all = "camelCase")]
#[repr(u8)]
pub enum AzureDevOpsLicenseAccessLevelUpdate {
    Account {
        #[facet(rename = "accountLicenseType")]
        license: AzureDevOpsAccountLicenseType,
    },
    Msdn {
        #[facet(rename = "msdnLicenseType")]
        license: AzureDevOpsMsdnLicenseType,
    },
}

impl TryFrom<&AzureDevOpsLicenseType> for AzureDevOpsLicenseAccessLevelUpdate {
    type Error = eyre::Report;

    fn try_from(value: &AzureDevOpsLicenseType) -> Result<Self, Self::Error> {
        Ok(match value {
            AzureDevOpsLicenseType::Other(value)
            | AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Other(value))
            | AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Other(value)) => {
                eyre::bail!("unsupported license update: {value}");
            }
            AzureDevOpsLicenseType::Account(license) => Self::Account {
                license: license.clone(),
            },
            AzureDevOpsLicenseType::Msdn(license) => Self::Msdn {
                license: license.clone(),
            },
        })
    }
}
