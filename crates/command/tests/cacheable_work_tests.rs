use bstr::BString;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableWorkRequest;
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
