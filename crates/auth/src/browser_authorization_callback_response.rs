use crate::browser_token_response::BrowserTokenResponse;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_command::CancellationError;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use eyre::ensure;
use reqwest::Url;

/// A state-validated authorization response from the browser's loopback redirect.
///
/// Codes and service-provided descriptions must not be included in diagnostics.
/// See [the authorization-code response contract](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow#successful-response).
pub(crate) enum BrowserAuthorizationCallbackResponse {
    Authorized { code: String },
    Cancelled,
    Denied { error: &'static str },
}

impl BrowserAuthorizationCallbackResponse {
    pub(crate) fn parse(request_target: &str, expected_state: &str) -> Result<Self> {
        let callback_url = Url::parse(&format!("http://localhost{request_target}"))
            .wrap_err("parsing the browser OAuth callback URL")?;
        let query = callback_url.query_pairs().collect::<Vec<_>>();
        for name in ["state", "code", "error", "error_subcode"] {
            ensure!(
                query.iter().filter(|(key, _)| key == name).count() <= 1,
                "Entra browser authentication returned duplicate {name} parameters"
            );
        }
        let query_value = |name: &str| {
            query
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_ref())
        };

        // Error responses must prove the same state as successful responses.
        // An unrelated request must not impersonate the user's cancellation.
        ensure!(
            query_value("state") == Some(expected_state),
            "Entra browser authentication returned an invalid state"
        );
        if let Some(error) = query_value("error") {
            return Ok(
                if error == "access_denied" && query_value("error_subcode") == Some("cancel") {
                    Self::Cancelled
                } else {
                    Self::Denied {
                        error: BrowserTokenResponse::diagnostic_error(Some(error)),
                    }
                },
            );
        }
        let code = query_value("code")
            .filter(|code| !code.is_empty())
            .ok_or_else(|| {
                eyre::eyre!("Entra browser authentication callback did not contain a code")
            })?;
        Ok(Self::Authorized {
            code: code.to_owned(),
        })
    }

    pub(crate) fn response_body(&self) -> &'static str {
        match self {
            Self::Authorized { .. } => {
                "<!doctype html><title>Cloud Terrastodon</title><p>Authorization received. You can close this tab.</p>"
            }
            Self::Cancelled => {
                "<!doctype html><title>Cloud Terrastodon</title><p>Login cancelled. You can close this tab.</p>"
            }
            Self::Denied { .. } => {
                "<!doctype html><title>Cloud Terrastodon</title><p>Login denied. See the terminal for details. You can close this tab.</p>"
            }
        }
    }

    pub(crate) fn into_code(self, resource: AzureCredentialResource) -> Result<String> {
        match self {
            Self::Authorized { code } => Ok(code),
            Self::Cancelled => Err(CancellationError::new("browser login was cancelled").into()),
            Self::Denied { error } => {
                bail!("Entra browser authentication for {resource:?} failed ({error})");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_encoded_code_and_matching_state() {
        let code = BrowserAuthorizationCallbackResponse::parse(
            "/?code=code%2Bvalue&state=expected%20state",
            "expected state",
        )
        .unwrap()
        .into_code(AzureCredentialResource::MicrosoftGraph)
        .unwrap();
        assert_eq!(code, "code+value");
    }

    #[test]
    fn validates_state_before_honoring_success_or_cancellation() {
        for request in [
            "/?code=code&state=unexpected",
            "/?error=access_denied&error_subcode=cancel&state=unexpected",
            "/?error=access_denied&error_subcode=cancel",
        ] {
            let error = BrowserAuthorizationCallbackResponse::parse(request, "expected")
                .err()
                .expect("the callback must validate its state");
            assert!(error.to_string().contains("invalid state"));
        }
    }

    #[test]
    fn distinguishes_user_cancellation_from_a_permission_denial() {
        let error = BrowserAuthorizationCallbackResponse::parse(
            "/?error=access_denied&error_subcode=cancel&state=expected",
            "expected",
        )
        .unwrap()
        .into_code(AzureCredentialResource::MicrosoftGraph)
        .unwrap_err();
        assert!(error.is::<CancellationError>());
        assert!(error.to_string().contains("cancelled"));

        let error = BrowserAuthorizationCallbackResponse::parse(
            "/?error=access_denied&error_description=sensitive-details&state=expected",
            "expected",
        )
        .unwrap()
        .into_code(AzureCredentialResource::MicrosoftGraph)
        .unwrap_err();
        assert!(!error.is::<CancellationError>());
        assert!(error.to_string().contains("access_denied"));
        assert!(!format!("{error:?}").contains("sensitive-details"));
    }

    #[test]
    fn rejects_ambiguous_callback_parameters_without_exposing_values() {
        for request in [
            "/?code=sensitive-code&state=expected&state=sensitive-state",
            "/?code=sensitive-code&code=other-code&state=expected",
            "/?error=access_denied&error=sensitive-error&state=expected",
        ] {
            let error = BrowserAuthorizationCallbackResponse::parse(request, "expected")
                .err()
                .expect("ambiguous parameters must be rejected");
            assert!(error.to_string().contains("duplicate"));
            assert!(!format!("{error:?}").contains("sensitive"));
        }
    }
}
