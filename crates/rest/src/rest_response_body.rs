use facet_json::RawJson;

#[derive(Clone, Debug)]
pub enum RestResponseBody {
    Json(RawJson<'static>),
    Text(String),
}
impl PartialEq for RestResponseBody {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RestResponseBody::Json(a), RestResponseBody::Json(b)) => {
                facet_json::from_str::<facet_value::Value>(a.as_str()).ok()
                    == facet_json::from_str::<facet_value::Value>(b.as_str()).ok()
            }
            (RestResponseBody::Text(a), RestResponseBody::Text(b)) => {
                a == b
                    || facet_json::from_str::<facet_value::Value>(a.as_str()).ok()
                        == facet_json::from_str::<facet_value::Value>(b.as_str()).ok()
            }
            (RestResponseBody::Text(a), RestResponseBody::Json(b))
            | (RestResponseBody::Json(b), RestResponseBody::Text(a)) => {
                facet_json::from_str::<facet_value::Value>(a.as_str()).ok()
                    == facet_json::from_str::<facet_value::Value>(b.as_str()).ok()
            }
        }
    }
}

pub fn parse_response_body(content: String) -> RestResponseBody {
    match facet_json::from_str::<RawJson<'static>>(&content) {
        Ok(_) => RestResponseBody::Json(pretty_json_body(content)),
        Err(_) => RestResponseBody::Text(content),
    }
}

fn pretty_json_body(content: String) -> RawJson<'static> {
    let Ok(value) = facet_json::from_str::<facet_value::Value>(&content) else {
        return RawJson::from_owned(content);
    };
    match facet_json::to_string_pretty(&value) {
        Ok(pretty) => RawJson::from_owned(pretty),
        Err(_) => RawJson::from_owned(content),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct RestResponseBodyProxy(RawJson<'static>);

impl TryFrom<RestResponseBodyProxy> for RestResponseBody {
    type Error = eyre::Report;

    fn try_from(value: RestResponseBodyProxy) -> Result<Self, Self::Error> {
        let body = value.0;
        if body.as_str().trim_start().starts_with('"') {
            let text = facet_json::from_str::<String>(body.as_str())
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(RestResponseBody::Text(text))
        } else if let Some(body) = try_decode_serialized_json_body(body.as_str())? {
            Ok(RestResponseBody::Json(body))
        } else {
            Ok(RestResponseBody::Json(body))
        }
    }
}

fn try_decode_serialized_json_body(body: &str) -> Result<Option<RawJson<'static>>, eyre::Report> {
    if !body.trim_start().starts_with('[') {
        return Ok(None);
    }

    // Cached RawJson bodies are serialized by facet as a single-element string
    // array containing the original JSON document.
    let Ok(mut chunks) = facet_json::from_str::<Vec<String>>(body) else {
        return Ok(None);
    };
    if chunks.len() != 1 {
        return Ok(None);
    }

    let content = chunks.remove(0);
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return Ok(None);
    }

    facet_json::from_str::<RawJson<'static>>(&content)
        .map(|_| Some(pretty_json_body(content)))
        .or(Ok(None))
}

impl TryFrom<&RestResponseBody> for RestResponseBodyProxy {
    type Error = eyre::Report;

    fn try_from(value: &RestResponseBody) -> Result<Self, Self::Error> {
        match value {
            RestResponseBody::Json(value) => Ok(Self(value.clone())),
            RestResponseBody::Text(value) => {
                let json =
                    facet_json::to_string(value).map_err(|error| eyre::eyre!("{error:?}"))?;
                Ok(Self(RawJson::from_owned(json)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RestResponseBody;
    use super::parse_response_body;
    use facet_json::RawJson;

    fn pretty_json(input: &str) -> String {
        facet_json::to_string_pretty(&facet_json::from_str::<facet_value::Value>(input).unwrap())
            .unwrap()
    }

    #[test]
    fn parses_json_response_body() {
        let body = parse_response_body("{\"hello\":\"world\"}".to_string());
        assert_eq!(
            body,
            RestResponseBody::Json(RawJson::from_owned(pretty_json(r#"{"hello":"world"}"#)))
        );
    }

    #[test]
    fn preserves_text_response_body() {
        let body = parse_response_body("not json".to_string());
        assert_eq!(body, RestResponseBody::Text("not json".to_string()));
    }
}
