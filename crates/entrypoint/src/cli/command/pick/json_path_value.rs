use facet::Facet;
use facet_value::VString;
use facet_value::Value;
use jsonpath_rust::JsonPath;
use jsonpath_rust::query::queryable::Queryable;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// The JSONPath engine requires arrays and objects of its own queryable type.
#[derive(Debug, Clone, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub(super) enum JsonPathValue {
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
    Scalar(Value),
}

impl From<&Value> for JsonPathValue {
    fn from(value: &Value) -> Self {
        if let Some(array) = value.as_array() {
            return Self::Array(array.iter().map(Self::from).collect());
        }
        if let Some(object) = value.as_object() {
            return Self::Object(
                object
                    .iter()
                    .map(|(key, value)| (key.as_str().to_owned(), Self::from(value)))
                    .collect(),
            );
        }
        Self::Scalar(value.clone())
    }
}

impl Default for JsonPathValue {
    fn default() -> Self {
        Self::Scalar(Value::NULL)
    }
}

impl From<&str> for JsonPathValue {
    fn from(value: &str) -> Self {
        Self::Scalar(Value::from(value))
    }
}

impl From<String> for JsonPathValue {
    fn from(value: String) -> Self {
        Self::Scalar(Value::from(value))
    }
}

impl From<bool> for JsonPathValue {
    fn from(value: bool) -> Self {
        Self::Scalar(Value::from(value))
    }
}

impl From<i64> for JsonPathValue {
    fn from(value: i64) -> Self {
        Self::Scalar(Value::from(value))
    }
}

impl From<f64> for JsonPathValue {
    fn from(value: f64) -> Self {
        if value.is_finite() {
            Self::Scalar(Value::from(value))
        } else {
            Self::default()
        }
    }
}

impl From<Vec<Self>> for JsonPathValue {
    fn from(value: Vec<Self>) -> Self {
        Self::Array(value)
    }
}

impl PartialEq for JsonPathValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            (Self::Scalar(a), Self::Scalar(b)) => match (a.as_number(), b.as_number()) {
                // Preserve the engine's distinction between integers and floats
                // for its custom array-membership functions.
                (Some(a), Some(b)) => a.is_float() == b.is_float() && a == b,
                _ => a == b,
            },
            _ => false,
        }
    }
}

impl Queryable for JsonPathValue {
    fn get(&self, key: &str) -> Option<&Self> {
        let Self::Object(object) = self else {
            return None;
        };
        let key = if key.starts_with('\'') && key.ends_with('\'') {
            key.trim_matches('\'')
        } else if key.starts_with('"') && key.ends_with('"') {
            key.trim_matches('"')
        } else {
            key
        };
        object.get(key)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<Vec<(&str, &Self)>> {
        match self {
            Self::Object(object) => Some(
                object
                    .iter()
                    .map(|(key, value)| (key.as_str(), value))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Self::Scalar(value) => value.as_string().map(VString::as_str),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Scalar(value) => {
                let number = value.as_number()?;
                if number.is_integer() {
                    number.to_i64()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Scalar(value) => Some(value.as_number()?.to_f64_lossy()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Scalar(value) => value.as_bool(),
            _ => None,
        }
    }

    fn null() -> Self {
        Self::default()
    }

    fn extension_custom(name: &str, args: Vec<Cow<'_, Self>>) -> Self {
        let [left, right] = args.as_slice() else {
            return Self::default();
        };
        let Some(right) = right.as_array() else {
            return Self::default();
        };
        match name {
            "in" => Self::from(right.contains(left.as_ref())),
            "nin" => Self::from(!right.contains(left.as_ref())),
            "none_of" | "any_of" | "subset_of" => {
                let Some(left) = left.as_array() else {
                    return Self::default();
                };
                let mut matches = left.iter().map(|item| right.contains(item));
                Self::from(match name {
                    "none_of" => !matches.any(|contains| contains),
                    "any_of" => matches.any(|contains| contains),
                    _ => matches.all(|contains| contains),
                })
            }
            _ => Self::default(),
        }
    }
}

impl JsonPath for JsonPathValue {}
