use serde::de::Error;
use std::hash::Hash;
use std::str::FromStr;
use tofu_types::prelude::TofuProviderBlock;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use uuid::Uuid;

use crate::prelude::TenantId;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionId(pub Uuid);

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl FromStr for SubscriptionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SubscriptionId(uuid::Uuid::parse_str(s)?))
    }
}

impl Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for SubscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded.parse().map_err(D::Error::custom)?;
        Ok(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionUserKind {
    #[serde(rename = "user")]
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionUser {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionState {
    Enabled,
}

/// `az cloud list --output table`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AzureCloudKind {
    AzureCloud,
    AzureChinaCloud,
    AzureUSGovernment,
    AzureGermanCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    #[serde(rename = "cloudName")]
    pub cloud_name: AzureCloudKind,
    pub id: SubscriptionId,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
    pub name: String,
    pub state: SubscriptionState,
    #[serde(rename = "tenantId")]
    pub tenant_id: TenantId,
    pub user: SubscriptionUser,
}
impl Hash for Subscription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Subscription {}
impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl Subscription {
    pub fn into_provider_block(self) -> TofuProviderBlock {
        TofuProviderBlock::AzureRM {
            alias: Some(self.name),
            subscription_id: Some(self.id.to_string()),
        }
    }
}
impl From<Subscription> for TofuProviderBlock {
    fn from(value: Subscription) -> Self {
        value.into_provider_block()
    }
}