use std::collections::BTreeSet;

use facet_reflect::HasFields;
use facet_reflect::Peek;

use super::arena_address_source::ArenaAddressSource;
use super::breadcrumbs::Breadcrumbs;
use super::projected_field::ProjectedField;
use super::query_plan::{QueryPlan, QueryPlanPoll};
use super::work_budget::WorkBudget;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct BreadcrumbContextField {
    selection: ProjectedField,
    field_shape: String,
}

impl BreadcrumbContextField {
    fn new(selection: ProjectedField, field_shape: impl Into<String>) -> Self {
        Self {
            selection,
            field_shape: field_shape.into(),
        }
    }

    pub(crate) const fn selection(&self) -> &ProjectedField {
        &self.selection
    }

    pub(crate) fn field_shape(&self) -> &str {
        &self.field_shape
    }

    pub(crate) fn label(&self) -> String {
        format!("{} ({})", self.selection.label(), self.field_shape)
    }
}

/// Bounded reflection metadata observed from one breadcrumb-query prefix.
///
/// This is metadata only: no addressed value, JSON document, or materialized
/// query result crosses the engine boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BreadcrumbContextSnapshot {
    shapes: Vec<String>,
    fields: Vec<BreadcrumbContextField>,
    inspected: usize,
    complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BreadcrumbContextValueSnapshot {
    values: Vec<String>,
    inspected: usize,
    complete: bool,
}

impl BreadcrumbContextSnapshot {
    pub(crate) fn inspect(
        source: &ArenaAddressSource<'_>,
        breadcrumbs: Breadcrumbs,
        max_work: usize,
        max_choices: usize,
    ) -> Result<Self, String> {
        if max_work == 0 {
            return Err("breadcrumb context inspection needs a positive work budget".to_owned());
        }
        if max_choices == 0 {
            return Err("breadcrumb context inspection needs a positive choice limit".to_owned());
        }

        let mut query = QueryPlan::new(breadcrumbs).evaluate(source);
        let mut work = WorkBudget::new(max_work);
        let mut shapes = BTreeSet::new();
        let mut fields = BTreeSet::new();
        let mut choices_truncated = false;
        let complete = loop {
            match query.poll_next(&mut work) {
                QueryPlanPoll::Item(address) => {
                    let Ok(value) = source.resolve(&address) else {
                        continue;
                    };
                    let owner_shape =
                        cloud_terrastodon_registry::describe_shape(value.shape()).to_owned();
                    insert_bounded(
                        &mut shapes,
                        owner_shape.clone(),
                        max_choices,
                        &mut choices_truncated,
                    );
                    collect_named_fields(
                        value.peek(),
                        &owner_shape,
                        &mut fields,
                        max_choices,
                        &mut choices_truncated,
                    );
                }
                QueryPlanPoll::Pending => break false,
                QueryPlanPoll::Complete => break true,
            }
        };
        let inspected = query.inspected();

        Ok(Self {
            shapes: shapes.into_iter().collect(),
            fields: fields.into_iter().collect(),
            inspected,
            complete: complete && !choices_truncated,
        })
    }

    pub(crate) fn shapes(&self) -> &[String] {
        &self.shapes
    }

    pub(crate) fn fields(&self) -> &[BreadcrumbContextField] {
        &self.fields
    }

    pub(crate) const fn inspected(&self) -> usize {
        self.inspected
    }

    pub(crate) const fn complete(&self) -> bool {
        self.complete
    }

    pub(crate) fn inspect_values(
        source: &ArenaAddressSource<'_>,
        breadcrumbs: Breadcrumbs,
        field_shape: &str,
        field_name: &str,
        max_work: usize,
        max_choices: usize,
    ) -> Result<BreadcrumbContextValueSnapshot, String> {
        if max_work == 0 || max_choices == 0 {
            return Err("breadcrumb value inspection needs positive limits".to_owned());
        }
        let mut query = QueryPlan::new(breadcrumbs).evaluate(source);
        let mut work = WorkBudget::new(max_work);
        let mut values = BTreeSet::new();
        let mut choices_truncated = false;
        let complete = loop {
            match query.poll_next(&mut work) {
                QueryPlanPoll::Item(address) => collect_field_values(
                    source,
                    &address,
                    field_shape,
                    field_name,
                    max_choices,
                    &mut values,
                    &mut choices_truncated,
                ),
                QueryPlanPoll::Pending => break false,
                QueryPlanPoll::Complete => break true,
            }
        };
        Ok(BreadcrumbContextValueSnapshot {
            values: values.into_iter().collect(),
            inspected: query.inspected(),
            complete: complete && !choices_truncated,
        })
    }
}

