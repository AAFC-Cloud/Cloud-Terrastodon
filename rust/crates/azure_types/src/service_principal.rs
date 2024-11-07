use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ServicePrincipalId(Uuid);
impl UuidWrapper for ServicePrincipalId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(ServicePrincipalId);