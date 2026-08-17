use super::breadcrumb::{Breadcrumb, ValueFilterOperator};
use super::slot_id::SlotId;
use super::tab::Tab;
use super::value_path::ValuePathSegment;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TabHeaderSnapshot {
    slot: SlotId,
    name: String,
    breadcrumb_count: usize,
    first_visible_breadcrumb: usize,
    breadcrumb_labels: Vec<String>,
}

impl TabHeaderSnapshot {
    pub(crate) fn observe(slot: SlotId, tab: &Tab, max_breadcrumbs: usize) -> Self {
        let breadcrumb_count = tab.breadcrumbs().operations().len();
        let first_visible_breadcrumb = breadcrumb_count.saturating_sub(max_breadcrumbs);
        let breadcrumb_labels = tab
            .breadcrumbs()
            .operations()
            .iter()
            .skip(first_visible_breadcrumb)
            .map(breadcrumb_label)
            .collect();
        Self {
            slot,
            name: truncate(tab.name(), 96),
            breadcrumb_count,
            first_visible_breadcrumb,
            breadcrumb_labels,
        }
    }

    pub(crate) const fn slot(&self) -> SlotId {
        self.slot
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn breadcrumb_count(&self) -> usize {
        self.breadcrumb_count
    }

    pub(crate) const fn first_visible_breadcrumb(&self) -> usize {
        self.first_visible_breadcrumb
    }

    pub(crate) fn breadcrumb_labels(&self) -> &[String] {
        &self.breadcrumb_labels
    }
}

fn breadcrumb_label(breadcrumb: &Breadcrumb) -> String {
    match breadcrumb {
        Breadcrumb::Projection { root_slot_id, path } => {
            let mut label = format!("slot {root_slot_id}");
            for segment in path.iter().take(4) {
                match segment {
                    ValuePathSegment::Field(field) => {
                        label.push('.');
                        label.push_str(&truncate(field, 32));
                    }
                    ValuePathSegment::Index(index) => {
                        label.push_str(&format!("[{index}]"));
                    }
                    ValuePathSegment::Key(key) => {
                        label.push_str(&format!("[{}]", truncate(key, 32)));
                    }
                }
            }
            if path.len() > 4 {
                label.push_str("…");
            }
            label
        }
        Breadcrumb::ShapeFilter { included_shapes } => {
            let mut shapes = included_shapes
                .iter()
                .take(3)
                .map(|shape| truncate(shape, 32))
                .collect::<Vec<_>>()
                .join(" | ");
            if included_shapes.len() > 3 {
                shapes.push_str(&format!(" | … +{}", included_shapes.len() - 3));
            }
            format!("shape {shapes}")
        }
        Breadcrumb::AddressKindFilter {
            include_roots,
            include_descendants,
        } => match (*include_roots, *include_descendants) {
            (true, true) => "roots + descendants".to_owned(),
            (true, false) => "roots only".to_owned(),
            (false, true) => "descendants only".to_owned(),
            (false, false) => "no addresses".to_owned(),
        },
        Breadcrumb::ValueFilter {
            field_name,
            operator,
            value,
            ..
        } => {
            let operator = match operator {
                ValueFilterOperator::Equals => "=",
                ValueFilterOperator::NotEquals => "!=",
                ValueFilterOperator::Contains => "contains",
            };
            format!(
                "{} {operator} {}",
                truncate(field_name, 40),
                truncate(value, 48)
            )
        }
        Breadcrumb::Pop => "pop".to_owned(),
        Breadcrumb::ProjectFields {
            mode,
            included_fields,
        } => format!(
            "{} ({})",
            mode.label(),
            included_fields
                .iter()
                .take(3)
                .map(|field| truncate(&field.label(), 40))
                .collect::<Vec<_>>()
                .join(" | ")
        ),
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut result = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;

    #[test]
    fn tab_header_observation_is_bounded_independently_of_query_length() {
        let tab = Tab::new(
            "a very ordinary tab",
            Breadcrumbs::new((0..1_000).map(|_| Breadcrumb::Pop).collect()),
        );
        let snapshot = TabHeaderSnapshot::observe(SlotId::new(4), &tab, 8);

        assert_eq!(snapshot.breadcrumb_count(), 1_000);
        assert_eq!(snapshot.first_visible_breadcrumb(), 992);
        assert_eq!(snapshot.breadcrumb_labels().len(), 8);
    }

    #[test]
    fn projected_field_breadcrumbs_use_the_user_facing_operation_names() {
        let tab = Tab::new(
            "projection",
            Breadcrumbs::new(vec![Breadcrumb::ProjectFields {
                mode: super::super::breadcrumb::ProjectFieldsMode::Map,
                included_fields: vec![crate::object_explorer::ProjectedField::new("Thing", "name")],
            }]),
        );

        let snapshot = TabHeaderSnapshot::observe(SlotId::new(4), &tab, 8);

        assert_eq!(
            snapshot.breadcrumb_labels(),
            ["project to fields (Thing.name)"]
        );
    }
}
