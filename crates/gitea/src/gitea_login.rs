use crate::GiteaInstanceUrl;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GiteaLogin {
    pub name: String,
    pub url: GiteaInstanceUrl,
    #[serde(default)]
    pub ssh_host: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(
        rename = "default",
        default,
        deserialize_with = "deserialize_default_flag"
    )]
    pub is_default: bool,
}

fn deserialize_default_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawFlag {
        Bool(bool),
        String(String),
        Integer(i64),
    }

    let raw = RawFlag::deserialize(deserializer)?;
    Ok(match raw {
        RawFlag::Bool(value) => value,
        RawFlag::String(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        ),
        RawFlag::Integer(value) => value != 0,
    })
}
