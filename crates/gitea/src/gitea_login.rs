use crate::GiteaInstanceUrl;
use facet::Facet;
use facet_json::RawJson;

#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct GiteaLogin {
    pub name: String,
    pub url: GiteaInstanceUrl,
    #[facet(default)]
    pub ssh_host: Option<String>,
    #[facet(default)]
    pub user: Option<String>,
    // https://github.com/facet-rs/facet/issues/2363
    #[facet(rename = "default", default = default_default_flag_raw_json())]
    pub is_default: RawJson<'static>,
}

impl GiteaLogin {
    pub fn is_default(&self) -> bool {
        is_default_flag(self.is_default.as_str())
    }
}

fn default_default_flag_raw_json() -> RawJson<'static> {
    RawJson::from_owned("false".to_string())
}

fn is_default_flag(value: &str) -> bool {
    let value = value.trim();
    let value = value.strip_prefix('"').unwrap_or(value);
    let value = value.strip_suffix('"').unwrap_or(value);
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes"
    )
}

cloud_terrastodon_registry::register_thing!(GiteaLogin);

#[cfg(test)]
mod tests {
    use super::GiteaLogin;

    #[test]
    fn it_deserializes_tea_default_flag_as_string() -> eyre::Result<()> {
        let json = r#"{
            "name": "example",
            "url": "https://gitea.example.com",
            "ssh_host": "gitea.example.com",
            "user": "redac",
            "default": "true"
        }"#;

        let login = facet_json::from_str::<GiteaLogin>(json)?;

        assert!(login.is_default());
        Ok(())
    }

    #[test]
    fn it_deserializes_tea_default_flag_false_string() -> eyre::Result<()> {
        let login = login_from_default_value(r#""false""#)?;

        assert!(!login.is_default());
        Ok(())
    }

    #[test]
    fn it_deserializes_default_flag_as_bool() -> eyre::Result<()> {
        let login = login_from_default_value("true")?;

        assert!(login.is_default());
        Ok(())
    }

    #[test]
    fn it_deserializes_default_flag_as_integer() -> eyre::Result<()> {
        let login = login_from_default_value("1")?;

        assert!(login.is_default());
        Ok(())
    }

    #[test]
    fn it_defaults_missing_default_flag_to_false() -> eyre::Result<()> {
        let login = facet_json::from_str::<GiteaLogin>(
            r#"{
                "name": "agr",
                "url": "https://gitea.example.com"
            }"#,
        )?;

        assert!(!login.is_default());
        Ok(())
    }

    fn login_from_default_value(default_value: &str) -> eyre::Result<GiteaLogin> {
        facet_json::from_str::<GiteaLogin>(&format!(
            r#"{{
                "name": "agr",
                "url": "https://gitea.example.com",
                "default": {default_value}
            }}"#
        ))
        .map_err(Into::into)
    }
}

