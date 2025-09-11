use serde::Deserialize;
use serde::Serialize;

/// Shared "All" privilege used across key/secret/certificate access policy privilege enums
/// to reduce duplication. Deserializes case-insensitively for common variants of "All".
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub enum KeyVaultAccessPolicyAllPrivilege {
    #[serde(alias = "All", alias = "all", alias = "ALL")]
    #[default]
    All,
}
