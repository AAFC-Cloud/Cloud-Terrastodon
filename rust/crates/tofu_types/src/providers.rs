#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderReference {
    Alias {
        kind: TofuProviderKind,
        name: String,
    },
    Default {
        kind: Option<TofuProviderKind>,
    },
}
impl TofuProviderReference {
    pub fn kind(&self) -> Option<&TofuProviderKind> {
        match self {
            TofuProviderReference::Alias { kind, .. } => Some(kind),
            TofuProviderReference::Default { kind } => kind.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderKind {
    AzureRM,
    AzureAD,
    Other(String),
}
impl TofuProviderKind {
    pub fn provider_prefix(&self) -> &str {
        match self {
            TofuProviderKind::AzureRM => "azurerm",
            TofuProviderKind::AzureAD => "azuread",
            TofuProviderKind::Other(s) => s.as_str(),
        }
    }
}
impl std::fmt::Display for TofuProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}
