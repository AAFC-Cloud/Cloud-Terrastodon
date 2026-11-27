use crate::bmc_helix_ticket_column_fields::BmcHelixTicketColumnFields;
use crate::bmc_helix_ticket_query_body::BmcHelixTicketQueryBody;
use crate::bmc_helix_ticket_query_values::BmcHelixTicketQueryValues;
use crate::bmc_helix_work_order_list_request::with_sign_in_hint;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicket;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketDataPage;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketFieldId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use eyre::Report;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use facet_value::Value;
use http::Method;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tracing::warn;

// Field order and sorting match the observed Shared Ticket Console request.
const RECORD_DEFINITION: &str = "com.bmc.dsm.itsm-applications:ITSM Tickets CRD";
const TICKET_FIELDS: [u32; 26] = [
    488101210, 488100390, 488100470, 488100190, 488100020, 488100240, 488100098, 488100270,
    488100010, 488100080, 488100090, 379, 488100260, 488100650, 488100920, 488100150, 488101260,
    488100330, 488100970, 488100850, 488101870, 488100940, 488104000, 488104010, 488104020,
    1000003278,
];
const SORT_FIELD: BmcHelixTicketFieldId = BmcHelixTicketFieldId::new(488100020);
const OPEN_STATUSES: [&str; 27] = [
    "Draft",
    "Assigned",
    "In Progress",
    "Pending",
    "Waiting Approval",
    "Planning",
    "Work In Progress",
    "New",
    "Staged",
    "Request For Change",
    "Request For Authorization",
    "Planning In Progress",
    "Scheduled For Review",
    "Scheduled For Approval",
    "Scheduled",
    "Implementation In Progress",
    "Under Review",
    "Under Investigation",
    "Assigned To Vendor",
    "Scheduled For Correction",
    "Build Approval",
    "Close Down Approval",
    "Deployment Approval",
    "Initiation Approval",
    "Planning Approval",
    "Registered",
    "Test Approval",
];

/// Lists open, assigned tickets across the signed-in user's support groups.
///
/// Runs the [Shared Ticket Console](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm262/Administering/Enabling-and-customizing-Shared-Ticket-Console/Customizing-Shared-Ticket-Console/)
/// preset against the `ITSM Tickets CRD` using Innovation Suite's
/// [RecordInstanceDataPageQuery](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is251/Developing-applications-by-using-BMC-Helix-Innovation-Studio/Developing-and-deploying-code-based-applications/Customizing-an-application-with-REST-APIs/Retrieving-record-instances-using-REST-API/).
/// Innovation Suite expands the support-group expression for the current account.
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct BmcHelixTicketListRequest {
    /// AR System origin that issued the selected Helix account's JWT.
    pub server: BmcHelixServerUrl,
    /// Configured Innovation Suite origin that receives the ticket query.
    pub innovation_suite_url: BmcHelixServerUrl,
    pub account_id: CloudTerrastodonAccountId,
    /// Maximum number of tickets to return across all pages.
    pub limit: NonZeroUsize,
}

impl BmcHelixTicketListRequest {
    pub const DEFAULT_LIMIT: NonZeroUsize = NonZeroUsize::MIN.saturating_add(49);

    fn authentication(&self) -> AuthenticationSelection {
        AuthenticationSelection::BmcHelixInnovationSuite {
            account_id: self.account_id,
            server_id: self.server.id(),
        }
    }

    /// Retrieve the [record definition](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Helix-Innovation-Suite/is253/Release-notes-and-notices/25-3-enhancements-and-patches/) so optional columns can use the server's numeric field IDs.
    async fn column_fields(
        &self,
        executor: &CloudTerrastodonExecutor,
        cache_key: &CacheKey,
    ) -> Result<Arc<BmcHelixTicketColumnFields>> {
        let mut url = self
            .innovation_suite_url
            .as_url()
            .join("api/rx/application/record/recorddefinition")?;
        url.path_segments_mut()
            .map_err(|()| eyre::eyre!("Helix record definition URL must support path segments"))?
            .push(RECORD_DEFINITION);
        RestRequest::from_method_and_url_for_service(
            Method::GET,
            url,
            RestService::BmcHelixInnovationSuite,
        )?
        .authentication(self.authentication())
        .headers(RequestHeaders::from_header(
            "x-requested-by",
            "Cloud Terrastodon",
        )?)
        .cache(cache_key.join("record-definition")?)
        .receive_converted_cached::<Value, BmcHelixTicketColumnFields>(
            executor,
            CacheOperationOwner::of::<Self>(),
        )
        .await
    }
}

