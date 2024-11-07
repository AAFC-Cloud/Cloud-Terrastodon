use serde::de::Error;
use std::ops::Deref;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use uuid::Uuid;

use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AppId(Uuid);
impl UuidWrapper for AppId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(AppId);