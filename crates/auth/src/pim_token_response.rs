use facet::Facet;

/// Sensitive OAuth response used while polling a PIM device authorization.
///
/// [Device authorization token response](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code#successful-authentication-response).
/// Extended lifetime fields follow the supplemental
/// [MSAL response model](https://azuread.github.io/microsoft-authentication-library-for-js/ref/types/_azure_msal_common.ServerAuthorizationTokenResponse.html).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct PimTokenResponse {
    #[facet(sensitive)]
    pub access_token: Option<String>,
    #[facet(sensitive)]
    pub refresh_token: Option<String>,
    #[facet(sensitive)]
    pub id_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_in: Option<u64>,
    pub ext_expires_in: Option<u64>,
    pub refresh_in: Option<u64>,
    pub error: Option<String>,
    #[facet(sensitive)]
    pub error_description: Option<String>,
    pub error_codes: Option<Vec<u64>>,
    pub timestamp: Option<String>,
    pub trace_id: Option<String>,
    pub correlation_id: Option<String>,
    pub suberror: Option<String>,
}

impl PimTokenResponse {
    pub(crate) fn decode(body: &str) -> eyre::Result<Self> {
        facet_json::from_str(body).map_err(|_| {
            eyre::eyre!("Microsoft Graph token response could not be decoded; response suppressed")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_token_response_does_not_disclose_credentials() {
        let result = PimTokenResponse::decode(r#"{"access_token":"synthetic-secret","error":{}}"#);
        let error = result.err().expect("malformed response must fail");
        assert!(!format!("{error:?}").contains("synthetic-secret"));
    }
}
