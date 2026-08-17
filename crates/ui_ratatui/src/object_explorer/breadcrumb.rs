use super::projected_field::ProjectedField;
use super::value_path::ValuePathSegment;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, facet::Facet)]
#[repr(C)]
pub(crate) enum ValueFilterOperator {
    Equals,
    NotEquals,
    Contains,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, facet::Facet)]
#[repr(C)]
pub(crate) enum ProjectFieldsMode {
    Extend,
    Map,
}

impl ProjectFieldsMode {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Extend => "extend from fields",
            Self::Map => "project to fields",
        }
    }
}

/// One explicit operation in a Tab's lazy reflected-address query.
///
/// Slot-kind filtering is intentionally absent: fields/elements are
/// ValueAddresses, not synthetic view/projection slots. Root-vs-descendant
/// filtering can be introduced as an address predicate when the migrated UI
/// needs that distinction.
#[derive(Clone, Debug, Eq, PartialEq, facet::Facet)]
#[repr(C)]
pub(crate) enum Breadcrumb {
    Projection {
        root_slot_id: u64,
        path: Vec<ValuePathSegment>,
    },
    ShapeFilter {
        included_shapes: Vec<String>,
    },
    AddressKindFilter {
        include_roots: bool,
        include_descendants: bool,
    },
    ValueFilter {
        field_shape: String,
        field_name: String,
        operator: ValueFilterOperator,
        value: String,
    },
    Pop,
    ProjectFields {
        mode: ProjectFieldsMode,
        included_fields: Vec<ProjectedField>,
    },
}

impl Breadcrumb {
    pub(crate) fn projection(root_slot_id: u64, path: Vec<ValuePathSegment>) -> Self {
        Self::Projection { root_slot_id, path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_breadcrumb_operations_round_trip_without_a_ui_model() {
        let operations = vec![
            Breadcrumb::projection(
                7,
                vec![
                    ValuePathSegment::Index(3),
                    ValuePathSegment::Field("permission_objects".to_owned()),
                ],
            ),
            Breadcrumb::ShapeFilter {
                included_shapes: vec!["AzureDevOpsProjectPermissionObject".to_owned()],
            },
            Breadcrumb::AddressKindFilter {
                include_roots: false,
                include_descendants: true,
            },
            Breadcrumb::ValueFilter {
                field_shape: "String".to_owned(),
                field_name: "displayName".to_owned(),
                operator: ValueFilterOperator::Equals,
                value: "Project Administrators".to_owned(),
            },
            Breadcrumb::ProjectFields {
                mode: ProjectFieldsMode::Map,
                included_fields: vec![ProjectedField::new("Thing", "name")],
            },
            Breadcrumb::Pop,
        ];
        let breadcrumbs = crate::object_explorer::breadcrumbs::Breadcrumbs::new(operations.clone());

        assert_eq!(breadcrumbs.operations(), operations);
    }
}
