#[cfg(windows)]
use crate::read_azure_devops_pat_from_credential_manager;
use reqwest::header::HeaderValue;
use std::io::Write;
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
impl AzureDevOpsPersonalAccessToken {
    pub fn into_authorization_header_value(&self) -> HeaderValue {
        let mut buf = b"Basic ".to_vec();
        {
            let username = "";
            let password = self;
            let mut encoder =
                base64::write::EncoderWriter::new(&mut buf, &base64::prelude::BASE64_STANDARD);
            let _ = write!(encoder, "{username}:");
            let _ = write!(encoder, "{password}");
        }
        let mut header = HeaderValue::from_bytes(&buf).expect("base64 is always valid HeaderValue");
        header.set_sensitive(true);
        header
    }
}
impl From<AzureDevOpsPersonalAccessToken> for HeaderValue {
    fn from(value: AzureDevOpsPersonalAccessToken) -> Self {
        value.into_authorization_header_value()
    }
}

pub async fn get_azure_devops_pat() -> eyre::Result<AzureDevOpsPersonalAccessToken> {
    #[cfg(windows)]
    return Ok(read_azure_devops_pat_from_credential_manager()?);
    #[cfg(not(windows))]
    return Ok(AzureDevOpsPersonalAccessToken::new(env!(
        "AZDO_PERSONAL_ACCESS_TOKEN"
    )));
}

#[cfg(test)]
mod test {
    use crate::get_azure_devops_pat;

    #[tokio::test]
    #[ignore]
    pub async fn non_empty() -> eyre::Result<()> {
        let credential = get_azure_devops_pat().await?;
        assert!(!credential.is_empty());
        Ok(())
    }
}
