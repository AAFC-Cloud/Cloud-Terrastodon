use crate::AzureBearerToken;
use crate::AzureRestResource;
use crate::browser_opener::open_browser_url;
use base64::Engine;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use reqwest::Client;
use reqwest::Url;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::debug;
use tracing::instrument;
use uuid::Uuid;

const TOKEN_REFRESH_BUFFER_SECONDS: i64 = 120;

/// Tokens acquired by one interactive CLI invocation.
///
/// The cache is owned by `AuthContext`, rather than being process-global, so
/// independent invocations and tests cannot share credentials accidentally.
#[derive(Clone, Default)]
pub struct BrowserTokenCache {
    entries: Arc<Mutex<HashMap<BrowserTokenCacheKey, BrowserCachedToken>>>,
    refresh_tokens: Arc<Mutex<HashMap<BrowserRefreshTokenKey, String>>>,
}

impl std::fmt::Debug for BrowserTokenCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserTokenCache")
            .field("entries", &"<sensitive>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BrowserTokenCacheKey {
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    scopes: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BrowserRefreshTokenKey {
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
}

#[derive(Clone)]
struct BrowserCachedToken {
    access_token: AzureAccessToken<AzureBearerToken>,
    refresh_token: Option<String>,
}

impl std::fmt::Debug for BrowserCachedToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserCachedToken")
            .field("access_token", &self.access_token)
            .field("refresh_token", &"***redacted***")
            .finish()
    }
}

impl BrowserTokenCache {
    pub(crate) fn from_session(session: Option<&BrowserSession>) -> Self {
        let cache = Self::default();
        if let Some(session) = session {
            // This cache is only seeded with the opaque refresh token. It is
            // never included in Debug output or request artifacts.
            if let Ok(mut refresh_tokens) = cache.refresh_tokens.try_lock() {
                refresh_tokens.insert(
                    BrowserRefreshTokenKey {
                        tenant_id: session.tenant_id,
                        client_id: session.client_id,
                    },
                    session.refresh_token.clone(),
                );
            }
        }
        cache
    }

    async fn get(&self, key: &BrowserTokenCacheKey) -> Option<BrowserCachedToken> {
        self.entries.lock().await.get(key).cloned()
    }

    async fn insert(&self, key: BrowserTokenCacheKey, token: BrowserCachedToken) {
        if let Some(refresh_token) = token.refresh_token.as_deref() {
            self.refresh_tokens.lock().await.insert(
                BrowserRefreshTokenKey {
                    tenant_id: key.tenant_id,
                    client_id: key.client_id,
                },
                refresh_token.to_owned(),
            );
        }
        self.entries.lock().await.insert(key, token);
    }

    async fn refresh_token(
        &self,
        tenant_id: AzureTenantId,
        client_id: EntraApplicationClientId,
    ) -> Option<String> {
        self.refresh_tokens
            .lock()
            .await
            .get(&BrowserRefreshTokenKey {
                tenant_id,
                client_id,
            })
            .cloned()
    }
}

#[derive(Clone)]
pub(crate) struct BrowserAccessToken {
    pub(crate) access_token: AzureAccessToken<AzureBearerToken>,
    pub(crate) refresh_token: Option<String>,
}

#[derive(Clone, Facet)]
pub struct BrowserSession {
    pub(crate) tenant_id: AzureTenantId,
    pub(crate) client_id: EntraApplicationClientId,
    #[facet(sensitive)]
    pub(crate) refresh_token: String,
}

#[derive(Clone, Facet)]
struct BrowserSessionMetadata {
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
}

impl std::fmt::Debug for BrowserSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserSession")
            .field("tenant_id", &self.tenant_id)
            .field("client_id", &self.client_id)
            .field("refresh_token", &"***redacted***")
            .finish()
    }
}

const BROWSER_SESSION_FILE: &str = "browser_session.json";

