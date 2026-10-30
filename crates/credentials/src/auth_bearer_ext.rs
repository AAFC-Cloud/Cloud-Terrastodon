use crate::AzureBearerToken;
use crate::AzureDevOpsPersonalAccessToken;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use cloud_terrastodon_azure_types::AzureAccessToken;
use eyre::Result;
use eyre::WrapErr;
use reqwest::header::HeaderValue;

pub trait AuthBearerExt {
    fn as_authorization_header_value(&self) -> Result<HeaderValue>;
}
impl AuthBearerExt for str {
    fn as_authorization_header_value(&self) -> Result<HeaderValue> {
        let encoded = BASE64_STANDARD.encode(format!(":{self}"));
        let mut header = HeaderValue::from_str(&format!("Basic {encoded}"))
            .wrap_err("Encoding Basic authorization header")?;
        header.set_sensitive(true);
        Ok(header)
    }
}
impl AuthBearerExt for String {
    fn as_authorization_header_value(&self) -> Result<HeaderValue> {
        self.as_str().as_authorization_header_value()
    }
}
impl<T: AsRef<str>> AuthBearerExt for AzureAccessToken<T> {
    fn as_authorization_header_value(&self) -> Result<HeaderValue> {
        bearer_header(self.access_token.as_ref())
    }
}
impl AuthBearerExt for AzureDevOpsPersonalAccessToken {
    fn as_authorization_header_value(&self) -> Result<HeaderValue> {
        self.as_str().as_authorization_header_value()
    }
}
impl AuthBearerExt for AzureBearerToken {
    fn as_authorization_header_value(&self) -> Result<HeaderValue> {
        bearer_header(self.as_str())
    }
}

fn bearer_header(token: &str) -> Result<HeaderValue> {
    let mut header = HeaderValue::from_str(&format!("Bearer {token}"))
        .wrap_err("Access token contains invalid HTTP header bytes")?;
    header.set_sensitive(true);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureBearerToken;
    use crate::AzureDevOpsPersonalAccessToken;

    #[test]
    fn bearer_and_pat_headers_are_distinct() {
        let bearer = AzureBearerToken::new("access-token")
            .as_authorization_header_value()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(bearer, "Bearer access-token");

        let pat = AzureDevOpsPersonalAccessToken::new("pat-token")
            .as_authorization_header_value()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        assert!(pat.starts_with("Basic "));
        assert_ne!(bearer, pat);
    }

    #[test]
    fn invalid_bearer_header_bytes_return_an_error() {
        let token = AzureBearerToken::new("synthetic\r\ninvalid");
        let error = token.as_authorization_header_value().unwrap_err();
        assert!(error.to_string().contains("invalid HTTP header bytes"));
        assert!(!format!("{error:?}").contains("synthetic"));
    }
}
