use arbitrary::Arbitrary;
use http::Method;
use std::ops::Deref;

#[derive(facet::Facet, Debug, Clone)]
#[facet(opaque, proxy = String)]
pub struct HumantimeDurationCli(pub humantime::Duration);

impl<'a> Arbitrary<'a> for HumantimeDurationCli {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let seconds = u64::arbitrary(u)? % 86_400;
        Ok(Self(humantime::Duration::from(std::time::Duration::from_secs(seconds))))
    }
}
impl Deref for HumantimeDurationCli {
    type Target = humantime::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for HumantimeDurationCli {
    type Error = humantime::DurationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

impl From<&HumantimeDurationCli> for String {
    fn from(value: &HumantimeDurationCli) -> Self {
        value.0.to_string()
    }
}

#[derive(facet::Facet, Debug, Clone, PartialEq, Eq)]
#[facet(opaque, proxy = String)]
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
cloud_terrastodon_registry::register_thing!(HumantimeDurationCli);
cloud_terrastodon_registry::register_arbitrary!(HumantimeDurationCli);
cloud_terrastodon_registry::register_thing!(HttpMethodCli);
cloud_terrastodon_registry::register_arbitrary!(HttpMethodCli);

