#[macro_export]
macro_rules! impl_uuid_traits {
    ($type:ty) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_ref().to_string().as_str())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                let uuid = Uuid::from_str(&s).map_err(D::Error::custom)?;
                Ok(Self::new(uuid))
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_ref().to_string().as_str())
            }
        }

        impl FromStr for $type {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(Uuid::parse_str(s)?))
            }
        }

        impl std::ops::Deref for $type {
            type Target = Uuid;

            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }
    };
}
