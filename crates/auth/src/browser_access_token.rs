use crate::AccountSignInPrompt;
use crate::AzureBearerToken;
use crate::BrowserSession;
use crate::BrowserTokenCache;
use crate::CancellationScope;
use crate::browser_authorization_callback_listener::BrowserAuthorizationCallbackListener;
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
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use reqwest::Client;
#[cfg(test)]
use reqwest::Url;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;
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
            let key = BrowserTokenCacheKey {
                resource,
                scopes: format!("{} offline_access", resource.delegated_scope()),
            };
            // This lock spans all resources because they rotate one refresh
            // credential. It also prevents simultaneous initial account prompts.
            let mut session = cache.acquire(tenant_id, client_id).await?;
            if let Some(token) = session.fresh_token(&key) {
                return Ok(token);
            }

            let mut acquisition_resource = if session.refresh_token().is_some() {
                resource
            } else {
                initial_browser_resource(resource)
            };
            loop {
                let scopes = format!("{} offline_access", acquisition_resource.delegated_scope());
                let acquisition_key = BrowserTokenCacheKey {
                    resource: acquisition_resource,
                    scopes: scopes.clone(),
                };
                let token = if let Some(refresh_token) = session.refresh_token() {
                    request_with_refresh_token(
                        tenant_id,
                        client_id,
                        acquisition_resource,
                        &scopes,
                        refresh_token,
                        &cancellation,
                    )
                    .await
                    .wrap_err("refreshing the saved browser session; sign in again if Entra requires interactive authentication")?
                } else {
                    session.begin_initial_login(allow_interactive)?;
                    // Initial consent covers named delegated permissions. ACR
                    // first acquires Graph, then uses that session's refresh
                    // token for its separate .default request.
                    request_with_authorization_code(
                        tenant_id,
                        client_id,
                        acquisition_resource,
                        &delegated_login_scopes(),
                        &AccountSignInPrompt::print,
                        &cancellation,
                    )
                    .await?
                };

                let access_token = cancellation
                    .finish_required(
                        "persisting the acquired browser session and updating its token cache",
                        |_| async {
                            let access_token = token.access_token.clone();
                            // Publish the identity transition even if durable saving
                            // fails. That failure must never permit a new account
                            // handoff within this provider's existing partition.
                            session.remember(acquisition_key, token);
                            if let Some(refresh_token) = session.refresh_token() {
                                let saved = BrowserSession {
                                    tenant_id,
                                    client_id,
                                    refresh_token: refresh_token.to_owned(),
                                };
                                if let Some(store) = &cache.credential_store {
                                    // Serialization errors must not retain the
                                    // refresh credential in their diagnostics.
                                    store.write(&facet_json::to_string(&saved).map_err(|_| eyre::eyre!("serializing the refreshed account credential failed"))?)?;
                                }
                            }
                            eyre::Ok(access_token)
                        },
                    )
                    .await?;
                if acquisition_resource == resource {
                    return Ok(access_token);
                }

                // The bootstrap credential is already retained under its actual
                // audience and persisted before cancellation or ACR failure can
                // interrupt the next exchange. Never repeat the account handoff.
                cancellation.bail_if_cancelled()?;
                if session.refresh_token().is_none() {
                    bail!("Entra browser login did not return a refresh token");
                }
                acquisition_resource = resource;
            }
        })
        .await
}

/// Select an audience present in the named-permission consent request. ACR's
/// .default scope is acquired through the resulting refresh credential instead.
fn initial_browser_resource(resource: AzureCredentialResource) -> AzureCredentialResource {
    match resource {
        AzureCredentialResource::AzureContainerRegistry => AzureCredentialResource::MicrosoftGraph,
        resource => resource,
    }
}

