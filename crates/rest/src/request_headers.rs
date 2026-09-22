use eyre::Result;
use eyre::WrapErr;
use facet_json::RawJson;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHeaders(BTreeMap<String, RequestHeaderValues>);

impl RequestHeaders {
    /// Builds a header collection containing one validated header.
    pub fn from_header(name: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let value = value.into();
        HeaderName::try_from(name.as_str())
            .wrap_err_with(|| format!("Invalid header name {name:?}"))?;
        HeaderValue::try_from(value.as_str())
            .wrap_err_with(|| format!("Invalid value for header {name:?}"))?;
        let mut parsed = Self(BTreeMap::new());
        parsed.append(name, value);
        Ok(parsed)
    }

    pub fn from_json_str(headers: &str) -> Result<Self> {
        let raw_headers = facet_json::from_str::<BTreeMap<String, RawJson<'static>>>(headers)
            .map_err(|error| eyre::eyre!("{error:?}"))
            .wrap_err("Parsing request headers JSON")?;
        let mut parsed = BTreeMap::new();
        for (name, values) in raw_headers {
            let values = if let Ok(value) = facet_json::from_str::<String>(values.as_str()) {
                RequestHeaderValues::One(value)
            } else {
                let values = facet_json::from_str::<Vec<String>>(values.as_str())
                    .map_err(|error| eyre::eyre!("{error:?}"))
                    .wrap_err_with(|| format!("Parsing values for header {name:?}"))?;
                RequestHeaderValues::Many(values)
            };
            parsed.insert(name, values);
        }
        Ok(Self(parsed))
    }

    /// Parses request headers in the conventional `name: value` form.
    pub fn from_header_lines<'a, I>(headers: I) -> Result<Option<Self>>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut parsed = Self(BTreeMap::new());

        for header in headers {
            let (name, value) = header.split_once(':').ok_or_else(|| {
                eyre::eyre!("Invalid request header {header:?}; expected the format 'name: value'")
            })?;
            let name = name.trim();
            let value = value.trim();

            HeaderName::try_from(name).wrap_err_with(|| format!("Invalid header name {name:?}"))?;
            HeaderValue::try_from(value)
                .wrap_err_with(|| format!("Invalid value for header {name:?}"))?;
            parsed.append(name.to_owned(), value.to_owned());
        }

        Ok((!parsed.0.is_empty()).then_some(parsed))
    }

    /// Appends all values from `other`, retaining repeated header values.
    pub fn merge(mut self, other: Self) -> Self {
        for (name, values) in other.0 {
            for value in values.iter() {
                self.append(name.clone(), value.to_owned());
            }
        }
        self
    }

    fn append(&mut self, name: String, value: String) {
        match self.0.entry(name) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(RequestHeaderValues::One(value));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                match values {
                    RequestHeaderValues::One(existing) => {
                        let existing = std::mem::take(existing);
                        *values = RequestHeaderValues::Many(vec![existing, value]);
                    }
                    RequestHeaderValues::Many(values) => values.push(value),
                }
            }
        }
    }

    pub fn to_json_pretty(&self) -> Result<String> {
        let values = self
            .0
            .iter()
            .map(|(name, values)| {
                let values = match values {
                    RequestHeaderValues::One(value) => vec![value.clone()],
                    RequestHeaderValues::Many(values) => values.clone(),
                };
                (name.clone(), values)
            })
            .collect::<BTreeMap<_, _>>();
        facet_json::to_string_pretty(&values).map_err(|error| eyre::eyre!("{error:?}"))
    }

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

    RequestHeaders::from_json_str(&headers).map(Some)
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

    #[test]
    fn builds_a_single_header() -> eyre::Result<()> {
        let headers = RequestHeaders::from_header("Content-Type", "application/json-patch+json")?;
        assert_eq!(
            headers.to_header_map()?.get("content-type").unwrap(),
            "application/json-patch+json"
        );
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
    fn parses_repeated_header_lines() -> eyre::Result<()> {
        let headers = RequestHeaders::from_header_lines([
            "ConsistencyLevel: eventual",
            "Accept: application/json",
            "X-Test: one",
            "X-Test: two",
        ])?
        .unwrap()
        .to_header_map()?;

        assert_eq!(headers.get("consistencylevel").unwrap(), "eventual");
        assert_eq!(headers.get("accept").unwrap(), "application/json");
        let values = headers
            .get_all("x-test")
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values, vec!["one", "two"]);
        Ok(())
    }

    #[test]
    fn rejects_header_without_separator() {
        let error = RequestHeaders::from_header_lines(["missing-separator"]).unwrap_err();
        assert!(error.to_string().contains("name: value"));
    }
}
