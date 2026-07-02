use chrono::Utc;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::eyre;
use facet_value::Destructured;
use facet_value::Value;
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;
use tracing::warn;
#[async_trait::async_trait]
pub trait Config:
    Sized
    + Default
    + std::fmt::Debug
    + Sync
    + facet::Facet<'static>
    + Clone
    + Send
    + 'static
    + PartialEq
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
            let user_json: Value = match facet_json::from_str(&content) {
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
                    facet_value::to_value(&Self::default())?
                }
            };

            // Get the default config as JSON.
            let default_json = facet_value::to_value(&Self::default())?;
            // Merge the user config (if present) into the default config.
            let merged_json = merge_json(default_json, user_json);
            // Decode the merged JSON-shaped value.
            facet_value::from_value(merged_json)
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
        let content = facet_json::to_string_pretty(self)?;
        debug!("Writing config to {:?}", path);
        fs::write(&path, content).await?;
        Ok(())
    }

    async fn modify_and_save<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) + Send,
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
    match (default.destructure(), user.destructure()) {
        // Both default and user are objects: merge key by key.
        (Destructured::Object(mut default_map), Destructured::Object(user_map)) => {
            for (key, user_value) in user_map {
                let entry = default_map.remove(key.as_str()).unwrap_or(Value::NULL);
                default_map.insert(key, merge_json(entry, user_value));
            }
            default_map.into()
        }
        // In all other cases, take the user value.
        (_, user_value) => destructured_to_value(user_value),
    }
}

fn destructured_to_value(value: Destructured) -> Value {
    match value {
        Destructured::Null => Value::NULL,
        Destructured::Bool(value) => value.into(),
        Destructured::Number(value) => value.into(),
        Destructured::String(value) => value.into(),
        Destructured::Bytes(value) => value.into(),
        Destructured::Array(value) => value.into(),
        Destructured::Object(value) => value.into(),
        Destructured::DateTime(value) => value.into(),
        Destructured::QName(value) => value.into(),
        Destructured::Uuid(value) => value.into(),
        Destructured::Char(value) => value.into(),
        _ => unreachable!("unknown facet-value::Destructured variant"),
    }
}
