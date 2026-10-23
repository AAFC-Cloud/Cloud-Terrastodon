use arbitrary::Arbitrary;
use http::Method;
use std::ops::Deref;

/// An HTTP method parsed from a command-line string.
#[derive(facet::Facet, Debug, Clone, PartialEq, Eq)]
#[facet(proxy = String, opaque)]
pub struct HttpMethodCli(pub Method);

impl<'a> Arbitrary<'a> for HttpMethodCli {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let method = match u.int_in_range(0..=4)? {
            0 => Method::GET,
            1 => Method::POST,
            2 => Method::PUT,
            3 => Method::DELETE,
            _ => Method::PATCH,
        };
        Ok(Self(method))
    }
}
impl Deref for HttpMethodCli {
    type Target = Method;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for HttpMethodCli {
    type Error = http::method::InvalidMethod;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

impl From<&HttpMethodCli> for String {
    fn from(value: &HttpMethodCli) -> Self {
        value.0.to_string()
    }
}
cloud_terrastodon_registry::register_thing!(HttpMethodCli);
cloud_terrastodon_registry::register_arbitrary!(HttpMethodCli);
