use serde::Deserialize;
use serde::Deserializer;
use std::str::FromStr;

/// https://github.com/serde-rs/serde/issues/1098 - Ability to use default value even if set to null
pub fn deserialize_default_if_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Deserialize a string into an `Option<T>`, returning `None` if the string is empty.
pub fn deserialize_none_if_empty_string<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    // let str = String::deserialize(deserializer)?;
    let str: Option<String> = deserialize_default_if_null(deserializer)?;
    match str {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => T::from_str(&s).map_err(serde::de::Error::custom).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::SubscriptionId;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct MyThing {
        #[serde(default)]
        #[serde(deserialize_with = "super::deserialize_none_if_empty")]
        pub my_option: Option<SubscriptionId>,
    }

    #[test]
    fn it_works() -> eyre::Result<()> {
        let json = json!({});
        let thing: MyThing = serde_json::from_value(json)?;
        dbg!(&thing);
        assert!(thing.my_option.is_none());
        Ok(())
    }

    #[test]
    fn empty_string_is_none() -> eyre::Result<()> {
        let json = json!({ "my_option": "" });
        let thing: MyThing = serde_json::from_value(json)?;
        dbg!(&thing);
        assert!(thing.my_option.is_none());
        Ok(())
    }

    #[test]
    fn null_is_none() -> eyre::Result<()> {
        let json = json!({ "my_option": null });
        let thing: MyThing = serde_json::from_value(json)?;
        dbg!(&thing);
        assert!(thing.my_option.is_none());
        Ok(())
    }

    #[test]
    fn nil_uuid_is_some() -> eyre::Result<()> {
        let raw: Vec<u8> = (0..128).map(|_| rand::random::<u8>()).collect();
        let mut u = Unstructured::new(&raw);
        let id = SubscriptionId::arbitrary(&mut u)?;
        let json = json!({ "my_option": id });
        let thing: MyThing = serde_json::from_value(json)?;
        dbg!(&thing);
        assert!(thing.my_option.is_some());
        assert_eq!(thing.my_option, Some(id));
        Ok(())
    }
}
