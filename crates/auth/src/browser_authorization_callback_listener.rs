use crate::browser_authorization_callback_response::BrowserAuthorizationCallbackResponse;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::io::ErrorKind;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpSocket;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::debug;

/// Own the loopback sockets for one bounded browser authorization attempt.
///
/// `localhost` may resolve to either address family. All sockets remain private
/// so they cannot outlive this attempt or be rebound to a non-loopback address.
/// See [Microsoft's loopback redirect guidance](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url#localhost-exceptions).
pub(crate) struct BrowserAuthorizationCallbackListener {
    ipv4: TcpListener,
    ipv6: Option<TcpListener>,
}

impl BrowserAuthorizationCallbackListener {
    pub(crate) async fn bind() -> Result<Self> {
        let ipv4 = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .wrap_err("binding the local OAuth callback listener")?;
        let port = ipv4.local_addr()?.port();
        let ipv6 = Self::bind_ipv6(port)?;
        Ok(Self { ipv4, ipv6 })
    }

    fn bind_ipv6(port: u16) -> Result<Option<TcpListener>> {
        let socket = match TcpSocket::new_v6() {
            Ok(socket) => socket,
            Err(error) => {
                // No port has been selected yet, so an unavailable IPv6 socket
                // cannot redirect the browser to somebody else's listener.
                debug!(%error, "IPv6 sockets are unavailable; browser callback uses IPv4");
                return Ok(None);
            }
        };
        let listener = match socket
            .bind(SocketAddr::from((Ipv6Addr::LOCALHOST, port)))
            .and_then(|()| socket.listen(128))
        {
            Ok(listener) => Some(listener),
            // Never fall back from an occupied or exclusively reserved port,
            // which Windows can report as either AddrInUse or PermissionDenied.
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::AddrNotAvailable | ErrorKind::Unsupported
                ) =>
            {
                debug!(%error, "IPv6 loopback is unavailable; browser callback uses IPv4");
                None
            }
            Err(error) => return Err(error).wrap_err("binding the IPv6 OAuth callback listener"),
        };
        Ok(listener)
    }

    pub(crate) fn port(&self) -> Result<u16> {
        Ok(self.ipv4.local_addr()?.port())
    }

    pub(crate) async fn receive_code(
        self,
        expected_state: &str,
        resource: AzureCredentialResource,
    ) -> Result<String> {
        timeout(Duration::from_secs(600), async {
            let mut requests = FuturesUnordered::new();
            loop {
                tokio::select! {
                    connection = self.accept(), if requests.len() < 8 => {
                        let stream = connection.wrap_err("accepting the browser OAuth callback")?;
                        requests.push(Self::read_request(stream));
                    }
                    request = requests.next(), if !requests.is_empty() => {
                        let Some(request) = request else { continue; };
                        let (mut stream, request_target) = match request {
                            Ok(request) => request,
                            Err(error) => {
                                // Browsers may open idle speculative connections or
                                // close them without sending any HTTP request.
                                debug!(%error, "Ignoring an incomplete browser callback connection");
                                continue;
                            }
                        };
                        if !request_target.starts_with("/?") {
                            if let Err(error) = Self::write_response(&mut stream, "404 Not Found", "Not found.").await {
                                debug!(%error, "Could not answer an unrelated loopback request");
                            }
                            continue;
                        }
                        let callback = BrowserAuthorizationCallbackResponse::parse(&request_target, expected_state);
                        let (status, body) = match &callback {
                            Ok(callback) => ("200 OK", callback.response_body()),
                            Err(_) => ("400 Bad Request", "Invalid authentication callback. Return to Cloud Terrastodon for details."),
                        };
                        // Browser delivery is bounded and cannot hide a denial or
                        // cancellation if the tab disconnects before reading it.
                        if let Err(error) = Self::write_response(&mut stream, status, body).await {
                            debug!(%error, "Could not deliver the browser authentication response");
                        }
                        return callback?.into_code(resource);
                    }
                }
            }
        }).await.wrap_err("timed out waiting for the browser OAuth callback; check that the browser can reach localhost")?
    }

    async fn accept(&self) -> std::io::Result<TcpStream> {
        let accepted = match &self.ipv6 {
            Some(ipv6) => tokio::select! {
                accepted = self.ipv4.accept() => accepted,
                accepted = ipv6.accept() => accepted,
            },
            None => self.ipv4.accept().await,
        };
        accepted.map(|(stream, _)| stream)
    }

    async fn read_request(mut stream: TcpStream) -> Result<(TcpStream, String)> {
        timeout(Duration::from_secs(30), async move {
            let mut buffer = vec![0_u8; 16 * 1024];
            let mut length = 0;
            loop {
                let read = stream
                    .read(&mut buffer[length..])
                    .await
                    .wrap_err("reading the browser OAuth callback")?;
                if read == 0 {
                    bail!("browser callback connection closed before completing its HTTP headers");
                }
                length += read;
                if buffer[..length]
                    .windows(4)
                    .any(|window| window == b"\r\n\r\n")
                {
                    break;
                }
                if length == buffer.len() {
                    bail!("browser OAuth callback request exceeded the 16 KiB limit");
                }
            }
            let request = String::from_utf8_lossy(&buffer[..length]);
            let target = request
                .lines()
                .next()
                .and_then(|line| {
                    let mut fields = line.split_whitespace();
                    let method = fields.next()?;
                    let target = fields.next()?;
                    (method == "GET").then_some(target)
                })
                .ok_or_else(|| {
                    eyre::eyre!("browser OAuth callback did not contain an HTTP GET request target")
                })?
                .to_owned();
            Ok((stream, target))
        })
        .await
        .wrap_err("timed out reading browser OAuth callback headers")?
    }

    async fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<()> {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len(),
        );
        timeout(Duration::from_secs(2), async {
            stream.write_all(response.as_bytes()).await?;
            stream.shutdown().await
        })
        .await
        .wrap_err("timed out responding to the browser OAuth callback")?
        .wrap_err("responding to the browser OAuth callback")
    }
}

#[cfg(test)]
#[path = "browser_authorization_callback_listener_tests.rs"]
mod tests;
