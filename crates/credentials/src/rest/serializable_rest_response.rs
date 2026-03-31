use crate::RestResponseBody;
use crate::parse_response_body;
use eyre::Result;
use eyre::WrapErr;
use reqwest::Response;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableRestResponse {
    pub status: u16,
    pub ok: bool,
    pub reason_phrase: Option<String>,
    pub headers: BTreeMap<String, Vec<String>>,
    pub body: RestResponseBody,
}

impl SerializableRestResponse {
    pub fn new(status: http::StatusCode, headers: &HeaderMap, content: String) -> Self {
        Self {
            status: status.as_u16(),
            ok: status.is_success(),
            reason_phrase: status.canonical_reason().map(str::to_owned),
            headers: serialize_headers(headers),
            body: parse_response_body(content),
        }
    }

    pub async fn from_response(response: Response) -> Result<Self> {
        let status = response.status();
        let headers = response.headers().clone();
        let content = response.text().await?;
        Ok(Self::new(status, &headers, content))
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|(key, values)| {
            if key.eq_ignore_ascii_case(name) {
                values.first().map(String::as_str)
            } else {
                None
            }
        })
    }

    pub fn into_json_body(self) -> Result<Value> {
        match self.body {
            RestResponseBody::Json(body) => Ok(body),
            RestResponseBody::Text(content) => serde_json::from_str(&content)
                .wrap_err("Expected REST response body to contain JSON"),
        }
    }
}

pub fn serialize_headers(headers: &HeaderMap) -> BTreeMap<String, Vec<String>> {
    let mut serialized = BTreeMap::<String, Vec<String>>::new();
    for (name, value) in headers {
        let value = value
            .to_str()
            .map(str::to_owned)
            .unwrap_or_else(|_| String::from_utf8_lossy(value.as_bytes()).into_owned());
        serialized.entry(name.to_string()).or_default().push(value);
    }
    serialized
}

#[cfg(test)]
mod tests {
    use super::RestResponseBody;
    use super::SerializableRestResponse;
    use super::serialize_headers;
    use crate::parse_response_body;
    use http::StatusCode;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[test]
    fn parses_json_response_body() {
        let body = parse_response_body("{\"hello\":\"world\"}".to_string());
        assert_eq!(
            body,
            RestResponseBody::Json(serde_json::json!({"hello": "world"}))
        );
    }

    #[test]
    fn preserves_text_response_body() {
        let body = parse_response_body("not json".to_string());
        assert_eq!(body, RestResponseBody::Text("not json".to_string()));
    }

    #[test]
    fn serializes_repeated_headers() {
        let mut headers = HeaderMap::new();
        headers.append("x-test", HeaderValue::from_static("a"));
        headers.append("x-test", HeaderValue::from_static("b"));
        headers.append("content-type", HeaderValue::from_static("application/json"));

        let serialized = serialize_headers(&headers);
        assert_eq!(
            serialized.get("x-test").unwrap(),
            &vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            serialized.get("content-type").unwrap(),
            &vec!["application/json".to_string()]
        );
    }

    #[test]
    fn looks_up_headers_case_insensitively() {
        let response = SerializableRestResponse {
            status: 202,
            ok: true,
            reason_phrase: Some("Accepted".to_string()),
            headers: std::collections::BTreeMap::from([(
                String::from("Location"),
                vec![String::from("https://example.test/poll")],
            )]),
            body: RestResponseBody::Text(String::new()),
        };
        assert_eq!(
            response.header("location"),
            Some("https://example.test/poll")
        );
    }

    #[test]
    fn builds_response_from_status_headers_and_content() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &headers,
            "{\"hello\":\"world\"}".to_string(),
        );
        assert!(response.ok);
        assert_eq!(response.status, 200);
        assert_eq!(response.reason_phrase.as_deref(), Some("OK"));
        assert_eq!(
            response.body,
            RestResponseBody::Json(serde_json::json!({"hello": "world"}))
        );
    }
}
