use std::str::FromStr;

use crate::prelude::AsTofuString;
use crate::providers::TofuProvider;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use hcl::edit::structure::Block;
use indoc::formatdoc;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub enum TofuAzureRMDataKind {
    PolicyDefinition,
    PolicySetDefinition,
    Other(String),
}
impl TofuAzureRMDataKind {
    pub fn supported_variants() -> Vec<TofuAzureRMDataKind> {
        vec![
            TofuAzureRMDataKind::PolicyDefinition,
            TofuAzureRMDataKind::PolicySetDefinition,
        ]
    }
}
impl AsRef<str> for TofuAzureRMDataKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::PolicyDefinition => "policy_definition",
            Self::PolicySetDefinition => "policy_set_definition",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for TofuAzureRMDataKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeking = s.trim_start_matches(TofuProvider::AzureRM.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(anyhow!("no variant matches"))
    }
}

#[derive(Debug, Clone)]
pub enum TofuDataReference {
    AzureRM {
        kind: TofuAzureRMDataKind,
        name: String,
    },
    Other {
        provider: TofuProvider,
        kind: String,
        name: String,
    },
}
impl TofuDataReference {
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
                TofuProvider::AzureRM.provider_prefix(),
                kind.as_ref()
            ),
            Self::Other { provider, kind, .. } => format!("{}_{}", provider, kind),
        }
    }
    pub fn name_label(&self) -> &str {
        match self {
            Self::AzureRM { name, .. } => name.as_str(),
            Self::Other { name, .. } => name.as_str(),
        }
    }
}
impl std::fmt::Display for TofuDataReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression_str())
    }
}

pub enum TofuDataBlock {
    LookupByName {
        reference: TofuDataReference,
        name: String,
    },
    UserLookup {
        label: String,
        user_principal_names: Vec<String>,
    },
}

impl AsTofuString for TofuDataBlock {
    fn as_tofu_string(&self) -> String {
        match self {
            TofuDataBlock::LookupByName { reference, name } => {
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
            TofuDataBlock::UserLookup {
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
                    user_principal_names.iter().map(|x| format!("      \"{}\",", x)).join("\n")
                }
            }
        }
    }
}

impl TryFrom<TofuDataBlock> for Block {
    type Error = anyhow::Error;

    fn try_from(value: TofuDataBlock) -> Result<Self> {
        let body = value
            .as_tofu_string()
            .parse::<hcl::edit::structure::Body>()
            .context("should be valid body")?;
        body.into_blocks()
            .next()
            .ok_or(anyhow!("parsed body should contain the import block"))
    }
}
