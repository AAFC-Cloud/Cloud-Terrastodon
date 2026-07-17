use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use opener::open_browser;
use reqwest::Client;
use reqwest::Url;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;
use std::time::Duration;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tracing::debug;
use tracing::instrument;
use uuid::Uuid;

const PIM_CLIENT_ID_ENV: &str = "CLOUD_TERRASTODON_PIM_CLIENT_ID";
const PIM_SCOPES_ENV: &str = "CLOUD_TERRASTODON_PIM_GRAPH_SCOPES";
const PIM_AUTH_FLOW_ENV: &str = "CLOUD_TERRASTODON_PIM_AUTH_FLOW";

// These are the delegated permissions required by the legacy Entra-role PIM
// endpoints currently used by Cloud Terrastodon. The newer
// roleAssignmentScheduleRequests endpoint uses RoleAssignmentSchedule.* and
// RoleManagement.* instead; callers can select those with PIM_SCOPES_ENV.
const DEFAULT_PIM_GRAPH_SCOPES: &[&str] = &[
    "https://graph.microsoft.com/User.Read",
    "https://graph.microsoft.com/PrivilegedAccess.Read.AzureAD",
    "https://graph.microsoft.com/PrivilegedAccess.ReadWrite.AzureAD",
    "offline_access",
];

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
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    message: Option<String>,
}

#[derive(Debug, Facet)]
struct TokenResponse {
    access_token: Option<String>,
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
    let client_id = pim_client_id()?;
    let scopes = pim_graph_scopes();

