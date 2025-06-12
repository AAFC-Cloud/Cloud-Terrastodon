use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use arbitrary::Arbitrary;
use eyre::Context;
use serde::de::Error;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use uuid::Uuid;

pub const SUBSCRIPTION_ID_PREFIX: &str = "/subscriptions/";

#[derive(Debug, Eq, PartialEq, Clone, Copy, Arbitrary, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(Uuid);
impl SubscriptionId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
    pub fn try_new<T>(uuid: T) -> eyre::Result<Self>
    where
        T: TryInto<Uuid>,
        T::Error: Into<eyre::Error>,
    {
        let uuid = uuid
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert to Uuid")?;
        Ok(Self(uuid))
    }
}
impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.hyphenated()))
    }
}
impl Deref for SubscriptionId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SubscriptionId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Uuid> for SubscriptionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}
impl serde::Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
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
        format!("{}{}", SUBSCRIPTION_ID_PREFIX, self.0)
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        let expanded = expanded
            .strip_prefix(SUBSCRIPTION_ID_PREFIX)
            .ok_or_else(|| {
                eyre::eyre!(
                    "Expected subscription id to start with {} but got {}",
                    SUBSCRIPTION_ID_PREFIX,
                    expanded
                )
            })?;
        Ok(Self(expanded.parse()?))
    }

    fn as_scope_impl(&self) -> crate::prelude::ScopeImpl {
        ScopeImpl::Subscription(*self)
    }

    fn kind(&self) -> crate::prelude::ScopeImplKind {
        ScopeImplKind::Subscription
    }
}

impl FromStr for SubscriptionId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix(SUBSCRIPTION_ID_PREFIX).unwrap_or(s);
        let id: eyre::Result<Uuid, _> = s.parse();
        let id = id.wrap_err_with(|| format!("Parsing subscription id from {s:?}"))?;
        Ok(Self(id))
    }
}
impl TryFrom<&str> for SubscriptionId {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod test {
    use super::SubscriptionId;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        // a random guid
        let _id = SubscriptionId::try_new("ba53fb6a-867e-413b-8c91-53fb5ff77d70")?;
        Ok(())
    }
}
