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
        impl From<&$name> for String {
            fn from(value: &$name) -> Self {
                value.as_ref().to_string()
            }
        }
        impl TryFrom<String> for $name {
            type Error = eyre::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
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
        #[cfg(test)]
        mod facet_json_roundtrip {
            use super::*;

            #[test]
            fn uuid_newtype_json_round_trips_through_facet() -> eyre::Result<()> {
                let value =
                    facet_json::from_str::<$name>("\"00000000-0000-0000-0000-000000000000\"")?;
                assert_eq!(value, $name::new(uuid::Uuid::nil()));
                let reparsed = facet_json::from_str::<$name>(&facet_json::to_string(&value)?)?;
                assert_eq!(value, reparsed);
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_facet_string_proxy {
    ($name:ty, $value:ident => $serialized:expr) => {
        impl From<&$name> for String {
            fn from($value: &$name) -> Self {
                $serialized
            }
        }

        impl TryFrom<String> for $name {
            type Error = <$name as std::str::FromStr>::Err;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_facet_string_proxy_serialize {
    ($name:ty, $value:ident => $serialized:expr) => {
        impl From<&$name> for String {
            fn from($value: &$name) -> Self {
                $serialized
            }
        }
    };
}