pub(crate) fn load_browser_session() -> Result<Option<BrowserSession>> {
    let path = AppDir::Config.join(BROWSER_SESSION_FILE);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).wrap_err("reading the local browser authentication session");
        }
    };
    #[cfg(windows)]
    {
        let metadata: BrowserSessionMetadata = facet_json::from_str(&content)
            .map_err(|error| eyre::eyre!("{error:?}"))
            .wrap_err("deserializing the browser authentication session metadata")?;
        let target = browser_session_target(&metadata.tenant_id, &metadata.client_id)?;
        let Some(refresh_token) = crate::read_credential_from_manager(&target)? else {
            return Ok(None);
        };
        if refresh_token.trim().is_empty() {
            return Ok(None);
        }
        return Ok(Some(BrowserSession {
            tenant_id: metadata.tenant_id,
            client_id: metadata.client_id,
            refresh_token,
        }));
    }
    #[cfg(not(windows))]
    facet_json::from_str::<BrowserSession>(&content)
        .map(|session| (!session.refresh_token.trim().is_empty()).then_some(session))
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("deserializing the local browser authentication session")
}

pub(crate) fn store_browser_session(session: &BrowserSession) -> Result<()> {
    let path = AppDir::Config.join(BROWSER_SESSION_FILE);
    let Some(parent) = path.parent() else {
        bail!("browser authentication session path has no parent directory");
    };
    std::fs::create_dir_all(parent).wrap_err("creating the browser authentication directory")?;
    #[cfg(windows)]
    let content = facet_json::to_string_pretty(&BrowserSessionMetadata {
        tenant_id: session.tenant_id,
        client_id: session.client_id,
    })
    .map_err(|error| eyre::eyre!("{error:?}"))
    .wrap_err("serializing the browser authentication session metadata")?;
    #[cfg(not(windows))]
    let content = facet_json::to_string_pretty(session)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("serializing the browser authentication session")?;

    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write as _;
    let mut file = options
        .open(&path)
        .wrap_err("opening the browser authentication session file")?;
    file.write_all(content.as_bytes())
        .wrap_err("writing the browser authentication session")?;
    #[cfg(unix)]
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .wrap_err("restricting browser authentication session permissions")?;
    #[cfg(windows)]
    {
        let target = browser_session_target(&session.tenant_id, &session.client_id)?;
        crate::write_credential_to_manager(&target, &session.refresh_token)
            .wrap_err("persisting the browser refresh token in Windows Credential Manager")?;
    }
    Ok(())
}

pub(crate) fn delete_browser_session(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
) {
    let path = AppDir::Config.join(BROWSER_SESSION_FILE);
    #[cfg(not(windows))]
    if let Ok(Some(session)) = load_browser_session()
        && (session.tenant_id != tenant_id || session.client_id != client_id)
    {
        return;
    }
    #[cfg(windows)]
    if let Ok(Some(session)) = load_browser_session()
        && (session.tenant_id != tenant_id || session.client_id != client_id)
    {
        return;
    }
    #[cfg(windows)]
    if let Ok(Some(_session)) = load_browser_session() {
        if let Ok(target) = browser_session_target(&tenant_id, &client_id)
            && let Err(error) = crate::delete_credential_from_manager(&target)
        {
            debug!(?error, "Could not delete the Windows browser refresh token");
        }
    }
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        debug!(
            ?error,
            "Could not delete the local browser authentication session"
        );
    }
}

#[cfg(windows)]
fn browser_session_target(
    tenant_id: &AzureTenantId,
    client_id: &EntraApplicationClientId,
) -> Result<crate::WindowsCredentialManagerTargetName> {
    crate::WindowsCredentialManagerTargetName::try_new(format!(
        "cloud-terrastodon:browser:refresh-token:{tenant_id}:{client_id}"
    ))
}

