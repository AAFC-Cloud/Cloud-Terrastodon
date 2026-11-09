#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestHttpError;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::cell::Cell;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_cancellation::CancellationToken;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Notify;

struct Operation;
struct Wrapper;
static DECODES: AtomicUsize = AtomicUsize::new(0);
static CONVERSIONS: AtomicUsize = AtomicUsize::new(0);

// Neither Clone nor Facet: this is a domain conversion, not a second JSON decode.
#[derive(Debug)]
struct ConvertedNumber(u64);
impl TryFrom<u64> for ConvertedNumber {
    type Error = eyre::Report;
    fn try_from(value: u64) -> Result<Self> {
        CONVERSIONS.fetch_add(1, Ordering::SeqCst);
        eyre::ensure!(
            value != 0,
            "synthetic zero is rejected by the domain conversion"
        );
        Ok(Self(value))
    }
}
impl TryFrom<i64> for ConvertedNumber {
    type Error = eyre::Report;
    fn try_from(value: i64) -> Result<Self> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Facet)]
#[facet(proxy = u64)]
struct CountedNumber(u64);
impl TryFrom<u64> for CountedNumber {
    type Error = eyre::Report;
    fn try_from(value: u64) -> Result<Self> {
        DECODES.fetch_add(1, Ordering::SeqCst);
        Ok(Self(value))
    }
}
impl From<&CountedNumber> for u64 {
    fn from(value: &CountedNumber) -> Self {
        value.0
    }
}

/// A local TCP peer exercising the actual transport, never a production endpoint.
struct LocalServer {
    url: reqwest::Url,
    requests: Arc<AtomicUsize>,
    content_types: Arc<Mutex<Vec<Vec<String>>>>,
    observed: Arc<Notify>,
    task: tokio::task::JoinHandle<()>,
}
impl LocalServer {
    async fn start(status: u16, body: &'static str, gate: Option<Arc<Notify>>) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let url = reqwest::Url::parse(&format!("http://{}/synthetic", listener.local_addr()?))?;
        let requests = Arc::new(AtomicUsize::new(0));
        let content_types = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::new(Notify::new());
        let request_count = Arc::clone(&requests);
        let request_content_types = Arc::clone(&content_types);
        let observed_request = Arc::clone(&observed);
        let task = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let header_end = loop {
                    let mut bytes = [0; 4096];
                    let count = stream.read(&mut bytes).await.unwrap();
                    assert_ne!(count, 0);
                    request.extend_from_slice(&bytes[..count]);
                    if let Some(index) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                        break index + 4;
                    }
                };
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or_default();
                let content_types = headers
                    .lines()
                    .filter_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-type")
                            .then(|| value.trim().to_owned())
                    })
                    .collect();
                while request.len() < header_end + length {
                    let mut bytes = [0; 4096];
                    let count = stream.read(&mut bytes).await.unwrap();
                    assert_ne!(count, 0);
                    request.extend_from_slice(&bytes[..count]);
                }
                request_content_types.lock().unwrap().push(content_types);
                request_count.fetch_add(1, Ordering::SeqCst);
                observed_request.notify_one();
                if let Some(gate) = &gate {
                    gate.notified().await;
                }
                let response = format!(
                    "HTTP/1.1 {status} Synthetic\r\nContent-Type: application/json\r\nx-ms-continuationtoken: synthetic-next\r\nRetry-After: 1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                // A cancelled read may deliberately close before the response arrives.
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        Ok(Self {
            url,
            requests,
            content_types,
            observed,
            task,
        })
    }
    fn request(&self, method: Method) -> Result<RestRequest> {
        Ok(RestRequest::from_method_and_url_for_service(
            method,
            self.url.clone(),
            RestService::MicrosoftGraph,
        )?
        .authentication(AuthenticationSelection::AzureBearer {
            auth_source: AzureBearerAuthSource::Browser,
            tenant_id: "11111111-1111-1111-1111-111111111111".parse()?,
        })
        .cache(key()))
    }
}

