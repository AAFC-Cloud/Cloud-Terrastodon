use crate::GiteaInstanceUrl;
use facet::Facet;

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct GiteaLogin {
    pub name: String,
    pub url: GiteaInstanceUrl,
    #[facet(default)]
    pub ssh_host: Option<String>,
    #[facet(default)]
    pub user: Option<String>,
    #[facet(rename = "default", default, proxy = DefaultFlagProxy)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
#[facet(untagged)]
#[repr(C)]
enum DefaultFlagProxy {
    Bool(bool),
    String(String),
    Integer(i64),
}

impl From<DefaultFlagProxy> for bool {
    fn from(value: DefaultFlagProxy) -> Self {
        match value {
            DefaultFlagProxy::Bool(value) => value,
            DefaultFlagProxy::String(value) => matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "true" | "1" | "yes"
            ),
            DefaultFlagProxy::Integer(value) => value != 0,
        }
    }
}

impl From<&bool> for DefaultFlagProxy {
    fn from(value: &bool) -> Self {
        Self::Bool(*value)
    }
}