#[derive(Debug, Facet)]
#[facet(rename_all = "snake_case")]
struct TokenResponse {
    #[facet(sensitive)]
    access_token: Option<String>,
    #[facet(sensitive)]
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Fetch a resource-specific delegated token using authorization code + PKCE.
///
/// The caller supplies the client registration and tenant explicitly. This
/// function performs no Azure API calls; it only contacts the Entra authorize
/// and token endpoints when the browser source is actually selected.
pub(crate) async fn fetch_browser_access_token(
    cache: &BrowserTokenCache,
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    allow_interactive: bool,
) -> Result<AzureAccessToken<AzureBearerToken>> {
    let scopes = format!("{} offline_access", resource.delegated_scope());
    let key = BrowserTokenCacheKey {
        tenant_id,
        client_id,
        resource,
        scopes: scopes.clone(),
    };

    let cached = cache.get(&key).await;
    if let Some(cached_token) = cached.as_ref()
        && is_token_fresh(&cached_token.access_token)
    {
        return Ok(cached_token.access_token.clone());
    }

    let refresh_token = if let Some(refresh_token) = cached
        .as_ref()
        .and_then(|cached_token| cached_token.refresh_token.as_deref())
    {
        Some(refresh_token.to_owned())
    } else {
        cache.refresh_token(tenant_id, client_id).await
    };
    let token = if let Some(refresh_token) = refresh_token.as_deref() {
        match request_with_refresh_token(tenant_id, client_id, resource, &scopes, refresh_token)
            .await
        {
            Ok(mut token) => {
                if token.refresh_token.is_none() {
                    token.refresh_token = Some(refresh_token.to_owned());
                }
                token
            }
            Err(error) if refresh_token_needs_interactive_login(&error) => {
                if !allow_interactive {
                    return Err(error).wrap_err(
                        "browser refresh token requires interactive reauthentication, which is unavailable in a headless context",
                    );
                }
                debug!(
                    ?error,
                    ?resource,
                    "Stored browser refresh token requires interactive login"
                );
                delete_browser_session(tenant_id, client_id);
                request_with_authorization_code(tenant_id, client_id, resource, &scopes).await?
            }
            Err(error) => return Err(error),
        }
    } else {
        if !allow_interactive {
            bail!(
                "browser authentication has no stored refresh token and interactive authentication is unavailable in a headless context"
            );
        }
        request_with_authorization_code(tenant_id, client_id, resource, &scopes).await?
    };

    if let Some(refresh_token) = token.refresh_token.as_deref() {
        store_browser_session(&BrowserSession {
            tenant_id,
            client_id,
            refresh_token: refresh_token.to_owned(),
        })?;
    }
    let access_token = token.access_token.clone();
    cache
        .insert(
            key,
            BrowserCachedToken {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
            },
        )
        .await;
    Ok(access_token)
}

/// Complete one delegated browser login and persist the resulting refresh
/// token for later resource-specific token exchanges.
pub async fn login_browser_session(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
) -> Result<()> {
    // Request consent for every delegated resource during the one browser
    // handoff. Entra can then redeem the resulting refresh token for one
    // resource at a time without forcing a second browser prompt when the
    // caller first reaches ARM or Azure DevOps.
    let scopes = delegated_login_scopes();
    let token = request_with_authorization_code(
        tenant_id,
        client_id,
        AzureRestResource::MicrosoftGraph,
        &scopes,
    )
    .await?;
    let refresh_token = token
        .refresh_token
        .clone()
        .ok_or_else(|| eyre::eyre!("Entra browser login did not return a refresh token"))?;
    store_browser_session(&BrowserSession {
        tenant_id,
        client_id,
        refresh_token,
    })?;
    Ok(())
}

fn delegated_login_scopes() -> String {
    [
        AzureRestResource::MicrosoftGraph.delegated_scope(),
        AzureRestResource::AzureResourceManager.delegated_scope(),
        AzureRestResource::AzureDevOps.delegated_scope(),
        "offline_access",
    ]
    .join(" ")
}

/// Acquire a delegated token for the PIM module with caller-provided scopes.
/// The PIM module keeps its existing Windows Credential Manager refresh-token
/// persistence, while this module owns the shared callback and token parsing.
pub(crate) async fn fetch_browser_access_token_with_scopes(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    scopes: &str,
) -> Result<BrowserAccessToken> {
    request_with_authorization_code(tenant_id, client_id, resource, scopes).await
}

pub(crate) async fn refresh_browser_access_token(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    scopes: &str,
    refresh_token: &str,
) -> Result<BrowserAccessToken> {
    request_with_refresh_token(tenant_id, client_id, resource, scopes, refresh_token).await
}

#[instrument(level = "debug", skip_all)]
async fn request_with_authorization_code(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    scopes: &str,
) -> Result<BrowserAccessToken> {
    let authority = format!("https://login.microsoftonline.com/{tenant_id}");
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .wrap_err("binding the local OAuth callback listener")?;
    let callback_port = listener
        .local_addr()
        .wrap_err("getting the local OAuth callback address")?
        .port();
    let redirect_uri = format!("http://localhost:{callback_port}");
    let state = Uuid::new_v4().to_string();
    let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(code_verifier.as_bytes()));
    let authorization_url = build_authorization_url(
        &authority,
        &client_id,
        &redirect_uri,
        scopes,
        &state,
        &code_challenge,
    );

