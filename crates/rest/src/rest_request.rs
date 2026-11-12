use crate::MicrosoftContinuationToken;
use crate::RequestHeaders;
use crate::RestOutputFormat;
use crate::RestResponseBody;
use crate::RestService;
use crate::SerializableRestResponse;
use crate::decoded_rest_response::DecodedRestResponse;
use crate::execute_rest_request;
use crate::rest_request_fingerprint::RestRequestFingerprint;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactFilePath;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use http::Method;
use reqwest::Url;
use std::collections::BTreeMap;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationReportExt;
use tracing::Instrument;
use tracing::Span;
use tracing::info_span;

type ResponseFailureExtraFiles = fn(&Report) -> BTreeMap<PathBuf, bstr::BString>;

/// A transport request containing authentication selection rather than credentials.
/// Owned receives execute independently; explicit cached receives share decoded values.
#[derive(Clone)]
pub struct RestRequest {
    pub complete_on_cancellation: bool,
    pub authentication: AuthenticationSelection,
    pub service: RestService,
    pub method: Method,
    pub url: Url,
    pub body: Option<String>,
    pub headers: Option<RequestHeaders>,
    pub cache_key: Option<CacheKey>,
    pub output_format: RestOutputFormat,
    pub failure_extra_files: Option<ResponseFailureExtraFiles>,
    pub artifact_retention: ArtifactRetention,
    /// Bound the HTTP exchange, including body delivery. Credential acquisition,
    /// decoding, artifact persistence and required completion retain their own lifetime.
    pub transport_timeout: Option<Duration>,
}

impl std::fmt::Debug for RestRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RestRequest")
            .field("authentication", &self.authentication)
            .field("service", &self.service)
            .field("method", &self.method)
            .field("summary", &self.context())
            .field("cache_key", &self.cache_key)
            .field("artifact_retention", &self.artifact_retention)
            .field("transport_timeout", &self.transport_timeout)
            .finish_non_exhaustive()
    }
}

