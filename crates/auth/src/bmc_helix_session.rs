use cloud_terrastodon_auth_types::BmcHelixAuthorization;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_bmc_helix_types::BmcHelixUsername;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use http::StatusCode;
use http::header::AUTHORIZATION;
use http::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::redirect::Policy;
use reqwest::tls::Version;
use std::time::Duration;
use url::form_urlencoded;

fn credential_client() -> Result<Client> {
    // These exchanges carry passwords or tokens. Keep both the outgoing form
    // and the response outside general REST diagnostics, and never redirect.
    Client::builder()
        .min_tls_version(Version::TLS_1_2)
        .redirect(Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .wrap_err("constructing the Helix credential transport")
}

pub(crate) async fn login(
    server: &BmcHelixServerUrl,
    username: &BmcHelixUsername,
    password: &str,
    cancellation: &CancellationScope,
) -> Result<String> {
    let url = server.as_url().join("api/jwt/login")?;
    let form = form_urlencoded::Serializer::new(String::new())
        .append_pair("username", username.as_str())
        .append_pair("password", password)
        .finish();
    cancellation.bail_if_cancelled()?;
    let response = credential_client()?
        .post(url)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form)
        .send()
        .await
        .wrap_err("contacting the Helix sign-in endpoint")?;
    ensure!(
        response.status().is_success(),
        "Helix sign-in failed with HTTP {}",
        response.status()
    );
    let token = response.text().await.wrap_err("reading the Helix token")?;
    let token = token.trim();
    BmcHelixAuthorization::new(token)?;
    Ok(token.to_owned())
}

pub(crate) async fn logout(
    server: &BmcHelixServerUrl,
    token: &str,
    _cancellation: &CancellationScope,
) -> Result<()> {
    let url = server.as_url().join("api/jwt/logout")?;
    let authorization = BmcHelixAuthorization::new(token)?;
    let status = credential_client()?
        .post(url)
        .header(AUTHORIZATION, authorization.header())
        .send()
        .await
        .wrap_err("contacting the Helix sign-out endpoint")?
        .status();
    ensure!(
        status.is_success() || status == StatusCode::UNAUTHORIZED,
        "Helix sign-out failed with HTTP {status}"
    );
    Ok(())
}
