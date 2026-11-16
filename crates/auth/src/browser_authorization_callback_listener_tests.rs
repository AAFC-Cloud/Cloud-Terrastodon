use super::BrowserAuthorizationCallbackListener;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_command::CancellationError;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::time::timeout;

async fn send_request(address: SocketAddr, target: &str) -> String {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream
        .write_all(format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .await
        .unwrap();
    // Close the client write half first so the client owns TIME_WAIT on Unix;
    // the completed server's listening port must remain immediately reusable.
    stream.shutdown().await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

async fn assert_listener_released(address: SocketAddr) {
    // Check socket ownership directly. Windows can delay refusing a connection
    // to a closed port, so connection timeouts do not establish a leaked socket.
    let replacement = TcpListener::bind(address).await.unwrap_or_else(|error| {
        panic!("authorization listener did not release {address}: {error}")
    });
    drop(replacement);
}

#[tokio::test]
async fn cancellation_bypasses_idle_preconnection_and_releases_both_listeners() {
    let listener = BrowserAuthorizationCallbackListener::bind().await.unwrap();
    let ipv4 = SocketAddr::from((Ipv4Addr::LOCALHOST, listener.port().unwrap()));
    let ipv6 = listener
        .ipv6
        .as_ref()
        .map(|listener| listener.local_addr().unwrap());
    let idle_preconnection = TcpStream::connect(ipv4).await.unwrap();

    let (result, response) = timeout(Duration::from_secs(2), async {
        tokio::join!(
            listener.receive_code("expected", AzureCredentialResource::MicrosoftGraph),
            send_request(
                ipv4,
                "/?error=access_denied&error_subcode=cancel&state=expected"
            ),
        )
    })
    .await
    .expect("an idle preconnection must not delay the cancellation callback");

    assert!(result.unwrap_err().is::<CancellationError>());
    assert!(response.contains("Login cancelled"));
    // Retain the speculative connection through completion: its presence must
    // not keep either listening socket alive after the attempt ends.
    assert_listener_released(ipv4).await;
    if let Some(ipv6) = ipv6 {
        assert_listener_released(ipv6).await;
    }
    drop(idle_preconnection);
}

#[tokio::test]
async fn closed_preconnection_and_favicon_do_not_consume_authorization_attempt() {
    let listener = BrowserAuthorizationCallbackListener::bind().await.unwrap();
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, listener.port().unwrap()));
    drop(TcpStream::connect(address).await.unwrap());

    let (result, response) = timeout(Duration::from_secs(2), async {
        tokio::join!(
            listener.receive_code("expected", AzureCredentialResource::MicrosoftGraph),
            async {
                let response = send_request(address, "/favicon.ico").await;
                assert!(response.starts_with("HTTP/1.1 404 Not Found\r\n"));
                send_request(address, "/?code=local-test-code&state=expected").await
            },
        )
    })
    .await
    .expect("unrelated browser connections must not consume the real callback");

    assert_eq!(result.unwrap(), "local-test-code");
    assert!(response.contains("Authorization received"));
}

#[tokio::test]
async fn ipv6_loopback_accepts_authorization_and_denial_when_available() {
    for (target, expected_code) in [
        (
            "/?code=ipv6-test-code&state=expected",
            Some("ipv6-test-code"),
        ),
        ("/?error=access_denied&state=expected", None),
    ] {
        let listener = BrowserAuthorizationCallbackListener::bind().await.unwrap();
        let Some(ipv6) = listener.ipv6.as_ref() else {
            return;
        };
        let address = ipv6.local_addr().unwrap();
        let (result, response) = timeout(Duration::from_secs(2), async {
            tokio::join!(
                listener.receive_code("expected", AzureCredentialResource::MicrosoftGraph),
                send_request(address, target),
            )
        })
        .await
        .expect("IPv6 callbacks must reach the authorization listener promptly");

        if let Some(expected_code) = expected_code {
            assert_eq!(result.unwrap(), expected_code);
            assert!(response.contains("Authorization received"));
        } else {
            let error = result.unwrap_err();
            assert!(!error.is::<CancellationError>());
            assert!(error.to_string().contains("access_denied"));
            assert!(response.contains("Login denied"));
        }
        assert_listener_released(address).await;
    }
}

#[tokio::test]
async fn closing_tab_after_cancellation_headers_preserves_cancellation() {
    let listener = BrowserAuthorizationCallbackListener::bind().await.unwrap();
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, listener.port().unwrap()));
    let (result, ()) = timeout(Duration::from_secs(2), async {
        tokio::join!(
            listener.receive_code("expected", AzureCredentialResource::MicrosoftGraph),
            async {
                let mut stream = TcpStream::connect(address).await.unwrap();
                stream
                    .write_all(
                        b"GET /?error=access_denied&error_subcode=cancel&state=expected HTTP/1.1\r\nHost: localhost\r\n\r\n",
                    )
                    .await
                    .unwrap();
                // The tab closes without reading the response. Its delivery is
                // best effort and must not replace the received cancellation.
                drop(stream);
            },
        )
    })
    .await
    .expect("a closed browser tab must not hold the authorization attempt open");

    assert!(result.unwrap_err().is::<CancellationError>());
    assert_listener_released(address).await;
}
