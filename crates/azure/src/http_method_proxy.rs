use eyre::Result;
use facet::Facet;
use http::Method;

#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub(crate) struct HttpMethodProxy(String);

impl TryFrom<HttpMethodProxy> for Method {
    type Error = http::method::InvalidMethod;

    fn try_from(value: HttpMethodProxy) -> Result<Self, Self::Error> {
        Method::from_bytes(value.0.as_bytes())
    }
}

impl From<&Method> for HttpMethodProxy {
    fn from(value: &Method) -> Self {
        Self(value.as_str().to_string())
    }
}
