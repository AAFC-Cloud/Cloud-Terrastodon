#![allow(
    clippy::disallowed_types,
    reason = "The tea login wire type and its generated Facet code retain bool, string, and number default-flag forms without an Azure domain dependency."
)]

use crate::GiteaInstanceUrl;
use arbitrary::Arbitrary;
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

impl<'a> Arbitrary<'a> for GiteaLogin {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let is_default = bool::arbitrary(u)?;
        Ok(Self {
            name: String::arbitrary(u)?,
            url: GiteaInstanceUrl::arbitrary(u)?,
            ssh_host: Option::<String>::arbitrary(u)?,
            user: Option::<String>::arbitrary(u)?,
            is_default: RawJson::from_owned(is_default.to_string()),
        })
    }
}

impl GiteaLogin {
    /// Select one explicit external profile for an exact instance URL.
    /// Defaults are deliberately irrelevant when a service instance is supplied.
    pub fn for_instance(
        logins: impl IntoIterator<Item = Self>,
        instance: &GiteaInstanceUrl,
    ) -> eyre::Result<Self> {
        let mut matches = logins.into_iter().filter(|login| login.url == *instance);
        let login = matches
            .next()
            .ok_or_else(|| eyre::eyre!("No tea login matches the selected Gitea instance"))?;
        eyre::ensure!(
            matches.next().is_none(),
            "Multiple tea logins match the selected Gitea instance; configure a unique profile"
        );
        eyre::ensure!(
            !login.name.is_empty(),
            "The matching tea login has an empty profile name"
        );
        Ok(login)
    }

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
cloud_terrastodon_registry::register_arbitrary!(GiteaLogin);
cloud_terrastodon_registry::register_arbitrary!(Vec<GiteaLogin>);

#[cfg(test)]
mod tests {
    use super::GiteaLogin;
    use crate::GiteaInstanceUrl;
    use facet_json::RawJson;

    #[test]
    fn instance_selection_never_falls_back_to_an_unrelated_default_or_ambiguous_profile()
    -> eyre::Result<()> {
        let selected: GiteaInstanceUrl = "https://selected.example.invalid".parse()?;
        let other: GiteaInstanceUrl = "https://other.example.invalid".parse()?;
        let profile = |name: &str, url: GiteaInstanceUrl, default: bool| GiteaLogin {
            name: name.to_owned(),
            url,
            ssh_host: None,
            user: None,
            is_default: RawJson::from_owned(default.to_string()),
        };
        let unrelated_default = profile("unrelated", other, true);
        assert!(GiteaLogin::for_instance([unrelated_default.clone()], &selected).is_err());
        let matching = profile("matching", selected.clone(), false);
        assert_eq!(
            GiteaLogin::for_instance([unrelated_default, matching.clone()], &selected)?.name,
            "matching"
        );
        assert!(
            GiteaLogin::for_instance(
                [matching, profile("second", selected.clone(), true)],
                &selected
            )
            .is_err()
        );
        assert!(
            GiteaLogin::for_instance([profile("", selected.clone(), true)], &selected).is_err()
        );
        Ok(())
    }

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
