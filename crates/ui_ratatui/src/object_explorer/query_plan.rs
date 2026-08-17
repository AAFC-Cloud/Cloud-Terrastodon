use super::arena_address_source::ArenaAddressSource;
use super::breadcrumb::Breadcrumb;
use super::breadcrumb::ProjectFieldsMode;
use super::breadcrumb::ValueFilterOperator;
use super::breadcrumbs::Breadcrumbs;
use super::pop_coalescer::AdjacentPop;
use super::preorder_cursor::PreorderCursor;
use super::projected_field::ProjectedField;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use super::work_budget::WorkBudget;
use facet_reflect::HasFields;
use facet_reflect::Peek;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
struct SelectionStage {
    projections: Vec<ValueAddress>,
    shape_filters: Vec<BTreeSet<String>>,
    address_kind: Option<(bool, bool)>,
    value_filters: Vec<ValueFilter>,
    project_fields: Option<ProjectFieldsSelection>,
}

#[derive(Clone, Debug)]
struct ProjectFieldsSelection {
    mode: ProjectFieldsMode,
    included_fields: BTreeSet<ProjectedField>,
}

#[derive(Clone, Debug, Default)]
enum QueryStart {
    #[default]
    Everything,
    Projection(ValueAddress),
    Empty,
}

impl SelectionStage {
    fn add(&mut self, operation: Breadcrumb) {
        match operation {
            Breadcrumb::Projection { root_slot_id, path } => {
                let mut address = ValueAddress::root(SlotId::new(root_slot_id));
                for segment in path {
                    address = address.child(segment);
                }
                self.projections.push(address);
            }
            Breadcrumb::ShapeFilter { included_shapes } => {
                self.shape_filters
                    .push(included_shapes.into_iter().collect());
            }
            Breadcrumb::AddressKindFilter {
                include_roots,
                include_descendants,
            } => {
                self.address_kind = Some((include_roots, include_descendants));
            }
            Breadcrumb::ValueFilter {
                field_shape,
                field_name,
                operator,
                value,
            } => self.value_filters.push(ValueFilter {
                field_shape,
                field_name,
                operator,
                value,
            }),
            Breadcrumb::ProjectFields {
                mode,
                included_fields,
            } => {
                self.project_fields = Some(ProjectFieldsSelection {
                    mode,
                    included_fields: included_fields.into_iter().collect(),
                })
            }
            Breadcrumb::Pop => unreachable!("Pop terminates a selection stage"),
        }
    }

    fn is_empty(&self) -> bool {
        self.projections.is_empty()
            && self.shape_filters.is_empty()
            && self.address_kind.is_none()
            && self.value_filters.is_empty()
            && self.project_fields.is_none()
    }

    fn base_matches(
        &self,
        source: &ArenaAddressSource<'_>,
        address: &ValueAddress,
        stats: &QueryPlanStatsCell,
    ) -> bool {
        if !self
            .projections
            .iter()
            .all(|projection| is_at_or_below(address, projection))
        {
            return false;
        }
        if let Some((include_roots, include_descendants)) = self.address_kind {
            let is_root = address.path().segments().is_empty();
            if (is_root && !include_roots) || (!is_root && !include_descendants) {
                return false;
            }
        }
        stats.record_reflection();
        let Ok(value) = source.resolve(address) else {
            return false;
        };
        let shape = cloud_terrastodon_registry::describe_shape(value.shape());
        if !self
            .shape_filters
            .iter()
            .all(|included| included.contains(&shape))
        {
            return false;
        }
        self.value_filters
            .iter()
            .all(|filter| value_filter_matches_address_or_ancestor(source, address, filter, stats))
    }

