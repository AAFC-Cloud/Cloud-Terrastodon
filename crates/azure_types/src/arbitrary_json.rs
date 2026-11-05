#![allow(
    clippy::disallowed_types,
    reason = "ArbitraryJson owns the RawJson codec boundary for opaque JSON values"
)]

use arbitrary::Arbitrary;
use facet_json::RawJson;
use facet_reflect::Peek;

/// An owned opaque JSON value with support for offline arbitrary generation.
///
/// Serialization preserves the JSON value rather than escaping it as text.
/// Prefer concrete records for modeled schemas; use this for intentionally
/// opaque values and open property bags.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = RawJson<'static>)]
pub struct ArbitraryJson(RawJson<'static>);

impl ArbitraryJson {
    #[must_use]
    pub fn object() -> Self {
        Self(RawJson::from_owned("{}".to_string()))
    }

    /// Encode a string scalar without reflective serialization. The formatter's
    /// string representation is a quoted, escaped JSON string; other dynamic
    /// value kinds are deliberately not accepted at this boundary.
    pub fn from_text(value: &str) -> Self {
        Self(RawJson::from_owned(facet_value::format_value(
            &facet_value::Value::from(value),
        )))
    }

    /// Serializes a Facet value into the raw JSON representation used by
    /// Azure API payloads.
    pub fn try_from_facet<T: facet::Facet<'static>>(value: &T) -> eyre::Result<Self> {
        Ok(RawJson::from_owned(facet_json::to_string(value)?).into())
    }

    /// Serialize a reflected field using its effective Facet representation.
    pub fn try_from_peek(value: Peek<'_, '_>) -> eyre::Result<Self> {
        Ok(RawJson::from_owned(facet_json::peek_to_string(value)?).into())
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

#[cfg(test)]
mod tests {
    use super::ArbitraryJson;

    #[test]
    fn text_scalars_preserve_escaping_and_unicode() -> eyre::Result<()> {
        for text in [
            "",
            "null",
            "a\"b\\c",
            "\0\u{0008}\u{001f}\n\r\t",
            "Équipe 😀",
        ] {
            let encoded = ArbitraryJson::from_text(text);
            let decoded: String = facet_json::from_str(encoded.as_ref())?;
            assert_eq!(decoded, text);
        }
        Ok(())
    }
}
