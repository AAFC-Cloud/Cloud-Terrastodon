use crate::RestHttpError;
use crate::RestOutputFormat;
use crate::RestResponseBody;
use crate::RestResponseBodyProxy;
use crate::RestResponseHeaders;
use crate::http_status_code_proxy::HttpStatusCodeProxy;
use crate::http_status_code_proxy::arbitrary_status_code;
use crate::parse_response_body;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use facet::Facet;
use facet_json::RawJson;
use http::StatusCode;
use reqwest::Response;
use reqwest::header::HeaderMap;
use std::io::Write;

#[derive(Arbitrary, Clone, Debug, PartialEq, facet::Facet)]
pub struct SerializableRestResponse {
    /// A validated HTTP status, serialized as its numeric code in cache artifacts.
    #[facet(opaque, proxy = HttpStatusCodeProxy)]
    #[arbitrary(with = arbitrary_status_code)]
    pub status: StatusCode,
    pub ok: bool,
    pub reason_phrase: Option<String>,
    pub headers: RestResponseHeaders,
    #[facet(proxy = RestResponseBodyProxy)]
    pub body: RestResponseBody,
}

impl SerializableRestResponse {
    pub fn new(status: StatusCode, headers: &HeaderMap, content: String) -> Self {
        Self {
            status,
            ok: status.is_success(),
            reason_phrase: status.canonical_reason().map(str::to_owned),
            headers: RestResponseHeaders::from(headers),
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

    /// Decode a JSON response, rejecting HTML pages even when their HTTP status
    /// indicates success. Authentication redirects can end at such a page.
    ///
    /// Content-Type is not required: some APIs omit it or return JSON as text.
    /// The JSON decoder still validates the actual body. Failure diagnostics
    /// describe the response without copying potentially sensitive page content.
    pub fn decode_json_body<T: Facet<'static>>(&self) -> Result<T> {
        let body = match &self.body {
            RestResponseBody::Json(body) => body.as_str(),
            RestResponseBody::Text(body) => body.as_str(),
        };
        let content_type = self.header("content-type");
        let is_html = content_type.is_some_and(|content_type| {
            let media_type = content_type.split(';').next().unwrap_or_default().trim();
            media_type.eq_ignore_ascii_case("text/html")
                || media_type.eq_ignore_ascii_case("application/xhtml+xml")
        });
        if is_html
            || body
                .trim_start_matches('\u{feff}')
                .trim_start()
                .starts_with('<')
        {
            bail!(
                "Expected a JSON REST response, but received HTML/XML (HTTP {}, Content-Type: {}). \
                 This may be an authentication redirect or sign-in/sign-out page; check the request's authentication context and permissions.",
                self.status,
                content_type.unwrap_or("not provided"),
            );
        }
        facet_json::from_str::<T>(body).wrap_err_with(|| {
            format!(
                "Deserializing REST response into {} (HTTP {}, Content-Type: {})",
                std::any::type_name::<T>(),
                self.status,
                content_type.unwrap_or("not provided"),
            )
        })
    }

    /// Reject an unsuccessful status with a [`RestHttpError`].
    ///
    /// The numeric HTTP status is authoritative, including for cached responses;
    /// the redundant serialized `ok` flag does not establish success or absence.
    /// Callers can handle the concrete error or convert it into an error report.
    pub fn error_for_status(&self) -> Result<(), RestHttpError> {
        if self.status.is_success() {
            Ok(())
        } else {
            Err(RestHttpError {
                status: self.status,
                reason_phrase: self.reason_phrase.clone(),
            })
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

        self.error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SerializableRestResponse;
    use crate::RestHttpError;
    use crate::RestOutputFormat;
    use crate::RestResponseBody;
    use eyre::Report;
    use http::StatusCode;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[test]
    fn unsuccessful_status_remains_typed_through_context_and_output() {
        let mut response = SerializableRestResponse::new(
            StatusCode::NOT_FOUND,
            &HeaderMap::new(),
            String::from("missing"),
        );
        // A stale serialized success flag must not turn a real 404 into success.
        response.ok = true;
        let error: RestHttpError = response.error_for_status().unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        let error = Report::new(error).wrap_err("Looking up a user");
        assert_eq!(
            error.downcast_ref::<RestHttpError>().unwrap().status,
            StatusCode::NOT_FOUND
        );

        let mut output = Vec::new();
        let error = response
            .write(RestOutputFormat::Text, &mut output)
            .unwrap_err();
        assert_eq!(output, b"missing\n");
        assert_eq!(
            error.downcast_ref::<RestHttpError>().unwrap().status,
            StatusCode::NOT_FOUND
        );
    }

    fn pretty_json(input: &str) -> String {
        facet_json::to_string_pretty(&facet_json::from_str::<facet_value::Value>(input).unwrap())
            .unwrap()
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
        assert_eq!(response.status, StatusCode::OK);
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
        assert!(serialized.contains(r#""status":200"#));
        assert!(serialized.contains(r#""body":["{\n  \"hello\": \"world\"\n}"]"#));
        let reparsed: SerializableRestResponse = facet_json::from_str(&serialized)?;
        assert_eq!(reparsed.status, StatusCode::OK);

        assert_eq!(
            reparsed.into_json_body()?.as_str(),
            pretty_json(r#"{"hello":"world"}"#)
        );
        Ok(())
    }

    #[test]
    fn round_trips_non_json_bodies_as_text() -> eyre::Result<()> {
        for content in [
            "",
            "plain text with \"quotes\" and\na newline",
            "<!DOCTYPE html>\n<html><title>Synthetic sign-out page</title></html>",
        ] {
            let response = SerializableRestResponse::new(
                StatusCode::OK,
                &HeaderMap::new(),
                content.to_owned(),
            );
            let serialized = facet_json::to_string(&response)?;
            let reparsed: SerializableRestResponse = facet_json::from_str(&serialized)?;
            let RestResponseBody::Text(text) = &reparsed.body else {
                panic!("cached non-JSON content must remain a text body");
            };
            assert_eq!(text, content);
            assert!(reparsed.into_json_body().is_err());
        }
        Ok(())
    }

    #[test]
    fn round_trips_json_arrays_without_recursively_unwrapping_their_strings() -> eyre::Result<()> {
        for content in [
            r#"[]"#,
            r#"["westus"]"#,
            r#"["westus", "eastus"]"#,
            r#"["\"<html>synthetic</html>\""]"#,
            r#"["{\"nested\":true}"]"#,
            r#"[{"nested":[1,2]}]"#,
        ] {
            let response = SerializableRestResponse::new(
                StatusCode::OK,
                &HeaderMap::new(),
                content.to_owned(),
            );
            let serialized = facet_json::to_string(&response)?;
            let reparsed: SerializableRestResponse = facet_json::from_str(&serialized)?;
            assert!(matches!(reparsed.body, RestResponseBody::Json(_)));
            assert_eq!(reparsed.into_json_body()?.as_str(), pretty_json(content));
        }
        Ok(())
    }

    #[test]
    fn typed_json_decode_rejects_successful_html_without_exposing_page_contents() -> eyre::Result<()>
    {
        let content = "\n<!DOCTYPE html><html>synthetic-sensitive-page-content</html>";
        for (content_type, content) in [
            (None, content),
            (Some("text/html; charset=utf-8"), content),
            (
                Some("Application/XHTML+XML; charset=UTF-8"),
                "synthetic-sensitive-page-content",
            ),
            (
                Some("Text/HTML; charset=UTF-8"),
                "synthetic-sensitive-page-content",
            ),
            (Some("application/json"), content),
        ] {
            let mut headers = HeaderMap::new();
            if let Some(content_type) = content_type {
                headers.insert("content-type", HeaderValue::from_static(content_type));
            }
            let response =
                SerializableRestResponse::new(StatusCode::OK, &headers, content.to_owned());
            // Exercise both the fresh response and the representation loaded
            // from an existing success artifact; HTML must fail identically.
            let serialized = facet_json::to_string(&response)?;
            let reparsed: SerializableRestResponse = facet_json::from_str(&serialized)?;
            for response in [response, reparsed] {
                let error = response
                    .decode_json_body::<facet_value::Value>()
                    .unwrap_err();
                let diagnostic = format!("{error:?}");
                assert!(diagnostic.contains("HTML/XML"));
                assert!(diagnostic.contains("200"));
                assert!(diagnostic.contains("authentication"));
                assert!(!diagnostic.contains("synthetic-sensitive-page-content"));
            }
        }
        Ok(())
    }

    #[test]
    fn typed_json_decode_accepts_json_with_missing_or_nonstandard_content_type() -> eyre::Result<()>
    {
        for content_type in [
            None,
            Some("text/plain"),
            Some("application/json"),
            Some("application/vnd.synthetic+json; charset=utf-8"),
        ] {
            let mut headers = HeaderMap::new();
            if let Some(content_type) = content_type {
                headers.insert("content-type", HeaderValue::from_static(content_type));
            }
            let response = SerializableRestResponse::new(
                StatusCode::OK,
                &headers,
                facet_json::to_string(&vec!["first", "second"])?,
            );
            assert_eq!(
                response.decode_json_body::<Vec<String>>()?,
                ["first", "second"]
            );
        }
        Ok(())
    }

    #[test]
    fn typed_json_decode_rejects_malformed_json_even_with_success_status_and_json_header() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &headers,
            String::from("{\"unfinished\":"),
        );
        let error = response
            .decode_json_body::<facet_value::Value>()
            .unwrap_err();
        assert!(error.to_string().contains("Deserializing REST response"));
        assert!(
            error.chain().count() > 1,
            "preserve the JSON decoder failure"
        );
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
        for body in [r#"["westus"]"#, r#"["westus", "eastus"]"#, r#"["42"]"#] {
            let cached_response = format!(
                r#"{{
            "status": 200,
            "ok": true,
            "reason_phrase": "OK",
            "headers": {{}},
            "body": {body}
        }}"#
            );

            let response: SerializableRestResponse = facet_json::from_str(&cached_response)?;

            assert_eq!(response.into_json_body()?.as_str(), body);
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(SerializableRestResponse);
cloud_terrastodon_registry::register_arbitrary!(SerializableRestResponse);
