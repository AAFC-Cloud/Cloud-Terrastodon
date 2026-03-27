use crate::SubscriptionId;
use crate::tenant_id::AzureTenantId;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct AccessToken<T> {
    #[serde(rename = "accessToken")]
    pub access_token: T,
    #[serde(deserialize_with = "crate::serde_helpers::deserialize_local_date_time_from_epoch")]
    pub expires_on: DateTime<Local>,
    pub subscription: SubscriptionId,
    pub tenant: AzureTenantId,
    #[serde(rename = "tokenType")]
    pub token_type: TokenType,
}

impl<T> Debug for AccessToken<T> {
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Bearer,
}
