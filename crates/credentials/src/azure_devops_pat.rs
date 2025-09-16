use crate::AuthBearerExt;
#[cfg(windows)]
use crate::read_azure_devops_pat_from_credential_manager;
use reqwest::header::HeaderValue;
use std::ops::Deref;

#[derive(Debug)]
pub struct AzureDevOpsPersonalAccessToken {
    inner: String,
}
impl AzureDevOpsPersonalAccessToken {
    pub fn new(inner: impl Into<String>) -> Self {
        Self {
            inner: inner.into(),
        }
    }
}
impl AsRef<str> for AzureDevOpsPersonalAccessToken {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}
impl Deref for AzureDevOpsPersonalAccessToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::fmt::Display for AzureDevOpsPersonalAccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl<'de> serde::Deserialize<'de> for AzureDevOpsPersonalAccessToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}
impl From<AzureDevOpsPersonalAccessToken> for HeaderValue {
    fn from(value: AzureDevOpsPersonalAccessToken) -> Self {
        value.as_authorization_header_value()
    }
}

pub async fn get_azure_devops_pat() -> eyre::Result<AzureDevOpsPersonalAccessToken> {
    #[cfg(windows)]
    return read_azure_devops_pat_from_credential_manager();
    #[cfg(not(windows))]
    return Ok(AzureDevOpsPersonalAccessToken::new(env!(
        "AZDO_PERSONAL_ACCESS_TOKEN"
    )));
}

#[cfg(test)]
mod test {
    use crate::get_azure_devops_pat;

    #[tokio::test]
    pub async fn non_empty() -> eyre::Result<()> {
        let credential = get_azure_devops_pat().await.ok();
        if let Some(credential) = credential {
            assert!(!credential.is_empty());
        } else {
            println!("No credential found, skipping test");
        }
        Ok(())
    }
}
