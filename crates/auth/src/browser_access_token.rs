use crate::AzureBearerToken;
use crate::BrowserSession;
use crate::BrowserTokenCache;
use crate::CancellationScope;
use crate::browser_opener::open_browser_url;
#[cfg(windows)]
use crate::browser_session_metadata::BrowserSessionMetadata;
use crate::browser_token_cache_key::BrowserTokenCacheKey;
use crate::browser_token_response::BrowserTokenResponse;
use base64::Engine;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use reqwest::Client;
use reqwest::Url;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tracing::debug;
use tracing::instrument;
use uuid::Uuid;

/// A validated access credential and optional successor refresh credential.
#[derive(Clone)]
pub(crate) struct BrowserAccessToken {
    pub(crate) access_token: AzureAccessToken<AzureBearerToken>,
    pub(crate) refresh_token: Option<String>,
}

pub(crate) const BROWSER_SESSION_FILE: &str = "browser_session.json";
#[cfg(windows)]
pub(crate) const BROWSER_REFRESH_TOKEN_TARGET_PREFIX: &str =
    "cloud-terrastodon:browser:refresh-token";

pub(crate) fn load_browser_session() -> Result<Option<BrowserSession>> {
    // Parsing diagnostics can include excerpts of the persisted credential.
    // Keep the failure context without exposing the source text.
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
        let metadata: BrowserSessionMetadata = facet_json::from_str(&content).map_err(|_| {
            eyre::eyre!("could not deserialize the browser authentication session metadata")
        })?;
        let target = browser_session_target(&metadata.tenant_id, &metadata.client_id)?;
        let Some(refresh_token) = crate::read_credential_from_manager(&target)? else {
            return Ok(None);
        };
        if refresh_token.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(BrowserSession {
            tenant_id: metadata.tenant_id,
            client_id: metadata.client_id,
            refresh_token,
        }))
    }
    #[cfg(not(windows))]
    facet_json::from_str::<BrowserSession>(&content)
        .map(|session| (!session.refresh_token.trim().is_empty()).then_some(session))
        .map_err(|_| eyre::eyre!("could not deserialize the local browser authentication session"))
}

pub(crate) fn store_browser_session(session: &BrowserSession) -> Result<()> {
    // Serialization diagnostics must not expose the sensitive session value.
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
    .map_err(|_| eyre::eyre!("could not serialize the browser authentication session metadata"))?;
    #[cfg(not(windows))]
    let content = facet_json::to_string_pretty(session)
        .map_err(|_| eyre::eyre!("could not serialize the browser authentication session"))?;

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

#[cfg(not(windows))]
pub(crate) fn delete_browser_session(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
) {
    let path = AppDir::Config.join(BROWSER_SESSION_FILE);
    if let Ok(Some(session)) = load_browser_session()
        && (session.tenant_id != tenant_id || session.client_id != client_id)
    {
        return;
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
        "{BROWSER_REFRESH_TOKEN_TARGET_PREFIX}:{tenant_id}:{client_id}"
    ))
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
    resource: AzureCredentialResource,
    allow_interactive: bool,
    cancellation: &CancellationScope,
) -> Result<AzureAccessToken<AzureBearerToken>> {
    cancellation
        .cancellable_scope(|cancellation| async move {
            let scopes = format!("{} offline_access", resource.delegated_scope());
            let key = BrowserTokenCacheKey {
                resource,
                scopes: scopes.clone(),
            };
            // This lock spans all resources because they rotate one refresh
            // credential. It also prevents simultaneous initial account prompts.
            let mut session = cache.acquire(tenant_id, client_id).await?;
            if let Some(token) = session.fresh_token(&key) {
                return Ok(token);
            }

            let token = if let Some(refresh_token) = session.refresh_token() {
                request_with_refresh_token(
                    tenant_id,
                    client_id,
                    resource,
                    &scopes,
                    refresh_token,
                    &cancellation,
                )
                .await
                .wrap_err("refreshing the admitted browser session; sign in with a new authentication context if reauthentication is required")?
            } else {
                session.begin_initial_login(allow_interactive)?;
                // Consent to every supported audience during the one account
                // handoff. Later audiences must use the admitted refresh chain,
                // and token redemption still filters to this request's resource.
                request_with_authorization_code(
                    tenant_id,
                    client_id,
                    resource,
                    &delegated_login_scopes(),
                    &cancellation,
                )
                .await?
            };

            cancellation
                .finish_required(
                    "persisting the acquired browser session and updating its token cache",
                    |_| async move {
                        let access_token = token.access_token.clone();
                        // Publish the identity transition even if durable saving
                        // fails. That failure must never permit a new account
                        // handoff within this provider's existing partition.
                        session.remember(key, token);
                        if let Some(refresh_token) = session.refresh_token() {
                            store_browser_session(&BrowserSession {
                                tenant_id,
                                client_id,
                                refresh_token: refresh_token.to_owned(),
                            })?;
                        }
                        Ok(access_token)
                    },
                )
                .await
        })
        .await
}

