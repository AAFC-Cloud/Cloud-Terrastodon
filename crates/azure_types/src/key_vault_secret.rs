use crate::prelude::KeyVaultSecretId;
use crate::prelude::KeyVaultSecretName;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyVaultSecretAttributes {
    pub created: DateTime<Utc>,
    pub enabled: bool,
    #[serde(default)]
    pub expires: Option<DateTime<Utc>>,
    #[serde(default)]
    pub not_before: Option<DateTime<Utc>>,
    pub recoverable_days: i64,
    pub recovery_level: String,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyVaultSecret {
    pub attributes: KeyVaultSecretAttributes,
    #[serde(default)]
    pub content_type: Option<String>,
    pub id: KeyVaultSecretId,
    #[serde(default)]
    pub managed: Option<bool>,
    pub name: KeyVaultSecretName,
    #[serde(default)]
    pub tags: serde_json::Value, // map but keep flexible
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_sample() -> eyre::Result<()> {
        let raw = r#"{
            "attributes": {
              "created": "2025-09-02T20:03:05.761Z",
              "enabled": true,
              "expires": null,
              "notBefore": null,
              "recoverableDays": 90,
              "recoveryLevel": "Recoverable+Purgeable",
              "updated": "2025-09-02T20:03:05.761Z"
            },
            "contentType": "",
            "id": "https://my-kv-name.vault.azure.net/secrets/BruhBruhBruh",
            "managed": null,
            "name": "BruhBruhBruh",
            "tags": {}
          }"#;
        let secret: KeyVaultSecret = serde_json::from_str(raw)?;
        assert_eq!(secret.name.to_string(), "BruhBruhBruh");
        Ok(())
    }
}
