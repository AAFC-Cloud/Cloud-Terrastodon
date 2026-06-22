use bstr::BString;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableWorkRequest;
use cloud_terrastodon_command::CachedWorkSpec;
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
    let executions = Arc::new(AtomicUsize::new(0));
    let cache_key = unique_cache_key();

    let first = EchoWorkRequest {
        cache_key: cache_key.clone(),
        message: "hello".to_string(),
        executions: executions.clone(),
    };
    let second = EchoWorkRequest {
        cache_key,
        message: "hello".to_string(),
        executions: executions.clone(),
    };

    assert_eq!(first.await?, "hello");
    assert_eq!(second.await?, "hello");
    assert_eq!(executions.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn cached_work_writes_extra_files() -> eyre::Result<()> {
    let cache_key = unique_cache_key();
    let cache_dir = cache_key.path_on_disk();

    let result = run_cached_work(CachedWorkSpec {
        cache_key,
        context: "extra file test".to_string(),
        debug_inputs: BTreeMap::new(),
        extra_files: Some(|raw: &EchoRaw| {
            BTreeMap::from([(PathBuf::from("extra.txt"), raw.message.clone().into())])
        }),
        executor_kind: "test".to_string(),
        output_type: std::any::type_name::<String>().to_string(),
        execute_raw: || async {
            Ok(EchoRaw {
                message: "hello-extra".to_string(),
            })
        },
        decode: EchoWorkRequest::decode,
    })
    .await?;

    assert_eq!(result, "hello-extra");
    assert_eq!(
        tokio::fs::read_to_string(cache_dir.join("extra.txt")).await?,
        "hello-extra"
    );
    Ok(())
}
