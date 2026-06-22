use crate::RequestHeaders;
use crate::RestResponseBody;
use crate::RestService;
use crate::SerializableRestResponse;
use crate::execute_rest_request;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableWorkRequest;
use cloud_terrastodon_command::CachedWorkSpec;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::run_cached_work;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use http::Method;
use reqwest::Url;
use std::collections::BTreeMap;
use std::future::Future;
use std::panic::Location;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use std::time::Instant;
use tracing::Instrument;
use tracing::debug;
use tracing::info_span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RestOutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct RestRequest {
    pub service: RestService,
    pub method: Method,
    pub url: Url,
    pub body: Option<String>,
    pub headers: Option<RequestHeaders>,
    pub tenant: Option<AzureTenantId>,
    pub cache_key: Option<CacheKey>,
    pub output_format: RestOutputFormat,
}

impl RestRequest {
    pub fn new(method: Method, url: impl AsRef<str>) -> Result<Self> {
        let url_string = url.as_ref().to_string();
        let url =
            Url::parse(&url_string).with_context(|| format!("parsing URL '{}'", url_string))?;
        let service = RestService::infer(&url).wrap_err_with(|| {
            format!("unsupported REST host '{}'", url.host_str().unwrap_or(""))
        })?;
        Ok(Self {
            service,
            method,
            url,
            body: None,
            headers: None,
            tenant: None,
            cache_key: None,
            output_format: RestOutputFormat::default(),
        })
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn headers(mut self, headers: RequestHeaders) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn tenant(mut self, tenant: AzureTenantId) -> Self {
        self.tenant = Some(tenant);
        self
    }

    pub fn cache(mut self, cache_key: CacheKey) -> Self {
        self.cache_key = Some(cache_key);
        self
    }

    pub fn use_cache(mut self, cache_key: Option<CacheKey>) -> Self {
        self.cache_key = cache_key;
        self
    }

    pub fn output_format(mut self, output_format: RestOutputFormat) -> Self {
        self.output_format = output_format;
        self
    }

    fn context(&self) -> String {
        format!("rest {} {}", self.method, self.url)
    }

    fn debug_inputs(&self) -> BTreeMap<PathBuf, bstr::BString> {
        let mut inputs = BTreeMap::new();
        if let Some(body) = &self.body {
            inputs.insert(PathBuf::from("body.json"), body.clone().into());
        }
        if let Some(headers) = &self.headers
            && let Ok(json) = headers.to_json_pretty()
        {
            inputs.insert(PathBuf::from("headers.json"), json.into());
        }
        inputs
    }

    #[track_caller]
    pub fn execute_without_cache(
        self,
    ) -> impl Future<Output = Result<SerializableRestResponse>> + Send {
        self.execute_without_cache_from(Location::caller())
    }

    fn execute_without_cache_from(
        self,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<SerializableRestResponse>> + Send {
        let context = self.context();
        let location = RelativeLocation::from(caller).to_string();
        async move {
            self.execute_without_cache_inner()
                .await
                .wrap_err(format!(
                    "RestRequest::execute_without_cache failed, called from {location}"
                ))
                .wrap_err(format!("Invoking REST request failed: {context}"))
        }
    }

    async fn execute_without_cache_inner(self) -> Result<SerializableRestResponse> {
        let start = Instant::now();
        let response = execute_rest_request(
            self.service,
            self.method,
            self.url,
            self.body,
            self.headers,
            self.tenant,
        )
        .await?;
        let serialized = SerializableRestResponse::from_response(response).await?;
        let elapsed = start.elapsed();
        debug!(
            elapsed_ms = elapsed.as_millis(),
            "REST request completed in {}",
            humantime::format_duration(elapsed)
        );
        Ok(serialized)
    }

    #[track_caller]
    pub fn receive_raw_with_decoder<T, Decode>(
        self,
        decode: Decode,
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: Send + 'static,
        Decode: FnOnce(SerializableRestResponse) -> Result<T> + Send + 'static,
    {
        self.receive_raw_with_decoder_from(decode, Location::caller())
    }

    fn receive_raw_with_decoder_from<T, Decode>(
        self,
        decode: Decode,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: Send + 'static,
        Decode: FnOnce(SerializableRestResponse) -> Result<T> + Send + 'static,
    {
        let context = self.context();
        let location = RelativeLocation::from(caller).to_string();
        let span = info_span!(
            "rest_receive",
            summary = %context,
            ?self.cache_key,
            location = %location,
        )
        .or_current();

        async move {
            let Some(cache_key) = self.cache_key.clone() else {
                return decode(self.execute_without_cache_from(caller).await?);
            };

            let debug_inputs = self.debug_inputs();
            run_cached_work(CachedWorkSpec {
                cache_key,
                context: context.clone(),
                debug_inputs,
                extra_files: Some(rest_response_extra_files),
                executor_kind: "rest".to_string(),
                output_type: std::any::type_name::<T>().to_string(),
                execute_raw: move || Box::pin(self.execute_without_cache_from(caller)),
                decode,
            })
            .await
            .wrap_err(format!(
                "RestRequest::receive_raw_with_decoder failed, called from {location}"
            ))
            .wrap_err(format!("Invoking REST request failed: {context}"))
        }
        .instrument(span)
    }

    #[track_caller]
    pub fn receive_raw(self) -> impl Future<Output = Result<SerializableRestResponse>> + Send {
        self.receive_raw_from(Location::caller())
    }

    fn receive_raw_from(
        self,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<SerializableRestResponse>> + Send {
        self.receive_raw_with_decoder_from(Ok, caller)
    }

    #[track_caller]
    pub fn receive<T>(self) -> impl Future<Output = Result<T>> + Send
    where
        T: Facet<'static> + Send + 'static,
    {
        self.receive_with_validator_from(Ok, Location::caller())
    }

    #[track_caller]
    pub fn receive_with_validator<T, F>(
        self,
        validator: F,
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: Facet<'static> + Send + 'static,
        F: FnOnce(T) -> Result<T> + Send + 'static,
    {
        self.receive_with_validator_from(validator, Location::caller())
    }

    fn receive_with_validator_from<T, F>(
        self,
        validator: F,
        caller: &'static Location<'static>,
    ) -> impl Future<Output = Result<T>> + Send
    where
        T: Facet<'static> + Send + 'static,
        F: FnOnce(T) -> Result<T> + Send + 'static,
    {
        self.receive_raw_with_decoder_from(
            |response| {
                if !response.ok {
                    bail!(
                        "REST call failed with status {}: {}",
                        response.status,
                        response.reason_phrase.as_deref().unwrap_or("Unknown error")
                    );
                }
                let parsed = facet_json::from_str::<T>(response.into_json_body()?.as_str())
                    .map_err(|error| eyre::eyre!("{error:?}"))
                    .wrap_err_with(|| {
                        format!(
                            "Deserializing REST response into {}",
                            std::any::type_name::<T>()
                        )
                    })?;
                validator(parsed)
            },
            caller,
        )
    }
}

fn rest_response_extra_files(
    response: &SerializableRestResponse,
) -> BTreeMap<PathBuf, bstr::BString> {
    let mut files = BTreeMap::new();
    if let RestResponseBody::Json(body) = &response.body {
        files.insert(PathBuf::from("response.body.json"), body.as_str().into());
    }
    files
}

#[async_trait]
impl CacheableWorkRequest for RestRequest {
    type Raw = SerializableRestResponse;
    type Output = SerializableRestResponse;

    fn cache_key(&self) -> CacheKey {
        self.cache_key.clone().unwrap_or(CacheKey {
            path: PathBuf::from_iter(["rest", "uncached"]),
            valid_for: Duration::ZERO,
        })
    }

    fn context(&self) -> String {
        RestRequest::context(self)
    }

    fn debug_inputs(&self) -> BTreeMap<PathBuf, bstr::BString> {
        RestRequest::debug_inputs(self)
    }

    async fn execute_raw(self) -> Result<Self::Raw> {
        self.execute_without_cache().await
    }

    fn decode(raw: Self::Raw) -> Result<Self::Output> {
        Ok(raw)
    }
}

impl std::future::IntoFuture for RestRequest {
    type Output = Result<SerializableRestResponse>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    #[track_caller]
    fn into_future(self) -> Self::IntoFuture {
        let caller = Location::caller();
        Box::pin(async move { self.receive_raw_from(caller).await })
    }
}

impl cloud_terrastodon_command::CacheInvalidatableIntoFuture for RestRequest {
    type WithInvalidation =
        Pin<Box<dyn Future<Output = <Self as std::future::IntoFuture>::Output> + Send>>;

    #[track_caller]
    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        let caller = Location::caller();
        Box::pin(async move {
            if invalidate_cache {
                <RestRequest as CacheableWorkRequest>::cache_key(&self)
                    .invalidate()
                    .await?;
            }
            self.receive_raw_from(caller).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;

    #[test]
    fn rest_response_extra_files_include_json_body() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let response = SerializableRestResponse::new(
            http::StatusCode::OK,
            &headers,
            "{\"hello\":\"world\"}".to_string(),
        );

        let files = rest_response_extra_files(&response);

        assert_eq!(
            files
                .get(&PathBuf::from("response.body.json"))
                .map(|value| value.as_ref()),
            Some(&b"{\"hello\":\"world\"}"[..])
        );
    }
}
