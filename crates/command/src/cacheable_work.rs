use crate::ArtifactMetadata;
use crate::CacheKey;
use crate::CancellationScope;
use crate::CommandOutput;
use crate::artifact_cache;
use async_trait::async_trait;
use bstr::BString;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use teamy_cancellation::CancellationReportExt;

type RawExtraFilesFn<Raw> = fn(&Raw) -> BTreeMap<PathBuf, BString>;
type FailureExtraFilesFn = fn(&eyre::Report) -> BTreeMap<PathBuf, BString>;

#[async_trait]
pub trait CacheableWorkRequest: Sized + Send {
    type Raw: Facet<'static> + Send + 'static;
    type Output: Send + 'static;

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
    Decode: FnOnce(Raw) -> Result<Output> + Send + 'static,
    Output: Send + 'static,
{
    pub cache_key: CacheKey,
    pub context: String,
    pub debug_inputs: BTreeMap<PathBuf, BString>,
    pub extra_files: Option<RawExtraFilesFn<Raw>>,
    pub failure_extra_files: Option<FailureExtraFilesFn>,
    pub executor_kind: String,
    pub output_type: String,
    pub execute_raw: Exec,
    pub decode: Decode,
}

#[derive(facet::Facet)]
struct WorkFingerprint<'a> {
    cache_path: String,
    context: &'a str,
    debug_inputs: BTreeMap<String, BString>,
}

