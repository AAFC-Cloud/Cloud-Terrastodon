use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use http::HeaderValue;
use std::fmt;

/// Sensitive authorization for the [BMC Helix AR System REST API](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/AR-System/ars213/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/Access-and-authentication-for-the-REST-API/).
///
/// The token travels only through credential storage and the transport boundary.
#[derive(Clone)]
pub struct BmcHelixAuthorization {
    header: HeaderValue,
}

impl BmcHelixAuthorization {
    pub fn new(token: &str) -> Result<Self> {
        ensure!(
            !token.is_empty() && !token.bytes().any(|byte| byte.is_ascii_whitespace()),
            "Helix returned an empty or malformed authorization token"
        );
        let mut header = HeaderValue::from_str(&format!("AR-JWT {token}"))
            .wrap_err("Helix token cannot be represented as an authorization header")?;
        header.set_sensitive(true);
        Ok(Self { header })
    }

    pub fn header(&self) -> &HeaderValue {
        &self.header
    }
}

impl fmt::Debug for BmcHelixAuthorization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BmcHelixAuthorization([REDACTED])")
    }
}
