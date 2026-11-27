use crate::bmc_helix_work_order_list_request::with_sign_in_hint;
use crate::bmc_helix_work_order_list_request::work_order_id_filter;
use crate::bmc_helix_work_order_list_request::work_order_page_request;
use arbitrary::Arbitrary;
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
use eyre::Report;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use std::sync::Arc;
use std::time::Duration;

/// Retrieves an individual [work order](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Service-Request-Management/srm2002/Integrating/Web-services-for-Work-Order-Management/GetListOfWorkOrders/)
/// by qualifying the form query on its visible ticket number.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
pub struct BmcHelixWorkOrderGetRequest {
    pub server: BmcHelixServerUrl,
    pub account_id: CloudTerrastodonAccountId,
    pub work_order_id: BmcHelixWorkOrderId,
}

impl HasCacheKey for BmcHelixWorkOrderGetRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "bmc-helix".to_owned(),
                self.server.as_url().scheme().to_owned(),
                sanitize_windows_path_component(self.server.as_url().authority()),
                "account".to_owned(),
                self.account_id.to_string(),
                "work-order".to_owned(),
                "get".to_owned(),
                sanitize_windows_path_component(self.work_order_id.as_str()),
            ])?,
            valid_for: Duration::from_mins(5),
        })
    }
}

impl InvalidatesOwnCacheKey for BmcHelixWorkOrderGetRequest {}

impl ExecutableRequest for BmcHelixWorkOrderGetRequest {
    type Error = Report;
    type Output = Arc<BmcHelixWorkOrder>;

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
            // The work order number is the visible ID. AR's entry-by-ID endpoint uses
            // the internal Request ID, so qualify the form query instead.
            let filter = work_order_id_filter(&self.work_order_id);
            let cache_key = self.cache_key()?;
            let mut work_order = None;
            for offset in 0..2 {
                executor.cancellation().bail_if_cancelled()?;
                let page = work_order_page_request(
                    &self.server,
                    self.account_id,
                    Some(&filter),
                    offset,
                    2 - offset,
                )?
                .cache(cache_key.join(format!("page-{offset}"))?)
                .receive_cached::<BmcHelixWorkOrderPage>(
                    &executor,
                    CacheOperationOwner::of::<Self>(),
                )
                .await
                .map_err(|error| with_sign_in_hint(error, self.account_id, &self.server))?;
                match page.entries.as_slice() {
                    [] => break,
                    [entry] if work_order.is_none() => {
                        ensure!(
                            entry.values.id == self.work_order_id,
                            "Helix returned work order {} when {} was requested",
                            entry.values.id,
                            self.work_order_id
                        );
                        work_order = Some(Arc::new(entry.values.clone()));
                    }
                    _ => {
                        bail!("Work order {} matched multiple entries", self.work_order_id);
                    }
                }
                // Even a one-entry page can be capped by the server.
            }
            let Some(work_order) = work_order else {
                bail!(
                    "Work order {} was not found or is not visible to this account",
                    self.work_order_id
                );
            };
            Ok(work_order)
        })
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixWorkOrderGetRequest);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixWorkOrderGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    BmcHelixWorkOrderGetRequest => Arc<BmcHelixWorkOrder>,
    effects = [Read]
);