    fn matches(
        &self,
        source: &ArenaAddressSource<'_>,
        address: &ValueAddress,
        stats: &QueryPlanStatsCell,
    ) -> bool {
        let base_match = self.base_matches(source, address, stats);
        let Some(project_fields) = &self.project_fields else {
            return base_match;
        };
        let parent_match = match address.path().segments().last() {
            Some(ValuePathSegment::Field(field_name)) => address.parent().is_some_and(|parent| {
                if !self.base_matches(source, &parent, stats) {
                    return false;
                }
                stats.record_reflection();
                source.resolve(&parent).is_ok_and(|parent_value| {
                    project_fields
                        .included_fields
                        .contains(&ProjectedField::new(
                            cloud_terrastodon_registry::describe_shape(parent_value.shape()),
                            field_name,
                        ))
                })
            }),
            _ => false,
        };
        match project_fields.mode {
            ProjectFieldsMode::Extend => base_match || parent_match,
            ProjectFieldsMode::Map => {
                if parent_match {
                    true
                } else if base_match {
                    stats.record_reflection();
                    source
                        .resolve(address)
                        .is_ok_and(|value| !is_object(value.peek()))
                } else {
                    false
                }
            }
        }
    }

    fn allowed_shapes(&self) -> Option<BTreeSet<String>> {
        let mut filters = self.shape_filters.iter();
        let mut allowed = filters.next()?.clone();
        for filter in filters {
            allowed.retain(|shape| filter.contains(shape));
        }
        Some(allowed)
    }

    fn query_start(&self) -> QueryStart {
        let Some(first) = self.projections.first() else {
            return QueryStart::Everything;
        };
        let mut most_specific = first.clone();
        for projection in self.projections.iter().skip(1) {
            if is_at_or_below(projection, &most_specific) {
                most_specific = projection.clone();
            } else if !is_at_or_below(&most_specific, projection) {
                return QueryStart::Empty;
            }
        }
        QueryStart::Projection(most_specific)
    }
}

#[derive(Clone, Debug)]
struct ValueFilter {
    field_shape: String,
    field_name: String,
    operator: ValueFilterOperator,
    value: String,
}

#[derive(Clone, Debug)]
enum QueryOperator {
    Select(SelectionStage),
    Pop,
}

/// Compiled, immutable form of a Breadcrumbs query program.
///
/// Operators are order-preserving filters or adjacent Pop transforms. No
/// operator sorts, revisits, or materializes the address stream.
#[derive(Clone, Debug)]
pub(crate) struct QueryPlan {
    operators: Vec<QueryOperator>,
}

impl QueryPlan {
    pub(crate) fn new(breadcrumbs: Breadcrumbs) -> Self {
        let mut operators = Vec::new();
        let mut selection = SelectionStage::default();
        for operation in breadcrumbs.into_operations() {
            match operation {
                Breadcrumb::Pop => {
                    if !selection.is_empty() {
                        operators.push(QueryOperator::Select(std::mem::take(&mut selection)));
                    }
                    operators.push(QueryOperator::Pop);
                }
                operation @ Breadcrumb::ProjectFields { .. } => {
                    selection.add(operation);
                    operators.push(QueryOperator::Select(std::mem::take(&mut selection)));
                }
                operation => selection.add(operation),
            }
        }
        if !selection.is_empty() {
            operators.push(QueryOperator::Select(selection));
        }
        Self { operators }
    }

    pub(crate) fn evaluate<'source, 'arena>(
        &self,
        source: &'source ArenaAddressSource<'arena>,
    ) -> QueryPlanIter<'source, 'arena>
    where
        'arena: 'source,
    {
        let stats = Rc::new(QueryPlanStatsCell::default());
        let (pruner, start) = self
            .operators
            .first()
            .and_then(|operator| match operator {
                QueryOperator::Select(selection) => {
                    Some((BasePruner::new(selection), selection.query_start()))
                }
                QueryOperator::Pop => None,
            })
            .unwrap_or_default();
        let input = PrunedPreorder::new(source, pruner, start, Rc::clone(&stats));
        let operators = self
            .operators
            .clone()
            .into_iter()
            .map(RuntimeQueryOperator::from)
            .collect();
        QueryPlanIter {
            source,
            input,
            operators,
            stats,
            complete: false,
        }
    }
}

enum RuntimeQueryOperator {
    Select(SelectionStage),
    Pop(AdjacentPop),
}

impl From<QueryOperator> for RuntimeQueryOperator {
    fn from(value: QueryOperator) -> Self {
        match value {
            QueryOperator::Select(selection) => Self::Select(selection),
            QueryOperator::Pop => Self::Pop(AdjacentPop::default()),
        }
    }
}

