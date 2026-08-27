use std::fmt::Debug;

/// A resource access token obtained from Entra. This type is intentionally
/// distinct from an Azure DevOps PAT so the REST layer cannot accidentally use
/// PAT Basic authentication for a federated token.
#[derive(Clone, Eq, PartialEq, facet::Facet)]
#[facet(transparent)]
pub struct AzureBearerToken(#[facet(sensitive)] String);

impl AzureBearerToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AzureBearerToken {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Debug for AzureBearerToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("AzureBearerToken")
            .field(&"***redacted***")
            .finish()
    }
}
