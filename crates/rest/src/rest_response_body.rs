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

#[cfg(test)]
mod tests {
    use super::RestResponseBody;
    use super::parse_response_body;

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
}
