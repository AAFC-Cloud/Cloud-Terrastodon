use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct GiteaSearchResults<T> {
    #[serde(default)]
    pub data: Vec<T>,
    #[serde(default)]
    pub ok: bool,
}
