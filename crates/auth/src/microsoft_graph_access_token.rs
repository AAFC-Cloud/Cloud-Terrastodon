/// Sensitive bearer credential issued for Microsoft Graph PIM operations.
#[derive(Clone)]
pub struct MicrosoftGraphAccessToken(String);

impl MicrosoftGraphAccessToken {
    pub(crate) fn new(token: String) -> Self {
        Self(token)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for MicrosoftGraphAccessToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MicrosoftGraphAccessToken")
            .field("access_token", &"***redacted***")
            .finish()
    }
}
