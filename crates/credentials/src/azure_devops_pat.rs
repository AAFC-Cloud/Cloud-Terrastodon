use crate::AuthBearerExt;
#[cfg(windows)]
use crate::read_azure_devops_pat_from_credential_manager;
use reqwest::header::HeaderValue;
use std::ops::Deref;

#[derive(Debug, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsPersonalAccessToken(String);
impl AzureDevOpsPersonalAccessToken {
    pub fn new(inner: impl Into<String>) -> Self {
        Self(inner.into())
    }
}
impl AsRef<str> for AzureDevOpsPersonalAccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Deref for AzureDevOpsPersonalAccessToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for AzureDevOpsPersonalAccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<AzureDevOpsPersonalAccessToken> for HeaderValue {
    fn from(value: AzureDevOpsPersonalAccessToken) -> Self {
        value.as_authorization_header_value()
    }
}

pub async fn get_azure_devops_personal_access_token_from_credential_manager()
-> eyre::Result<AzureDevOpsPersonalAccessToken> {
    #[cfg(windows)]
    return read_azure_devops_pat_from_credential_manager();
    #[cfg(not(windows))]
    return Ok(AzureDevOpsPersonalAccessToken::new(std::env::var(
        "AZDO_PERSONAL_ACCESS_TOKEN",
    )?));
}

#[cfg(test)]
mod test {
    use crate::get_azure_devops_personal_access_token_from_credential_manager;

    #[tokio::test]
    pub async fn non_empty() -> eyre::Result<()> {
        let credential = get_azure_devops_personal_access_token_from_credential_manager()
            .await
            .ok();
        if let Some(credential) = credential {
            assert!(!credential.is_empty());
        } else {
            println!("No credential found, skipping test");
        }
        Ok(())
    }
}