impl<'a> Arbitrary<'a> for BmcHelixTicketListRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            server: BmcHelixServerUrl::arbitrary(input)?,
            innovation_suite_url: BmcHelixServerUrl::arbitrary(input)?,
            account_id: CloudTerrastodonAccountId::arbitrary(input)?,
            limit: NonZeroUsize::MIN.saturating_add(input.int_in_range(0..=49)?),
        })
    }
}

impl HasCacheKey for BmcHelixTicketListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "bmc-helix".to_owned(),
                self.server.as_url().scheme().to_owned(),
                sanitize_windows_path_component(self.server.as_url().authority()),
                "account".to_owned(),
                self.account_id.to_string(),
                "tickets".to_owned(),
                "my-support-groups-assigned".to_owned(),
            ])?,
            valid_for: Duration::from_mins(5),
        })
    }
}

impl InvalidatesOwnCacheKey for BmcHelixTicketListRequest {}

impl ExecutableRequest for BmcHelixTicketListRequest {
    type Error = Report;
    type Output = Arc<Vec<BmcHelixTicket>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        let selection = self.authentication();
        executor.validate_authentication_target(&selection, self.innovation_suite_url.as_url())?;
        executor.with_authentication_lease(&selection)
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let limit = self.limit.get();
            i32::try_from(limit)
                .wrap_err("Helix ticket limit exceeds the DataPageQuery maximum")?;
            let cache_key = self.cache_key()?;
            let url = self
                .innovation_suite_url
                .as_url()
                .join("api/rx/application/datapage")?;
            let columns = match self.column_fields(&executor, &cache_key).await {
                Ok(columns) => columns,
                Err(error) => {
                    executor.cancellation().bail_if_cancelled()?;
                    warn!(error = %format_args!("{error:#}"), "Could not read Helix ticket column definitions");
                    Arc::new(BmcHelixTicketColumnFields::default())
                }
            };
            let mut fields = TICKET_FIELDS.map(BmcHelixTicketFieldId::new).to_vec();
            for id in columns.ids() {
                if !fields.contains(&id) {
                    fields.push(id);
                }
            }
            let property_selection = fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let query_expression = open_support_group_ticket_expression();
            let mut tickets = Vec::new();
            while tickets.len() < limit {
                executor.cancellation().bail_if_cancelled()?;
                let start_index = tickets.len();
                let page_size = (limit - start_index).min(Self::DEFAULT_LIMIT.get());
                let body = BmcHelixTicketQueryBody {
                    values: BmcHelixTicketQueryValues {
                        data_page_type: "com.bmc.arsys.rx.application.record.datapage.RecordInstanceDataPageQuery",
                        page_size: page_size.to_string(),
                        start_index: start_index.to_string(),
                        record_definition: RECORD_DEFINITION,
                        should_include_total_size: "false",
                        property_selection: &property_selection,
                        sort_by: SORT_FIELD.to_string(),
                        query_expression: &query_expression,
                    },
                };
                // The REST fingerprint retains the full page body and RX destination.
                let page = RestRequest::from_method_and_url_for_service(
                    Method::POST,
                    url.clone(),
                    RestService::BmcHelixInnovationSuite,
                )?
                .cancellation_safe()
                .authentication(self.authentication())
                .headers(RequestHeaders::from_header(
                    "x-requested-by",
                    "Cloud Terrastodon",
                )?)
                .body(facet_json::to_string(&body)?)
                .cache(cache_key.join(format!("page-{start_index}"))?)
                .receive_converted_cached::<BmcHelixTicketDataPage, Vec<BmcHelixTicket>>(
                    &executor,
                    CacheOperationOwner::of::<Self>(),
                )
                .await
                .map_err(|error| with_sign_in_hint(error, self.account_id, &self.server))?;
                if page.is_empty() {
                    break;
                }
                tickets.extend(
                    page.iter()
                        .take(page_size)
                        .cloned()
                        .map(|ticket| columns.project(ticket)),
                );
                // A server can cap pages. Keep paging until the limit or an empty page.
            }
            Ok(Arc::new(tickets))
        })
    }
}

fn open_support_group_ticket_expression() -> String {
    let status = BmcHelixTicketFieldId::STATUS;
    let support_group = BmcHelixTicketFieldId::SUPPORT_GROUP;
    let assignee = BmcHelixTicketFieldId::ASSIGNEE;
    let open_statuses = OPEN_STATUSES
        .map(|value| format!("'{status}' = \"{value}\""))
        .join(" OR ");
    format!(
        "({open_statuses}) AND ('{support_group}' IN (\"$MYSUPPORTGROUP$\")) AND ('{assignee}' != Null)"
    )
}

cloud_terrastodon_registry::register_thing!(BmcHelixTicketListRequest);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixTicketListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    BmcHelixTicketListRequest => Arc<Vec<BmcHelixTicket>>,
    effects = [Read]
);
