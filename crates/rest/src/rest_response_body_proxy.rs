use crate::RestResponseBody;
use crate::rest_response_body::pretty_json_body;
use eyre::WrapErr;
use facet::Facet;
use facet_json::RawJson;

/// Preserves the existing raw-body representation used by response cache artifacts.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(transparent)]
pub struct RestResponseBodyProxy(RawJson<'static>);

impl TryFrom<RestResponseBodyProxy> for RestResponseBody {
    type Error = eyre::Report;

    fn try_from(value: RestResponseBodyProxy) -> Result<Self, Self::Error> {
        let body = try_decode_serialized_body(value.0.as_str()).unwrap_or(value.0);
        if body.as_str().trim_start().starts_with('"') {
            let text = facet_json::from_str::<String>(body.as_str())
                .wrap_err("Decoding cached REST text body")?;
            Ok(RestResponseBody::Text(text))
        } else {
            Ok(RestResponseBody::Json(body))
        }
    }
}

fn try_decode_serialized_body(body: &str) -> Option<RawJson<'static>> {
    if !body.trim_start().starts_with('[') {
        return None;
    }

    // Facet serializes cached RawJson as a one-string array containing the
    // original JSON document. Text bodies use a JSON-quoted string inside that
    // envelope, so unwrap before deciding whether the response was text or JSON.
    let Ok(mut chunks) = facet_json::from_str::<Vec<String>>(body) else {
        return None;
    };
    if chunks.len() != 1 {
        return None;
    }

    let content = chunks.remove(0);
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') && !trimmed.starts_with('"') {
        // Preserve older cache artifacts that stored ordinary JSON string
        // arrays directly rather than using the RawJson envelope.
        return None;
    }

    facet_json::from_str::<RawJson<'static>>(&content).ok()?;
    Some(pretty_json_body(content))
}

impl TryFrom<&RestResponseBody> for RestResponseBodyProxy {
    type Error = eyre::Report;

    fn try_from(value: &RestResponseBody) -> Result<Self, Self::Error> {
        match value {
            RestResponseBody::Json(value) => Ok(Self(value.clone())),
            RestResponseBody::Text(value) => {
                let json =
                    facet_json::to_string(value).wrap_err("Encoding REST text body for cache")?;
                Ok(Self(RawJson::from_owned(json)))
            }
        }
    }
}
