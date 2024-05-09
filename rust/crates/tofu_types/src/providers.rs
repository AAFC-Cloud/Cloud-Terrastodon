#[derive(Debug, Clone)]
pub enum TofuProvider {
    AzureRM,
    AzureAD,
    Other(String),
}
impl TofuProvider {
    pub fn provider_prefix(&self) -> &str {
        match self {
            TofuProvider::AzureRM => "azurerm",
            TofuProvider::AzureAD => "azuread",
            TofuProvider::Other(s) => s.as_str(),
        }
    }
}
impl std::fmt::Display for TofuProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}
