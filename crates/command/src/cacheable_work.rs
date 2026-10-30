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

#[expect(
    clippy::double_must_use,
    reason = "async-trait adds #[must_use] to an already must-use future"
)]
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

    let cached_output = match cancellation
        .cancellable_scope(|_| {
            artifact_cache::get_cached_output(&cache_key, &context, &debug_inputs, &fingerprint)
        })
        .await
    {
        Ok(output) => output,
        Err(error) if error.is_cancellation() => return Err(error),
        Err(error) => {
            tracing::debug!(?cache_key, %error, "Cache load failed");
            None
        }
    };
    cancellation.bail_if_cancelled()?;
    let from_cache = cached_output.is_some();
    let fresh_raw = if from_cache {
        None
    } else {
        Some(execute_raw().await?)
    };

    // Once execution has produced an outcome, retain it even if cancellation
    // arrives while decoding. Only a successfully decoded result is reusable;
    // rejected outcomes belong in failure artifacts, including mutation results.
    cancellation
        .finish_required(
            "decoding request output and retaining its artifacts",
            |_| async {
                let (output, mut extra_files, decoded) =
                    tokio::task::spawn_blocking(move || -> Result<_> {
                        if let Some(raw) = fresh_raw {
                            let extra_files = extra_files.map(|f| f(&raw)).unwrap_or_default();
                            let output = CommandOutput {
                                stdout: crate::json::to_vec_pretty(&raw)
                                    .context("serializing cached work raw output to JSON")?
                                    .into(),
                                stderr: BString::default(),
                                status: 0,
                            };
                            // Decode the actual response, not a serialized roundtrip
                            // whose cache representation may differ from its payload.
                            let decoded = decode(raw);
                            Ok((output, extra_files, decoded))
                        } else {
                            let output = cached_output.ok_or_else(|| {
                                eyre::eyre!(
                                    "cached work has neither a cached output nor a fresh response"
                                )
                            })?;
                            let raw = crate::json::from_slice::<Raw>(&output.stdout)
                                .context("Deserializing cached work output");
                            let extra_files = raw
                                .as_ref()
                                .ok()
                                .and_then(|raw| extra_files.map(|f| f(raw)))
                                .unwrap_or_default();
                            let decoded = raw.and_then(decode);
                            Ok((output, extra_files, decoded))
                        }
                    })
                    .await??;
                match decoded {
                    Ok(result) => {
                        if !from_cache {
                            if let Err(error) = artifact_cache::write_output_with_extra_files(
                                &cache_key.path_on_disk(),
                                &context,
                                &debug_inputs,
                                &output,
                                &metadata,
                                &extra_files,
                            )
                            .await
                            {
                                artifact_cache::note_cache_write_failure(&error);
                            } else {
                                if let Err(error) = artifact_cache::put_memory_cache_entry(
                                    &cache_key,
                                    &metadata.fingerprint,
                                    &output,
                                ) {
                                    artifact_cache::note_cache_write_failure(&error);
                                }
                            }
                        }
                        Ok(result)
                    }
                    // A cancelled decoder has not rejected the cached payload, and
                    // its outcome is already retained in the existing cache entry.
                    Err(error) if from_cache && error.is_cancellation() => Err(error),
                    Err(error) => {
                        let invalidation_error = if from_cache {
                            artifact_cache::invalidate_cache_entry(&cache_key)
                                .await
                                .err()
                        } else {
                            None
                        };
                        let error = match invalidation_error {
                            Some(ref invalidation_error) => error.wrap_err(format!(
                                "Could not invalidate rejected cache entry: {invalidation_error:#}"
                            )),
                            None => error,
                        };
                        if let Some(failure_extra_files) = failure_extra_files {
                            extra_files.extend(failure_extra_files(&error));
                        }
                        let dump_result = artifact_cache::write_failure_with_extra_files(
                            Some(&cache_key),
                            &context,
                            &debug_inputs,
                            &output,
                            &metadata,
                            Some(&format!("{error:?}")),
                            &extra_files,
                        )
                        .await;
                        let dump_dir = match dump_result {
                            Ok(dump_dir) => dump_dir,
                            Err(dump_error) => {
                                return Err(error).wrap_err(format!(
                                    "Could not retain rejected response artifacts: {dump_error:#}"
                                ));
                            }
                        };
                        // Never replay a rejected cached operation automatically:
                        // the executor may mutate state and both closures are FnOnce.
                        let disposition = if invalidation_error.is_some() {
                            "rejected entry may remain reusable"
                        } else if from_cache {
                            "rejected cache entry was invalidated"
                        } else {
                            "response was not cached"
                        };
                        Err(error).wrap_err(format!(
                            "Decoded cached work failed; {disposition}, dumped to {dump_dir:?}"
                        ))
                    }
                }
            },
        )
        .await
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
