use crate::AzureDevOpsLicenseType;
use arbitrary::Arbitrary;
use std::str::FromStr;

#[derive(Clone, Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
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

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleSource {
    Account,
    Msdn,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleMsdnLicenseType {
    None,
    Eligible,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleGitHubLicenseType {
    None,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseRuleStatus {
    Active,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
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
    AzureDevOpsLicenseRuleMsdnLicenseType,
    {
        "none" => Self::None,
        "eligible" => Self::Eligible,
    },
    value => match value {
        AzureDevOpsLicenseRuleMsdnLicenseType::None => "none".to_owned(),
        AzureDevOpsLicenseRuleMsdnLicenseType::Eligible => "eligible".to_owned(),
        AzureDevOpsLicenseRuleMsdnLicenseType::Other(value) => value.clone(),
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
    use crate::AzureDevOpsLicenseRule;

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
