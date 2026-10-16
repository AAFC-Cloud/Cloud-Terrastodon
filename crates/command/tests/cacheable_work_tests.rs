use bstr::BString;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableWorkRequest;
use cloud_terrastodon_command::CachedWorkSpec;
use cloud_terrastodon_command::CancellationError;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::run_cached_work;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
struct EchoWorkRequest {
    cancellation: CancellationScope,
    cache_key: CacheKey,
    message: String,
    executions: Arc<AtomicUsize>,
}

#[derive(Debug, facet::Facet)]
struct EchoRaw {
    message: String,
}

#[cloud_terrastodon_command::async_trait]
impl CacheableWorkRequest for EchoWorkRequest {
    type Raw = EchoRaw;
    type Output = String;

    fn cache_key(&self) -> CacheKey {
        self.cache_key.clone()
    }

    fn context(&self) -> String {
        format!("echo {}", self.message)
    }

    fn debug_inputs(&self) -> BTreeMap<PathBuf, BString> {
        BTreeMap::from([(PathBuf::from("input.txt"), self.message.clone().into())])
    }

    async fn execute_raw(self) -> eyre::Result<Self::Raw> {
        self.executions.fetch_add(1, Ordering::SeqCst);
        Ok(EchoRaw {
            message: self.message,
        })
    }

    fn decode(raw: Self::Raw) -> eyre::Result<Self::Output> {
        Ok(raw.message)
    }
}

cloud_terrastodon_command::impl_cacheable_work_into_future!(EchoWorkRequest);

fn unique_cache_key() -> CacheKey {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos()
        .to_string();
    CacheKey::new(PathBuf::from_iter([
        "tests",
        "cacheable_work",
        suffix.as_str(),
    ]))
}

#[tokio::test]
async fn reuses_cached_in_process_work_without_reexecuting() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let executions = Arc::new(AtomicUsize::new(0));
            let cache_key = unique_cache_key();

            let first = EchoWorkRequest {
                cancellation: cancellation.clone(),
                cache_key: cache_key.clone(),
                message: "hello".to_string(),
                executions: executions.clone(),
            };
            let second = EchoWorkRequest {
                cancellation: cancellation.clone(),
                cache_key,
                message: "hello".to_string(),
                executions: executions.clone(),
            };

            assert_eq!(first.await?, "hello");
            assert_eq!(second.await?, "hello");
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cached_work_writes_extra_files() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let cache_key = unique_cache_key();
            let cache_dir = cache_key.path_on_disk();

            let result = run_cached_work(
                CachedWorkSpec {
                    cache_key,
                    context: "extra file test".to_string(),
                    debug_inputs: BTreeMap::new(),
                    extra_files: Some(|raw: &EchoRaw| {
                        BTreeMap::from([(PathBuf::from("extra.txt"), raw.message.clone().into())])
                    }),
                    failure_extra_files: None,
                    executor_kind: "test".to_string(),
                    output_type: std::any::type_name::<String>().to_string(),
                    execute_raw: || async {
                        Ok(EchoRaw {
                            message: "hello-extra".to_string(),
                        })
                    },
                    decode: EchoWorkRequest::decode,
                },
                &cancellation,
            )
            .await?;

            assert_eq!(result, "hello-extra");
            assert_eq!(
                tokio::fs::read_to_string(cache_dir.join("extra.txt")).await?,
                "hello-extra"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cached_work_writes_failure_extra_files() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let cache_key = unique_cache_key();
            let failed_parent = cache_key.path_on_disk().join("failed");

            let result: eyre::Result<String> = run_cached_work(
                CachedWorkSpec {
                    cache_key: cache_key.clone(),
                    context: "failure extra file test".to_string(),
                    debug_inputs: BTreeMap::new(),
                    extra_files: None,
                    failure_extra_files: Some(|error: &eyre::Report| {
                        BTreeMap::from([(
                            PathBuf::from("failed-extra.txt"),
                            error.to_string().into(),
                        )])
                    }),
                    executor_kind: "test".to_string(),
                    output_type: std::any::type_name::<String>().to_string(),
                    execute_raw: || async {
                        Ok(EchoRaw {
                            message: "hello-failure-extra".to_string(),
                        })
                    },
                    decode: |_| eyre::bail!("decode failed"),
                },
                &cancellation,
            )
            .await;

            let _error = result.expect_err("decode should fail");

            let mut entries = tokio::fs::read_dir(&failed_parent).await?;
            let dump_dir = entries
                .next_entry()
                .await?
                .expect("failure dump should be written")
                .path();
            assert!(entries.next_entry().await?.is_none());

            assert_eq!(
                tokio::fs::read_to_string(dump_dir.join("failed-extra.txt")).await?,
                "decode failed"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelled_request_with_invalidation_disabled_never_executes() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let executions = Arc::new(AtomicUsize::new(0));
            let request = EchoWorkRequest {
                cancellation: cancellation.clone(),
                cache_key: unique_cache_key(),
                message: "cancelled request".to_owned(),
                executions: executions.clone(),
            };
            cancellation.request_cancel("cancel before request admission");

            let error = request
                .with_invalidation(false)
                .await
                .expect_err("cancelled requests must be rejected before execution");
            assert!(
                error.downcast_ref::<CancellationError>().is_some(),
                "{error:#}"
            );
            assert_eq!(executions.load(Ordering::SeqCst), 0);
            Ok(())
        })
        .await
}
