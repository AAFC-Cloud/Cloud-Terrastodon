use eyre::Result;
use eyre::WrapErr;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum RequestHeaderValues {
    One(String),
    Many(Vec<String>),
}

impl RequestHeaderValues {
    fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            RequestHeaderValues::One(value) => Box::new(std::iter::once(value.as_str())),
            RequestHeaderValues::Many(values) => Box::new(values.iter().map(String::as_str)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RequestHeaders(BTreeMap<String, RequestHeaderValues>);

impl RequestHeaders {
    pub fn to_header_map(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (name, values) in &self.0 {
            let header_name = HeaderName::try_from(name.as_str())
                .wrap_err_with(|| format!("Invalid header name {name:?}"))?;
            for value in values.iter() {
                let header_value = HeaderValue::try_from(value)
                    .wrap_err_with(|| format!("Invalid value for header {name:?}"))?;
                headers.append(header_name.clone(), header_value);
            }
        }
        Ok(headers)
    }
}

pub async fn read_optional_headers(headers: Option<String>) -> Result<Option<RequestHeaders>> {
    let Some(headers) = headers else {
        return Ok(None);
    };

    let headers = if let Some(file_path) = headers.strip_prefix('@') {
        std::fs::read_to_string(file_path)
            .wrap_err_with(|| format!("Reading headers from {file_path}"))?
    } else {
        headers
    };

    serde_json::from_str::<RequestHeaders>(&headers)
        .map(Some)
        .wrap_err("Parsing request headers JSON")
}

#[cfg(test)]
mod tests {
    use super::RequestHeaders;
    use super::read_optional_headers;

    #[tokio::test]
    async fn parses_string_header_values() -> eyre::Result<()> {
        let headers =
            read_optional_headers(Some(r#"{"content-type":"application/json"}"#.to_string()))
                .await?
                .unwrap();
        let headers = headers.to_header_map()?;
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        Ok(())
    }

    #[tokio::test]
    async fn parses_multi_value_headers() -> eyre::Result<()> {
        let headers = read_optional_headers(Some(r#"{"x-test":["a","b"]}"#.to_string()))
            .await?
            .unwrap();
        let headers = headers.to_header_map()?;
        let values = headers
            .get_all("x-test")
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(values, vec!["a".to_string(), "b".to_string()]);
        Ok(())
    }

    #[test]
    fn deserializes_request_headers() -> eyre::Result<()> {
        let _headers = serde_json::from_str::<RequestHeaders>(
            r#"{"content-type":"application/json","x-test":["a","b"]}"#,
        )?;
        Ok(())
    }
}
