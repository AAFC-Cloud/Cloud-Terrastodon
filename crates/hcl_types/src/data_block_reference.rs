use crate::prelude::AzureRmDataBlockKind;
use crate::prelude::ProviderKind;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DataBlockReference {
    AzureRM {
        kind: AzureRmDataBlockKind,
        name: String,
    },
    Other {
        provider: ProviderKind,
        kind: String,
        name: String,
    },
}
impl DataBlockReference {
    pub fn expression_str(&self) -> String {
        format!("data.{}.{}", self.kind_label(), self.name_label())
    }
    pub fn id_expression_str(&self) -> String {
        format!("{}.id", self.expression_str())
    }
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            DataBlockReference::AzureRM { .. } => ProviderKind::AzureRM,
            DataBlockReference::Other { provider, .. } => provider.to_owned(),
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            DataBlockReference::AzureRM { kind, .. } => kind.as_ref(),
            DataBlockReference::Other { kind, .. } => kind.as_ref(),
        }
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
impl std::fmt::Display for DataBlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression_str())
    }
}
