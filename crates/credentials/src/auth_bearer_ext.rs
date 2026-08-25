use crate::AzureBearerToken;
use crate::AzureDevOpsPersonalAccessToken;
use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use cloud_terrastodon_azure_types::AzureAccessToken;
use reqwest::header::HeaderValue;
use std::io::Write;

pub trait AuthBearerExt {
    fn as_authorization_header_value(&self) -> HeaderValue;
}
impl AuthBearerExt for str {
    fn as_authorization_header_value(&self) -> HeaderValue {
        let mut buf = b"Basic ".to_vec();
        {
            let username = "";
            let password = self;
            let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
            encoder
                .write_fmt(format_args!("{username}:{password}"))
                .unwrap();
        }
        let mut header = HeaderValue::from_bytes(&buf).expect("base64 is always valid HeaderValue");
        header.set_sensitive(true);
        header
    }
}
impl AuthBearerExt for String {
    fn as_authorization_header_value(&self) -> HeaderValue {
        self.as_str().as_authorization_header_value()
    }
}
impl<T: AsRef<str>> AuthBearerExt for AzureAccessToken<T> {
    fn as_authorization_header_value(&self) -> HeaderValue {
        bearer_header(self.access_token.as_ref())
    }
}
impl AuthBearerExt for AzureDevOpsPersonalAccessToken {
    fn as_authorization_header_value(&self) -> HeaderValue {
        self.as_str().as_authorization_header_value()
    }
}
impl AuthBearerExt for AzureBearerToken {
    fn as_authorization_header_value(&self) -> HeaderValue {
        bearer_header(self.as_str())
    }
}

fn bearer_header(token: &str) -> HeaderValue {
    let mut header = HeaderValue::from_str(&format!("Bearer {token}"))
        .expect("access tokens are expected to contain valid HTTP header bytes");
    header.set_sensitive(true);
    header
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
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(bearer, "Bearer access-token");

        let pat = AzureDevOpsPersonalAccessToken::new("pat-token")
            .as_authorization_header_value()
            .to_str()
            .unwrap()
            .to_owned();
        assert!(pat.starts_with("Basic "));
        assert_ne!(bearer, pat);
    }
}
