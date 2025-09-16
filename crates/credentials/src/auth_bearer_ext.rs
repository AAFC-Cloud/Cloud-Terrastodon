use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use cloud_terrastodon_azure_types::prelude::AccessToken;
use reqwest::header::HeaderValue;
use std::io::Write;

use crate::AzureDevOpsPersonalAccessToken;

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
            encoder.write_fmt(format_args!("{username}:{password}")).unwrap();
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
impl<T: AsRef<str>> AuthBearerExt for AccessToken<T> {
    fn as_authorization_header_value(&self) -> HeaderValue {
        self.access_token.as_ref().as_authorization_header_value()
    }
}
impl AuthBearerExt for AzureDevOpsPersonalAccessToken {
    fn as_authorization_header_value(&self) -> HeaderValue {
        self.as_str().as_authorization_header_value()
    }
}