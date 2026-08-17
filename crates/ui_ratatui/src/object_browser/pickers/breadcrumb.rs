use crate::object_explorer::{
    Breadcrumb, ProjectFieldsMode, ProjectedField, ValueAddress, ValueFilterOperator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BreadcrumbPickerChoice {
    Add(Breadcrumb),
    PromptValue {
        edit_index: Option<usize>,
        initial_value: Option<String>,
        field_shape: String,
        field_name: String,
        operator: ValueFilterOperator,
    },
    PickShapes {
        edit_index: Option<usize>,
        initially_included: Vec<String>,
    },
    PickFields {
        edit_index: Option<usize>,
        mode: ProjectFieldsMode,
        initially_included: Vec<ProjectedField>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BreadcrumbPickerRow {
    label: String,
    choice: BreadcrumbPickerChoice,
}

impl BreadcrumbPickerRow {
    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) const fn choice(&self) -> &BreadcrumbPickerChoice {
        &self.choice
    }
}

pub(crate) struct BreadcrumbPicker {
    rows: Vec<BreadcrumbPickerRow>,
    selected: usize,
}

impl BreadcrumbPicker {
    pub(crate) fn new(selected: Option<(&ValueAddress, &str)>, breadcrumbs: &[Breadcrumb]) -> Self {
        let mut rows = vec![BreadcrumbPickerRow {
            label: "filter shapes…".to_owned(),
            choice: BreadcrumbPickerChoice::PickShapes {
                edit_index: None,
                initially_included: Vec::new(),
            },
        }];
        if let Some((address, shape)) = selected {
            if let Some(crate::object_explorer::ValuePathSegment::Field(field_name)) =
                address.path().segments().last()
            {
                for (label, operator) in [
                    ("equals", ValueFilterOperator::Equals),
                    ("contains", ValueFilterOperator::Contains),
                    ("does not equal", ValueFilterOperator::NotEquals),
                ] {
                    rows.push(BreadcrumbPickerRow {
                        label: format!("filter {field_name} {label} …"),
                        choice: BreadcrumbPickerChoice::PromptValue {
                            edit_index: None,
                            initial_value: None,
                            field_shape: shape.to_owned(),
                            field_name: field_name.clone(),
                            operator,
                        },
                    });
                }
            }
            rows.push(BreadcrumbPickerRow {
                label: format!("project from {address}"),
                choice: BreadcrumbPickerChoice::Add(Breadcrumb::projection(
                    address.root_id().get(),
                    address.path().segments().to_vec(),
                )),
            });
        }
        rows.extend(breadcrumbs.iter().enumerate().filter_map(
            |(index, breadcrumb)| match breadcrumb {
                Breadcrumb::ShapeFilter { included_shapes } => Some(BreadcrumbPickerRow {
                    label: format!("edit breadcrumb {}: filter shapes", index + 1),
                    choice: BreadcrumbPickerChoice::PickShapes {
                        edit_index: Some(index),
                        initially_included: included_shapes.clone(),
                    },
                }),
                Breadcrumb::ProjectFields {
                    mode,
                    included_fields,
                } => Some(BreadcrumbPickerRow {
                    label: format!("edit breadcrumb {}: {}", index + 1, mode.label()),
                    choice: BreadcrumbPickerChoice::PickFields {
                        edit_index: Some(index),
                        mode: *mode,
                        initially_included: included_fields.clone(),
                    },
                }),
                Breadcrumb::ValueFilter {
                    field_shape,
                    field_name,
                    operator,
                    value,
                } => Some(BreadcrumbPickerRow {
                    label: format!("edit breadcrumb {}: filter {field_name}", index + 1),
                    choice: BreadcrumbPickerChoice::PromptValue {
                        edit_index: Some(index),
                        initial_value: Some(value.clone()),
                        field_shape: field_shape.clone(),
                        field_name: field_name.clone(),
                        operator: *operator,
                    },
                }),
                _ => None,
            },
        ));
        rows.extend([
            BreadcrumbPickerRow {
                label: "pop to parents".to_owned(),
                choice: BreadcrumbPickerChoice::Add(Breadcrumb::Pop),
            },
            BreadcrumbPickerRow {
                label: ProjectFieldsMode::Extend.label().to_owned(),
                choice: BreadcrumbPickerChoice::PickFields {
                    edit_index: None,
                    mode: ProjectFieldsMode::Extend,
                    initially_included: Vec::new(),
                },
            },
            BreadcrumbPickerRow {
                label: ProjectFieldsMode::Map.label().to_owned(),
                choice: BreadcrumbPickerChoice::PickFields {
                    edit_index: None,
                    mode: ProjectFieldsMode::Map,
                    initially_included: Vec::new(),
                },
            },
            BreadcrumbPickerRow {
                label: "roots only".to_owned(),
                choice: BreadcrumbPickerChoice::Add(Breadcrumb::AddressKindFilter {
                    include_roots: true,
                    include_descendants: false,
                }),
            },
            BreadcrumbPickerRow {
                label: "descendants only".to_owned(),
                choice: BreadcrumbPickerChoice::Add(Breadcrumb::AddressKindFilter {
                    include_roots: false,
                    include_descendants: true,
                }),
            },
        ]);
        Self { rows, selected: 0 }
    }

    pub(crate) fn rows(&self) -> &[BreadcrumbPickerRow] {
        &self.rows
    }

    pub(crate) const fn selected_index(&self) -> usize {
        self.selected
    }

    pub(crate) fn selected(&self) -> Option<&BreadcrumbPickerRow> {
        self.rows.get(self.selected)
    }

    pub(crate) fn move_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + 1).min(self.rows.len() - 1);
        }
    }

    pub(crate) fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::{SlotId, ValuePathSegment};

    #[test]
    fn filter_shapes_is_the_default_breadcrumb_choice_without_an_initial_selection() {
        let address = ValueAddress::root(SlotId::new(5))
            .child(ValuePathSegment::Index(3))
            .child(ValuePathSegment::Field("display_name".to_owned()));
        let picker = BreadcrumbPicker::new(Some((&address, "String")), &[]);

        assert_eq!(picker.selected_index(), 0);
        assert_eq!(picker.selected().unwrap().label(), "filter shapes…");
        assert!(matches!(
            picker.selected().unwrap().choice(),
            BreadcrumbPickerChoice::PickShapes {
                edit_index: None,
                initially_included,
            } if initially_included.is_empty()
        ));
    }

    #[test]
    fn selected_reflected_field_still_offers_generic_value_filters() {
        let address = ValueAddress::root(SlotId::new(5))
            .child(ValuePathSegment::Index(3))
            .child(ValuePathSegment::Field("display_name".to_owned()));
        let picker = BreadcrumbPicker::new(Some((&address, "String")), &[]);
        let row = picker
            .rows()
            .iter()
            .find(|row| row.label() == "filter display_name equals …")
            .expect("the selected reflected field contributes a value filter");

        assert_eq!(row.label(), "filter display_name equals …");
        assert!(matches!(
            row.choice(),
            BreadcrumbPickerChoice::PromptValue {
                field_shape,
                field_name,
                operator: ValueFilterOperator::Equals,
                ..
            } if field_shape == "String" && field_name == "display_name"
        ));
    }

    #[test]
    fn existing_shape_and_field_breadcrumbs_reopen_as_edit_choices() {
        let picker = BreadcrumbPicker::new(
            None,
            &[
                Breadcrumb::ShapeFilter {
                    included_shapes: vec!["Thing".to_owned()],
                },
                Breadcrumb::ProjectFields {
                    mode: ProjectFieldsMode::Map,
                    included_fields: vec![ProjectedField::new("Thing", "name")],
                },
            ],
        );

        assert_eq!(picker.rows()[0].label(), "filter shapes…");
        assert_eq!(
            picker.rows()[2].label(),
            "edit breadcrumb 2: project to fields"
        );
        assert!(matches!(
            picker.rows()[1].choice(),
            BreadcrumbPickerChoice::PickShapes {
                edit_index: Some(0),
                initially_included,
            } if initially_included == &["Thing"]
        ));
        assert!(
            picker
                .rows()
                .iter()
                .any(|row| row.label() == "extend from fields")
        );
    }
}
