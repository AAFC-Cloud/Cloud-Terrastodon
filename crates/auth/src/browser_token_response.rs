use crate::AzureBearerToken;
use crate::browser_access_token::BrowserAccessToken;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::TokenType;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use http::StatusCode;

/// Success or error returned by the Entra v2 token endpoint.
///
/// [Protocol response](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow#successful-response),
/// with optional extended-lifetime fields from the
/// [supplemental MSAL response model](https://azuread.github.io/microsoft-authentication-library-for-js/ref/types/_azure_msal_common.ServerAuthorizationTokenResponse.html).
/// This response deliberately has no `Debug`: error descriptions and malformed
/// credential fields must never become diagnostic output.
#[derive(Default, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct BrowserTokenResponse {
    #[facet(sensitive)]
    pub(crate) access_token: Option<String>,
    #[facet(sensitive)]
    pub(crate) refresh_token: Option<String>,
    #[facet(sensitive)]
    pub(crate) id_token: Option<String>,
    pub(crate) token_type: Option<String>,
    pub(crate) expires_in: Option<u64>,
    pub(crate) ext_expires_in: Option<u64>,
    pub(crate) refresh_in: Option<u64>,
    pub(crate) scope: Option<String>,
    pub(crate) error: Option<String>,
    #[facet(sensitive)]
    pub(crate) error_description: Option<String>,
    pub(crate) error_codes: Option<Vec<u64>>,
    pub(crate) timestamp: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) correlation_id: Option<String>,
    pub(crate) suberror: Option<String>,
}

impl BrowserTokenResponse {
    pub(crate) fn decode(body: &str) -> Result<Self> {
        // Facet's source diagnostic can contain input excerpts, including tokens.
        // Deliberately omit that source at this credential boundary.
        facet_json::from_str(body)
            .map_err(|_| eyre::eyre!("could not decode the Entra browser token response"))
    }

    pub(crate) fn into_access_token(
        self,
        status: StatusCode,
        tenant_id: AzureTenantId,
        resource: AzureCredentialResource,
    ) -> Result<BrowserAccessToken> {
        if !status.is_success() || self.error.is_some() {
            let error = Self::diagnostic_error(self.error.as_deref());
            bail!("Entra browser token request for {resource:?} failed with {status} ({error})");
        }
        let access_token = self
            .access_token
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| eyre::eyre!("Entra browser token response omitted an access token"))?;
        if !self
            .token_type
            .as_deref()
            .is_some_and(|token_type| token_type.eq_ignore_ascii_case("bearer"))
        {
            bail!("Entra browser token response did not specify a Bearer token for {resource:?}");
        }
        let expires_in = self
            .expires_in
            .ok_or_else(|| eyre::eyre!("Entra browser token response omitted expires_in"))?;
        let expires_in = i64::try_from(expires_in)
            .wrap_err("Entra browser token expiry is outside the supported range")?;
        let validity = TimeDelta::try_seconds(expires_in).ok_or_else(|| {
            eyre::eyre!("Entra browser token expiry is outside the supported range")
        })?;
        let expires_on = Local::now().checked_add_signed(validity).ok_or_else(|| {
            eyre::eyre!("Entra browser token expiry is outside the supported range")
        })?;
        if self
            .refresh_token
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            bail!("Entra browser token response included an empty refresh token");
        }
        Ok(BrowserAccessToken {
            access_token: AzureAccessToken {
                access_token: AzureBearerToken::new(access_token),
                expires_on,
                subscription: None,
                tenant: tenant_id,
                token_type: TokenType::Bearer,
            },
            refresh_token: self.refresh_token,
        })
    }

    /// Only emit known protocol vocabulary, never untrusted response text.
    pub(crate) fn diagnostic_error(error: Option<&str>) -> &'static str {
        match error {
            Some("invalid_request") => "invalid_request",
            Some("invalid_grant") => "invalid_grant",
            Some("unauthorized_client") => "unauthorized_client",
            Some("invalid_client") => "invalid_client",
            Some("unsupported_grant_type") => "unsupported_grant_type",
            Some("invalid_resource") => "invalid_resource",
            Some("interaction_required") => "interaction_required",
            Some("temporarily_unavailable") => "temporarily_unavailable",
            Some("consent_required") => "consent_required",
            Some("invalid_scope") => "invalid_scope",
            Some("login_required") => "login_required",
            Some("access_denied") => "access_denied",
            _ => "unrecognized_error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_secret_responses_never_enter_diagnostics() {
        let error = BrowserTokenResponse::decode(
            "{\"access_token\":\"synthetic-secret\",\"expires_in\": invalid}",
        )
        .err()
        .unwrap();
        assert!(!format!("{error:?}").contains("synthetic-secret"));
        let error = BrowserTokenResponse {
            error: Some("synthetic-secret-error".to_owned()),
            error_description: Some("synthetic-secret-description".to_owned()),
            ..Default::default()
        }
        .into_access_token(
            StatusCode::BAD_REQUEST,
            "11111111-1111-1111-1111-111111111111".parse().unwrap(),
            AzureCredentialResource::MicrosoftGraph,
        )
        .err()
        .unwrap();
        assert!(!format!("{error:?}").contains("synthetic-secret"));
    }

    #[test]
    fn rejects_failed_exchange_even_with_access_token_and_unrepresentable_expiry() {
        for (status, expiry) in [(StatusCode::BAD_REQUEST, 3600), (StatusCode::OK, u64::MAX)] {
            let error = BrowserTokenResponse {
                access_token: Some("synthetic-secret".to_owned()),
                token_type: Some("Bearer".to_owned()),
                expires_in: Some(expiry),
                ..Default::default()
            }
            .into_access_token(
                status,
                "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                AzureCredentialResource::MicrosoftGraph,
            )
            .err()
            .unwrap();
            assert!(!format!("{error:?}").contains("synthetic-secret"));
        }
    }
}