impl RestRequest {
    /// Parse an absolute service URL without acquiring execution resources.
    #[track_caller]
    pub fn new(method: Method, url: impl AsRef<str>) -> Result<Self> {
        Self::from_method_and_url(
            method,
            Url::parse(url.as_ref()).wrap_err("parsing REST URL")?,
        )
    }
    /// Construct from an already parsed URL and infer its service family.
    #[track_caller]
    pub fn from_method_and_url(method: Method, url: Url) -> Result<Self> {
        let service =
            RestService::infer(&url).ok_or_else(|| eyre::eyre!("unsupported REST host"))?;
        Self::from_method_and_url_for_service(method, url, service)
    }
    /// A trusted SDK boundary may select the service for a validated Server URL.
    #[track_caller]
    pub fn from_method_and_url_for_service(
        method: Method,
        url: Url,
        service: RestService,
    ) -> Result<Self> {
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
            "REST destinations must be absolute HTTP(S) URLs"
        );
        ensure!(
            url.username().is_empty() && url.password().is_none(),
            "REST destinations must not contain embedded credentials"
        );
        Ok(Self {
            complete_on_cancellation: !matches!(
                method,
                Method::GET | Method::HEAD | Method::OPTIONS
            ),
            authentication: AuthenticationSelection::Anonymous,
            service,
            method,
            url,
            body: None,
            headers: None,
            cache_key: None,
            output_format: RestOutputFormat::default(),
            failure_extra_files: None,
            artifact_retention: ArtifactRetention::Full,
            transport_timeout: None,
        })
    }
    /// Select non-secret authentication intent; the executor supplies credentials.
    pub fn authentication(mut self, selection: AuthenticationSelection) -> Self {
        self.authentication = selection;
        self
    }
    /// Set the serialized request body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    pub fn headers(mut self, headers: RequestHeaders) -> Self {
        self.headers = Some(headers);
        self
    }
    pub fn cache(mut self, key: CacheKey) -> Self {
        self.cache_key = Some(key);
        self
    }
    pub fn use_cache(mut self, key: Option<CacheKey>) -> Self {
        self.cache_key = key;
        self
    }
    pub fn output_format(mut self, format: RestOutputFormat) -> Self {
        self.output_format = format;
        self
    }
    pub fn failure_extra_files(mut self, files: ResponseFailureExtraFiles) -> Self {
        self.failure_extra_files = Some(files);
        self
    }
    pub fn artifact_retention(mut self, retention: ArtifactRetention) -> Self {
        self.artifact_retention = retention;
        self
    }
    /// Limit only the HTTP exchange, including response-body delivery. A timeout
    /// remains an execution error that still passes through artifact retention.
    pub fn transport_timeout(mut self, timeout: Duration) -> Self {
        self.transport_timeout = Some(timeout);
        self
    }
    /// Mark a documented read-only operation as safely interruptible.
    pub fn cancellation_safe(mut self) -> Self {
        self.complete_on_cancellation = false;
        self
    }

    fn context(&self) -> String {
        if self.artifact_retention == ArtifactRetention::Disabled {
            "REST request with suppressed sensitive diagnostics".to_owned()
        } else {
            format!("rest {} {}", self.method, self.url)
        }
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            matches!(self.url.scheme(), "http" | "https")
                && self.url.host_str().is_some()
                && self.url.username().is_empty()
                && self.url.password().is_none(),
            "REST destination must be an absolute HTTP(S) URL without credentials"
        );
        let compatible = matches!(
            (self.service, self.authentication),
            (
                RestService::AzureRetailPrices,
                AuthenticationSelection::Anonymous
            ) | (
                RestService::AzureDevOps,
                AuthenticationSelection::AzureDevOpsPersonalAccessToken { .. }
            ) | (
                RestService::AzureDevOps
                    | RestService::MicrosoftGraph
                    | RestService::AzureResourceManager
                    | RestService::AzureKeyVault
                    | RestService::AzureContainerRegistry,
                AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection { .. })
            )
        );
        ensure!(
            compatible,
            "REST service requires an explicit compatible authentication selection"
        );
        if let Some(headers) = &self.headers {
            let headers = headers.to_header_map()?;
            ensure!(
                !headers.contains_key(http::header::AUTHORIZATION)
                    && !headers.contains_key(http::header::PROXY_AUTHORIZATION),
                "REST authorization must come from the executor, not request headers"
            );
        }
        Ok(())
    }
    fn fingerprint(&self) -> Result<NonEmptyString> {
        let mut headers = BTreeMap::<String, Vec<String>>::new();
        if let Some(values) = &self.headers {
            for (name, value) in &values.to_header_map()? {
                headers
                    .entry(name.as_str().to_owned())
                    .or_default()
                    .push(value.to_str()?.to_owned());
            }
        }
        let value = RestRequestFingerprint {
            service: format!("{:?}", self.service),
            method: self.method.to_string(),
            url: self.url.to_string(),
            body: self.body.clone(),
            headers,
        };
        let bytes = facet_json::to_vec(&value).map_err(|error| eyre::eyre!("{error:?}"))?;
        NonEmptyString::new(blake3::hash(&bytes).to_hex().to_string())
    }
    fn artifact_key(&self, caller: &'static Location<'static>) -> Result<CacheKey> {
        match &self.cache_key {
            Some(key) => Ok(key.clone()),
            None => Ok(CacheKey {
                scope: CacheScope::try_from_components([
                    "uncached".to_owned(),
                    caller.file().to_owned(),
                    caller.line().to_string(),
                    caller.column().to_string(),
                ])?,
                valid_for: Duration::ZERO,
            }),
        }
    }
    fn inputs(&self) -> Result<BTreeMap<ArtifactFilePath, Vec<u8>>> {
        let mut inputs = BTreeMap::new();
        if self.artifact_retention == ArtifactRetention::Disabled {
            return Ok(inputs);
        }
        if let Some(body) = &self.body {
            inputs.insert(
                ArtifactFilePath::new("body.json")?,
                body.as_bytes().to_vec(),
            );
        }
        if let Some(headers) = &self.headers {
            inputs.insert(
                ArtifactFilePath::new("headers.json")?,
                headers.to_json_pretty()?.into_bytes(),
            );
        }
        Ok(inputs)
    }
    fn diagnostics(
        &self,
        response: Option<&SerializableRestResponse>,
        error: Option<&Report>,
    ) -> Result<BTreeMap<ArtifactFilePath, Vec<u8>>> {
        let mut files = BTreeMap::new();
        if self.artifact_retention == ArtifactRetention::Disabled {
            return Ok(files);
        }
        files.insert(
            ArtifactFilePath::new("context.txt")?,
            self.context().into_bytes(),
        );
        if let Some(response) = response {
            files.insert(
                ArtifactFilePath::new("response.headers.json")?,
                facet_json::to_vec_pretty(&response.headers)
                    .map_err(|error| eyre::eyre!("{error:?}"))?,
            );
            files.insert(
                ArtifactFilePath::new("status.txt")?,
                response.status.as_u16().to_string().into_bytes(),
            );
            let (name, body) = match &response.body {
                RestResponseBody::Json(body) => ("response.body.json", body.as_str()),
                RestResponseBody::Text(body) => ("response.body.txt", body.as_str()),
            };
            files.insert(ArtifactFilePath::new(name)?, body.as_bytes().to_vec());
        }
        if let Some(error) = error {
            files.insert(
                ArtifactFilePath::new("error.txt")?,
                format!("{error:?}").into_bytes(),
            );
            if let Some(extra) = self.failure_extra_files {
                for (path, bytes) in extra(error) {
                    let path = path
                        .to_str()
                        .ok_or_else(|| eyre::eyre!("REST diagnostic filename is not Unicode"))?;
                    files.insert(
                        ArtifactFilePath::new(path.replace('\\', "/"))?,
                        bytes.to_vec(),
                    );
                }
            }
        }
        Ok(files)
    }
    async fn with_policy<C, F, T>(
        executor: &CloudTerrastodonExecutor,
        selection: AuthenticationSelection,
        protected: bool,
        callback: C,
    ) -> Result<T>
    where
        C: FnOnce(CloudTerrastodonExecutor) -> F,
        F: Future<Output = Result<T>>,
    {
        let executor = executor.with_authentication_lease(&selection)?;
        executor
            .with_account_cancellation(move |executor| async move {
                if protected {
                    executor
                        .deferred("complete REST response, decoding, and artifacts", callback)
                        .await
                } else {
                    executor.cancellable_scope(callback).await
                }
            })
            .await
    }
    async fn execute_transport(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<SerializableRestResponse> {
        let response = execute_rest_request(
            self.service,
            self.method.clone(),
            self.url.clone(),
            self.body.clone(),
            self.headers.clone(),
            self.transport_timeout,
            &self.authentication,
            executor,
        )
        .await?;
        SerializableRestResponse::from_response(response).await
    }
    fn decode_owned<T: Facet<'static>>(
        response: &SerializableRestResponse,
    ) -> Result<DecodedRestResponse<T>> {
        response.error_for_status()?;
        Ok(DecodedRestResponse {
            value: response.decode_json_body()?,
            response: response.clone(),
        })
    }
    fn decode_shared<T: Facet<'static>>(
        response: &SerializableRestResponse,
    ) -> Result<DecodedRestResponse<Arc<T>>> {
        response.error_for_status()?;
        Ok(DecodedRestResponse {
            value: Arc::new(response.decode_json_body()?),
            response: response.clone(),
        })
    }
    async fn decode<T: Send + 'static>(
        &self,
        response: SerializableRestResponse,
        decoder: fn(&SerializableRestResponse) -> Result<T>,
        caller: &'static Location<'static>,
    ) -> Result<(SerializableRestResponse, Result<T>)> {
        let span = Span::current();
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        let sensitive = self.artifact_retention == ArtifactRetention::Disabled;
        tokio::task::spawn_blocking(move || {
            tracing::dispatcher::with_default(&dispatch, || {
                span.in_scope(|| {
                    let decoded = decoder(&response).map_err(|mut error| {
                        if sensitive {
                            return eyre::eyre!(
                                "sensitive REST response could not be decoded as {}",
                                std::any::type_name::<T>()
                            );
                        }
                        error.handler_mut().track_caller(caller);
                        error
                    });
                    (response, decoded)
                })
            })
        })
        .await
        .wrap_err("REST decoding worker failed")
    }
    async fn canonical<T: Send + 'static>(
        &self,
        executor: &CloudTerrastodonExecutor,
        entry: ArtifactEntry,
        reusable: bool,
        decoder: fn(&SerializableRestResponse) -> Result<T>,
        caller: &'static Location<'static>,
    ) -> Result<WorkOutput<T>> {
        if reusable && let Some(loaded) = entry.load(executor.cancellation()).await? {
            let span = Span::current();
            let dispatch = tracing::dispatcher::get_default(Clone::clone);
            let (loaded, decoded) = tokio::task::spawn_blocking(move || {
                tracing::dispatcher::with_default(&dispatch, || {
                    span.in_scope(|| {
                        let decoded = facet_json::from_slice::<SerializableRestResponse>(
                            &loaded.artifacts.payload,
                        )
                        .map_err(|error| eyre::eyre!("{error:?}"))
                        .and_then(|response| decoder(&response))
                        .map_err(|mut error| {
                            error.handler_mut().track_caller(caller);
                            error
                        });
                        (loaded, decoded)
                    })
                })
            })
            .await
            .wrap_err("cached REST decoding worker failed")?;
            return match decoded {
                Ok(value) => Ok(loaded.decoded(value)),
                Err(error) => executor.cancellation().finish_required("reject invalid REST artifact and retain diagnostics", |scope| async move {
                    let retained = async {
                        entry.reject(&scope).await?;
                        let mut artifacts = loaded.artifacts;
                        artifacts.diagnostics.extend(self.diagnostics(None, Some(&error))?);
                        if let Some(attempt) = entry.begin_attempt(self.inputs()?, &scope).await? { attempt.finish(artifacts, ArtifactObservation::now(), &scope).await?; }
                        eyre::Ok(())
                    }.await;
                    let error = match retained {
                        Ok(()) => error,
                        Err(retention) => error.wrap_err(format!("Rejecting or retaining the failed REST artifact also failed: {retention:#}")),
                    };
                    Err(error).wrap_err_with(|| format!("Canonical REST cache decoding failed; called from {}", RelativeLocation::from(caller)))
                }).await,
            };
        }
        let attempt = entry
            .begin_attempt(self.inputs()?, executor.cancellation())
            .await?;
        let outcome = self.execute_transport(executor).await;
        let observed = ArtifactObservation::now();
        executor.cancellation().finish_required("decode and retain observed REST outcome", |scope| async move {
            let (response, decoded) = match outcome {
                Ok(response) => { let (response, decoded) = self.decode(response, decoder, caller).await?; (Some(response), decoded) }
                Err(error) => (None, Err(error)),
            };
            let diagnostics = self.diagnostics(response.as_ref(), decoded.as_ref().err());
            let stored = async {
                if let Some(attempt) = attempt {
                    let payload = response.as_ref().map(facet_json::to_vec).transpose().map_err(|error| eyre::eyre!("{error:?}"))?.unwrap_or_default();
                    eyre::Ok(Some(attempt.finish(ArtifactSet { payload, diagnostics: diagnostics? }, observed, &scope).await?))
                } else { Ok(None) }
            }.await;
            match decoded {
                Ok(value) => {
                    let stored = stored?;
                    if reusable && let Some(stored) = &stored { entry.publish(stored, &scope).await?; }
                    Ok(WorkOutput::new(value, observed))
                }
                Err(error) => {
                    let error = if self.artifact_retention == ArtifactRetention::Disabled && !error.is_cancellation() { eyre::eyre!("sensitive REST execution failed") } else { error };
                    match stored {
                        Ok(stored) => Err(error).wrap_err_with(|| format!("REST execution or decoding failed; called from {}; artifacts: {:?}", RelativeLocation::from(caller), stored.as_ref().map(|value| value.directory()))),
                        Err(retention) => Err(error).wrap_err_with(|| format!("REST execution or decoding failed; called from {}; retaining its artifacts also failed: {retention:#}", RelativeLocation::from(caller))),
                    }
                }
            }
        }).await
    }
    fn receive_owned_from<'a, T: Send + 'static>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        decoder: fn(&SerializableRestResponse) -> Result<T>,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        let span = info_span!("rest_receive", summary=%self.context(), location=%RelativeLocation::from(caller));
        async move {
            Self::with_policy(
                executor,
                self.authentication,
                self.complete_on_cancellation,
                move |child| async move {
                    self.validate()?;
                    child.validate_authentication_target(&self.authentication, &self.url)?;
                    let selection = self.authentication;
                    child
                        .uncached_work_with_artifacts(
                            ExecutorKind::Rest,
                            self.artifact_key(caller)?,
                            CacheOperationOwner::of::<Self>(),
                            &selection,
                            self.fingerprint()?,
                            self.artifact_retention,
                            |attempt, entry| async move {
                                Ok(self
                                    .canonical(&attempt, entry, false, decoder, caller)
                                    .await?
                                    .value)
                            },
                        )?
                        .await
                },
            )
            .await
        }
        .instrument(span)
    }
    fn receive_shared_from<'a, T: Send + Sync + 'static>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
        decoder: fn(&SerializableRestResponse) -> Result<T>,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a {
        let span = info_span!("rest_receive_cached", summary=%self.context(), location=%RelativeLocation::from(caller));
        async move {
            Self::with_policy(
                executor,
                self.authentication,
                self.complete_on_cancellation,
                move |child| async move {
                    self.validate()?;
                    child.validate_authentication_target(&self.authentication, &self.url)?;
                    ensure!(
                        !self.complete_on_cancellation,
                        "mutating REST requests must use owned receives and cannot share execution"
                    );
                    let key = self.cache_key.clone().ok_or_else(|| {
                        eyre::eyre!("cached REST receive requires an explicit cache key")
                    })?;
                    let selection = self.authentication;
                    child
                        .cached_work_with_artifacts(
                            ExecutorKind::Rest,
                            key,
                            owner,
                            &selection,
                            self.fingerprint()?,
                            WorkReuse::ReusableRead,
                            self.artifact_retention,
                            |attempt, entry| async move {
                                self.canonical(&attempt, entry, true, decoder, caller).await
                            },
                        )?
                        .await
                },
            )
            .await
        }
        .instrument(span)
    }
    /// Execute independently and return an owned decoded body, even when a key is supplied.
    #[track_caller]
    pub fn receive<'a, T: Facet<'static> + Send + 'static>(
        self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        let result = self.receive_owned_from(executor, Self::decode_owned::<T>, Location::caller());
        async move { Ok(result.await?.value) }
    }
    /// Return an independently observed response, including unsuccessful HTTP statuses.
    #[track_caller]
    pub fn receive_raw<'a>(
        self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<SerializableRestResponse>> + Send + 'a {
        self.receive_owned_from(
            executor,
            |response| Ok(response.clone()),
            Location::caller(),
        )
    }
    /// Share one accepted JSON body. Warm hits perform no token lookup or decoding.
    #[track_caller]
    pub fn receive_cached<'a, T: Facet<'static> + Send + Sync + 'static>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a {
        let result = self.receive_shared_from(
            executor,
            owner,
            Self::decode_shared::<T>,
            Location::caller(),
        );
        async move { Ok(Arc::clone(&result.await?.value)) }
    }
    /// Decode and convert one independently executed response inside artifact handling.
    /// Use this when the wire format needs a canonical domain conversion.
    #[track_caller]
    pub fn receive_converted<'a, Wire, T>(
        self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + Send + 'a
    where
        Wire: Facet<'static> + 'static,
        T: TryFrom<Wire> + Send + 'static,
        T::Error: Into<Report>,
    {
        let result = self.receive_owned_from(
            executor,
            |response| {
                let decoded = Self::decode_owned::<Wire>(response)?;
                Ok(DecodedRestResponse {
                    value: T::try_from(decoded.value).map_err(Into::into)?,
                    response: decoded.response,
                })
            },
            Location::caller(),
        );
        async move { Ok(result.await?.value) }
    }
    /// Share the converted domain value, without repeating conversion on memory hits.
    ///
    /// The operation owner identifies this canonical conversion. Unlike the
    /// mapper/validator APIs, conversion runs only on decode, so it must not
    /// contain caller-specific policy checks. Conversion failures retain response
    /// artifacts just like JSON decoding failures.
    /// Both wire and domain types participate in the in-memory cache contract.
    #[track_caller]
    pub fn receive_converted_cached<'a, Wire, T>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a
    where
        Wire: Facet<'static> + 'static,
        T: TryFrom<Wire> + Send + Sync + 'static,
        T::Error: Into<Report>,
    {
        let result = self.receive_shared_from(
            executor,
            owner,
            |response| {
                let decoded = Self::decode_owned::<Wire>(response)?;
                Ok((
                    PhantomData::<fn() -> Wire>,
                    DecodedRestResponse {
                        value: Arc::new(T::try_from(decoded.value).map_err(Into::into)?),
                        response: decoded.response,
                    },
                ))
            },
            Location::caller(),
        );
        async move { Ok(Arc::clone(&result.await?.1.value)) }
    }
    /// Share successful raw responses. HTTP failures retain artifacts and return RestHttpError.
    #[track_caller]
    pub fn receive_raw_cached<'a>(
        self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<SerializableRestResponse>>> + Send + 'a {
        self.receive_shared_from(
            executor,
            owner,
            |response| {
                response.error_for_status()?;
                Ok(response.clone())
            },
            Location::caller(),
        )
    }
    async fn policy_failure<T: 'static>(
        &self,
        executor: &CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
        caller: &'static Location<'static>,
        response: &SerializableRestResponse,
        error: Report,
    ) -> Report {
        if error.is_cancellation() {
            return error;
        }
        if self.artifact_retention == ArtifactRetention::Disabled {
            return eyre::eyre!("sensitive REST caller policy rejected its result");
        }
        let retained = async {
            let mut key = self.artifact_key(caller)?;
            key.valid_for = Duration::ZERO;
            let entry = executor.cache()?.artifacts::<T>(
                ExecutorKind::Rest,
                key,
                owner,
                executor.admit(&self.authentication)?,
                self.fingerprint()?,
                self.artifact_retention,
            )?;
            if let Some(attempt) = entry
                .begin_attempt(self.inputs()?, executor.cancellation())
                .await?
            {
                attempt
                    .finish(
                        ArtifactSet {
                            payload: facet_json::to_vec(response)
                                .map_err(|error| eyre::eyre!("{error:?}"))?,
                            diagnostics: self.diagnostics(Some(response), Some(&error))?,
                        },
                        ArtifactObservation::now(),
                        executor.cancellation(),
                    )
                    .await?;
            }
            eyre::Ok(())
        }
        .await;
        match retained {
            Ok(()) => error.wrap_err(format!(
                "REST caller policy failed; called from {}",
                RelativeLocation::from(caller)
            )),
            Err(retention) => error.wrap_err(format!(
                "REST caller policy failed; retaining diagnostics failed: {retention:#}"
            )),
        }
    }
    /// Apply caller-specific decoding to an independently observed raw response.
    #[track_caller]
    pub fn receive_raw_with_decoder<'a, T, F>(
        self,
        decode: F,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: 'a,
        F: FnOnce(SerializableRestResponse) -> Result<T> + 'a,
    {
        let caller = Location::caller();
        Self::with_policy(
            executor,
            self.authentication,
            self.complete_on_cancellation,
            move |child| async move {
                let response = self
                    .clone()
                    .receive_owned_from(&child, |response| Ok(response.clone()), caller)
                    .await?;
                match decode(response.clone()) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(self
                        .policy_failure::<SerializableRestResponse>(
                            &child,
                            CacheOperationOwner::of::<Self>(),
                            caller,
                            &response,
                            error,
                        )
                        .await),
                }
            },
        )
        .instrument(info_span!("rest_response_policy",location=%RelativeLocation::from(caller)))
    }
    #[track_caller]
    pub fn receive_with_mapper<'a, T, H, F>(
        self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<(T, H)>> + 'a
    where
        T: Facet<'static> + Send + 'static,
        H: 'a,
        F: FnOnce(T, &SerializableRestResponse) -> Result<(T, H)> + 'a,
    {
        self.receive_with_mapper_from(mapper, executor, Location::caller())
    }
    fn receive_with_mapper_from<'a, T, H, F>(
        self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<(T, H)>> + 'a
    where
        T: Facet<'static> + Send + 'static,
        H: 'a,
        F: FnOnce(T, &SerializableRestResponse) -> Result<(T, H)> + 'a,
    {
        Self::with_policy(
            executor,
            self.authentication,
            self.complete_on_cancellation,
            move |child| async move {
                let decoded = self
                    .clone()
                    .receive_owned_from(&child, Self::decode_owned::<T>, caller)
                    .await?;
                match mapper(decoded.value, &decoded.response) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(self
                        .policy_failure::<DecodedRestResponse<T>>(
                            &child,
                            CacheOperationOwner::of::<Self>(),
                            caller,
                            &decoded.response,
                            error,
                        )
                        .await),
                }
            },
        )
        .instrument(info_span!("rest_response_policy",location=%RelativeLocation::from(caller)))
    }
    #[track_caller]
    pub fn receive_with_validator<'a, T, F>(
        self,
        validator: F,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: Facet<'static> + Send + 'static,
        F: FnOnce(T) -> Result<T> + 'a,
    {
        let result = self.receive_with_mapper_from(
            move |value, _| validator(value).map(|value| (value, ())),
            executor,
            Location::caller(),
        );
        async move { Ok(result.await?.0) }
    }
    #[track_caller]
    pub fn receive_with_mapper_cached<'a, T, H, F>(
        self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<(Arc<T>, H)>> + 'a
    where
        T: Facet<'static> + Send + Sync + 'static,
        H: 'a,
        F: FnOnce(Arc<T>, &SerializableRestResponse) -> Result<(Arc<T>, H)> + 'a,
    {
        self.receive_with_mapper_cached_from(mapper, executor, owner, Location::caller())
    }
    fn receive_with_mapper_cached_from<'a, T, H, F>(
        self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<(Arc<T>, H)>> + 'a
    where
        T: Facet<'static> + Send + Sync + 'static,
        H: 'a,
        F: FnOnce(Arc<T>, &SerializableRestResponse) -> Result<(Arc<T>, H)> + 'a,
    {
        Self::with_policy(
            executor,
            self.authentication,
            self.complete_on_cancellation,
            move |child| async move {
                let decoded = self
                    .clone()
                    .receive_shared_from(&child, owner, Self::decode_shared::<T>, caller)
                    .await?;
                match mapper(Arc::clone(&decoded.value), &decoded.response) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(self
                        .policy_failure::<DecodedRestResponse<Arc<T>>>(
                            &child,
                            owner,
                            caller,
                            &decoded.response,
                            error,
                        )
                        .await),
                }
            },
        )
        .instrument(info_span!("rest_response_policy",location=%RelativeLocation::from(caller)))
    }
    #[track_caller]
    pub fn receive_with_validator_cached<'a, T, F>(
        self,
        validator: F,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + 'a
    where
        T: Facet<'static> + Send + Sync + 'static,
        F: FnOnce(&T) -> Result<()> + 'a,
    {
        let result = self.receive_with_mapper_cached_from(
            move |value: Arc<T>, _| {
                validator(&value)?;
                Ok((value, ()))
            },
            executor,
            owner,
            Location::caller(),
        );
        async move { Ok(result.await?.0) }
    }
    #[track_caller]
    pub fn receive_with_ms_continuation_token<'a, T: Facet<'static> + Send + 'static>(
        self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<(T, Option<MicrosoftContinuationToken>)>> + 'a {
        self.receive_with_mapper_from(
            |value, response| Ok((value, Self::continuation(response)?)),
            executor,
            Location::caller(),
        )
    }
    #[track_caller]
    pub fn receive_with_ms_continuation_token_cached<
        'a,
        T: Facet<'static> + Send + Sync + 'static,
    >(
        self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<(Arc<T>, Option<MicrosoftContinuationToken>)>> + 'a {
        self.receive_with_mapper_cached_from(
            |value, response| Ok((value, Self::continuation(response)?)),
            executor,
            owner,
            Location::caller(),
        )
    }
    fn continuation(
        response: &SerializableRestResponse,
    ) -> Result<Option<MicrosoftContinuationToken>> {
        response
            .header("x-ms-continuationtoken")
            .or_else(|| response.header("x-ms-continuation-token"))
            .map(MicrosoftContinuationToken::try_new)
            .transpose()
    }
}

