use crate::RequestHeaders;
use crate::RestService;
use crate::SerializableRestResponse;
use crate::execute_rest_request;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableWorkRequest;
use cloud_terrastodon_command::CachedWorkSpec;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::run_cached_work;
use eyre::Context;
use eyre::ContextCompat;
use eyre::Result;
use eyre::bail;
use http::Method;
use reqwest::Url;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;

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
            && let Ok(json) = serde_json::to_string_pretty(headers)
        {
            inputs.insert(PathBuf::from("headers.json"), json.into());
        }
        inputs
    }

    pub async fn execute_without_cache(self) -> Result<SerializableRestResponse> {
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

    pub async fn receive_raw_with_decoder<T, Decode>(self, decode: Decode) -> Result<T>
    where
        T: Send + 'static,
        Decode: FnOnce(SerializableRestResponse) -> Result<T> + Send + 'static,
    {
        let Some(cache_key) = self.cache_key.clone() else {
            return decode(self.execute_without_cache().await?);
        };

        let context = self.context();
        let debug_inputs = self.debug_inputs();
        run_cached_work(CachedWorkSpec {
            cache_key,
            context,
            debug_inputs,
            executor_kind: "rest".to_string(),
            output_type: std::any::type_name::<T>().to_string(),
            execute_raw: move || Box::pin(self.execute_without_cache()),
            decode,
        })
        .await
    }

    pub async fn receive_raw(self) -> Result<SerializableRestResponse> {
        self.receive_raw_with_decoder(Ok).await
    }

    pub async fn receive<T>(self) -> Result<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        self.receive_with_validator(Ok).await
    }

    pub async fn receive_with_validator<T, F>(self, validator: F) -> Result<T>
    where
        T: DeserializeOwned + Send + 'static,
        F: FnOnce(T) -> Result<T> + Send + 'static,
    {
        self.receive_raw_with_decoder(|response| {
            if !response.ok {
                bail!(
                    "REST call failed with status {}: {}",
                    response.status,
                    response.reason_phrase.as_deref().unwrap_or("Unknown error")
                );
            }
            let parsed =
                serde_json::from_value(response.into_json_body()?).wrap_err_with(|| {
                    format!(
                        "Deserializing REST response into {}",
                        std::any::type_name::<T>()
                    )
                })?;
            validator(parsed)
        })
        .await
    }
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

cloud_terrastodon_command::impl_cacheable_work_into_future!(RestRequest);
