use crate::AzureDevOpsAccountLicenseType;
use crate::AzureDevOpsLicenseType;
use crate::AzureDevOpsMsdnLicenseType;
use arbitrary::Arbitrary;
use facet::Facet;
use std::str::FromStr;

/// License information from [AccessLevel 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#accesslevel).
#[derive(Clone, Debug, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsLicenseRule {
    pub licensing_source: AzureDevOpsLicenseRuleSource,
    pub account_license_type: AzureDevOpsAccountLicenseType,
    pub msdn_license_type: AzureDevOpsMsdnLicenseType,
    /// Omitted by the documented 7.1 AccessLevel response.
    pub git_hub_license_type: Option<AzureDevOpsLicenseRuleGitHubLicenseType>,
    pub license_display_name: String,
    pub status: AzureDevOpsLicenseRuleStatus,
    pub status_message: String,
    pub assignment_source: AzureDevOpsLicenseRuleAssignmentSource,
}

impl AzureDevOpsLicenseRule {
    /// Whether this rule grants the selected license through its active source.
    /// An inactive family field must never identify a group for license revocation.
    pub fn matches_license(&self, license: &AzureDevOpsLicenseType) -> bool {
        match (&self.licensing_source, license) {
            (AzureDevOpsLicenseRuleSource::Account, AzureDevOpsLicenseType::Account(license)) => {
                &self.account_license_type == license
            }
            (AzureDevOpsLicenseRuleSource::Msdn, AzureDevOpsLicenseType::Msdn(license)) => {
                &self.msdn_license_type == license
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleSource {
    Account,
    Msdn,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleGitHubLicenseType {
    None,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleStatus {
    Active,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleAssignmentSource {
    Unknown,
    GroupRule,
    Other(String),
}

macro_rules! impl_string_backed_enum {
    ($ty:ty, {$($wire:literal => $variant:expr,)*}, $value:ident => $serialized:expr) => {
        impl FromStr for $ty {
            type Err = eyre::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(match value {
                    $($wire => $variant,)*
                    other => Self::Other(other.to_owned()),
                })
            }
        }

        impl From<&$ty> for String {
            fn from($value: &$ty) -> Self {
                $serialized
            }
        }

        impl TryFrom<String> for $ty {
            type Error = eyre::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }
    };
}

impl_string_backed_enum!(
    AzureDevOpsLicenseRuleSource,
    {
        "account" => Self::Account,
        "msdn" => Self::Msdn,
    },
    value => match value {
        AzureDevOpsLicenseRuleSource::Account => "account".to_owned(),
        AzureDevOpsLicenseRuleSource::Msdn => "msdn".to_owned(),
        AzureDevOpsLicenseRuleSource::Other(value) => value.clone(),
    }
);

impl_string_backed_enum!(
    AzureDevOpsLicenseRuleGitHubLicenseType,
    {
        "none" => Self::None,
    },
    value => match value {
        AzureDevOpsLicenseRuleGitHubLicenseType::None => "none".to_owned(),
        AzureDevOpsLicenseRuleGitHubLicenseType::Other(value) => value.clone(),
    }
);

impl_string_backed_enum!(
    AzureDevOpsLicenseRuleStatus,
    {
        "active" => Self::Active,
    },
    value => match value {
        AzureDevOpsLicenseRuleStatus::Active => "active".to_owned(),
        AzureDevOpsLicenseRuleStatus::Other(value) => value.clone(),
    }
);

impl_string_backed_enum!(
    AzureDevOpsLicenseRuleAssignmentSource,
    {
        "unknown" => Self::Unknown,
        "groupRule" => Self::GroupRule,
    },
    value => match value {
        AzureDevOpsLicenseRuleAssignmentSource::Unknown => "unknown".to_owned(),
        AzureDevOpsLicenseRuleAssignmentSource::GroupRule => "groupRule".to_owned(),
        AzureDevOpsLicenseRuleAssignmentSource::Other(value) => value.clone(),
    }
);

cloud_terrastodon_registry::register_thing!(AzureDevOpsLicenseRule);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsLicenseRule);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matching_uses_the_active_license_family() {
        // Responses carry both family fields. Only licensingSource identifies
        // which one grants the group's license, even when names coincide.
        let mut rule = AzureDevOpsLicenseRule {
            licensing_source: AzureDevOpsLicenseRuleSource::Account,
            account_license_type: AzureDevOpsAccountLicenseType::Professional,
            msdn_license_type: AzureDevOpsMsdnLicenseType::Professional,
            git_hub_license_type: None,
            license_display_name: "Professional".into(),
            status: AzureDevOpsLicenseRuleStatus::Active,
            status_message: String::new(),
            assignment_source: AzureDevOpsLicenseRuleAssignmentSource::GroupRule,
        };
        let account = AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Professional);
        let msdn = AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Professional);
        assert!(rule.matches_license(&account));
        assert!(!rule.matches_license(&msdn));

        rule.licensing_source = AzureDevOpsLicenseRuleSource::Msdn;
        assert!(!rule.matches_license(&account));
        assert!(rule.matches_license(&msdn));

        rule.licensing_source = AzureDevOpsLicenseRuleSource::Other("future".into());
        assert!(!rule.matches_license(&account));
        assert!(!rule.matches_license(&msdn));
    }

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
        let rule: AzureDevOpsLicenseRule = facet_json::from_str(json)?;
        assert_eq!(rule.license_display_name, "Basic");
        assert!(rule.status_message.is_empty());

        Ok(())
    }
}
