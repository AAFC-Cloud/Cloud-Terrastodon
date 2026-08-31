use crate::browser_access_token::fetch_browser_access_token_with_scopes;
use crate::browser_access_token::form_urlencoded;
use crate::browser_access_token::refresh_browser_access_token;
use crate::pim_client_id;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use reqwest::Client;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;

const PIM_SCOPES_ENV: &str = "CLOUD_TERRASTODON_PIM_GRAPH_SCOPES";
const PIM_AUTH_FLOW_ENV: &str = "CLOUD_TERRASTODON_PIM_AUTH_FLOW";

pub const PIM_APPLICATION_DISPLAY_NAME: &str = "Cloud Terrastodon PIM";

// These are the delegated permissions required by the legacy Entra-role PIM
// endpoints currently used by Cloud Terrastodon. The newer
// roleAssignmentScheduleRequests endpoint uses RoleAssignmentSchedule.* and
// RoleManagement.* instead; callers can select those with PIM_SCOPES_ENV.
pub const DEFAULT_PIM_GRAPH_SCOPES: &[&str] = &[
    "https://graph.microsoft.com/User.Read",
    "https://graph.microsoft.com/PrivilegedAccess.Read.AzureAD",
    "https://graph.microsoft.com/PrivilegedAccess.ReadWrite.AzureAD",
    "offline_access",
];

#[cfg(windows)]
const PIM_REFRESH_TOKEN_TARGET_PREFIX: &str = "cloud-terrastodon:pim:refresh-token";

#[derive(Clone)]
pub struct MicrosoftGraphAccessToken(String);

impl MicrosoftGraphAccessToken {
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

#[derive(Debug, Facet)]
struct DeviceCodeResponse {
    #[facet(sensitive)]
    device_code: String,
    #[facet(sensitive)]
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    message: Option<String>,
}

#[derive(Debug, Facet)]
struct TokenResponse {
    #[facet(sensitive)]
    access_token: Option<String>,
    #[facet(sensitive)]
    refresh_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Acquire a delegated Graph token using the interactive browser flow.
///
/// Set `CLOUD_TERRASTODON_PIM_AUTH_FLOW=device_code` to use the explicitly
/// named device-code fallback instead. The browser flow is the default because
/// device-code authentication cannot satisfy this tenant's managed-device
/// Conditional Access policy.
pub async fn fetch_pim_graph_access_token(
    tenant_id: AzureTenantId,
) -> Result<MicrosoftGraphAccessToken> {
    let client_id = pim_client_id(&tenant_id).await?;
    let scopes = pim_graph_scopes();

    if let Some(refresh_token) = load_pim_refresh_token(&tenant_id, &client_id)? {
        match refresh_browser_access_token(
            tenant_id,
            client_id,
            crate::AzureRestResource::MicrosoftGraph,
            &scopes,
            &refresh_token,
        )
        .await
        {
            Ok(token) => {
                if let Some(next_refresh_token) = token.refresh_token {
                    store_pim_refresh_token(&tenant_id, &client_id, &next_refresh_token)?;
                }
                return Ok(MicrosoftGraphAccessToken(
                    token.access_token.access_token.as_str().to_owned(),
                ));
            }
            Err(error) if refresh_token_needs_interactive_login(&error) => {
                debug!(error = ?error, "Stored Microsoft Graph refresh token was rejected; deleting it");
                delete_pim_refresh_token(&tenant_id, &client_id);
            }
            Err(error) => return Err(error),
        }
    }

    if std::env::var(PIM_AUTH_FLOW_ENV).as_deref() == Ok("device_code") {
        fetch_pim_graph_access_token_device_code(tenant_id).await
    } else {
        fetch_pim_graph_access_token_interactive(tenant_id).await
    }
}

/// Acquire a delegated Graph token through a localhost OAuth authorization-code
/// callback, equivalent to Azure CLI's non-device interactive login path.
pub async fn fetch_pim_graph_access_token_interactive(
    tenant_id: AzureTenantId,
) -> Result<MicrosoftGraphAccessToken> {
    let client_id = pim_client_id(&tenant_id).await?;
    let scopes = pim_graph_scopes();
    let token = fetch_browser_access_token_with_scopes(
        tenant_id,
        client_id,
        crate::AzureRestResource::MicrosoftGraph,
        &scopes,
    )
    .await?;
    if let Some(refresh_token) = token.refresh_token {
        store_pim_refresh_token(&tenant_id, &client_id, &refresh_token)?;
    }
    Ok(MicrosoftGraphAccessToken(
        token.access_token.access_token.as_str().to_owned(),
    ))
}

/// Acquire a delegated Microsoft Graph token using the app registration's
/// public-client device-code flow.
pub async fn fetch_pim_graph_access_token_device_code(
    tenant_id: AzureTenantId,
) -> Result<MicrosoftGraphAccessToken> {
    let client_id = pim_client_id(&tenant_id).await?;
    let scopes = pim_graph_scopes();
    let authority = format!("https://login.microsoftonline.com/{tenant_id}");
    let client = Client::new();

    let device_code_url = format!("{authority}/oauth2/v2.0/devicecode");
    let device_code_response = client
        .post(device_code_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("client_id", client_id.to_string().as_str()),
            ("scope", scopes.as_str()),
        ]))
        .send()
        .await?;
    let device_code_status = device_code_response.status();
    let device_code_body = device_code_response.text().await?;
    if !device_code_status.is_success() {
        bail!(
            "Microsoft Graph device-code request failed with {device_code_status}: {}",
            oauth_error_description(&device_code_body)
        );
    }
    let device_code: DeviceCodeResponse = facet_json::from_str(&device_code_body)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("deserializing the Microsoft Graph device-code response")?;

