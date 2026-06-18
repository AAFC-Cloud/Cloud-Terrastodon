use std::ops::ControlFlow;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use teamy_mft::query::QueryNeedle;
use teamy_mft::query::QueryPlan;
use teamy_mft::query::QueryRule;
use teamy_mft::query::QuerySession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftwareQuerySummary {
    pub query: &'static str,
    pub result_count: usize,
}

pub const SOFTWARE_QUERIES: [&'static str; 5] = [
    ".git",
    "package.json",
    "package-lock.json",
    "Cargo.toml",
    "Cargo.lock",
];

/// # Errors
///
/// Returns an error if the underlying teamy-mft query fails.
/// Cancellation is best-effort and returns the summaries collected so far.
pub fn list_software_counts_with_cancel(
    cancel: Option<&AtomicBool>,
) -> eyre::Result<Vec<SoftwareQuerySummary>> {
    let mut session = QuerySession::published_index_only()?;
    let mut rtn = Vec::with_capacity(SOFTWARE_QUERIES.len());
    for query in SOFTWARE_QUERIES {
        let mut count = 0;
        session.visit_rows_with_cancel(
            QueryPlan::single_rule(QueryRule::EqualsCaseInsensitive(QueryNeedle::new(query))),
            cancel,
            |_row| {
                count += 1;
                Ok(ControlFlow::Continue(()))
            },
        )?;
        if cancel.is_some_and(|cancel| cancel.load(Ordering::Relaxed)) {
            break;
        }
        rtn.push(SoftwareQuerySummary {
            query,
            result_count: count,
        });
    }
    Ok(rtn)
}
