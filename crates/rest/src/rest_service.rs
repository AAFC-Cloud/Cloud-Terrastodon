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

#[cfg(test)]
mod tests {
    use super::RestService;
    use reqwest::Url;

    #[test]
    fn infers_microsoft_graph() {
        let url = Url::parse("https://graph.microsoft.com/v1.0/organization").unwrap();
        assert_eq!(RestService::infer(&url), Some(RestService::MicrosoftGraph));
    }

    #[test]
    fn infers_azure_resource_manager() {
        let url = Url::parse("https://management.azure.com/subscriptions?api-version=2020-01-01")
            .unwrap();
        assert_eq!(
            RestService::infer(&url),
            Some(RestService::AzureResourceManager)
        );
    }

    #[test]
    fn infers_azure_devops_hosts() {
        for host in [
            "https://dev.azure.com/example/_apis/projects?api-version=7.1",
            "https://vssps.dev.azure.com/example/_apis/graph/users?api-version=7.1-preview.1",
            "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0",
        ] {
            let url = Url::parse(host).unwrap();
            assert_eq!(RestService::infer(&url), Some(RestService::AzureDevOps));
        }
    }
}