fn work_fingerprint(
    cache_key: &CacheKey,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
) -> Result<String> {
    let fingerprint = WorkFingerprint {
        cache_path: cache_key.path.to_string_lossy().into_owned(),
        context,
        debug_inputs: debug_inputs
            .iter()
            .map(|(path, contents)| (path.to_string_lossy().into_owned(), contents.clone()))
            .collect(),
    };
    let bytes = crate::json::to_vec(&fingerprint)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

async fn execute_and_cache_output<Exec, ExecFuture, Raw>(
    cache_key: &CacheKey,
    context: &str,
    debug_inputs: &BTreeMap<PathBuf, BString>,
    metadata: &ArtifactMetadata,
    extra_files: Option<RawExtraFilesFn<Raw>>,
    execute_raw: Exec,
    cancellation: &CancellationScope,
) -> Result<CommandOutput>
where
    Exec: FnOnce() -> ExecFuture + Send,
    ExecFuture: Future<Output = Result<Raw>> + Send,
    Raw: Facet<'static> + Send + 'static,
{
    let raw = execute_raw().await?;
    cancellation
        .finish_required("persisting request output and cache artifacts", |_| async {
            let (output, extra_files) = tokio::task::spawn_blocking(move || -> Result<_> {
                let extra_files = extra_files.map(|f| f(&raw)).unwrap_or_default();
                let stdout = crate::json::to_vec_pretty(&raw)
                    .context("serializing cached work raw output to JSON")?;
                let output = CommandOutput {
                    stdout: BString::from(stdout),
                    stderr: BString::default(),
                    status: 0,
                };
                Ok((output, extra_files))
            })
            .await??;
            if let Err(error) = artifact_cache::write_output_with_extra_files(
                &cache_key.path_on_disk(),
                context,
                debug_inputs,
                &output,
                metadata,
                &extra_files,
            )
            .await
            {
                artifact_cache::note_cache_write_failure(&error);
            } else {
                artifact_cache::put_memory_cache_entry(cache_key, &metadata.fingerprint, &output);
            }
            Ok(output)
        })
        .await
}

pub async fn run_cached_work<Exec, ExecFuture, Decode, Raw, Output>(
    spec: CachedWorkSpec<Exec, ExecFuture, Decode, Raw, Output>,
    cancellation: &CancellationScope,
) -> Result<Output>
where
    Exec: FnOnce() -> ExecFuture + Send,
    ExecFuture: Future<Output = Result<Raw>> + Send,
    Decode: FnOnce(Raw) -> Result<Output> + Send,
    Raw: Facet<'static> + Send + 'static,
    Output: Send + 'static,
{
    let CachedWorkSpec {
        cache_key,
        context,
        debug_inputs,
        extra_files,
        failure_extra_files,
        executor_kind,
        output_type,
        execute_raw,
        decode,
    } = spec;
    let fingerprint = work_fingerprint(&cache_key, &context, &debug_inputs)?;
    let metadata = ArtifactMetadata::new(&fingerprint, executor_kind, output_type);

    let output = match cancellation
        .cancellable_scope(|_| {
            artifact_cache::get_cached_output(&cache_key, &context, &debug_inputs, &fingerprint)
        })
        .await
    {
        Ok(Some(output)) => {
            cancellation.bail_if_cancelled()?;
            output
        }
        Ok(None) => {
            cancellation.bail_if_cancelled()?;
            execute_and_cache_output(
                &cache_key,
                &context,
                &debug_inputs,
                &metadata,
                extra_files,
                execute_raw,
                cancellation,
            )
            .await?
        }
        Err(error) if error.is_cancellation() => return Err(error),
        Err(error) => {
            tracing::debug!(?cache_key, %error, "Cache load failed");
            cancellation.bail_if_cancelled()?;
            execute_and_cache_output(
                &cache_key,
                &context,
                &debug_inputs,
                &metadata,
                extra_files,
                execute_raw,
                cancellation,
            )
            .await?
        }
    };

    let raw_result = tokio::task::spawn_blocking(move || -> Result<_> {
        let raw = crate::json::from_slice::<Raw>(&output.stdout)?;
        let extra_files = extra_files.map(|f| f(&raw)).unwrap_or_default();
        Ok((output, raw, extra_files))
    })
    .await?
    .wrap_err(format!(
        "Deserializing cached work output{}",
        artifact_cache::cache_clean_recommendation(Some(&cache_key))
    ))?;
    let (output, raw, extra_files) = raw_result;
    let decode_result = tokio::task::spawn_blocking(move || decode(raw)).await?;
    match decode_result {
        Ok(result) => Ok(result),
        Err(error) if error.is_cancellation() => Err(error),
        Err(error) => {
            let mut extra_files = extra_files;
            if let Some(failure_extra_files) = failure_extra_files {
                extra_files.extend(failure_extra_files(&error));
            }
            let dump_dir = artifact_cache::write_failure_with_extra_files(
                Some(&cache_key),
                &context,
                &debug_inputs,
                &output,
                &metadata,
                Some(&format!("{error:?}")),
                &extra_files,
            )
            .await?;
            Err(error).wrap_err(format!(
                "Decoded cached work failed, dumped to {:?}{}",
                dump_dir,
                artifact_cache::cache_clean_recommendation(Some(&cache_key))
            ))
        }
    }
}

#[doc(hidden)]
pub async fn run_cached_work_request<Request>(
    request: Request,
    cancellation: &CancellationScope,
) -> Result<Request::Output>
where
    Request: CacheableWorkRequest + 'static,
{
    let cache_key = request.cache_key();
    let context = request.context();
    let debug_inputs = request.debug_inputs();
    run_cached_work(
        CachedWorkSpec {
            cache_key,
            context,
            debug_inputs,
            extra_files: None,
            failure_extra_files: None,
            executor_kind: "in_process".to_string(),
            output_type: std::any::type_name::<Request::Output>().to_string(),
            execute_raw: move || request.execute_raw(),
            decode: Request::decode,
        },
        cancellation,
    )
    .await
}

#[macro_export]
macro_rules! impl_cacheable_work_into_future {
    ($ty:ty) => {
        impl ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableWorkRequest>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send>>;

            fn into_future(mut self) -> Self::IntoFuture {
                let parent = self.cancellation.clone();
                Box::pin(parent.cancellable_scope(move |scope| async move {
                    self.cancellation = scope.clone();
                    $crate::run_cached_work_request(self, &scope).await
                }))
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
                        self.cancellation
                            .deferred(
                                "finishing filesystem and memory cache invalidation",
                                |_| async {
                                    <$ty as $crate::CacheableWorkRequest>::cache_key(&self)
                                        .invalidate()
                                        .await
                                },
                            )
                            .await?;
                    }
                    ::std::future::IntoFuture::into_future(self).await
                })
            }
        }
    };

    ($ty:ty, $lt:lifetime) => {
        impl<$lt> ::std::future::IntoFuture for $ty {
            type Output = ::eyre::Result<<$ty as $crate::CacheableWorkRequest>::Output>;
            type IntoFuture =
                ::std::pin::Pin<Box<dyn ::std::future::Future<Output = Self::Output> + Send + $lt>>;

            fn into_future(mut self) -> Self::IntoFuture {
                let parent = self.cancellation.clone();
                Box::pin(parent.cancellable_scope(move |scope| async move {
                    self.cancellation = scope.clone();
                    $crate::run_cached_work_request(self, &scope).await
                }))
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
                        self.cancellation
                            .deferred(
                                "finishing filesystem and memory cache invalidation",
                                |_| async {
                                    <$ty as $crate::CacheableWorkRequest>::cache_key(&self)
                                        .invalidate()
                                        .await
                                },
                            )
                            .await?;
                    }
                    ::std::future::IntoFuture::into_future(self).await
                })
            }
        }
    };
}
