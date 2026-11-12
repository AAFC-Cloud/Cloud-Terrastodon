#![cfg(test)]

use cloud_terrastodon_config::Config;
use facet::Facet;
use std::path::PathBuf;

tokio::task_local! {
    // Every test owns its root. Missing test setup fails here instead of
    // falling back to the application's real configuration directory.
    static TEST_CONFIG_ROOT: PathBuf;
}

#[derive(Debug, Facet, Clone, PartialEq)]
pub struct MyTestConfigV1 {
    pub bloop: String,
}

impl Default for MyTestConfigV1 {
    fn default() -> Self {
        Self {
            bloop: "Ahoy, world!".to_string(),
        }
    }
}

#[derive(Debug, Facet, Eq, PartialEq, Clone)]
pub struct MyTestConfigV2 {
    pub bleep: bool,
    pub bloop: String,
}

impl Default for MyTestConfigV2 {
    fn default() -> Self {
        Self {
            bleep: true,
            bloop: "Ahoy, world!".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Config for MyTestConfigV1 {
    const FILE_SLUG: &'static str = "config_test";

    fn config_path() -> PathBuf {
        TEST_CONFIG_ROOT.with(|root| root.join(format!("{}.json", Self::FILE_SLUG)))
    }
}
#[async_trait::async_trait]
impl Config for MyTestConfigV2 {
    const FILE_SLUG: &'static str = "config_test";

    fn config_path() -> PathBuf {
        TEST_CONFIG_ROOT.with(|root| root.join(format!("{}.json", Self::FILE_SLUG)))
    }
}

#[tokio::test]
pub async fn test_config() -> eyre::Result<()> {
    let directory = tempfile::tempdir()?;
    TEST_CONFIG_ROOT
        .scope(directory.path().to_owned(), async {
            let config = MyTestConfigV1::load().await?;
            assert_eq!(config, MyTestConfigV1::default());
            Ok(())
        })
        .await
}

#[tokio::test]
pub async fn multi_load() -> eyre::Result<()> {
    let directory = tempfile::tempdir()?;
    TEST_CONFIG_ROOT
        .scope(directory.path().to_owned(), async {
            let config1 = MyTestConfigV1::load().await?;
            let config2 = MyTestConfigV1::load().await?;
            assert_eq!(config1, MyTestConfigV1::default());
            assert_eq!(config2, config1);
            Ok(())
        })
        .await
}

#[tokio::test]
pub async fn upgrade_works() -> eyre::Result<()> {
    let directory = tempfile::tempdir()?;
    TEST_CONFIG_ROOT
        .scope(directory.path().to_owned(), async {
            let mut config = MyTestConfigV1::load().await?;
            let user_modified_v1_config = MyTestConfigV1 {
                bloop: "bologna".to_string(),
            };
            config
                .modify_and_save(|cfg| *cfg = user_modified_v1_config.clone())
                .await?;
            drop(config);

            // A newly introduced field gets its default without overwriting
            // the user's existing value from the previous schema.
            let config = MyTestConfigV2::load().await?;
            assert_eq!(config.bloop, user_modified_v1_config.bloop);
            assert_eq!(config.bleep, MyTestConfigV2::default().bleep);
            assert_ne!(config, MyTestConfigV2::default());
            Ok(())
        })
        .await
}
