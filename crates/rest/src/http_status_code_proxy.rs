use arbitrary::Error as ArbitraryError;
use arbitrary::Unstructured;
use http::StatusCode;
use http::status::InvalidStatusCode;

/// Preserve numeric status codes on the wire while validating them through
/// [`StatusCode::from_u16`]. Invalid numbers fail decoding instead of entering a
/// response whose status cannot be classified.
#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[facet(transparent)]
pub(crate) struct HttpStatusCodeProxy(u16);

impl TryFrom<HttpStatusCodeProxy> for StatusCode {
    type Error = InvalidStatusCode;

    fn try_from(value: HttpStatusCodeProxy) -> Result<Self, Self::Error> {
        Self::from_u16(value.0)
    }
}

impl From<&StatusCode> for HttpStatusCodeProxy {
    fn from(value: &StatusCode) -> Self {
        Self(value.as_u16())
    }
}

/// Generate the full numeric range accepted by the pinned `http` crate,
/// including unregistered codes, without manufacturing an invalid status.
pub(crate) fn arbitrary_status_code(input: &mut Unstructured<'_>) -> arbitrary::Result<StatusCode> {
    StatusCode::from_u16(input.int_in_range(100..=999)?)
        .map_err(|_| ArbitraryError::IncorrectFormat)
}

#[cfg(test)]
mod tests {
    use super::HttpStatusCodeProxy;
    use http::StatusCode;

    #[test]
    fn numeric_wire_conversion_accepts_valid_boundaries_and_rejects_invalid_statuses() {
        for code in [100, 200, 404, 599, 999] {
            let status = StatusCode::try_from(HttpStatusCodeProxy(code))
                .expect("the http crate accepts this numeric status");
            assert_eq!(status.as_u16(), code);
            assert_eq!(HttpStatusCodeProxy::from(&status).0, code);
        }
        for code in [0, 99, 1000, u16::MAX] {
            assert!(StatusCode::try_from(HttpStatusCodeProxy(code)).is_err());
        }
    }
}