    let fallback_message = format!(
        "Open {} and enter code {} to sign in.",
        device_code.verification_uri, device_code.user_code
    );
    println!(
        "{}",
        device_code.message.as_deref().unwrap_or(&fallback_message)
    );

    let token_url = format!("{authority}/oauth2/v2.0/token");
    let deadline = Instant::now() + Duration::from_secs(device_code.expires_in);
    let mut poll_interval = Duration::from_secs(device_code.interval.max(1));

    loop {
        if Instant::now() >= deadline {
            bail!("Microsoft Graph device-code authentication expired before sign-in completed");
        }
        tokio::time::sleep(poll_interval).await;

        let token_response = client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_urlencoded(&[
                ("client_id", client_id.to_string().as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", device_code.device_code.as_str()),
            ]))
            .send()
            .await?;
        let token_status = token_response.status();
        let token_body = token_response.text().await?;
        let token: TokenResponse = facet_json::from_str(&token_body)
            .map_err(|error| eyre::eyre!("{error:?}"))
            .wrap_err("deserializing the Microsoft Graph token response")?;

        if let Some(access_token) = token.access_token {
            if let Some(refresh_token) = token.refresh_token {
                store_pim_refresh_token(&tenant_id, &client_id, &refresh_token)?;
            }
            return Ok(MicrosoftGraphAccessToken(access_token));
        }

        match token.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                poll_interval += Duration::from_secs(5);
            }
            Some(error) => bail!(
                "Microsoft Graph device-code authentication failed ({error}): {}",
                token
                    .error_description
                    .as_deref()
                    .unwrap_or("no description")
            ),
            None if !token_status.is_success() => bail!(
                "Microsoft Graph token request failed with {token_status}: {}",
                oauth_error_description(&token_body)
            ),
            None => bail!("Microsoft Graph token response did not contain an access token"),
        }
    }
}

fn pim_graph_scopes() -> String {
    let mut scopes =
        std::env::var(PIM_SCOPES_ENV).unwrap_or_else(|_| DEFAULT_PIM_GRAPH_SCOPES.join(" "));
    if !scopes
        .split_whitespace()
        .any(|scope| scope == "offline_access")
    {
        scopes.push_str(" offline_access");
    }
    scopes
}

fn refresh_token_needs_interactive_login(error: &eyre::Report) -> bool {
    let error = error.to_string();
    [
        "(invalid_grant)",
        "(invalid_scope)",
        "(interaction_required)",
        "(login_required)",
        "(consent_required)",
    ]
    .iter()
    .any(|marker| error.contains(marker))
}

#[cfg(windows)]
fn pim_refresh_token_target(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
) -> Result<crate::WindowsCredentialManagerTargetName> {
    crate::WindowsCredentialManagerTargetName::try_new(format!(
        "{PIM_REFRESH_TOKEN_TARGET_PREFIX}:{tenant_id}:{client_id}"
    ))
}

#[cfg(windows)]
fn load_pim_refresh_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
) -> Result<Option<String>> {
    let target = pim_refresh_token_target(tenant_id, client_id)?;
    crate::read_credential_from_manager(&target)
}

#[cfg(not(windows))]
fn load_pim_refresh_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
) -> Result<Option<String>> {
    Ok(crate::browser_access_token::load_browser_session()?
        .filter(|session| session.tenant_id == *tenant_id && session.client_id == *client_id)
        .map(|session| session.refresh_token))
}

#[cfg(windows)]
fn store_pim_refresh_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
    refresh_token: &str,
) -> Result<()> {
    let target = pim_refresh_token_target(tenant_id, client_id)?;
    crate::write_credential_to_manager(&target, refresh_token)
        .wrap_err("persisting the Microsoft Graph refresh token in Windows Credential Manager")
}

#[cfg(not(windows))]
fn store_pim_refresh_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
    refresh_token: &str,
) -> Result<()> {
    crate::browser_access_token::store_browser_session(
        &crate::browser_access_token::BrowserSession {
            tenant_id: *tenant_id,
            client_id: *client_id,
            refresh_token: refresh_token.to_owned(),
        },
    )
}

#[cfg(windows)]
fn delete_pim_refresh_token(tenant_id: &AzureTenantId, client_id: &EntraApplicationClientId) {
    let target = match pim_refresh_token_target(tenant_id, client_id) {
        Ok(target) => target,
        Err(error) => {
            debug!(
                ?error,
                "Could not construct the rejected Microsoft Graph refresh token target"
            );
            return;
        }
    };
    if let Err(error) = crate::delete_credential_from_manager(&target) {
        debug!(
            ?error,
            "Could not delete the rejected Microsoft Graph refresh token"
        );
    }
}

#[cfg(not(windows))]
fn delete_pim_refresh_token(tenant_id: &AzureTenantId, client_id: &EntraApplicationClientId) {
    crate::browser_access_token::delete_browser_session(*tenant_id, *client_id);
}

fn oauth_error_description(body: &str) -> String {
    facet_json::from_str::<TokenResponse>(body)
        .ok()
        .and_then(|response| response.error_description)
        .unwrap_or_else(|| "no description".to_string())
}
