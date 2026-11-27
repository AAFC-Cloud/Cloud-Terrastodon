use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrder;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderPage;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestHttpError;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use eyre::Report;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use http::Method;
use http::StatusCode;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

/// Lists work orders visible to the selected account through the
/// [AR System form entries API](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/AR-System/ars221/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/Endpoints-in-AR-REST-API/).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct BmcHelixWorkOrderListRequest {
    pub server: BmcHelixServerUrl,
    pub account_id: CloudTerrastodonAccountId,
    /// Optional AR qualification for the `q` parameter.
    pub filter: Option<String>,
    /// Maximum number of work orders to return across all response pages.
    pub limit: NonZeroUsize,
}

impl BmcHelixWorkOrderListRequest {
    pub const DEFAULT_LIMIT: NonZeroUsize = NonZeroUsize::MIN.saturating_add(99);

    /// Look up a visible work order by its [unique work order ID](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Service-Request-Management/srm2002/Integrating/Web-services-for-Work-Order-Management/GetListOfWorkOrders/).
    /// The returned list is empty when the account cannot see that ID.
    pub fn for_work_order_id(
        server: BmcHelixServerUrl,
        account_id: CloudTerrastodonAccountId,
        id: &BmcHelixWorkOrderId,
    ) -> Self {
        Self {
            server,
            account_id,
            filter: Some(work_order_id_filter(id)),
            limit: NonZeroUsize::MIN,
        }
    }
}

pub(crate) fn work_order_id_filter(id: &BmcHelixWorkOrderId) -> String {
    format!("'Work Order ID' = \"{id}\"")
}

impl<'a> Arbitrary<'a> for BmcHelixWorkOrderListRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            server: BmcHelixServerUrl::arbitrary(input)?,
            account_id: CloudTerrastodonAccountId::arbitrary(input)?,
            // No AR qualification is valid against every server's form.
            filter: None,
            limit: NonZeroUsize::MIN.saturating_add(input.int_in_range(0..=99)?),
        })
    }
}

impl HasCacheKey for BmcHelixWorkOrderListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "bmc-helix".to_owned(),
                self.server.as_url().scheme().to_owned(),
                sanitize_windows_path_component(self.server.as_url().authority()),
                "account".to_owned(),
                self.account_id.to_string(),
                "work-order".to_owned(),
                "list".to_owned(),
            ])?,
            valid_for: Duration::from_mins(5),
        })
    }
}

impl InvalidatesOwnCacheKey for BmcHelixWorkOrderListRequest {}

impl ExecutableRequest for BmcHelixWorkOrderListRequest {
    type Error = Report;
    type Output = Arc<Vec<BmcHelixWorkOrder>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        let selection = AuthenticationSelection::BmcHelix {
            account_id: self.account_id,
            server_id: self.server.id(),
        };
        executor.validate_authentication_target(&selection, self.server.as_url())?;
        executor.with_authentication_lease(&selection)
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let limit = self.limit.get();
            // AR System accepts signed 32-bit limits and offsets.
            i32::try_from(limit)
                .wrap_err("Helix work order limit exceeds the AR System maximum")?;
            let cache_key = self.cache_key()?;
            let mut work_orders = Vec::new();
            while work_orders.len() < limit {
                executor.cancellation().bail_if_cancelled()?;
                let offset = work_orders.len();
                let page_limit = (limit - offset).min(100);
                // REST's URL fingerprint distinguishes filter and limit variants
                // under this shared, invalidatable list scope.
                let page = work_order_page_request(
                    &self.server,
                    self.account_id,
                    self.filter.as_deref(),
                    offset,
                    page_limit,
                )?
                .cache(cache_key.join(format!("page-{offset}"))?)
                .receive_cached::<BmcHelixWorkOrderPage>(
                    &executor,
                    CacheOperationOwner::of::<Self>(),
                )
                .await
                .map_err(|error| with_sign_in_hint(error, self.account_id, &self.server))?;
                if page.entries.is_empty() {
                    break;
                }
                work_orders.extend(
                    page.entries
                        .iter()
                        .take(page_limit)
                        .map(|entry| entry.values.clone()),
                );
                // A server-side cap may produce a short page with more results available.
            }
            Ok(Arc::new(work_orders))
        })
    }
}

pub(crate) fn work_order_page_request(
    server: &BmcHelixServerUrl,
    account_id: CloudTerrastodonAccountId,
    filter: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<RestRequest> {
    let mut url = server
        .as_url()
        .join("api/arsys/v1/entry/WOI:WorkOrderInterface")?;
    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("offset", &offset.to_string())
            .append_pair("limit", &limit.to_string())
            .append_pair("sort", "Request ID.asc");
        if let Some(filter) = filter.filter(|value| !value.trim().is_empty()) {
            query.append_pair("q", filter);
        }
    }
    Ok(
        RestRequest::from_method_and_url_for_service(Method::GET, url, RestService::BmcHelix)?
            .authentication(AuthenticationSelection::BmcHelix {
                account_id,
                server_id: server.id(),
            }),
    )
}

pub(crate) fn with_sign_in_hint(
    error: Report,
    account_id: CloudTerrastodonAccountId,
    server: &BmcHelixServerUrl,
) -> Report {
    if error
        .downcast_ref::<RestHttpError>()
        .is_some_and(|error| error.status == StatusCode::UNAUTHORIZED)
    {
        error.wrap_err(format!(
            "Helix rejected the saved session for {server}. Run `ct account logout {account_id} --server <name>`, then `ct account login {account_id} --server <name>`"
        ))
    } else {
        error
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixWorkOrderListRequest);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixWorkOrderListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    BmcHelixWorkOrderListRequest => Arc<Vec<BmcHelixWorkOrder>>,
    effects = [Read]
);