impl RuntimeQueryOperator {
    fn apply(
        &mut self,
        source: &ArenaAddressSource<'_>,
        address: ValueAddress,
        stats: &QueryPlanStatsCell,
    ) -> Option<ValueAddress> {
        match self {
            Self::Select(selection) => selection
                .matches(source, &address, stats)
                .then_some(address),
            Self::Pop(pop) => pop.apply(address),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct QueryPlanInstrumentation {
    pub(crate) addressed: usize,
    pub(crate) reflected: usize,
    pub(crate) matched: usize,
    pub(crate) pruned_subtrees: usize,
}

#[derive(Default)]
struct QueryPlanStatsCell {
    addressed: Cell<usize>,
    reflected: Cell<usize>,
    matched: Cell<usize>,
    pruned_subtrees: Cell<usize>,
}

impl QueryPlanStatsCell {
    fn record_addressed(&self) {
        self.addressed.set(self.addressed.get().saturating_add(1));
    }

    fn record_reflection(&self) {
        self.reflected.set(self.reflected.get().saturating_add(1));
    }

    fn record_match(&self) {
        self.matched.set(self.matched.get().saturating_add(1));
    }

    fn record_pruned_subtree(&self) {
        self.pruned_subtrees
            .set(self.pruned_subtrees.get().saturating_add(1));
    }

    fn snapshot(&self) -> QueryPlanInstrumentation {
        QueryPlanInstrumentation {
            addressed: self.addressed.get(),
            reflected: self.reflected.get(),
            matched: self.matched.get(),
            pruned_subtrees: self.pruned_subtrees.get(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QueryPlanPoll {
    Item(ValueAddress),
    Pending,
    Complete,
}

pub(crate) struct QueryPlanIter<'source, 'arena> {
    source: &'source ArenaAddressSource<'arena>,
    input: PrunedPreorder<'source, 'arena>,
    operators: Vec<RuntimeQueryOperator>,
    stats: Rc<QueryPlanStatsCell>,
    complete: bool,
}

impl QueryPlanIter<'_, '_> {
    pub(crate) fn inspected(&self) -> usize {
        self.stats.addressed.get()
    }

    pub(crate) fn pruned_subtrees(&self) -> usize {
        self.stats.pruned_subtrees.get()
    }

    pub(crate) fn instrumentation(&self) -> QueryPlanInstrumentation {
        self.stats.snapshot()
    }

    /// Advance the raw reflected address stream cooperatively.
    ///
    /// One unit is charged before each attempt to obtain a raw address. Query
    /// operators process only that address and never pull from their input, so
    /// a filter with no matches cannot hide an unbounded scan in this call.
    pub(crate) fn poll_next(&mut self, budget: &mut WorkBudget) -> QueryPlanPoll {
        if self.complete {
            return QueryPlanPoll::Complete;
        }
        while budget.try_consume() {
            let Some(address) = self.input.next() else {
                self.complete = true;
                return QueryPlanPoll::Complete;
            };
            let mut candidate = Some(address);
            for operator in &mut self.operators {
                let Some(address) = candidate else {
                    break;
                };
                candidate = operator.apply(self.source, address, self.stats.as_ref());
            }
            if let Some(address) = candidate {
                self.stats.record_match();
                return QueryPlanPoll::Item(address);
            }
        }
        QueryPlanPoll::Pending
    }
}

impl Iterator for QueryPlanIter<'_, '_> {
    type Item = ValueAddress;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut budget = WorkBudget::new(usize::MAX);
            match self.poll_next(&mut budget) {
                QueryPlanPoll::Item(address) => return Some(address),
                QueryPlanPoll::Complete => return None,
                QueryPlanPoll::Pending => {}
            }
        }
    }
}

#[derive(Default)]
struct BasePruner {
    projections: Vec<ValueAddress>,
    allowed_shapes: Option<BTreeSet<String>>,
    shape_cache: RefCell<BTreeMap<String, bool>>,
}

impl BasePruner {
    fn new(selection: &SelectionStage) -> Self {
        Self {
            projections: selection.projections.clone(),
            allowed_shapes: selection.allowed_shapes(),
            shape_cache: RefCell::new(BTreeMap::new()),
        }
    }

    fn should_descend(
        &self,
        source: &ArenaAddressSource<'_>,
        address: &ValueAddress,
        stats: &QueryPlanStatsCell,
    ) -> bool {
        if !self
            .projections
            .iter()
            .all(|projection| subtrees_intersect(address, projection))
        {
            return false;
        }
        let Some(allowed) = &self.allowed_shapes else {
            return true;
        };
        if allowed.is_empty() {
            return false;
        }
        stats.record_reflection();
        let Ok(value) = source.resolve(address) else {
            return false;
        };
        let shape_name = cloud_terrastodon_registry::describe_shape(value.shape());
        if let Some(reachable) = self.shape_cache.borrow().get(&shape_name) {
            return *reachable;
        }
        let reachable = crate::projection_shapes::projection_shape_names(value.shape())
            .iter()
            .any(|shape| allowed.contains(shape));
        self.shape_cache.borrow_mut().insert(shape_name, reachable);
        reachable
    }
}

struct PrunedPreorder<'source, 'arena> {
    source: &'source ArenaAddressSource<'arena>,
    cursor: PreorderCursor<'source, ArenaAddressSource<'arena>>,
    pruner: BasePruner,
    stats: Rc<QueryPlanStatsCell>,
}

impl<'source, 'arena> PrunedPreorder<'source, 'arena> {
    fn new(
        source: &'source ArenaAddressSource<'arena>,
        pruner: BasePruner,
        start: QueryStart,
        stats: Rc<QueryPlanStatsCell>,
    ) -> Self {
        let cursor = match start {
            QueryStart::Everything => PreorderCursor::new(source),
            QueryStart::Projection(address) => PreorderCursor::from_address(source, address),
            QueryStart::Empty => PreorderCursor::empty(source),
        };
        Self {
            source,
            cursor,
            pruner,
            stats,
        }
    }
}

impl Iterator for PrunedPreorder<'_, '_> {
    type Item = ValueAddress;

    fn next(&mut self) -> Option<Self::Item> {
        let source = self.source;
        let pruner = &self.pruner;
        let stats = Rc::clone(&self.stats);
        let address = self.cursor.next_with_descend(|address| {
            let descend = pruner.should_descend(source, address, stats.as_ref());
            if !descend {
                stats.record_pruned_subtree();
            } else {
                // PreorderCursor asks ArenaAddressSource for this address's
                // reflected children immediately after this predicate.
                stats.record_reflection();
            }
            descend
        })?;
        self.stats.record_addressed();
        Some(address)
    }
}

fn is_at_or_below(address: &ValueAddress, ancestor: &ValueAddress) -> bool {
    address.root_id() == ancestor.root_id()
        && address
            .path()
            .segments()
            .starts_with(ancestor.path().segments())
}

fn subtrees_intersect(left: &ValueAddress, right: &ValueAddress) -> bool {
    left.root_id() == right.root_id()
        && (left.path().segments().starts_with(right.path().segments())
            || right.path().segments().starts_with(left.path().segments()))
}

fn value_filter_matches_address_or_ancestor(
    source: &ArenaAddressSource<'_>,
    address: &ValueAddress,
    filter: &ValueFilter,
    stats: &QueryPlanStatsCell,
) -> bool {
    let mut candidate = Some(address.clone());
    while let Some(address) = candidate {
        stats.record_reflection();
        if source
            .resolve(&address)
            .is_ok_and(|value| object_has_matching_field(value.peek(), filter))
        {
            return true;
        }
        candidate = address.parent();
    }
    false
}

fn object_has_matching_field(value: Peek<'_, 'static>, filter: &ValueFilter) -> bool {
    let value = value.innermost_peek();
    if let Ok(object) = value.into_struct() {
        return object
            .fields()
            .any(|(field, child)| field_matches(field.effective_name(), child, filter));
    }
    if let Ok(object) = value.into_enum()
        && let Ok(variant) = object.active_variant()
    {
        return variant
            .data
            .fields
            .iter()
            .enumerate()
            .any(|(index, field)| {
                object
                    .field(index)
                    .ok()
                    .flatten()
                    .is_some_and(|child| field_matches(field.effective_name(), child, filter))
            });
    }
    false
}

fn field_matches(name: &str, value: Peek<'_, 'static>, filter: &ValueFilter) -> bool {
    if filter.field_name != "*" && filter.field_name != name {
        return false;
    }
    let shape = cloud_terrastodon_registry::describe_shape(value.shape());
    if filter.field_shape != "*" && filter.field_shape != shape {
        return false;
    }
    let Some(candidate) = scalar_text(value) else {
        return false;
    };
    match filter.operator {
        ValueFilterOperator::Equals => candidate == filter.value,
        ValueFilterOperator::NotEquals => candidate != filter.value,
        ValueFilterOperator::Contains => candidate.contains(&filter.value),
    }
}

fn scalar_text(value: Peek<'_, 'static>) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    if let Some(proxy) = value.shape().effective_proxy(None)
        && let Ok(owned) = value.custom_serialization_with_proxy(proxy)
    {
        let proxied = owned.as_peek();
        return proxied
            .as_str()
            .map(ToOwned::to_owned)
            .or_else(|| Some(proxied.to_string()));
    }
    let value = value.innermost_peek();
    if value.into_list_like().is_ok()
        || value.into_set().is_ok()
        || value.into_map().is_ok()
        || value.into_struct().is_ok()
        || value.into_tuple().is_ok()
    {
        return None;
    }
    Some(value.to_string())
}

fn is_object(value: Peek<'_, 'static>) -> bool {
    let value = value.innermost_peek();
    value.into_struct().is_ok() || value.into_enum().is_ok() || value.into_map().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena::Arena;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    #[derive(Clone, Debug, Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    struct Permission {
        display_name: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[facet(rename_all = "camelCase")]
    #[repr(C)]
    struct Member {
        display_name: String,
        permission_objects: Vec<Permission>,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn member(name: &str, roles: &[&str]) -> Member {
        Member {
            display_name: name.to_owned(),
            permission_objects: roles
                .iter()
                .map(|role| Permission {
                    display_name: (*role).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn project_admin_query_filters_then_pops_without_materializing_members() {
        let mut arena = Arena::default();
        let members = arena
            .insert_ready(runtime(vec![
                member("Ada", &["Project Administrators", "Project Administrators"]),
                member("Ben", &["Readers"]),
                member("Cy", &["Project Administrators"]),
            ]))
            .unwrap();
        let permission_shape = cloud_terrastodon_registry::describe_shape(Permission::SHAPE);
        let query = Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![permission_shape],
            },
            Breadcrumb::ValueFilter {
                field_shape: cloud_terrastodon_registry::describe_shape(String::SHAPE),
                field_name: "displayName".to_owned(),
                operator: ValueFilterOperator::Equals,
                value: "Project Administrators".to_owned(),
            },
            Breadcrumb::Pop,
            Breadcrumb::Pop,
        ]);
        let source = ArenaAddressSource::new(&arena);
        let mut result = QueryPlan::new(query).evaluate(&source);

        assert_eq!(
            result.by_ref().collect::<Vec<_>>(),
            vec![
                ValueAddress::root(members).child(ValuePathSegment::Index(0)),
                ValueAddress::root(members).child(ValuePathSegment::Index(2)),
            ]
        );
        assert!(result.inspected() < 32);
    }

    #[test]
    fn exact_projection_starts_at_its_address_instead_of_scanning_earlier_roots() {
        let mut arena = Arena::default();
        for index in 0..64 {
            arena
                .insert_ready(runtime(format!("earlier {index}")))
                .unwrap();
        }
        let target = arena.insert_ready(runtime(String::from("target"))).unwrap();
        let source = ArenaAddressSource::new(&arena);
        let query = Breadcrumbs::new(vec![Breadcrumb::projection(target.get(), Vec::new())]);
        let mut result = QueryPlan::new(query).evaluate(&source);
        let mut budget = WorkBudget::new(1);

        assert_eq!(
            result.poll_next(&mut budget),
            QueryPlanPoll::Item(ValueAddress::root(target))
        );
        assert_eq!(result.inspected(), 1);
    }

    #[test]
    fn project_fields_extend_and_map_preserve_preorder_without_duplicates() {
        let mut arena = Arena::default();
        let members = arena
            .insert_ready(runtime(vec![member("Ada", &["Readers"])]))
            .unwrap();
        let member_address = ValueAddress::root(members).child(ValuePathSegment::Index(0));
        let member_shape = cloud_terrastodon_registry::describe_shape(Member::SHAPE);
        let source = ArenaAddressSource::new(&arena);

        let extend = QueryPlan::new(Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![member_shape.clone()],
            },
            Breadcrumb::ProjectFields {
                mode: ProjectFieldsMode::Extend,
                included_fields: vec![
                    ProjectedField::new(member_shape.clone(), "displayName"),
                    ProjectedField::new(member_shape.clone(), "permissionObjects"),
                ],
            },
        ]))
        .evaluate(&source)
        .collect::<Vec<_>>();
        assert_eq!(
            extend,
            vec![
                member_address.clone(),
                member_address
                    .clone()
                    .child(ValuePathSegment::Field("displayName".to_owned())),
                member_address
                    .clone()
                    .child(ValuePathSegment::Field("permissionObjects".to_owned())),
            ]
        );

        let mapped = QueryPlan::new(Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![member_shape.clone()],
            },
            Breadcrumb::ProjectFields {
                mode: ProjectFieldsMode::Map,
                included_fields: vec![
                    ProjectedField::new(member_shape.clone(), "displayName"),
                    ProjectedField::new(member_shape, "permissionObjects"),
                ],
            },
        ]))
        .evaluate(&source)
        .collect::<Vec<_>>();
        assert_eq!(mapped, extend[1..]);

        let mapped_strings = QueryPlan::new(Breadcrumbs::new(vec![
            Breadcrumb::ShapeFilter {
                included_shapes: vec![cloud_terrastodon_registry::describe_shape(Member::SHAPE)],
            },
            Breadcrumb::ProjectFields {
                mode: ProjectFieldsMode::Map,
                included_fields: vec![
                    ProjectedField::new(
                        cloud_terrastodon_registry::describe_shape(Member::SHAPE),
                        "displayName",
                    ),
                    ProjectedField::new(
                        cloud_terrastodon_registry::describe_shape(Member::SHAPE),
                        "permissionObjects",
                    ),
                ],
            },
            Breadcrumb::ShapeFilter {
                included_shapes: vec![cloud_terrastodon_registry::describe_shape(String::SHAPE)],
            },
        ]))
        .evaluate(&source)
        .collect::<Vec<_>>();
        assert_eq!(mapped_strings, vec![extend[1].clone()]);
    }

