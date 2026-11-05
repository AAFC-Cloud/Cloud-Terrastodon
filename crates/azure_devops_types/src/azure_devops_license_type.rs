use crate::AzureDevOpsAccountLicenseType;
use crate::AzureDevOpsMsdnLicenseType;
use arbitrary::Arbitrary;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// A license identified by its family and kind.
///
/// Shares the account and MSDN vocabularies from [AccessLevel 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#accesslevel).
/// The combined entitlement/CLI representation remains `Account-Express`,
/// `Msdn-Enterprise`, etc.; individual REST fields use the family's own spelling.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseType {
    Account(AzureDevOpsAccountLicenseType),
    Msdn(AzureDevOpsMsdnLicenseType),
    /// Preserve opaque combined identifiers from other licensing sources.
    /// This combined vocabulary is open; it is not an account/MSDN REST value.
    #[arbitrary(skip)]
    Other(String),
}

impl AzureDevOpsLicenseType {
    /// The existing interactive choices, rather than every historical service value.
    pub const VARIANTS: &'static [Self] = &[
        Self::Account(AzureDevOpsAccountLicenseType::Express),
        Self::Account(AzureDevOpsAccountLicenseType::Stakeholder),
        Self::Account(AzureDevOpsAccountLicenseType::Advanced),
        Self::Msdn(AzureDevOpsMsdnLicenseType::Eligible),
        Self::Msdn(AzureDevOpsMsdnLicenseType::Enterprise),
        Self::Msdn(AzureDevOpsMsdnLicenseType::Professional),
    ];

    /// Existing estimate for the two priced account tiers; unpriced kinds
    /// contribute zero to the estimate, which does not establish that they are free.
    /// See <https://azure.microsoft.com/en-us/pricing/details/devops/azure-devops-services/>.
    pub fn cost_per_month_cad(&self) -> f64 {
        match self {
            Self::Account(AzureDevOpsAccountLicenseType::Express) => 8.30,
            Self::Account(AzureDevOpsAccountLicenseType::Advanced) => 71.93,
            _ => 0.0,
        }
    }
}

impl FromStr for AzureDevOpsLicenseType {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Shorthand belongs to the combined selection, never to a family's REST parser.
        match value {
            "Basic" | "basic" | "express" => {
                return Ok(Self::Account(AzureDevOpsAccountLicenseType::Express));
            }
            "Stakeholder" | "stakeholder" => {
                return Ok(Self::Account(AzureDevOpsAccountLicenseType::Stakeholder));
            }
            "Advanced" | "advanced" | "test" | "Test" => {
                return Ok(Self::Account(AzureDevOpsAccountLicenseType::Advanced));
            }
            "None" | "none" => return Ok(Self::Account(AzureDevOpsAccountLicenseType::None)),
            _ => {}
        }
        if let Some((family, spelling)) = value.split_once('-') {
            let mut name = spelling.to_owned();
            if let Some(first) = name.get_mut(..1) {
                first.make_ascii_lowercase();
            }
            match family {
                "Account" => {
                    return Ok(Self::Account(name.parse().unwrap_or_else(|_| {
                        AzureDevOpsAccountLicenseType::Other(spelling.to_owned())
                    })));
                }
                "Msdn" => {
                    return Ok(Self::Msdn(name.parse().unwrap_or_else(|_| {
                        AzureDevOpsMsdnLicenseType::Other(spelling.to_owned())
                    })));
                }
                _ => {}
            }
        }
        Ok(Self::Other(value.to_owned()))
    }
}

impl From<&AzureDevOpsLicenseType> for String {
    fn from(value: &AzureDevOpsLicenseType) -> Self {
        let (family, name) = match value {
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::None) => {
                return "None".to_owned();
            }
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Other(value)) => {
                return format!("Account-{value}");
            }
            AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Other(value)) => {
                return format!("Msdn-{value}");
            }
            AzureDevOpsLicenseType::Account(license) => ("Account", license.as_str()),
            AzureDevOpsLicenseType::Msdn(license) => ("Msdn", license.as_str()),
            AzureDevOpsLicenseType::Other(value) => return value.clone(),
        };
        let mut name = name.to_owned();
        if let Some(first) = name.get_mut(..1) {
            first.make_ascii_uppercase();
        }
        format!("{family}-{name}")
    }
}

