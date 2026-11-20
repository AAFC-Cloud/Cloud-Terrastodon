/// Interactive instructions for the owner of an account sign-in operation.
///
/// Prompts are short-lived credentials or authorization URLs. Present them only
/// to the user who initiated sign-in and clear them when the operation finishes.
pub enum AccountSignInPrompt {
    /// The application could not open the browser automatically.
    BrowserAuthorization { url: String },
    /// Azure CLI is waiting for an explicit or automatic device-code sign-in.
    AzureCliDeviceCode { code: String },
}

impl AccountSignInPrompt {
    /// Microsoft Entra's public-cloud device sign-in page.
    pub const AZURE_CLI_DEVICE_SIGN_IN_URL: &'static str = "https://aka.ms/devicelogin";

    pub(crate) fn print(self) {
        match self {
            Self::BrowserAuthorization { url } => {
                eprintln!(
                    "Open this URL in a browser to complete Cloud Terrastodon sign-in:\n{url}"
                );
            }
            Self::AzureCliDeviceCode { code } => {
                let url = Self::AZURE_CLI_DEVICE_SIGN_IN_URL;
                eprintln!(
                    "Open {url} in a browser and enter {code} to complete Azure CLI sign-in."
                );
            }
        }
    }
}
