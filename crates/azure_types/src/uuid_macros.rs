/// Implements common UUID-based traits & helpers for a newtype *tuple struct* or enum around a `uuid::Uuid`.
///
/// Struct usage (assumes a single tuple field `0` that is a `Uuid`):
/// ```ignore
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
/// pub struct UserId(uuid::Uuid);
/// impl_uuid_traits!(UserId);
/// ```
#[macro_export]
macro_rules! impl_uuid_newtype {
    ($name:ident) => {
        impl $name {
            pub fn new(uuid: impl Into<uuid::Uuid>) -> Self {
                Self(uuid.into())
            }
        }
        impl AsRef<uuid::Uuid> for $name {
            fn as_ref(&self) -> &uuid::Uuid {
                &self.0
            }
        }
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_ref().to_string().as_str())
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                use std::str::FromStr;
                let uuid = uuid::Uuid::from_str(&s).map_err(serde::de::Error::custom)?;
                Ok(Self::new(uuid))
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_ref().to_string().as_str())
            }
        }
        impl std::str::FromStr for $name {
            type Err = eyre::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(uuid::Uuid::parse_str(s)?))
            }
        }
        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }
    };
}
