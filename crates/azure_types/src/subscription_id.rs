use arbitrary::Arbitrary;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use uuid::Uuid;
use serde::de::Error;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash)]
pub struct SubscriptionId {
    pub inner: Uuid,
}
impl SubscriptionId {
    pub fn new(uuid: Uuid) -> Self {
        Self { inner: uuid }
    }
}
impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner.hyphenated()))
    }
}
impl Deref for SubscriptionId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for SubscriptionId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl From<Uuid> for SubscriptionId {
    fn from(value: Uuid) -> Self {
        Self { inner: value }
    }
}
impl FromStr for SubscriptionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix(SUBSCRIPTION_ID_PREFIX).unwrap_or(s);
        Ok(Self { inner: s.parse()? })
    }
}
impl serde::Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for SubscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}
impl Scope for SubscriptionId {
    fn expanded_form(&self) -> String {
        format!("{}{}", SUBSCRIPTION_ID_PREFIX, self.inner)
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        let expanded = expanded.strip_prefix(SUBSCRIPTION_ID_PREFIX).ok_or_else(|| {
            eyre::eyre!(
                "Expected subscription id to start with {} but got {}",
                SUBSCRIPTION_ID_PREFIX,
                expanded
            )
        })?;
        Ok(Self { inner: expanded.parse()? })

    }

    fn as_scope(&self) -> crate::prelude::ScopeImpl {
        ScopeImpl::Subscription(self.clone())
    }

    fn kind(&self) -> crate::prelude::ScopeImplKind {
        ScopeImplKind::Subscription
    }
}