    #[test]
    fn projection_and_address_kind_are_order_preserving_filters() {
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(vec![String::from("zero"), String::from("one")]))
            .unwrap();
        let second = arena.insert_ready(runtime(String::from("other"))).unwrap();
        let query = Breadcrumbs::new(vec![
            Breadcrumb::projection(first.get(), vec![ValuePathSegment::Index(1)]),
            Breadcrumb::AddressKindFilter {
                include_roots: false,
                include_descendants: true,
            },
        ]);
        let source = ArenaAddressSource::new(&arena);

        assert_eq!(
            QueryPlan::new(query).evaluate(&source).collect::<Vec<_>>(),
            vec![ValueAddress::root(first).child(ValuePathSegment::Index(1))]
        );
        assert!(arena.ready_value(second).is_some());
    }

    #[test]
    fn impossible_shape_subtree_is_pruned_from_metadata() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(
                (0..10_000).map(|index| index as u64).collect::<Vec<_>>(),
            ))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let query = Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
            included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                Permission::SHAPE,
            )],
        }]);
        let mut evaluation = QueryPlan::new(query).evaluate(&source);

        assert_eq!(evaluation.next(), None);
        assert_eq!(evaluation.inspected(), 1);
        assert_eq!(evaluation.pruned_subtrees(), 1);
    }
}
