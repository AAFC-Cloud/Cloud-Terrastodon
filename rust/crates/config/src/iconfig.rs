use chrono::Utc;
use cloud_terrastodon_core_pathing::AppDir;
use eyre::Result;
use eyre::eyre;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::{self};
use std::path::PathBuf;
use tokio::fs;
use tracing::warn;
#[async_trait::async_trait]
pub trait IConfig:
    Sized + Default + std::fmt::Debug + Sync + for<'de> Deserialize<'de> + Serialize
{
    /// Unique slug (used for generating the filename).
    const FILE_SLUG: &'static str;

    /// Compute the file path where this config is stored.
    fn config_path() -> PathBuf {
        AppDir::Config.join(format!("{}.json", Self::FILE_SLUG))
    }

    /// Asynchronously load the configuration with incremental upgrading.
    async fn load() -> Result<Self> {
        let path = Self::config_path();
        let instance = if path.exists() {
            let content = fs::read_to_string(&path).await?;
            // Try to parse file into a serde_json::Value
            let user_json: Value = match serde_json::from_str(&content) {
                Ok(val) => val,
                Err(err) => {
                    warn!(
                        "Failed to load config as valid json, will make a backup and will revert to defaults. Error: {}",
                        err
                    );
                    // If we fail, backup the original file and use the default.
                    let now = Utc::now().format("%Y%m%dT%H%M%SZ");
                    let backup_path =
                        path.with_file_name(format!("{}-{}.json.bak", Self::FILE_SLUG, now));
                    fs::copy(&path, &backup_path).await?;
                    // For upgrade purposes, we use the default JSON.
                    serde_json::to_value(&Self::default())?
                }
            };

            // Get the default config as JSON.
            let default_json = serde_json::to_value(&Self::default())?;
            // Merge the user config (if present) into the default config.
            let merged_json = merge_json(default_json, user_json);
            // Deserialize the merged JSON.
            serde_json::from_value(merged_json)
                .map_err(|e| eyre!("Failed to deserialize merged config: {}", e))?
        } else {
            Self::default()
        };


        Ok(instance)
    }

    /// Asynchronously save the configuration.
    async fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).await?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    async fn modify_and_save<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> () + Send,
    {
        f(self);
        self.save().await?;
        Ok(())
    }
}

/// A recursive merge function that takes the `default` value and overrides
/// with any keys found in `user` where the key exists in both objects.
/// If both values are objects, the merge is done recursively.
fn merge_json(default: Value, user: Value) -> Value {
    match (default, user) {
        // Both default and user are objects: merge key by key.
        (Value::Object(mut default_map), Value::Object(user_map)) => {
            for (key, user_value) in user_map {
                let entry = default_map.entry(key).or_insert(Value::Null);
                *entry = merge_json(entry.take(), user_value);
            }
            Value::Object(default_map)
        }
        // In all other cases, take the user value.
        (_, user_value) => user_value,
    }
}
