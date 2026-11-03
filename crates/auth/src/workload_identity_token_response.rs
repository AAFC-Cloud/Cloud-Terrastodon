use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use reqwest::StatusCode;

/// Private OAuth exchange response. Its body and decoder diagnostics are
/// sensitive because either can contain the assertion or newly issued token.
///
/// [Client credentials response](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-client-creds-grant-flow#successful-response).
/// Extended lifetime fields follow the supplemental
/// [MSAL response model](https://azuread.github.io/microsoft-authentication-library-for-js/ref/types/_azure_msal_common.ServerAuthorizationTokenResponse.html).
#[derive(Facet)]
#[facet(rename_all = "snake_case")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct WorkloadIdentityTokenResponse {
    #[facet(sensitive)]
    pub access_token: Option<String>,
    pub token_type: Option<String>,
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

impl WorkloadIdentityTokenResponse {
    pub(crate) fn decode(status: StatusCode, body: &str) -> Result<(String, u64)> {
        // Facet diagnostics include source excerpts. Do not retain that source
        // error at this secret-bearing response boundary.
        let token = facet_json::from_str::<Self>(body).map_err(|_| {
            eyre::eyre!(
                "Entra workload-identity endpoint returned an invalid token response ({status})"
            )
        })?;
        if !status.is_success() || token.error.is_some() || token.access_token.is_none() {
            let reason = match token.error.as_deref() {
                Some("invalid_client") => "invalid_client",
                Some("invalid_grant") => "invalid_grant",
                Some("invalid_request") => "invalid_request",
                Some("invalid_scope") => "invalid_scope",
                Some("unauthorized_client") => "unauthorized_client",
                Some("unsupported_grant_type") => "unsupported_grant_type",
                _ => "token exchange rejected",
            };
            bail!("Entra workload-identity token exchange failed ({status}): {reason}");
        }
        ensure!(
            token
                .token_type
                .as_deref()
                .is_some_and(|kind| kind.eq_ignore_ascii_case("Bearer")),
            "Entra workload-identity token exchange returned an unsupported token type"
        );
        let expires_in = token
            .expires_in
            .ok_or_else(|| eyre::eyre!("Entra workload-identity response omitted expires_in"))?;
        let access_token = token
            .access_token
            .ok_or_else(|| eyre::eyre!("Entra workload-identity response omitted access_token"))?;
        ensure!(
            !access_token.is_empty(),
            "Entra workload-identity response included an empty access_token"
        );
        Ok((access_token, expires_in))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_never_retain_response_body_or_decoder_excerpts() -> Result<()> {
        let (access_token, expires_in) = WorkloadIdentityTokenResponse::decode(
            StatusCode::OK,
            r#"{"access_token":"synthetic-token","token_type":"Bearer","expires_in":3600}"#,
        )?;
        assert_eq!(access_token, "synthetic-token");
        assert_eq!(expires_in, 3600);

        for (status, response) in [
            (
                StatusCode::BAD_REQUEST,
                r#"{"error":"invalid_client","error_description":"secret-assertion"}"#,
            ),
            (StatusCode::BAD_REQUEST, r#"{"error":"secret-assertion"}"#),
            (
                StatusCode::OK,
                r#"{"access_token":"secret-assertion","expires_in":"wrong-type"}"#,
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"access_token":"secret-assertion","expires_in":3600}"#,
            ),
        ] {
            let error = WorkloadIdentityTokenResponse::decode(status, response)
                .expect_err("invalid or rejected exchanges cannot return a credential");
            let diagnostic = format!("{error:?}");
            assert!(!diagnostic.contains("secret-assertion"));
            assert!(diagnostic.contains(&status.to_string()));
        }
        Ok(())
    }
}
