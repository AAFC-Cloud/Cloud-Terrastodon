use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    time::Duration,
};

use arbitrary::Arbitrary;
use http::HeaderMap;

#[derive(Debug, PartialEq, facet::Facet, Default, Clone, Arbitrary)]
#[facet(transparent)]
pub struct RestResponseHeaders(pub BTreeMap<String, Vec<String>>);

impl Deref for RestResponseHeaders {
    type Target = BTreeMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RestResponseHeaders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RestResponseHeaders {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.iter().find_map(|(key, values)| {
            if key.eq_ignore_ascii_case(name) {
                values.first().map(String::as_str)
            } else {
                None
            }
        })
    }
    
    pub fn resource_graph_quota_remaining(&self) -> Option<u64> {
        self.header("x-ms-user-quota-remaining")
            .and_then(|value| value.parse::<u64>().ok())
    }

    pub fn retry_after(&self) -> Option<Duration> {
        const RETRY_AFTER_HEADERS: [&str; 3] = [
            "x-ms-user-quota-resets-after",
            "x-ms-ratelimit-microsoft.costmanagement-clienttype-retry-after",
            "retry-after",
        ];
        for header in RETRY_AFTER_HEADERS {
            if let Some(value) = self.header(header) {
                if let Some(duration) = parse_hms_duration(value) {
                    return Some(duration);
                }
                if let Some(duration) = value.parse::<u64>().ok().map(Duration::from_secs) {
                    return Some(duration);
                }
            }
        }
        None
    }
}

fn parse_hms_duration(value: &str) -> Option<Duration> {
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<u64>().ok()?;
    let minutes = parts.next()?.parse::<u64>().ok()?;
    let seconds = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Duration::from_secs(
        hours * 60 * 60 + minutes * 60 + seconds,
    ))
}

impl From<&HeaderMap> for RestResponseHeaders {
    fn from(headers: &HeaderMap) -> Self {
        let mut serialized = BTreeMap::<String, Vec<String>>::new();
        for (name, value) in headers {
            let value = value
                .to_str()
                .map(str::to_owned)
                .unwrap_or_else(|_| String::from_utf8_lossy(value.as_bytes()).into_owned());
            serialized.entry(name.to_string()).or_default().push(value);
        }
        RestResponseHeaders(serialized)
    }
}
impl<K, V, VV> FromIterator<(K, V)> for RestResponseHeaders
where
    K: Into<String>,
    V: IntoIterator<Item = VV>,
    VV: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(headers: T) -> Self {
        let mut serialized = BTreeMap::<String, Vec<String>>::new();
        for (name, values) in headers {
            let name = name.into();
            let values = values.into_iter().map(|v| v.into()).collect();
            serialized.insert(name, values);
        }
        RestResponseHeaders(serialized)
    }
}
impl<const N: usize, K, V, VV> From<[(K, V); N]> for RestResponseHeaders
where
    K: Into<String>,
    V: IntoIterator<Item = VV>,
    VV: Into<String>,
{
    fn from(headers: [(K, V); N]) -> Self {
        headers.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderValue};

    use crate::{RestResponseBody, RestResponseHeaders, SerializableRestResponse};

    #[test]
    fn looks_up_headers_case_insensitively() {
        let response = SerializableRestResponse {
            status: 202,
            ok: true,
            reason_phrase: Some("Accepted".to_string()),
            headers: RestResponseHeaders::from([("Location", vec!["https://example.test/poll"])]),
            body: RestResponseBody::Text(String::new()),
        };
        assert_eq!(
            response.header("location"),
            Some("https://example.test/poll")
        );
    }

    #[test]
    fn serializes_repeated_headers() {
        let mut headers = HeaderMap::new();
        headers.append("x-test", HeaderValue::from_static("a"));
        headers.append("x-test", HeaderValue::from_static("b"));
        headers.append("content-type", HeaderValue::from_static("application/json"));

        let serialized = RestResponseHeaders::from(&headers);
        assert_eq!(
            serialized.get("x-test").unwrap(),
            &vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            serialized.get("content-type").unwrap(),
            &vec!["application/json".to_string()]
        );
    }
}