impl BreadcrumbContextValueSnapshot {
    pub(crate) fn values(&self) -> &[String] {
        &self.values
    }

    pub(crate) const fn inspected(&self) -> usize {
        self.inspected
    }

    pub(crate) const fn complete(&self) -> bool {
        self.complete
    }
}

fn collect_field_values(
    source: &ArenaAddressSource<'_>,
    address: &super::value_address::ValueAddress,
    field_shape: &str,
    field_name: &str,
    max_choices: usize,
    values: &mut BTreeSet<String>,
    truncated: &mut bool,
) {
    let mut consider = |candidate: &super::value_address::ValueAddress| {
        let Some(super::value_path::ValuePathSegment::Field(name)) =
            candidate.path().segments().last()
        else {
            return;
        };
        if name != field_name {
            return;
        }
        let Ok(value) = source.resolve(candidate) else {
            return;
        };
        if cloud_terrastodon_registry::describe_shape(value.shape()) != field_shape {
            return;
        }
        let Some(text) = scalar_text(value.peek()) else {
            return;
        };
        insert_bounded(values, text, max_choices, truncated);
    };

    consider(address);
    if let Ok(Some(children)) = source.reflected_children(address) {
        for child in children.take(max_choices.saturating_add(1)) {
            consider(&child);
        }
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

fn collect_named_fields(
    value: facet_reflect::Peek<'_, 'static>,
    owner_shape: &str,
    fields: &mut BTreeSet<BreadcrumbContextField>,
    max_choices: usize,
    truncated: &mut bool,
) {
    if value.shape().proxy.is_some() || !value.shape().format_proxies.is_empty() {
        return;
    }
    let value = value.innermost_peek();
    if let Ok(object) = value.into_struct() {
        for (field, child) in object.fields() {
            insert_bounded(
                fields,
                BreadcrumbContextField::new(
                    ProjectedField::new(owner_shape, field.effective_name()),
                    cloud_terrastodon_registry::describe_shape(child.shape()),
                ),
                max_choices,
                truncated,
            );
        }
        return;
    }
    if let Ok(object) = value.into_enum()
        && let Ok(variant) = object.active_variant()
    {
        for field in variant.data.fields {
            insert_bounded(
                fields,
                BreadcrumbContextField::new(
                    ProjectedField::new(owner_shape, field.effective_name()),
                    cloud_terrastodon_registry::describe_shape(field.shape()),
                ),
                max_choices,
                truncated,
            );
        }
    }
}

fn insert_bounded<T: Ord>(
    values: &mut BTreeSet<T>,
    value: T,
    max_choices: usize,
    truncated: &mut bool,
) {
    if values.contains(&value) {
        return;
    }
    if values.len() == max_choices {
        *truncated = true;
        return;
    }
    values.insert(value);
}

#[cfg(test)]
mod tests {
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    use super::*;
    use crate::object_explorer::arena::Arena;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct ContextThing {
        name: String,
        age: usize,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn context_metadata_is_reflected_lazily_from_the_query_prefix() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(ContextThing {
                name: "Ada".to_owned(),
                age: 42,
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let snapshot =
            BreadcrumbContextSnapshot::inspect(&source, Breadcrumbs::default(), 16, 16).unwrap();

        assert!(
            snapshot
                .shapes()
                .iter()
                .any(|shape| shape == "ContextThing")
        );
        assert!(snapshot.fields().iter().any(|field| {
            field.selection() == &ProjectedField::new("ContextThing", "name")
                && field.field_shape() == "String"
        }));
        assert!(snapshot.inspected() <= 16);
        assert!(snapshot.complete());
    }

    #[test]
    fn context_scan_obeys_work_budget_for_million_element_values() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let snapshot =
            BreadcrumbContextSnapshot::inspect(&source, Breadcrumbs::default(), 7, 16).unwrap();

        assert_eq!(snapshot.inspected(), 7);
        assert!(!snapshot.complete());
    }

    #[test]
    fn value_context_returns_bounded_scalar_values_for_a_reflected_field() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(vec![
                ContextThing {
                    name: "Ada".to_owned(),
                    age: 42,
                },
                ContextThing {
                    name: "Grace".to_owned(),
                    age: 37,
                },
            ]))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let values = BreadcrumbContextSnapshot::inspect_values(
            &source,
            Breadcrumbs::default(),
            "String",
            "name",
            32,
            8,
        )
        .unwrap();

        assert_eq!(values.values(), ["Ada", "Grace"]);
        assert!(values.complete());
        assert!(values.inspected() <= 32);
    }
}
