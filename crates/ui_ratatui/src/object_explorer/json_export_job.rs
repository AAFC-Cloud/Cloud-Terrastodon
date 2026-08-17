use super::arena_address_source::ArenaAddressSource;
use super::arena_query_command::CommandResponse;
use super::arena_query_session::JsonBatch;
use super::json_encoder::JsonEncoder;
use super::query_cursor::QueryCursor;
use super::query_plan::QueryPlan;
use super::query_progress::QueryProgressState;
use super::revision::QueryRevision;
use super::revision::ScanRevisionStamp;
use super::work_budget::WorkBudget;
use std::num::NonZeroUsize;

/// Engine-side state for one coherent, bounded JSON export session.
///
/// At most one serialized value is retained between batches. Since both the
/// outgoing fragment and lookahead are individually bounded by `max_bytes`,
/// peak serialized session data is at most `2 * max_bytes` while a response is
/// in flight. No RuntimeValue, Peek, or query result collection is retained.
pub(crate) struct JsonExportJob<'source, 'arena> {
    source: &'source ArenaAddressSource<'arena>,
    cursor: QueryCursor<'source, 'arena>,
    stamp: ScanRevisionStamp,
    emitted_any: bool,
    pending_json: Option<String>,
    peak_retained_serialized_bytes: usize,
}

impl<'source, 'arena> JsonExportJob<'source, 'arena>
where
    'arena: 'source,
{
    pub(crate) fn new(
        source: &'source ArenaAddressSource<'arena>,
        query_plan: QueryPlan,
        stamp: ScanRevisionStamp,
    ) -> Self {
        Self {
            source,
            cursor: QueryCursor::new(
                source,
                query_plan,
                stamp,
                NonZeroUsize::new(2).expect("export cache capacity is nonzero"),
            ),
            stamp,
            emitted_any: false,
            pending_json: None,
            peak_retained_serialized_bytes: 0,
        }
    }

    pub(crate) fn for_arena(
        source: &'source ArenaAddressSource<'arena>,
        query_plan: QueryPlan,
        arena_revision: super::revision::ArenaRevision,
    ) -> Self {
        Self::new(
            source,
            query_plan,
            ScanRevisionStamp {
                arena: arena_revision,
                query: QueryRevision::default(),
            },
        )
    }

    pub(crate) fn next_batch(
        &mut self,
        json_encoder: &mut JsonEncoder,
        max_work: usize,
        max_bytes: usize,
    ) -> CommandResponse<JsonBatch> {
        if max_work == 0 {
            return Err("JSON batch work budget must be greater than zero".to_owned());
        }
        if max_bytes == 0 {
            return Err("JSON batch byte budget must be greater than zero".to_owned());
        }

        let addressed_before = self.cursor.instrumentation().addressed;
        let mut work = WorkBudget::new(max_work);
        let mut fragment = String::new();
        let mut emitted = 0;
        let mut complete = false;

        if let Some(json) = self.pending_json.take() {
            self.append_json(&mut fragment, json, max_bytes)?;
            emitted += 1;
        }

        while fragment.len() < max_bytes {
            let progress = self.cursor.next(self.stamp, &mut work);
            match progress.into_state() {
                QueryProgressState::Ready(address) => {
                    let value = self.source.resolve(&address).map_err(|error| {
                        format!("query address {address:?} stopped resolving: {error}")
                    })?;
                    let json = json_encoder
                        .encode_pretty(value.peek())
                        .map_err(|error| format!("could not serialize {address:?}: {error}"))?;
                    let json = indent_array_item(&json);
                    self.cursor.record_serialized(1);
                    let separator_bytes = self.separator().len();
                    if separator_bytes + json.len() > max_bytes {
                        return Err(format!(
                            "serialized value at {address:?} needs {} bytes, exceeding the {max_bytes}-byte batch limit",
                            separator_bytes + json.len()
                        ));
                    }
                    if fragment.len() + separator_bytes + json.len() > max_bytes {
                        self.pending_json = Some(json);
                        break;
                    }
                    self.append_json(&mut fragment, json, max_bytes)?;
                    emitted += 1;
                }
                QueryProgressState::Pending => break,
                QueryProgressState::Complete => {
                    complete = true;
                    break;
                }
                QueryProgressState::Stale => {
                    return Err("coherent export cursor became stale".to_owned());
                }
                QueryProgressState::Cancelled => {
                    return Err("coherent export cursor was cancelled".to_owned());
                }
            }
        }

        let retained = fragment
            .len()
            .saturating_add(self.pending_json.as_ref().map_or(0, String::len));
        self.peak_retained_serialized_bytes = self.peak_retained_serialized_bytes.max(retained);
        debug_assert!(fragment.len() <= max_bytes);
        debug_assert!(
            self.pending_json
                .as_ref()
                .is_none_or(|json| json.len() <= max_bytes)
        );
        debug_assert!(work.spent() <= max_work);
        let inspected = self
            .cursor
            .instrumentation()
            .addressed
            .saturating_sub(addressed_before);
        debug_assert!(inspected <= max_work);
        Ok(JsonBatch {
            fragment,
            inspected,
            emitted,
            complete,
        })
    }

    pub(crate) fn peak_retained_serialized_bytes(&self) -> usize {
        self.peak_retained_serialized_bytes
    }

    fn append_json(
        &mut self,
        fragment: &mut String,
        json: String,
        max_bytes: usize,
    ) -> CommandResponse<()> {
        let separator = self.separator();
        let separator_bytes = separator.len();
        if fragment.len() + separator_bytes + json.len() > max_bytes {
            return Err(format!(
                "serialized value needs {} bytes, exceeding the remaining batch capacity",
                separator_bytes + json.len()
            ));
        }
        fragment.push_str(separator);
        fragment.push_str(&json);
        self.emitted_any = true;
        Ok(())
    }

    fn separator(&self) -> &'static str {
        if self.emitted_any { ",\n" } else { "\n" }
    }
}