impl TryFrom<String> for AzureDevOpsLicenseType {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Display for AzureDevOpsLicenseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from(self))?;
        match self {
            Self::Account(AzureDevOpsAccountLicenseType::Express) => f.write_str(" (Basic)"),
            Self::Account(AzureDevOpsAccountLicenseType::Advanced) => f.write_str(" (Basic+Test)"),
            _ => Ok(()),
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsLicenseType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsLicenseType);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureDevOpsLicenseAccessLevelUpdate;

    #[test]
    fn aliases_share_identity_and_update_family() -> eyre::Result<()> {
        let basic = "Basic".parse::<AzureDevOpsLicenseType>()?;
        let canonical = "Account-Express".parse::<AzureDevOpsLicenseType>()?;
        assert_eq!(basic, canonical);
        assert_eq!(String::from(&basic), "Account-Express");
        assert_eq!(basic.to_string(), "Account-Express (Basic)");
        assert!(matches!(
            AzureDevOpsLicenseAccessLevelUpdate::try_from(&basic)?,
            AzureDevOpsLicenseAccessLevelUpdate::Account {
                license: AzureDevOpsAccountLicenseType::Express
            }
        ));
        // Combined selection syntax must not leak into the account REST field.
        assert!("Basic".parse::<AzureDevOpsAccountLicenseType>().is_err());
        assert!(
            "Msdn-Professional"
                .parse::<AzureDevOpsAccountLicenseType>()
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn same_named_licenses_keep_distinct_families() -> eyre::Result<()> {
        let account = "Account-Professional".parse::<AzureDevOpsLicenseType>()?;
        let msdn = "Msdn-Professional".parse::<AzureDevOpsLicenseType>()?;
        assert_ne!(account, msdn);
        assert!(matches!(
            AzureDevOpsLicenseAccessLevelUpdate::try_from(&account)?,
            AzureDevOpsLicenseAccessLevelUpdate::Account {
                license: AzureDevOpsAccountLicenseType::Professional
            }
        ));
        assert!(matches!(
            AzureDevOpsLicenseAccessLevelUpdate::try_from(&msdn)?,
            AzureDevOpsLicenseAccessLevelUpdate::Msdn {
                license: AzureDevOpsMsdnLicenseType::Professional
            }
        ));
        Ok(())
    }

    #[test]
    fn selectable_licenses_preserve_combined_wire_identity() -> eyre::Result<()> {
        for license in AzureDevOpsLicenseType::VARIANTS {
            let text = facet_json::to_string(license)?;
            assert_eq!(
                facet_json::from_str::<AzureDevOpsLicenseType>(&text)?,
                *license
            );
            AzureDevOpsLicenseAccessLevelUpdate::try_from(license)?;
        }
        let none = "None".parse::<AzureDevOpsLicenseType>()?;
        assert_eq!(none, "Account-None".parse()?);
        assert_eq!(String::from(&none), "None");
        Ok(())
    }

    #[test]
    fn unknown_licenses_are_observable_but_not_mutation_selections() -> eyre::Result<()> {
        let unknown = "Custom-License".parse::<AzureDevOpsLicenseType>()?;
        assert_eq!(String::from(&unknown), "Custom-License");
        for license in [
            unknown,
            AzureDevOpsLicenseType::Account(AzureDevOpsAccountLicenseType::Other(
                "futureTier".into(),
            )),
            AzureDevOpsLicenseType::Msdn(AzureDevOpsMsdnLicenseType::Other("FutureTier".into())),
        ] {
            assert_eq!(
                String::from(&license).parse::<AzureDevOpsLicenseType>()?,
                license
            );
            assert!(AzureDevOpsLicenseAccessLevelUpdate::try_from(&license).is_err());
        }
        Ok(())
    }
}
