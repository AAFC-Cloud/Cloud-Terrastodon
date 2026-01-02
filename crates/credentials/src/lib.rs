//! This crate is towards avoiding the need to invoke the azure cli to perform what should be straightforward REST requests.
//! 
//! Unfortunately, the Azure CLI is useful because we can delegate all authentication to it.
//! 
//! If we take on that burden, then things get a bit more complicated.
//! 
//! A hybrid approach lets us read the credentials that Azure CLI writes in ~/.azure/msal_token_cache.bin and the Windows credential store, but this is still a high-complexity solution :(
//! 
//! For now, I think the best is to just keep using `az devops invoke` and `az rest` when the other commands fail us.
//! 
//! 
//! ...
//! 
//! not to mention that you have to pay for licenses for service principals in Azure DevOps... Stakeholder access isn't enough to read repos.
//! 
//! - https://devblogs.microsoft.com/devops/reducing-pat-usage-across-azure-devops/
//! - https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/oauth?view=azure-devops&source=recommendations
//! - https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra-oauth?view=azure-devops
//! 
//! For now (2025-08-22), I conclude that PATs are still the best mechanism for the needs of Cloud Terrastodon - a end-user CLI tool that can query anything from the portal; full access.
//! 
//! As the PAT-alternatives mature, they should be considered, but for now it's just not ready yet.
//! 
//! 
//! https://learn.microsoft.com/en-us/azure/devops/cli/log-in-via-pat?view=azure-devops&tabs=windows
//! 
//! > Set the `AZURE_DEVOPS_EXT_PAT` environment variable and run CLI commands without using az devops login.
//! 
//! I've had problems with an opaque rate limit where running 20 concurrent requests bricks the API, but running az devops login which asked for a PAT fixed it?
//! 
//! idk what to think.

mod auth_bearer_ext;
mod azure_access_token;
mod azure_claims;
mod azure_devops_pat;
mod azure_devops_rest_client;
mod azure_token_cache;
mod jwt;
mod windows_credential_manager;
pub use auth_bearer_ext::*;
pub use azure_claims::*;
pub use azure_devops_pat::*;
pub use azure_devops_rest_client::*;
pub use jwt::*;
#[cfg(windows)]
pub use windows_credential_manager::*;
