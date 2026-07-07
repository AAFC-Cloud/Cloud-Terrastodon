use arbitrary::Arbitrary;
use facet_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(transparent)]
pub struct ArbitraryJson(RawJson<'static>);

impl ArbitraryJson {
    #[must_use]
    pub fn object() -> Self {
        Self(RawJson::from_owned("{}".to_string()))
    }
}

impl<'a> Arbitrary<'a> for ArbitraryJson {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let json = match u.int_in_range(0..=3u8)? {
            0 => "null".to_string(),
            1 => "{}".to_string(),
            2 => "[]".to_string(),
            _ => facet_json::to_string(&String::arbitrary(u)?)
                .map_err(|_| arbitrary::Error::IncorrectFormat)?,
        };
        // todo: we could make it do nested fields and stuff
        Ok(Self(RawJson::from_owned(json)))
    }
}

impl From<RawJson<'static>> for ArbitraryJson {
    fn from(value: RawJson<'static>) -> Self {
        Self(value)
    }
}

impl From<ArbitraryJson> for RawJson<'static> {
    fn from(value: ArbitraryJson) -> Self {
        value.0
    }
}

impl From<&ArbitraryJson> for RawJson<'static> {
    fn from(value: &ArbitraryJson) -> Self {
        value.0.clone()
    }
}

impl AsRef<str> for ArbitraryJson {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for ArbitraryJson {
    type Target = RawJson<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

cloud_terrastodon_registry::register_thing!(ArbitraryJson);
cloud_terrastodon_registry::register_arbitrary!(ArbitraryJson);
