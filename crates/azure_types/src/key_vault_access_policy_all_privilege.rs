use serde::{Deserialize, Serialize};

/// Shared "All" privilege used across key/secret/certificate access policy privilege enums
/// to reduce duplication. Deserializes case-insensitively for common variants of "All".
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum KeyVaultAccessPolicyAllPrivilege {
    #[serde(alias = "All", alias = "all", alias = "ALL")]
    All,
}

impl Default for KeyVaultAccessPolicyAllPrivilege {
    fn default() -> Self { Self::All }
}