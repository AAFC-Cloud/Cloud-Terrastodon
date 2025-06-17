use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllOr<T> {
    All,
    AllTrusted,
    MicrosoftAdminPortals,
    None,
    Some(T),
}
impl<T> Serialize for AllOr<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AllOr::All => serializer.serialize_str("All"),
            AllOr::None => serializer.serialize_none(),
            AllOr::AllTrusted => serializer.serialize_str("AllTrusted"),
            AllOr::MicrosoftAdminPortals => serializer.serialize_str("MicrosoftAdminPortals"),
            AllOr::Some(value) => value.serialize(serializer),
        }
    }
}
impl<'de, T> Deserialize<'de> for AllOr<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AllOrVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for AllOrVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = AllOr<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("either the string \"all\" or a value of the inner type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.eq_ignore_ascii_case("all") {
                    Ok(AllOr::All)
                } else if value.eq_ignore_ascii_case("none") {
                    Ok(AllOr::None)
                } else if value.eq_ignore_ascii_case("alltrusted") {
                    Ok(AllOr::AllTrusted)
                } else if value.eq_ignore_ascii_case("microsoftadminportals") {
                    Ok(AllOr::MicrosoftAdminPortals)
                } else {
                    // If it's a string but not "all", try to deserialize the string as T
                    // This handles cases where T itself might be String or other string-deserializable types
                    T::deserialize(serde::de::value::StrDeserializer::new(value)).map(AllOr::Some)
                }
            }

            // For non-string values, deserialize directly as T
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::deserialize(serde::de::value::BoolDeserializer::new(value)).map(AllOr::Some)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::deserialize(serde::de::value::I64Deserializer::new(value)).map(AllOr::Some)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::deserialize(serde::de::value::U64Deserializer::new(value)).map(AllOr::Some)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::deserialize(serde::de::value::F64Deserializer::new(value)).map(AllOr::Some)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                T::deserialize(serde::de::value::SeqAccessDeserializer::new(seq)).map(AllOr::Some)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                T::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(AllOr::Some)
            }
        }

        deserializer.deserialize_any(AllOrVisitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let json_string = "\"all\"";
        let all_or: super::AllOr<String> = serde_json::from_str(json_string)?;
        assert_eq!(all_or, super::AllOr::All);

        let json_value = "\"some_value\"";
        let all_or: super::AllOr<String> = serde_json::from_str(json_value)?;
        assert_eq!(all_or, super::AllOr::Some("some_value".to_string()));

        let json_value = "42";
        let all_or: super::AllOr<i32> = serde_json::from_str(json_value)?;
        assert_eq!(all_or, super::AllOr::Some(42));

        Ok(())
    }
}
