use crate::ArtifactMetadata;
use crate::CacheKey;
use crate::CommandOutput;
use crate::artifact_cache;
use async_trait::async_trait;
use bstr::BString;
use eyre::Context;
use eyre::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;

#[async_trait]
pub trait CacheableWorkRequest: Sized + Send {
    type Raw: Serialize + DeserializeOwned + Send + 'static;
    type Output;

    fn cache_key(&self) -> CacheKey;
    fn context(&self) -> String;
    fn debug_inputs(&self) -> BTreeMap<PathBuf, BString> {
        BTreeMap::new()
    }
    async fn execute_raw(self) -> Result<Self::Raw>;
    fn decode(raw: Self::Raw) -> Result<Self::Output>;
}

#[derive(Debug)]
pub struct CachedWorkSpec<Exec, ExecFuture, Decode, Raw, Output>
where
    Exec: FnOnce() -> ExecFuture + Send,
    ExecFuture: Future<Output = Result<Raw>> + Send,
    Decode: FnOnce(Raw) -> Result<Output> + Send,
{
    pub cache_key: CacheKey,
    pub context: String,
    pub debug_inputs: BTreeMap<PathBuf, BString>,
    pub executor_kind: String,
    pub output_type: String,
    pub execute_raw: Exec,
    pub decode: Decode,
}

#[derive(Serialize)]
struct WorkFingerprint<'a> {
    cache_path: String,
    context: &'a str,
    debug_inputs: &'a BTreeMap<PathBuf, BString>,
}

fn work_fingerprint(
    cache_key: &CacheKey,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
) -> Result<String> {
    let fingerprint = WorkFingerprint {
        cache_path: cache_key.path.to_string_lossy().into_owned(),
        context,
        debug_inputs,
    };
    let bytes = serde_json::to_vec(&fingerprint)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

async fn execute_and_cache_output<Exec, ExecFuture, Raw>(
    cache_key: &CacheKey,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    metadata: &ArtifactMetadata,
    execute_raw: Exec,
) -> Result<CommandOutput>
where
    Exec: FnOnce() -> ExecFuture + Send,
    ExecFuture: Future<Output = Result<Raw>> + Send,
    Raw: Serialize + DeserializeOwned + Send + 'static,
{
    let raw = execute_raw().await?;
    let stdout =
        serde_json::to_vec_pretty(&raw).context("serializing cached work raw output to JSON")?;
    let output = CommandOutput {
        stdout: BString::from(stdout),
        stderr: BString::default(),
        status: 0,
    };
    if let Err(error) = artifact_cache::write_output(
        &cache_key.path_on_disk(),
        context,
        debug_inputs,
        &output,
        metadata,
    )
    .await
    {
        artifact_cache::note_cache_write_failure(&error);
    } else {
        artifact_cache::put_memory_cache_entry(cache_key, &metadata.fingerprint, &output);
    }
    Ok(output)
}

pub async fn run_cached_work<Exec, ExecFuture, Decode, Raw, Output>(
    spec: CachedWorkSpec<Exec, ExecFuture, Decode, Raw, Output>,
) -> Result<Output>
where
    Exec: FnOnce() -> ExecFuture + Send,
    ExecFuture: Future<Output = Result<Raw>> + Send,
    Decode: FnOnce(Raw) -> Result<Output> + Send,
    Raw: Serialize + DeserializeOwned + Send + 'static,
{
    let CachedWorkSpec {
        cache_key,
        context,
        debug_inputs,
        executor_kind,
        output_type,
        execute_raw,
        decode,
    } = spec;
    let fingerprint = work_fingerprint(&cache_key, &context, &debug_inputs)?;
    let metadata = ArtifactMetadata::new(&fingerprint, executor_kind, output_type);

    let output =
        match artifact_cache::get_cached_output(&cache_key, &context, &debug_inputs, &fingerprint)
            .await
        {
            Ok(Some(output)) => output,
            Ok(None) => {
                execute_and_cache_output(
                    &cache_key,
                    &context,
                    &debug_inputs,
                    &metadata,
                    execute_raw,
                )
                .await?
            }
            Err(error) => {
                tracing::debug!(?cache_key, %error, "Cache load failed");
                execute_and_cache_output(
                    &cache_key,
                    &context,
                    &debug_inputs,
                    &metadata,
                    execute_raw,
                )
                .await?
            }
        };

    let raw =
        serde_json::from_slice::<Raw>(&output.stdout).map_err(|error| eyre::Error::new(error))?;
    match decode(raw) {
        Ok(result) => Ok(result),
        Err(error) => {
            let dump_dir = artifact_cache::write_failure(
                Some(&cache_key),
                &context,
                &debug_inputs,
                &output,
                &metadata,
                Some(&format!("{error:?}")),
            )
            .await?;
            Err(error).wrap_err(format!(
                "Decoded cached work failed, dumped to {:?}",
                dump_dir
            ))
        }
    }
}

#[doc(hidden)]
pub async fn run_cached_work_request<Request>(request: Request) -> Result<Request::Output>
where
    Request: CacheableWorkRequest,
{
    let cache_key = request.cache_key();
    let context = request.context();
    let debug_inputs = request.debug_inputs();
    run_cached_work(CachedWorkSpec {
        cache_key,
        context,
        debug_inputs,
        executor_kind: "in_process".to_string(),
        output_type: std::any::type_name::<Request::Output>().to_string(),
        execute_raw: move || request.execute_raw(),
        decode: Request::decode,
    })
    .await
}

#[macro_export]
macro_rules! impl_cacheable_work_into_future {
    ($ty:ty) => {
        impl ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableWorkRequest>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::run_cached_work_request(self))
            }
        }

        impl $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation = ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output>
                        + Send,
                >,
            >;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <$ty as $crate::CacheableWorkRequest>::cache_key(&self)
                            .invalidate()
                            .await?;
                    }
                    $crate::run_cached_work_request(self).await
                })
            }
        }
    };

    ($ty:ty, $lt:lifetime) => {
        impl<$lt> ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableWorkRequest>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send + $lt>>;

            fn into_future(self) -> Self::IntoFuture {
                Box::pin($crate::run_cached_work_request(self))
            }
        }

        impl<$lt> $crate::CacheInvalidatableIntoFuture for $ty {
            type WithInvalidation = ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = <Self as ::std::future::IntoFuture>::Output>
                        + Send
                        + $lt,
                >,
            >;

            fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
                Box::pin(async move {
                    if invalidate_cache {
                        <$ty as $crate::CacheableWorkRequest>::cache_key(&self)
                            .invalidate()
                            .await?;
                    }
                    $crate::run_cached_work_request(self).await
                })
            }
        }
    };
}