/// Complete explicit browser login and verify the authenticated tenant-local
/// Graph user before returning its captured refresh credential to the account store.
pub(crate) async fn acquire_browser_session(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
    cancellation: &CancellationScope,
) -> Result<(BrowserSession, EntraUserId)> {
    let token = request_with_authorization_code(
        tenant_id,
        client_id,
        AzureCredentialResource::MicrosoftGraph,
        &delegated_login_scopes(),
        prompt,
        cancellation,
    )
    .await?;
    let principal =
        crate::entra_authenticated_user_identity::EntraAuthenticatedUserIdentity::verify(
            &token.access_token.access_token,
            cancellation,
        )
        .await?;
    Ok((
        BrowserSession {
            tenant_id,
            client_id,
            refresh_token: token
                .refresh_token
                .ok_or_else(|| eyre::eyre!("browser login did not return a refresh credential"))?,
        },
        principal,
    ))
}

fn delegated_login_scopes() -> String {
    // ACR exposes only .default, which cannot be mixed with named delegated
    // scopes during consent; its audience is requested separately on token refresh.
    [
        AzureCredentialResource::MicrosoftGraph.delegated_scope(),
        AzureCredentialResource::AzureResourceManager.delegated_scope(),
        AzureCredentialResource::AzureDevOps.delegated_scope(),
        AzureCredentialResource::AzureKeyVault.delegated_scope(),
        "offline_access",
    ]
    .join(" ")
}

/// Perform the authorization-code/PKCE exchange for explicit browser provisioning.
/// The named account owns refresh-credential persistence; this function owns
/// the callback and token parsing for the requested scopes.

#[instrument(level = "debug", skip_all)]
async fn request_with_authorization_code(
    tenant_id: AzureTenantId,
    client_id: EntraApplicationClientId,
    resource: AzureCredentialResource,
    scopes: &str,
    prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
    cancellation: &CancellationScope,
) -> Result<BrowserAccessToken> {
    cancellation
        .cancellable_scope(|_| async move {
            let authority = format!("https://login.microsoftonline.com/{tenant_id}");
            let listener = BrowserAuthorizationCallbackListener::bind().await?;
            let callback_port = listener.port()?;
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

            if open_browser_url(&authorization_url).is_err() {
                prompt(AccountSignInPrompt::BrowserAuthorization {
                    url: authorization_url,
                });
            }
            let authorization_code = listener.receive_code(&state, resource).await?;

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
        })
        .await
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
                    AzureCredentialResource::AzureKeyVault => {
                        scope.starts_with("https://vault.azure.net/")
                    }
                    AzureCredentialResource::AzureContainerRegistry => {
                        scope.starts_with("https://containerregistry.azure.net/")
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
    fn initial_browser_consent_covers_named_delegated_permissions() {
        let scopes = delegated_login_scopes();
        assert!(scopes.contains(AzureCredentialResource::MicrosoftGraph.delegated_scope()));
        assert!(scopes.contains(AzureCredentialResource::AzureResourceManager.delegated_scope()));
        assert!(scopes.contains(AzureCredentialResource::AzureDevOps.delegated_scope()));
        assert!(scopes.contains(AzureCredentialResource::AzureKeyVault.delegated_scope()));
        assert!(!scopes.contains("/.default"));
        assert!(
            scopes
                .split_whitespace()
                .any(|scope| scope == "offline_access")
        );
    }

    #[test]
    fn fresh_registry_login_redeems_a_consented_audience_before_requesting_default_scope() {
        let resource = AzureCredentialResource::AzureContainerRegistry;
        let initial_resource = initial_browser_resource(resource);
        let login_scopes = delegated_login_scopes();
        let redemption_scopes = token_exchange_scopes(initial_resource, &login_scopes);
        assert!(
            redemption_scopes
                .split_whitespace()
                .any(|scope| scope == initial_resource.delegated_scope())
        );
        assert!(
            redemption_scopes
                .split_whitespace()
                .any(|scope| scope == "offline_access")
        );
        assert!(!login_scopes.contains(resource.delegated_scope()));
        assert_ne!(initial_resource, resource);
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
