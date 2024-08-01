use crate::prelude::strip_prefix_case_insensitive;
use crate::prelude::SubscriptionScoped;
use crate::prelude::TenantId;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuProviderBlock;
use uuid::Uuid;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Clone)]
pub struct SubscriptionId {
    expanded: String,
}

impl PartialEq for SubscriptionId {
    fn eq(&self, other: &Self) -> bool {
        self.expanded.to_lowercase() == other.expanded.to_lowercase()
    }
}

impl Eq for SubscriptionId {}

impl Hash for SubscriptionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.expanded.to_lowercase().hash(state);
    }
}
impl SubscriptionId {
    pub fn new(uuid: Uuid) -> SubscriptionId {
        let expanded = format!("{}{}", SUBSCRIPTION_ID_PREFIX, uuid);
        SubscriptionId { expanded }
    }
    pub fn uuid(&self) -> Uuid {
        self.short_form()
            .parse()
            .expect("subscription slug should be valid UUID")
    }
}
impl SubscriptionScoped for SubscriptionId {}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expanded)
    }
}

impl Scope for SubscriptionId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> anyhow::Result<Self> {
        expanded.parse()
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Subscription
    }

    fn as_scope(&self) -> ScopeImpl {
        ScopeImpl::Subscription(self.clone())
    }
}

impl FromStr for SubscriptionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(strip_prefix_case_insensitive(s,SUBSCRIPTION_ID_PREFIX).unwrap_or(s))?;
        Ok(SubscriptionId::new(uuid))
    }
}

impl Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.uuid().to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for SubscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
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

impl HasScope for Subscription {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &Subscription {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
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
            alias: Some(self.name.sanitize()),
            subscription_id: Some(self.id.short_form().to_owned()),
        }
    }
}
impl From<Subscription> for TofuProviderBlock {
    fn from(value: Subscription) -> Self {
        value.into_provider_block()
    }
}