fn indent_array_item(json: &str) -> String {
    json.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::breadcrumb::Breadcrumb;
    use crate::object_explorer::breadcrumb::ValueFilterOperator;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn no_match_export_batch_obeys_raw_address_work_budget() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let query = QueryPlan::new(Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![cloud_terrastodon_registry::describe_shape(usize::SHAPE)],
            },
            Breadcrumb::ValueFilter {
                field_shape: "*".to_owned(),
                field_name: "missing".to_owned(),
                operator: ValueFilterOperator::Equals,
                value: "never".to_owned(),
            },
        ]));
        let mut job = JsonExportJob::for_arena(&source, query, arena.arena_revision());
        let mut encoder = JsonEncoder::default();

        let first = job.next_batch(&mut encoder, 7, 128).unwrap();
        let second = job.next_batch(&mut encoder, 11, 128).unwrap();

        assert_eq!(first.inspected, 7);
        assert_eq!(second.inspected, 11);
        assert_eq!(first.emitted + second.emitted, 0);
        assert!(!first.complete);
        assert!(!second.complete);
        assert_eq!(encoder.encoded_values(), 0);
    }

    #[test]
    fn serialized_fragment_and_lookahead_stay_within_two_batch_budgets() {
        let mut arena = Arena::default();
        for value in ["12345678", "abcdefgh", "ABCDEFGH"] {
            arena.insert_ready(runtime(String::from(value))).unwrap();
        }
        let source = ArenaAddressSource::new(&arena);
        let mut job = JsonExportJob::for_arena(
            &source,
            QueryPlan::new(Breadcrumbs::default()),
            arena.arena_revision(),
        );
        let mut encoder = JsonEncoder::default();
        let max_bytes = 32;

        let first = job.next_batch(&mut encoder, 8, max_bytes).unwrap();

        assert!(first.fragment.len() <= max_bytes);
        assert!(job.peak_retained_serialized_bytes() <= 2 * max_bytes);
        assert_eq!(first.emitted, 2);
    }
}
