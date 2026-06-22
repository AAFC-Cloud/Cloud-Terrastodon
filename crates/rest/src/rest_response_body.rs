use facet_json::RawJson;

#[derive(Debug, PartialEq)]
pub enum RestResponseBody {
    Json(RawJson<'static>),
    Text(String),
}

pub fn parse_response_body(content: String) -> RestResponseBody {
    match facet_json::from_str::<RawJson<'static>>(&content) {
        Ok(_) => RestResponseBody::Json(RawJson::from_owned(content)),
        Err(_) => RestResponseBody::Text(content),
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
        .map(|_| Some(RawJson::from_owned(content)))
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

    #[test]
    fn parses_json_response_body() {
        let body = parse_response_body("{\"hello\":\"world\"}".to_string());
        assert_eq!(
            body,
            RestResponseBody::Json(RawJson::from_owned("{\"hello\":\"world\"}".to_string()))
        );
    }

    #[test]
    fn preserves_text_response_body() {
        let body = parse_response_body("not json".to_string());
        assert_eq!(body, RestResponseBody::Text("not json".to_string()));
    }
}
