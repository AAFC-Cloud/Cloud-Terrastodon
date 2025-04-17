use cloud_terrastodon_core_config::iconfig::IConfig;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
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
impl IConfig for MyTestConfigV1 {
    const FILE_SLUG: &'static str = "config_test";
}
#[async_trait::async_trait]
impl IConfig for MyTestConfigV2 {
    const FILE_SLUG: &'static str = "config_test";
}

#[tokio::test]
pub async fn test_config() -> eyre::Result<()> {
    let config = MyTestConfigV1::load().await?;
    dbg!(&config);
    Ok(())
}

#[tokio::test]
pub async fn multi_load() -> eyre::Result<()> {
    let config1 = MyTestConfigV1::load().await;
    let config2 = MyTestConfigV1::load().await;
    assert!(config1.is_ok());
    assert!(config2.is_ok());
    Ok(())
}

#[tokio::test]
pub async fn upgrade_works() -> eyre::Result<()> {
    let mut config = MyTestConfigV1::load().await?;
    let user_modified_v1_config = MyTestConfigV1 {
        bloop: "bologna".to_string(),
    };
    config
        .modify_and_save(|cfg| *cfg = user_modified_v1_config.clone())
        .await?;
    drop(config);

    // pretend that we changed our struct to have a new field
    // when we load, the new field will not be present in the file
    // we want the old fields to be unmolested
    let config = MyTestConfigV2::load().await?;
    assert_eq!(config.bloop, user_modified_v1_config.bloop);
    assert_eq!(config.bleep, MyTestConfigV2::default().bleep);
    assert_ne!(config, MyTestConfigV2::default());
    Ok(())
}