    let callback = Box::pin(wait_for_authorization_code(listener, state, resource));
    match open_browser_url(&authorization_url) {
        Ok(()) => eprintln!(
            "Opened the browser for Cloud Terrastodon authentication. If the callback cannot reach this WSL instance, open this URL after forwarding port {}:\n{authorization_url}",
            callback_port
        ),
        Err(error) => eprintln!(
            "Could not open the browser automatically ({error}); open this URL manually:\n{authorization_url}"
        ),
    }
    let authorization_code = callback.await?;

    let client = Client::new();
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let token_response = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_urlencoded(&[
            ("client_id", client_id.to_string().as_str()),
            ("grant_type", "authorization_code"),
            ("code", authorization_code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", code_verifier.as_str()),
            ("scope", token_exchange_scopes(resource, scopes).as_str()),
        ]))
        .send()
        .await
        .wrap_err("requesting an Entra browser access token")?;
    parse_token_response(token_response, tenant_id, resource).await
}

async fn request_with_refresh_token(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureRestResource,
    scopes: &str,
    refresh_token: &str,
) -> Result<BrowserAccessToken> {
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
        .wrap_err("requesting an Entra access token from a browser refresh token")?;
    parse_token_response(token_response, tenant_id, resource).await
}

async fn parse_token_response(
    response: reqwest::Response,
    tenant_id: AzureTenantId,
    resource: AzureRestResource,
) -> Result<BrowserAccessToken> {
    let status = response.status();
    let body = response.text().await?;
    let token: TokenResponse = facet_json::from_str(&body)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("deserializing the Entra browser token response")?;

    if let Some(access_token) = token.access_token {
        if token
            .token_type
            .as_deref()
            .is_some_and(|token_type| !token_type.eq_ignore_ascii_case("bearer"))
        {
            bail!("Entra returned a non-Bearer token for {resource:?}");
        }
        let expires_in = token
            .expires_in
            .ok_or_else(|| eyre::eyre!("Entra browser token response omitted expires_in"))?;
        return Ok(BrowserAccessToken {
            access_token: AzureAccessToken {
                access_token: AzureBearerToken::new(access_token),
                expires_on: Local::now() + TimeDelta::seconds(expires_in as i64),
                subscription: None,
                tenant: tenant_id,
                token_type: cloud_terrastodon_azure_types::TokenType::Bearer,
            },
            refresh_token: token.refresh_token,
        });
    }

    let error = token.error.unwrap_or_else(|| "unknown_error".to_owned());
    let description = token
        .error_description
        .unwrap_or_else(|| "no description".to_owned());
    if error == "invalid_client" {
        bail!(
            "Entra rejected the {resource:?} browser client. Ensure the app registration allows public-client PKCE authentication and has the localhost redirect URI configured: {description}"
        );
    }
    bail!(
        "Entra browser token request for {resource:?} failed with {status} ({error}): {description}"
    );
}

#[instrument(level = "debug", skip_all)]
async fn wait_for_authorization_code(
    listener: TcpListener,
    expected_state: String,
    resource: AzureRestResource,
) -> Result<String> {
    debug!(
        ?resource,
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
    let mut request_length = 0;
    loop {
        let bytes_read = timeout(
            Duration::from_secs(30),
            stream.read(&mut request_buffer[request_length..]),
        )
        .await
        .wrap_err("reading the browser OAuth callback")??;
        request_length += bytes_read;
        if bytes_read == 0
            || request_buffer[..request_length]
                .windows(4)
                .any(|window| window == b"\r\n\r\n")
        {
            break;
        }
        if request_length == request_buffer.len() {
            bail!("browser OAuth callback request exceeded the 16 KiB limit");
        }
    }
    let request = String::from_utf8_lossy(&request_buffer[..request_length]);
    let request_target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| {
            eyre::eyre!("browser OAuth callback did not contain an HTTP request target")
        })?;
    let code = match parse_authorization_callback(request_target, &expected_state, resource) {
        Ok(code) => code,
        Err(error) => {
            write_callback_response(&mut stream, false).await?;
            return Err(error);
        }
    };
    write_callback_response(&mut stream, true).await?;
    Ok(code)
}

