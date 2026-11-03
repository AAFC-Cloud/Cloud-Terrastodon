use facet::Facet;

/// Sensitive device authorization response. Decoder excerpts must not disclose
/// the device credential or user code through ordinary error diagnostics.
///
/// [Device authorization response](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#device-authorization-response).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct PimDeviceCodeResponse {
    #[facet(sensitive)]
    pub device_code: String,
    #[facet(sensitive)]
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: Option<String>,
}

impl PimDeviceCodeResponse {
    pub(crate) fn decode(body: &str) -> eyre::Result<Self> {
        facet_json::from_str(body).map_err(|_| {
            eyre::eyre!(
                "Microsoft Graph device-code response could not be decoded; response suppressed"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_device_response_does_not_disclose_credentials() {
        let result = PimDeviceCodeResponse::decode(
            r#"{"device_code":"synthetic-secret","expires_in":"invalid"}"#,
        );
        let error = result.err().expect("malformed response must fail");
        assert!(!format!("{error:?}").contains("synthetic-secret"));
    }
}
