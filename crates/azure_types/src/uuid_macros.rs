#[macro_export]
macro_rules! impl_uuid_traits {
    ($type:ty) => {
        impl serde::Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_ref().to_string().as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                use serde::de::Error;
                use std::str::FromStr;
                let uuid = uuid::Uuid::from_str(&s).map_err(D::Error::custom)?;
                Ok(Self::new(uuid))
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_ref().to_string().as_str())
            }
        }

        impl std::str::FromStr for $type {
            type Err = eyre::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(uuid::Uuid::parse_str(s)?))
            }
        }

        impl std::ops::Deref for $type {
            type Target = uuid::Uuid;

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }
    };
}