fn parse_authorization_callback(
    request_target: &str,
    expected_state: &str,
    resource: AzureRestResource,
) -> Result<String> {
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

    if query_value("error").is_some() {
        bail!(
            "Entra browser authentication for {resource:?} failed ({}): {}",
            query_value("error").unwrap_or("unknown error"),
            query_value("error_description").unwrap_or("no description")
        );
    }
    if query_value("state") != Some(expected_state) {
        bail!("Entra browser authentication for {resource:?} returned an invalid state");
    }
    query_value("code")
        .map(str::to_owned)
        .ok_or_else(|| eyre::eyre!("Entra browser authentication callback did not contain a code"))
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

fn build_authorization_url(
    authority: &str,
    client_id: &EntraApplicationClientId,
    redirect_uri: &str,
    scopes: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    format!(
        "{authority}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&state={}&code_challenge={}&code_challenge_method=S256&prompt=select_account",
        url_encode(&client_id.to_string()),
        url_encode(redirect_uri),
        url_encode(scopes),
        url_encode(state),
        url_encode(code_challenge),
    )
}

fn is_token_fresh(token: &AzureAccessToken<AzureBearerToken>) -> bool {
    token.expires_on > Local::now() + TimeDelta::seconds(TOKEN_REFRESH_BUFFER_SECONDS)
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

pub(crate) fn form_urlencoded(parameters: &[(&str, &str)]) -> String {
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

fn token_exchange_scopes(resource: AzureRestResource, scopes: &str) -> String {
    scopes
        .split_whitespace()
        .filter(|scope| {
            *scope == "offline_access"
                || match resource {
                    AzureRestResource::MicrosoftGraph => {
                        // Graph permissions are accepted both as fully
                        // qualified resource scopes and as short permission
                        // names (for example `Directory.Read.All`). Azure
                        // DevOps uses a GUID-based resource scope without a
                        // URI scheme, so a short Graph name must also contain
                        // no resource-path separator.
                        scope.starts_with("https://graph.microsoft.com/") || !scope.contains('/')
                    }
                    AzureRestResource::AzureResourceManager => {
                        scope.starts_with("https://management.azure.com/")
                    }
                    AzureRestResource::AzureDevOps => {
                        scope.starts_with("499b84ac-1321-427f-aa17-267ca6975798/")
                    }
                }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_urlencoding_escapes_scopes_and_redirects() {
        let encoded = form_urlencoded(&[
            (
                "scope",
                "https://management.azure.com/user_impersonation offline_access",
            ),
            ("redirect_uri", "http://localhost:1234"),
        ]);
        assert_eq!(
            encoded,
            "scope=https%3A%2F%2Fmanagement.azure.com%2Fuser_impersonation%20offline_access&redirect_uri=http%3A%2F%2Flocalhost%3A1234"
        );
    }

    #[test]
    fn resource_cache_keys_are_distinct() {
        let tenant_id = "11111111-1111-1111-1111-111111111111"
            .parse::<AzureTenantId>()
            .unwrap();
        let client_id = "22222222-2222-2222-2222-222222222222"
            .parse::<EntraApplicationClientId>()
            .unwrap();
        let graph = BrowserTokenCacheKey {
            tenant_id,
            client_id,
            resource: AzureRestResource::MicrosoftGraph,
            scopes: "graph".to_owned(),
        };
        let arm = BrowserTokenCacheKey {
            resource: AzureRestResource::AzureResourceManager,
            ..graph.clone()
        };
        assert_ne!(graph, arm);
    }

    #[test]
    fn resource_scope_refresh_errors_can_fall_back_to_interactive_login() {
        let error = eyre::eyre!("Entra token request failed (invalid_scope)");
        assert!(refresh_token_needs_interactive_login(&error));
    }

    #[test]
    fn browser_login_requests_all_normal_delegated_resources() {
        let scopes = delegated_login_scopes();
        assert!(scopes.contains(AzureRestResource::MicrosoftGraph.delegated_scope()));
        assert!(scopes.contains(AzureRestResource::AzureResourceManager.delegated_scope()));
        assert!(scopes.contains(AzureRestResource::AzureDevOps.delegated_scope()));
        assert!(
            scopes
                .split_whitespace()
                .any(|scope| scope == "offline_access")
        );
    }

    #[test]
    fn authorization_code_redemption_scopes_are_single_resource() {
        let scopes = delegated_login_scopes();
        let arm_scopes = token_exchange_scopes(AzureRestResource::AzureResourceManager, &scopes);
        assert!(arm_scopes.contains(AzureRestResource::AzureResourceManager.delegated_scope()));
        assert!(arm_scopes.contains("offline_access"));
        assert!(!arm_scopes.contains(AzureRestResource::MicrosoftGraph.delegated_scope()));
        assert!(!arm_scopes.contains(AzureRestResource::AzureDevOps.delegated_scope()));
    }

    #[test]
    fn graph_redemption_keeps_short_permission_names() {
        let scopes = "User.Read Directory.Read.All https://management.azure.com/user_impersonation 499b84ac-1321-427f-aa17-267ca6975798/user_impersonation offline_access";
        let graph_scopes = token_exchange_scopes(AzureRestResource::MicrosoftGraph, scopes);
        assert_eq!(graph_scopes, "User.Read Directory.Read.All offline_access");
    }

    #[test]
    fn every_authorization_code_redemption_uses_one_resource() {
        let scopes = delegated_login_scopes();
        assert_eq!(
            token_exchange_scopes(AzureRestResource::MicrosoftGraph, &scopes),
            "https://graph.microsoft.com/User.Read offline_access"
        );
        assert_eq!(
            token_exchange_scopes(AzureRestResource::AzureResourceManager, &scopes),
            "https://management.azure.com/user_impersonation offline_access"
        );
        assert_eq!(
            token_exchange_scopes(AzureRestResource::AzureDevOps, &scopes),
            "499b84ac-1321-427f-aa17-267ca6975798/user_impersonation offline_access"
        );
    }

    #[test]
    fn callback_parser_accepts_encoded_code_and_matching_state() {
        let code = parse_authorization_callback(
            "/?code=code%2Bvalue&state=expected%20state",
            "expected state",
            AzureRestResource::MicrosoftGraph,
        )
        .unwrap();
        assert_eq!(code, "code+value");
    }

    #[test]
    fn callback_parser_rejects_invalid_state() {
        let error = parse_authorization_callback(
            "/?code=code&state=unexpected",
            "expected",
            AzureRestResource::MicrosoftGraph,
        )
        .unwrap_err();
        assert!(error.to_string().contains("invalid state"));
    }

    #[test]
    fn callback_parser_reports_provider_errors() {
        let error = parse_authorization_callback(
            "/?error=access_denied&error_description=cancelled%20by%20user&state=expected",
            "expected",
            AzureRestResource::AzureDevOps,
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("access_denied"));
        assert!(message.contains("cancelled by user"));
    }

    #[test]
    fn authorization_url_contains_pkce_and_dynamic_localhost_redirect() {
        let client_id = "22222222-2222-2222-2222-222222222222"
            .parse::<EntraApplicationClientId>()
            .unwrap();
        let scopes = delegated_login_scopes();
        let url = build_authorization_url(
            "https://login.microsoftonline.com/11111111-1111-1111-1111-111111111111",
            &client_id,
            "http://localhost:43721",
            &scopes,
            "state value",
            "pkce-challenge",
        );
        let parsed = Url::parse(&url).unwrap();
        assert_eq!(
            parsed.path(),
            "/11111111-1111-1111-1111-111111111111/oauth2/v2.0/authorize"
        );
        let query = parsed
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(
            query.get("response_type").map(|value| value.as_ref()),
            Some("code")
        );
        assert_eq!(
            query.get("redirect_uri").map(|value| value.as_ref()),
            Some("http://localhost:43721")
        );
        assert_eq!(
            query.get("state").map(|value| value.as_ref()),
            Some("state value")
        );
        assert_eq!(
            query
                .get("code_challenge_method")
                .map(|value| value.as_ref()),
            Some("S256")
        );
        assert_eq!(
            query.get("code_challenge").map(|value| value.as_ref()),
            Some("pkce-challenge")
        );
        let requested_scopes = query.get("scope").unwrap();
        assert!(requested_scopes.contains(AzureRestResource::AzureDevOps.delegated_scope()));
        assert!(requested_scopes.contains("offline_access"));
    }
}