impl HasCacheKey for RestRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        self.cache_key
            .clone()
            .ok_or_else(|| eyre::eyre!("REST invalidation requires an explicit cache key"))
    }
}
impl InvalidatesOwnCacheKey for RestRequest {}
impl ExecutableRequest for RestRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.validate_authentication_target(&self.authentication, &self.url)?;
        executor.with_authentication_lease(&self.authentication)
    }
    type Error = eyre::Report;
    type Output = SerializableRestResponse;
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move { self.receive_raw(&executor).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use http::HeaderValue;
    use http::StatusCode;

    #[test]
    fn explicit_service_accepts_server_hosts_without_inferencing_credentials() -> Result<()> {
        let url = Url::parse("https://server.example.invalid/tfs/Collection/_apis/projects")?;
        assert!(RestRequest::from_method_and_url(Method::GET, url.clone()).is_err());
        let request = RestRequest::from_method_and_url_for_service(
            Method::GET,
            url,
            RestService::AzureDevOps,
        )?;
        assert_eq!(request.service, RestService::AzureDevOps);
        assert!(
            request.validate().is_err(),
            "service selection cannot imply authentication"
        );
        for url in [
            "ftp://server.example.invalid/file",
            "https://user:secret@server.example.invalid/path",
        ] {
            assert!(
                RestRequest::from_method_and_url_for_service(
                    Method::GET,
                    Url::parse(url)?,
                    RestService::AzureDevOps
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn artifacts_retain_headers_and_distinguish_json_and_text_outcomes() -> Result<()> {
        let request = RestRequest::new(Method::GET, "https://prices.azure.com/api/retail/prices")?;
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response = SerializableRestResponse::new(
            StatusCode::OK,
            &headers,
            facet_json::to_string(&vec![1u64, 2])?,
        );
        let files = request.diagnostics(Some(&response), None)?;
        assert!(files.contains_key(&ArtifactFilePath::new("response.body.json")?));
        assert!(files.contains_key(&ArtifactFilePath::new("response.headers.json")?));
        let response = SerializableRestResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            &HeaderMap::new(),
            "synthetic text".to_owned(),
        );
        let files = request.diagnostics(Some(&response), None)?;
        assert_eq!(
            files
                .get(&ArtifactFilePath::new("response.body.txt")?)
                .map(Vec::as_slice),
            Some(b"synthetic text".as_slice())
        );
        assert!(!files.contains_key(&ArtifactFilePath::new("response.body.json")?));
        Ok(())
    }
}