    if let Some(refresh_token) = load_pim_refresh_token(&tenant_id, &client_id)? {
        match refresh_pim_graph_access_token(&tenant_id, &client_id, &scopes, &refresh_token).await
        {
            Ok(access_token) => return Ok(access_token),
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
#[instrument(level = "debug")]
pub async fn fetch_pim_graph_access_token_interactive(
    tenant_id: AzureTenantId,
) -> Result<MicrosoftGraphAccessToken> {
    debug!("Starting Microsoft Graph interactive authentication");
    let client_id = pim_client_id()?;
    let scopes = pim_graph_scopes();
    let authority = format!("https://login.microsoftonline.com/{tenant_id}");
    let redirect_uri;
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .wrap_err("binding the local OAuth callback listener")?;
    redirect_uri = format!(
        "http://localhost:{}",
        listener
            .local_addr()
            .wrap_err("getting the local OAuth callback address")?
            .port()
    );
    let state = Uuid::new_v4().to_string();
    let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    let authorization_url = format!(
        "{authority}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&state={}&code_challenge={}&code_challenge_method=S256&prompt=select_account",
        url_encode(&client_id.to_string()),
        url_encode(&redirect_uri),
        url_encode(&scopes),
        url_encode(&state),
        url_encode(&code_challenge),
    );

    let callback = Box::pin(wait_for_authorization_code(listener, state));
    if let Err(error) = open_browser(&authorization_url) {
        eprintln!(
            "Could not open the browser automatically ({error}); open this URL manually:\n{authorization_url}"
        );
    }
    let authorization_code = callback.await?;
    debug!("Received authorization code from browser callback");

    let client = Client::new();
    let token_url = format!("{authority}/oauth2/v2.0/token");
    debug!(%token_url, "Requesting Microsoft Graph access token");
    let token_response = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("client_id", client_id.to_string().as_str()),
            ("grant_type", "authorization_code"),
            ("code", authorization_code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", code_verifier.as_str()),
        ]))
        .send()
        .await?;
    let token_status = token_response.status();
    let token_body = token_response.text().await?;
    debug!("Received Microsoft Graph token response");
    let token: TokenResponse = facet_json::from_str(&token_body)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("deserializing the Microsoft Graph interactive token response")?;
    if let Some(access_token) = token.access_token {
        if let Some(refresh_token) = token.refresh_token {
            store_pim_refresh_token(&tenant_id, &client_id, &refresh_token)?;
        }
        return Ok(MicrosoftGraphAccessToken(access_token));
    }
    if token.error.as_deref() == Some("invalid_client") {
        bail!(
            "Microsoft Graph rejected the PIM app as a confidential client. Register http://localhost under the app registration's PublicClient redirect URIs (not Web), and keep public-client flow enabled."
        );
    }
    if let Some(error) = token.error {
        bail!(
            "Microsoft Graph interactive authentication failed ({error}): {}",
            token
                .error_description
                .as_deref()
                .unwrap_or("no description")
        );
    }
    bail!(
        "Microsoft Graph interactive token request failed with {token_status}: {}",
        oauth_error_description(&token_body)
    );
}

/// Acquire a delegated Microsoft Graph token using the app registration's
/// public-client device-code flow.
pub async fn fetch_pim_graph_access_token_device_code(
    tenant_id: AzureTenantId,
) -> Result<MicrosoftGraphAccessToken> {
    let client_id = pim_client_id()?;
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

#[instrument(level = "debug", skip_all)]
async fn wait_for_authorization_code(
    listener: TcpListener,
    expected_state: String,
) -> Result<String> {
    debug!(
        "Waiting for the browser OAuth callback on {}",
        listener
            .local_addr()
            .wrap_err("getting the local OAuth callback address")?
    );
    let accepted = timeout(Duration::from_secs(600), listener.accept())
        .await
        .wrap_err("waiting for the browser OAuth callback")?;
    let (mut stream, _) = accepted?;
    let mut request_buffer = vec![0_u8; 16 * 1024];
    let request_length = timeout(Duration::from_secs(30), stream.read(&mut request_buffer))
        .await
        .wrap_err("reading the browser OAuth callback")??;
    let request = String::from_utf8_lossy(&request_buffer[..request_length]);
    let request_target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| {
            eyre::eyre!("browser OAuth callback did not contain an HTTP request target")
        })?;
    let callback_url = Url::parse(&format!("http://localhost{request_target}"))
        .wrap_err("parsing the browser OAuth callback URL")?;
    let query = callback_url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    let query_value = |name: &str| {
        query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };

    debug!("Received browser OAuth callback");
    if query_value("error").is_some() {
        write_callback_response(&mut stream, false).await?;
        bail!(
            "Microsoft Graph browser authentication failed ({}): {}",
            query_value("error").unwrap_or("unknown error"),
            query_value("error_description").unwrap_or("no description")
        );
    }
    if query_value("state") != Some(expected_state.as_str()) {
        write_callback_response(&mut stream, false).await?;
        bail!("Microsoft Graph browser authentication returned an invalid state");
    }
    let code = query_value("code").map(str::to_owned).ok_or_else(|| {
        eyre::eyre!("Microsoft Graph browser authentication callback did not contain a code")
    })?;
    write_callback_response(&mut stream, true).await?;
    Ok(code)
}

async fn write_callback_response(stream: &mut tokio::net::TcpStream, success: bool) -> Result<()> {
    let body = if success {
        "<!doctype html><title>Cloud Terrastodon</title><p>Authentication complete. You can close this tab.</p>"
    } else {
        "<!doctype html><title>Cloud Terrastodon</title><p>Authentication failed. You can close this tab.</p>"
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await?;
    Ok(())
}

fn pim_client_id() -> Result<EntraApplicationClientId> {
    let client_id = std::env::var(PIM_CLIENT_ID_ENV).with_context(|| {
        format!(
            "{PIM_CLIENT_ID_ENV} is not set; set it to the Application (client) ID of the Cloud Terrastodon PIM app registration"
        )
    })?;
    if client_id.trim().is_empty() {
        bail!("{PIM_CLIENT_ID_ENV} must not be empty");
    }
    Ok(client_id.parse()?)
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

async fn refresh_pim_graph_access_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
    scopes: &str,
    refresh_token: &str,
) -> Result<MicrosoftGraphAccessToken> {
    let authority = format!("https://login.microsoftonline.com/{tenant_id}");
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let client = Client::new();
    let token_response = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("client_id", client_id.to_string().as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", scopes),
        ]))
        .send()
        .await
        .wrap_err("requesting a new Microsoft Graph access token from the stored refresh token")?;
    let token_status = token_response.status();
    let token_body = token_response.text().await?;
    let token: TokenResponse = facet_json::from_str(&token_body)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("deserializing the Microsoft Graph refresh-token response")?;

    if let Some(access_token) = token.access_token {
        if let Some(next_refresh_token) = token.refresh_token {
            store_pim_refresh_token(tenant_id, client_id, &next_refresh_token)?;
        }
        return Ok(MicrosoftGraphAccessToken(access_token));
    }

    let error = token.error.unwrap_or_else(|| "unknown_error".to_string());
    let description = token
        .error_description
        .unwrap_or_else(|| "no description".to_string());
    bail!(
        "Microsoft Graph refresh-token request failed with {token_status} ({error}): {description}"
    );
}

fn refresh_token_needs_interactive_login(error: &eyre::Report) -> bool {
    let error = error.to_string();
    [
        "(invalid_grant)",
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
    _tenant_id: &AzureTenantId,
    _client_id: &EntraApplicationClientId,
) -> Result<Option<String>> {
    Ok(None)
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
    _tenant_id: &AzureTenantId,
    _client_id: &str,
    _refresh_token: &str,
) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn delete_pim_refresh_token(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
) {
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
fn delete_pim_refresh_token(_tenant_id: &AzureTenantId, _client_id: &str) {}

fn form_urlencoded(parameters: &[(&str, &str)]) -> String {
    parameters
        .iter()
        .map(|(key, value)| format!("{}={}", url_encode(key), url_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn oauth_error_description(body: &str) -> String {
    facet_json::from_str::<TokenResponse>(body)
        .ok()
        .and_then(|response| response.error_description)
        .unwrap_or_else(|| "no description".to_string())
}
