use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use http::HeaderValue;
use http::header::InvalidHeaderValue;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Deref;
use std::str::FromStr;

/// An opaque Microsoft REST pagination token returned in a response header.
///
/// See Azure DevOps 7.1's [`x-ms-continuationtoken` and `continuationToken`
/// contract](https://learn.microsoft.com/en-us/rest/api/azure/devops/testplan/test-plans/list?view=azure-devops-rest-7.1#uri-parameters)
/// and Durable Functions' [`x-ms-continuation-token`
/// contract](https://learn.microsoft.com/en-us/azure/durable-task/durable-functions/durable-functions-http-api#get-all-instances-status).
///
/// Rejecting blank values is a library pagination safety constraint, not a
/// documented token grammar. Nonblank text is preserved exactly, without
/// trimming, decoding, or interpreting its contents. HTTP header conversion
/// checks transport syntax separately and can fail for a valid opaque token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct MicrosoftContinuationToken(String);

impl MicrosoftContinuationToken {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        ensure!(
            !value.trim().is_empty(),
            "Microsoft continuation tokens must not be blank"
        );
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for MicrosoftContinuationToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for MicrosoftContinuationToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for MicrosoftContinuationToken {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MicrosoftContinuationToken {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for MicrosoftContinuationToken {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for MicrosoftContinuationToken {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

// Facet's String proxy requires this owned conversion for serialization, so it
// clones. Use as_str() or deref coercion when only borrowed text is needed.
impl From<&MicrosoftContinuationToken> for String {
    fn from(value: &MicrosoftContinuationToken) -> Self {
        value.0.clone()
    }
}

impl From<MicrosoftContinuationToken> for String {
    fn from(value: MicrosoftContinuationToken) -> Self {
        value.0
    }
}

impl TryFrom<&MicrosoftContinuationToken> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: &MicrosoftContinuationToken) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<MicrosoftContinuationToken> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: MicrosoftContinuationToken) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl<'a> Arbitrary<'a> for MicrosoftContinuationToken {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut value = String::arbitrary(input)?;
        if value.trim().is_empty() {
            value.push('a');
        }
        Self::try_new(value).map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(MicrosoftContinuationToken);
cloud_terrastodon_registry::register_arbitrary!(MicrosoftContinuationToken);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_tokens() {
        for value in ["", " ", "\t\r\n", "\u{2003}"] {
            assert!(MicrosoftContinuationToken::try_new(value).is_err());
        }
    }

    #[test]
    fn preserves_opaque_token_text() {
        let text = "  synthetic+/=&?%token  ";
        let token: MicrosoftContinuationToken = text.parse().unwrap();
        let borrowed: &str = &token;
        assert_eq!(borrowed, text);
        assert_eq!(token.as_str(), text);
        assert_eq!(String::from(&token), text);
        assert_eq!(
            HeaderValue::try_from(&token).unwrap().as_bytes(),
            text.as_bytes()
        );
        assert_eq!(String::from(token), text);
    }

    #[test]
    fn header_syntax_is_checked_at_the_transport_boundary() {
        let token = MicrosoftContinuationToken::try_new("synthetic\r\ntoken").unwrap();
        assert!(HeaderValue::try_from(token).is_err());
    }
}
