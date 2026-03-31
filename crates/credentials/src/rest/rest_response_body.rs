use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RestResponseBody {
    Json(Value),
    Text(String),
}

pub fn parse_response_body(content: String) -> RestResponseBody {
    match serde_json::from_str::<Value>(&content) {
        Ok(value) => RestResponseBody::Json(value),
        Err(_) => RestResponseBody::Text(content),
    }
}
