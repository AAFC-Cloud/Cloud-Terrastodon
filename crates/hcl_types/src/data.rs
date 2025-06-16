use crate::prelude::AsHCLString;
use crate::providers::ProviderKind;
use eyre::Result;
use eyre::eyre;
use indoc::formatdoc;
use itertools::Itertools;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum AzureRMProviderDataBlockKind {
    PolicyDefinition,
    PolicySetDefinition,
    ResourceGroup,
    Other(String),
}
impl AzureRMProviderDataBlockKind {
    pub fn supported_variants() -> Vec<AzureRMProviderDataBlockKind> {
        vec![
            AzureRMProviderDataBlockKind::PolicyDefinition,
            AzureRMProviderDataBlockKind::PolicySetDefinition,
            AzureRMProviderDataBlockKind::ResourceGroup,
        ]
    }
}
impl AsRef<str> for AzureRMProviderDataBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::PolicyDefinition => "policy_definition",
            Self::PolicySetDefinition => "policy_set_definition",
            Self::ResourceGroup => "resource_group",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureRMProviderDataBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeking = s.trim_start_matches(ProviderKind::AzureRM.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(eyre!("no variant matches"))
    }
}

#[derive(Debug, Clone)]
pub enum HCLDataBlockReference {
    AzureRM {
        kind: AzureRMProviderDataBlockKind,
        name: String,
    },
    Other {
        provider: ProviderKind,
        kind: String,
        name: String,
    },
}
impl HCLDataBlockReference {
    pub fn expression_str(&self) -> String {
        format!("data.{}.{}", self.kind_label(), self.name_label())
    }
    pub fn id_expression_str(&self) -> String {
        format!("{}.id", self.expression_str())
    }
    pub fn kind_label(&self) -> String {
        match self {
            Self::AzureRM { kind, .. } => format!(
                "{}_{}",
                ProviderKind::AzureRM.provider_prefix(),
                kind.as_ref()
            ),
            Self::Other { provider, kind, .. } => format!("{provider}_{kind}"),
        }
    }
    pub fn name_label(&self) -> &str {
        match self {
            Self::AzureRM { name, .. } => name.as_str(),
            Self::Other { name, .. } => name.as_str(),
        }
    }
}
impl std::fmt::Display for HCLDataBlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression_str())
    }
}

pub enum HCLDataBlock {
    LookupByName {
        reference: HCLDataBlockReference,
        name: String,
    },
    UserLookup {
        label: String,
        user_principal_names: Vec<String>,
    },
}

impl AsHCLString for HCLDataBlock {
    fn as_hcl_string(&self) -> String {
        match self {
            HCLDataBlock::LookupByName { reference, name } => {
                let ref_kind = reference.kind_label();
                let ref_name = reference.name_label();
                formatdoc! {
                    r#"
                        data "{}" "{}" {{
                            name = "{}"
                        }}
                    "#,
                    ref_kind,
                    ref_name,
                    name
                }
            }
            HCLDataBlock::UserLookup {
                label,
                user_principal_names,
            } => {
                formatdoc! {
                    r#"
                        data "azuread_users" "{}" {{
                            user_principal_names = [
                                {}
                            ]
                        }}
                    "#,
                    label,
                    user_principal_names.iter().map(|x| format!("      \"{x}\",")).join("\n")
                }
            }
        }
    }
}
