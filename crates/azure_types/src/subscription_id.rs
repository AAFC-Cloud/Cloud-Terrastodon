use crate::prelude::SubscriptionScoped;
use crate::prelude::strip_prefix_case_insensitive;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use arbitrary::Arbitrary;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use uuid::Uuid;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash)]
pub struct SubscriptionId2 {
    pub inner: Uuid,
}
impl SubscriptionId2 {
    pub fn new(uuid: Uuid) -> Self {
        Self { inner: uuid }
    }
}
impl std::fmt::Display for SubscriptionId2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner.hyphenated()))
    }
}
impl Deref for SubscriptionId2 {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for SubscriptionId2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl From<Uuid> for SubscriptionId2 {
    fn from(value: Uuid) -> Self {
        Self { inner: value }
    }
}
impl FromStr for SubscriptionId2 {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { inner: s.parse()? })
    }
}
impl serde::Serialize for SubscriptionId2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for SubscriptionId2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <Uuid as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

///
///
///
///
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

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
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
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(
            strip_prefix_case_insensitive(s, SUBSCRIPTION_ID_PREFIX).unwrap_or(s),
        )?;
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
