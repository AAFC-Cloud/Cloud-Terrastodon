use reqwest::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestService {
    AzureDevOps,
    MicrosoftGraph,
    AzureResourceManager,
}

impl RestService {
    pub fn infer(url: &Url) -> Option<Self> {
        let host = url.host_str()?.to_ascii_lowercase();
        match host.as_str() {
            "graph.microsoft.com" => Some(Self::MicrosoftGraph),
            "management.azure.com" => Some(Self::AzureResourceManager),
            "dev.azure.com"
            | "vssps.dev.azure.com"
            | "vsrm.dev.azure.com"
            | "vsaex.dev.azure.com"
            | "app.vssps.visualstudio.com" => Some(Self::AzureDevOps),
            _ => None,
        }
    }
}
