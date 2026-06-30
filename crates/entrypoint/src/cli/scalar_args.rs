use http::Method;
use std::ops::Deref;

#[derive(facet::Facet, Debug, Clone)]
#[facet(opaque, proxy = String)]
pub struct HumantimeDurationCli(pub humantime::Duration);

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