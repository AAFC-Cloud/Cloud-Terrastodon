use crate::SubscriptionId;
use crate::tenant_id::AzureTenantId;
use chrono::DateTime;
use chrono::Local;
use std::fmt::Debug;

#[derive(PartialEq, facet::Facet)]
pub struct AzureAccessToken<T> {
    #[facet(rename = "accessToken")]
    #[facet(sensitive)]
    pub access_token: T,
    #[facet(proxy = crate::LocalDateTimeEpochSecondsProxy)]
    pub expires_on: DateTime<Local>,
    pub subscription: Option<SubscriptionId>,
    pub tenant: AzureTenantId,
    #[facet(rename = "tokenType")]
    pub token_type: TokenType,
}

impl<T> Debug for AzureAccessToken<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessToken")
            .field("access_token", &"***redacted***")
            .field("expires_on", &self.expires_on)
            .field("subscription", &self.subscription)
            .field("tenant", &self.tenant)
            .field("token_type", &self.token_type)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum TokenType {
    Bearer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_token_json_deserializes() -> eyre::Result<()> {
        let json = r#"
        {
            "accessToken": "secret",
            "expires_on": 1712345678,
            "subscription": "00000000-0000-0000-0000-000000000000",
            "tenant": "11111111-1111-1111-1111-111111111111",
            "tokenType": "Bearer"
        }
        "#;

        let token = facet_json::from_str::<AzureAccessToken<String>>(json)?;
        assert_eq!(token.access_token, "secret");
        assert_eq!(token.token_type, TokenType::Bearer);
        Ok(())
    }
}
