use crate::AzureBearerToken;
use crate::BrowserSession;
use crate::browser_access_token::BrowserAccessToken;
use crate::browser_token_cache_key::BrowserTokenCacheKey;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_azure_types::AzureAccessToken;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;

/// Mutable credentials for one browser authentication lifetime.
///
/// Private state makes the initial handoff a one-way transition. Once it begins,
/// only the resulting refresh-token chain may acquire another token; failure or
/// cancellation never permits another account selection under the same partition.
#[derive(Default)]
pub(crate) struct BrowserTokenSession {
    entries: HashMap<BrowserTokenCacheKey, AzureAccessToken<AzureBearerToken>>,
    refresh_token: Option<String>,
    initial_login_attempted: bool,
}

impl BrowserTokenSession {
    pub(crate) fn from_session(session: Option<&BrowserSession>) -> Self {
        Self {
            refresh_token: session.map(|session| session.refresh_token.clone()),
            initial_login_attempted: session.is_some(),
            ..Self::default()
        }
    }

    pub(crate) fn fresh_token(
        &self,
        key: &BrowserTokenCacheKey,
    ) -> Option<AzureAccessToken<AzureBearerToken>> {
        self.entries
            .get(key)
            .filter(|token| token.expires_on > Local::now() + TimeDelta::seconds(120))
            .cloned()
    }

    pub(crate) fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub(crate) fn begin_initial_login(&mut self, allow_interactive: bool) -> Result<()> {
        if self.initial_login_attempted {
            bail!(
                "this browser authentication session has no reusable refresh credential; begin a new authentication context to sign in again"
            );
        }
        if !allow_interactive {
            bail!(
                "browser authentication has no stored refresh token and interactive authentication is unavailable in a headless context"
            );
        }
        self.initial_login_attempted = true;
        Ok(())
    }

    pub(crate) fn remember(&mut self, key: BrowserTokenCacheKey, token: BrowserAccessToken) {
        // Refresh tokens rotate across resources. Always use the newest one,
        // including when an older resource-specific access token later expires.
        if token.refresh_token.is_some() {
            self.refresh_token = token.refresh_token;
        }
        self.initial_login_attempted = true;
        self.entries.insert(key, token.access_token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureCredentialResource;
    use cloud_terrastodon_azure_types::TokenType;

    fn token(refresh_token: Option<&str>) -> BrowserAccessToken {
        BrowserAccessToken {
            access_token: AzureAccessToken {
                access_token: AzureBearerToken::new("synthetic-access-token"),
                expires_on: Local::now() + TimeDelta::hours(1),
                subscription: None,
                tenant: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
                token_type: TokenType::Bearer,
            },
            refresh_token: refresh_token.map(str::to_owned),
        }
    }

    fn key(resource: AzureCredentialResource) -> BrowserTokenCacheKey {
        BrowserTokenCacheKey {
            resource,
            scopes: resource.delegated_scope().to_owned(),
        }
    }

    #[test]
    fn interrupted_initial_handoff_cannot_select_another_account() {
        let mut session = BrowserTokenSession::default();
        assert!(session.begin_initial_login(false).is_err());
        session.begin_initial_login(true).unwrap();
        // Dropping or failing the network handoff does not reset admission.
        assert!(session.begin_initial_login(true).is_err());
    }

    #[test]
    fn missing_refresh_token_does_not_allow_reauthentication() {
        let mut session = BrowserTokenSession::default();
        session.begin_initial_login(true).unwrap();
        session.remember(key(AzureCredentialResource::MicrosoftGraph), token(None));
        assert!(
            session
                .fresh_token(&key(AzureCredentialResource::MicrosoftGraph))
                .is_some()
        );
        assert!(
            session
                .fresh_token(&key(AzureCredentialResource::AzureDevOps))
                .is_none()
        );
        assert!(session.begin_initial_login(true).is_err());
    }

    #[test]
    fn refresh_rotation_is_shared_between_resources_and_preserved_when_omitted() {
        let mut session = BrowserTokenSession::default();
        session.remember(
            key(AzureCredentialResource::MicrosoftGraph),
            token(Some("first")),
        );
        session.remember(
            key(AzureCredentialResource::AzureDevOps),
            token(Some("rotated")),
        );
        session.remember(key(AzureCredentialResource::MicrosoftGraph), token(None));
        assert_eq!(session.refresh_token(), Some("rotated"));
        assert!(session.begin_initial_login(true).is_err());
    }
}