#[tokio::test]
async fn body_content_types_preserve_explicit_media_types_and_default_to_json() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "7", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            server
                .request(Method::PATCH)?
                .headers(RequestHeaders::from_header(
                    "Content-Type",
                    "application/json-patch+json",
                )?)
                .body("[]")
                .receive::<u64>(&executor)
                .await?;
            server
                .request(Method::POST)?
                .body("{}")
                .receive::<u64>(&executor)
                .await?;
            assert_eq!(
                *server.content_types.lock().unwrap(),
                [
                    vec!["application/json-patch+json".to_owned()],
                    vec!["application/json".to_owned()],
                ]
            );
            Ok(())
        })
        .await
}
impl Drop for LocalServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn key() -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "rest"]).unwrap(),
        valid_for: Duration::from_secs(3600),
    }
}
fn files_named(root: &Path, name: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut pending = vec![root.to_path_buf()];
    let mut matches = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                pending.push(entry.path());
            } else if entry.file_name() == name {
                matches.push(entry.path());
            }
        }
    }
    Ok(matches)
}

#[tokio::test]
async fn canonical_conversion_shares_non_clone_values_and_rejects_another_wire_contract()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "7", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            let owner = CacheOperationOwner::of::<ConvertedNumber>();
            let before = CONVERSIONS.load(Ordering::SeqCst);
            let first = server
                .request(Method::GET)?
                .receive_converted_cached::<u64, ConvertedNumber>(&executor, owner)
                .await?;
            let again = server
                .request(Method::GET)?
                .receive_converted_cached::<u64, ConvertedNumber>(&executor, owner)
                .await?;
            assert!(Arc::ptr_eq(&first, &again));
            assert_eq!(first.0, 7);
            assert_eq!(CONVERSIONS.load(Ordering::SeqCst), before + 1);
            assert!(
                server
                    .request(Method::GET)?
                    .receive_converted_cached::<i64, ConvertedNumber>(&executor, owner)
                    .await
                    .is_err()
            );
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rejected_domain_conversion_retains_artifacts_without_publishing_a_result() -> Result<()> {
    // Separate conversion avoids sharing the successful test's instrumentation.
    #[derive(Debug)]
    struct Rejected;
    impl TryFrom<u64> for Rejected {
        type Error = std::io::Error;
        fn try_from(_: u64) -> std::result::Result<Self, Self::Error> {
            Err(std::io::Error::other("synthetic conversion rejection"))
        }
    }
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "0", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            for _ in 0..2 {
                let error = server
                    .request(Method::GET)?
                    .receive_converted_cached::<u64, Rejected>(
                        &executor,
                        CacheOperationOwner::of::<Rejected>(),
                    )
                    .await
                    .unwrap_err();
                assert_eq!(
                    error.downcast_ref::<std::io::Error>().unwrap().to_string(),
                    "synthetic conversion rejection"
                );
            }
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            assert_eq!(
                files_named(executor.cache()?.root(), "response.body.json")?.len(),
                2
            );
            assert!(files_named(executor.cache()?.root(), "selection.json")?.is_empty());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn canonical_hits_share_decode_but_keep_metadata_and_every_callers_policy() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "7", None).await?;
            let authentication = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::bearer("synthetic-test-credential")?,
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            DECODES.store(0, Ordering::SeqCst);
            let request = server.request(Method::GET)?;
            let owner = CacheOperationOwner::of::<Operation>();
            let (first, joined) = executor
                .join2(
                    async |child| {
                        request
                            .clone()
                            .receive_cached::<CountedNumber>(&child, owner)
                            .await
                    },
                    async |child| {
                        request
                            .clone()
                            .receive_cached::<CountedNumber>(&child, owner)
                            .await
                    },
                )
                .await?;
            assert!(Arc::ptr_eq(&first, &joined));
            let calls = Cell::new(0);
            let label = String::from("borrowed policy output");
            let (mapped, borrowed) = request
                .clone()
                .receive_with_mapper_cached(
                    |value: Arc<CountedNumber>, response| {
                        calls.set(calls.get() + 1);
                        assert_eq!(
                            response.header("x-ms-continuationtoken"),
                            Some("synthetic-next")
                        );
                        Ok((value, label.as_str()))
                    },
                    &executor,
                    owner,
                )
                .await?;
            assert!(Arc::ptr_eq(&first, &mapped));
            assert!(std::ptr::eq(borrowed.as_ptr(), label.as_ptr()));
            let error = request
                .clone()
                .receive_with_validator_cached::<CountedNumber, _>(
                    |_| {
                        calls.set(calls.get() + 1);
                        eyre::bail!("synthetic caller rejection");
                    },
                    &executor,
                    owner,
                )
                .await
                .err()
                .unwrap();
            assert!(format!("{error:#}").contains("synthetic caller rejection"));
            let (retained, token) = request
                .clone()
                .receive_with_ms_continuation_token_cached::<CountedNumber>(&executor, owner)
                .await?;
            assert!(Arc::ptr_eq(&first, &retained));
            assert_eq!(token.as_deref(), Some("synthetic-next"));
            assert_eq!(calls.get(), 2);
            assert_eq!(DECODES.load(Ordering::SeqCst), 1);
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 1);
            authentication
                .allow_authenticated
                .store(false, Ordering::SeqCst);
            assert!(
                request
                    .receive_cached::<CountedNumber>(&executor, owner)
                    .await
                    .is_err()
            );
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelling_the_transport_leader_releases_a_waiters_retry() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let gate = Arc::new(Notify::new());
            let server = LocalServer::start(200, "11", Some(gate.clone())).await?;
            let authentication = Arc::new(TestAuthenticationProvider::new("synthetic-test-session", Authorization::bearer("synthetic-test-credential")?)?);
            let cache = WorkCache::temporary()?;
            let executor = CloudTerrastodonExecutor::new(cache.clone(), authentication.clone(), scope);
            let owner = CacheOperationOwner::of::<Operation>();
            let request = server.request(Method::GET)?;
            let leader_token = CancellationToken::new();
            let leader_request = request.clone();
            let leader = leader_token.cancellable_scope(|leader_scope| async {
                let leader_executor = CloudTerrastodonExecutor::new(cache, authentication.clone(), leader_scope);
                leader_request.receive_cached::<u64>(&leader_executor, owner).await
            });
            let mut leader = Box::pin(leader);
            tokio::time::timeout(Duration::from_secs(10), async {
                tokio::select! {
                    result = &mut leader => panic!("leader completed before the peer released its response: {result:?}"),
                    () = server.observed.notified() => (),
                }
            }).await?;

            let waiter = request.clone().receive_cached::<u64>(&executor, owner);
            tokio::pin!(waiter);
            std::future::poll_fn(|context| {
                assert!(std::future::Future::poll(waiter.as_mut(), context).is_pending());
                std::task::Poll::Ready(())
            }).await;
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            leader_token.request_cancel("cancel only the read which reached the peer first");
            assert!(tokio::time::timeout(Duration::from_secs(10), &mut leader).await?.is_err());
            // The scope future owns its suspended child until dropped, as an awaited job would be.
            drop(leader);
            assert!(files_named(executor.cache()?.root(), "selection.json")?.is_empty());
            assert!(files_named(executor.cache()?.root(), "response.body.json")?.is_empty());

            // Release the abandoned response so this serial peer can accept the retry.
            gate.notify_one();
            tokio::time::timeout(Duration::from_secs(10), async {
                tokio::select! {
                    result = &mut waiter => panic!("waiter completed before its response was released: {result:?}"),
                    () = server.observed.notified() => (),
                }
            }).await?;
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            gate.notify_one();
            let value = tokio::time::timeout(Duration::from_secs(10), &mut waiter).await??;
            assert_eq!(*value, 11);
            let warm = request.receive_cached::<u64>(&executor, owner).await?;
            assert!(Arc::ptr_eq(&value, &warm));
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 2);
            assert_eq!(files_named(executor.cache()?.root(), "response.body.json")?.len(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn wrappers_and_rest_envelopes_share_a_scope_without_sharing_their_type_claim() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "[1,2,3]", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            let request = server.request(Method::GET)?;
            let wrapper = || {
                executor.cached_work(
                    ExecutorKind::InProcess,
                    key(),
                    CacheOperationOwner::of::<Wrapper>(),
                    &request.authentication,
                    NonEmptyString::new("wrapper")?,
                    WorkReuse::ReusableRead,
                    async |child| {
                        Ok(WorkOutput::new(
                            request
                                .clone()
                                .receive_cached::<Vec<u64>>(
                                    &child,
                                    CacheOperationOwner::of::<Operation>(),
                                )
                                .await?
                                .len(),
                            ArtifactObservation::now(),
                        ))
                    },
                )
            };
            assert_eq!(*wrapper()?.await?, 3);
            assert_eq!(*wrapper()?.await?, 3);
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            executor
                .cache()?
                .invalidate(&key().scope, executor.cancellation())
                .await?;
            assert_eq!(*wrapper()?.await?, 3);
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn owned_mutations_never_share_and_keep_their_outcome_after_cancellation() -> Result<()> {
    let token = CancellationToken::new();
    token
        .with_async_scope(|scope| async {
            let gate = Arc::new(Notify::new());
            let server = LocalServer::start(200, "9", Some(gate.clone())).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            let request = server.request(Method::POST)?.body("synthetic input");
            let future = request.clone().receive::<u64>(&executor);
            let control = async {
                server.observed.notified().await;
                token.request_cancel("cancel after mutation reached local peer");
                gate.notify_one();
            };
            let (value, ()) = tokio::join!(future, control);
            assert_eq!(value?, 9);
            assert_eq!(
                files_named(executor.cache()?.root(), "response.body.json")?.len(),
                1
            );
            assert!(files_named(executor.cache()?.root(), "selection.json")?.is_empty());
            eyre::Ok(())
        })
        .await?;
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "9", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            let request = server.request(Method::POST)?;
            let (first, second) = executor
                .join2(
                    async |child| request.clone().receive::<u64>(&child).await,
                    async |child| request.clone().receive::<u64>(&child).await,
                )
                .await?;
            assert_eq!((first, second), (9, 9));
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            assert!(
                request
                    .receive_cached::<u64>(&executor, CacheOperationOwner::of::<Operation>())
                    .await
                    .is_err()
            );
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn failures_keep_http_metadata_and_raw_artifacts_without_becoming_hits() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(429, "{\"synthetic\":true}", None).await?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::bearer("synthetic-test-credential")?,
                )?),
                scope,
            );
            for _ in 0..2 {
                let error = server
                    .request(Method::GET)?
                    .receive_cached::<u64>(&executor, CacheOperationOwner::of::<Operation>())
                    .await
                    .unwrap_err();
                let error = error.downcast_ref::<RestHttpError>().unwrap();
                assert_eq!(error.status, http::StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(error.headers.retry_after(), Some(Duration::from_secs(1)));
            }
            assert_eq!(server.requests.load(Ordering::SeqCst), 2);
            assert_eq!(
                files_named(executor.cache()?.root(), "response.body.json")?.len(),
                2
            );
            assert!(files_named(executor.cache()?.root(), "selection.json")?.is_empty());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn sensitive_failures_and_credential_header_rejection_leave_no_artifacts() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let server = LocalServer::start(200, "synthetic-private-body", None).await?;
            let authentication = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::bearer("synthetic-test-credential")?,
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                authentication.clone(),
                scope,
            );
            let error = server
                .request(Method::GET)?
                .artifact_retention(ArtifactRetention::Disabled)
                .receive::<u64>(&executor)
                .await
                .unwrap_err();
            assert!(!format!("{error:?}").contains("synthetic-private-body"));
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_none()
            );
            let error = server
                .request(Method::GET)?
                .headers(RequestHeaders::from_header(
                    "Authorization",
                    "Bearer synthetic-override",
                )?)
                .receive_raw(&executor)
                .await
                .unwrap_err();
            assert!(error.to_string().contains("executor"));
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 1);
            assert_eq!(server.requests.load(Ordering::SeqCst), 1);
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_none()
            );
            Ok(())
        })
        .await
}
