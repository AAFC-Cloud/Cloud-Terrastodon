use crate::RestOutputFormat;
use crate::RestResponseBody;
use crate::RestResponseBodyProxy;
use crate::parse_response_body;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use facet_json::RawJson;
use reqwest::Response;
use reqwest::header::HeaderMap;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct SerializableRestResponse {
    pub status: u16,
    pub ok: bool,
    pub reason_phrase: Option<String>,
    pub headers: BTreeMap<String, Vec<String>>,
    #[facet(opaque, proxy = RestResponseBodyProxy)]
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

    pub fn into_json_body(self) -> Result<RawJson<'static>> {
        match self.body {
            RestResponseBody::Json(body) => Ok(body),
            RestResponseBody::Text(content) => facet_json::from_str::<RawJson<'static>>(&content)
                .map_err(|error| eyre::eyre!("{error:?}"))
                .wrap_err("Expected REST response body to contain JSON"),
        }
    }

    pub fn write(&self, output_format: RestOutputFormat, mut writer: impl Write) -> Result<()> {
        match output_format {
            RestOutputFormat::Text => match &self.body {
                RestResponseBody::Json(value) => writeln!(writer, "{}", value.as_str())?,
                RestResponseBody::Text(content) => writeln!(writer, "{}", content)?,
            },
            RestOutputFormat::Json => {
                writeln!(
                    writer,
                    "{}",
                    facet_json::to_string_pretty(self).map_err(|error| eyre::eyre!("{error:?}"))?
                )?;
            }
        }

        if !self.ok {
            bail!(
                "REST call failed with status {}: {}",
                self.status,
                self.reason_phrase.as_deref().unwrap_or("Unknown error")
            );
        }
        Ok(())
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
    use super::SerializableRestResponse;
    use super::serialize_headers;
    use crate::RestResponseBody;
    use http::StatusCode;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    fn pretty_json(input: &str) -> String {
        facet_json::to_string_pretty(&facet_json::from_str::<facet_value::Value>(input).unwrap())
            .unwrap()
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
    }

    #[test]
    fn round_trips_json_body_as_json() -> eyre::Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &headers,
            "{\"hello\":\"world\"}".to_string(),
        );

        let serialized = facet_json::to_string(&response)?;
        assert!(serialized.contains(r#""body":["{\n  \"hello\": \"world\"\n}"]"#));
        let reparsed: SerializableRestResponse = facet_json::from_str(&serialized)?;

        assert_eq!(
            reparsed.into_json_body()?.as_str(),
            pretty_json(r#"{"hello":"world"}"#)
        );
        Ok(())
    }

    #[test]
    fn decodes_cached_single_string_json_body() -> eyre::Result<()> {
        let cached_response = r#"{
            "status": 200,
            "ok": true,
            "reason_phrase": "OK",
            "headers": {},
            "body": ["{\"hello\":\"world\"}"]
        }"#;

        let response: SerializableRestResponse = facet_json::from_str(cached_response)?;

        assert_eq!(
            response.into_json_body()?.as_str(),
            pretty_json(r#"{"hello":"world"}"#)
        );
        Ok(())
    }

    #[test]
    fn preserves_json_array_body() -> eyre::Result<()> {
        let cached_response = r#"{
            "status": 200,
            "ok": true,
            "reason_phrase": "OK",
            "headers": {},
            "body": ["westus", "eastus"]
        }"#;

        let response: SerializableRestResponse = facet_json::from_str(cached_response)?;

        assert_eq!(
            response.into_json_body()?.as_str(),
            r#"["westus", "eastus"]"#
        );
        Ok(())
    }
}