/// Complete one delegated browser login and persist the resulting refresh
/// token for later resource-specific token exchanges.
pub async fn login_browser_session(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    cancellation: &CancellationScope,
) -> Result<()> {
    cancellation
        .cancellable_scope(|cancellation| async move {
            let cancellation = &cancellation;
            // Request consent for every delegated resource during the one browser
            // handoff. Entra can then redeem the resulting refresh token for one
            // resource at a time without forcing a second browser prompt when the
            // caller first reaches ARM or Azure DevOps.
            let scopes = delegated_login_scopes();
            let token = request_with_authorization_code(
                tenant_id,
                client_id,
                AzureCredentialResource::MicrosoftGraph,
                &scopes,
                cancellation,
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
        })
        .await
}

fn delegated_login_scopes() -> String {
    [
        AzureCredentialResource::MicrosoftGraph.delegated_scope(),
        AzureCredentialResource::AzureResourceManager.delegated_scope(),
        AzureCredentialResource::AzureDevOps.delegated_scope(),
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
    resource: AzureCredentialResource,
    scopes: &str,
    cancellation: &CancellationScope,
) -> Result<BrowserAccessToken> {
    cancellation
        .cancellable_scope(|cancellation| async move {
            let cancellation = &cancellation;
            request_with_authorization_code(tenant_id, client_id, resource, scopes, cancellation)
                .await
        })
        .await
}

pub(crate) async fn refresh_browser_access_token(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureCredentialResource,
    scopes: &str,
    refresh_token: &str,
    cancellation: &CancellationScope,
) -> Result<BrowserAccessToken> {
    request_with_refresh_token(
        tenant_id,
        client_id,
        resource,
        scopes,
        refresh_token,
        cancellation,
    )
    .await
}

#[instrument(level = "debug", skip_all)]
async fn request_with_authorization_code(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureCredentialResource,
    scopes: &str,
    cancellation: &CancellationScope,
) -> Result<BrowserAccessToken> {
    cancellation.cancellable_scope(|_| async move {
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

    }).await
}

async fn request_with_refresh_token(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureCredentialResource,
    scopes: &str,
    refresh_token: &str,
    cancellation: &CancellationScope,
) -> Result<BrowserAccessToken> {
    cancellation
        .cancellable_scope(|_| async move {
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
        })
        .await
}

async fn parse_token_response(
    response: reqwest::Response,
    tenant_id: AzureTenantId,
    resource: AzureCredentialResource,
) -> Result<BrowserAccessToken> {
    let status = response.status();
    let body = response
        .text()
        .await
        .wrap_err("reading the Entra browser token response")?;
    BrowserTokenResponse::decode(&body)?.into_access_token(status, tenant_id, resource)
}

#[instrument(level = "debug", skip_all)]
async fn wait_for_authorization_code(
    listener: TcpListener,
    expected_state: String,
    resource: AzureCredentialResource,
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
    resource: AzureCredentialResource,
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

    if let Some(error) = query_value("error") {
        let error = BrowserTokenResponse::diagnostic_error(Some(error));
        bail!("Entra browser authentication for {resource:?} failed ({error})");
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

fn token_exchange_scopes(resource: AzureCredentialResource, scopes: &str) -> String {
    scopes
        .split_whitespace()
        .filter(|scope| {
            *scope == "offline_access"
                || match resource {
                    AzureCredentialResource::MicrosoftGraph => {
                        // Graph permissions are accepted both as fully
                        // qualified resource scopes and as short permission
                        // names (for example `Directory.Read.All`). Azure
                        // DevOps uses a GUID-based resource scope without a
                        // URI scheme, so a short Graph name must also contain
                        // no resource-path separator.
                        scope.starts_with("https://graph.microsoft.com/") || !scope.contains('/')
                    }
                    AzureCredentialResource::AzureResourceManager => {
                        scope.starts_with("https://management.azure.com/")
                    }
                    AzureCredentialResource::AzureDevOps => {
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
    fn initial_browser_consent_covers_all_supported_audiences() {
        let scopes = delegated_login_scopes();
        assert!(scopes.contains(AzureCredentialResource::MicrosoftGraph.delegated_scope()));
        assert!(scopes.contains(AzureCredentialResource::AzureResourceManager.delegated_scope()));
        assert!(scopes.contains(AzureCredentialResource::AzureDevOps.delegated_scope()));
        assert!(
            scopes
                .split_whitespace()
                .any(|scope| scope == "offline_access")
        );
    }

    #[test]
    fn authorization_code_redemption_scopes_are_single_resource() {
        let scopes = delegated_login_scopes();
        let arm_scopes =
            token_exchange_scopes(AzureCredentialResource::AzureResourceManager, &scopes);
        assert!(
            arm_scopes.contains(AzureCredentialResource::AzureResourceManager.delegated_scope())
        );
        assert!(arm_scopes.contains("offline_access"));
        assert!(!arm_scopes.contains(AzureCredentialResource::MicrosoftGraph.delegated_scope()));
        assert!(!arm_scopes.contains(AzureCredentialResource::AzureDevOps.delegated_scope()));
    }

    #[test]
    fn graph_redemption_keeps_short_permission_names() {
        let scopes = "User.Read Directory.Read.All https://management.azure.com/user_impersonation 499b84ac-1321-427f-aa17-267ca6975798/user_impersonation offline_access";
        let graph_scopes = token_exchange_scopes(AzureCredentialResource::MicrosoftGraph, scopes);
        assert_eq!(graph_scopes, "User.Read Directory.Read.All offline_access");
    }

    #[test]
    fn every_authorization_code_redemption_uses_one_resource() {
        let scopes = delegated_login_scopes();
        assert_eq!(
            token_exchange_scopes(AzureCredentialResource::MicrosoftGraph, &scopes),
            "https://graph.microsoft.com/User.Read offline_access"
        );
        assert_eq!(
            token_exchange_scopes(AzureCredentialResource::AzureResourceManager, &scopes),
            "https://management.azure.com/user_impersonation offline_access"
        );
        assert_eq!(
            token_exchange_scopes(AzureCredentialResource::AzureDevOps, &scopes),
            "499b84ac-1321-427f-aa17-267ca6975798/user_impersonation offline_access"
        );
    }

    #[test]
    fn callback_parser_accepts_encoded_code_and_matching_state() {
        let code = parse_authorization_callback(
            "/?code=code%2Bvalue&state=expected%20state",
            "expected state",
            AzureCredentialResource::MicrosoftGraph,
        )
        .unwrap();
        assert_eq!(code, "code+value");
    }

    #[test]
    fn callback_parser_rejects_invalid_state() {
        let error = parse_authorization_callback(
            "/?code=code&state=unexpected",
            "expected",
            AzureCredentialResource::MicrosoftGraph,
        )
        .unwrap_err();
        assert!(error.to_string().contains("invalid state"));
    }

    #[test]
    fn callback_parser_reports_known_errors_without_untrusted_descriptions() {
        let error = parse_authorization_callback(
            "/?error=access_denied&error_description=cancelled%20by%20user&state=expected",
            "expected",
            AzureCredentialResource::AzureDevOps,
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("access_denied"));
        assert!(!message.contains("cancelled by user"));
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
        assert!(requested_scopes.contains(AzureCredentialResource::AzureDevOps.delegated_scope()));
        assert!(requested_scopes.contains("offline_access"));
    }
}
