use crate::object_explorer::BreadcrumbContextSnapshot;
use crate::object_explorer::ProjectFieldsMode;
use crate::object_explorer::ProjectedField;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use tokio::task::JoinHandle;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BreadcrumbPickerValue {
    Shape(String),
    Field(ProjectedField),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BreadcrumbPickerTarget {
    Shapes {
        edit_index: Option<usize>,
    },
    Fields {
        edit_index: Option<usize>,
        mode: ProjectFieldsMode,
    },
}

#[derive(Debug)]
pub(super) enum BreadcrumbPickerOutcome {
    Selected(Vec<BreadcrumbPickerValue>),
    Cancelled,
    Failed(String),
}

pub(super) struct BreadcrumbPickerTask {
    target: BreadcrumbPickerTarget,
    context_complete: bool,
    handle: JoinHandle<BreadcrumbPickerOutcome>,
}

impl BreadcrumbPickerTask {
    pub(super) fn shapes(
        snapshot: &BreadcrumbContextSnapshot,
        edit_index: Option<usize>,
        initially_included: &[String],
    ) -> Result<Self, String> {
        let mut labels = snapshot.shapes().iter().cloned().collect::<BTreeSet<_>>();
        labels.extend(initially_included.iter().cloned());
        let choices = labels
            .into_iter()
            .map(|shape| Choice {
                key: shape.clone(),
                value: BreadcrumbPickerValue::Shape(shape),
            })
            .collect::<Vec<_>>();
        Self::spawn(
            BreadcrumbPickerTarget::Shapes { edit_index },
            snapshot.complete(),
            "Filter shapes",
            choices,
            initially_included.to_vec(),
        )
    }

    pub(super) fn fields(
        snapshot: &BreadcrumbContextSnapshot,
        edit_index: Option<usize>,
        mode: ProjectFieldsMode,
        initially_included: &[ProjectedField],
    ) -> Result<Self, String> {
        let mut labels = snapshot
            .fields()
            .iter()
            .map(|field| (field.selection().clone(), field.label()))
            .collect::<BTreeMap<_, _>>();
        for field in initially_included {
            labels.entry(field.clone()).or_insert_with(|| field.label());
        }
        let initial = initially_included.iter().collect::<BTreeSet<_>>();
        let mut initial_keys = Vec::new();
        let choices = labels
            .into_iter()
            .map(|(field, label)| {
                if initial.contains(&field) {
                    initial_keys.push(label.clone());
                }
                Choice {
                    key: label,
                    value: BreadcrumbPickerValue::Field(field),
                }
            })
            .collect::<Vec<_>>();
        Self::spawn(
            BreadcrumbPickerTarget::Fields { edit_index, mode },
            snapshot.complete(),
            mode.label(),
            choices,
            initial_keys,
        )
    }

    fn spawn(
        target: BreadcrumbPickerTarget,
        context_complete: bool,
        header: &str,
        choices: Vec<Choice<BreadcrumbPickerValue>>,
        initially_selected: Vec<String>,
    ) -> Result<Self, String> {
        if choices.is_empty() {
            return Err("the breadcrumb prefix has no matching reflected choices".to_owned());
        }
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let header = header.to_owned();
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header(header)
                .set_auto_accept(false)
                .set_initial_selected(initially_selected)
                .terminal_coordinator(coordinator)
                .pick_many(choices)
                .await
            {
                Ok(selected) => BreadcrumbPickerOutcome::Selected(selected),
                Err(PickError::Cancelled) => BreadcrumbPickerOutcome::Cancelled,
                Err(error) => BreadcrumbPickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self {
            target,
            context_complete,
            handle,
        })
    }

    pub(super) const fn target(&self) -> &BreadcrumbPickerTarget {
        &self.target
    }

    pub(super) const fn context_complete(&self) -> bool {
        self.context_complete
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<BreadcrumbPickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("breadcrumb PickerTui task failed: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::Arena;
    use crate::object_explorer::ArenaAddressSource;
    use crate::object_explorer::Breadcrumbs;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct PickerThing {
        name: String,
    }

    #[test]
    fn field_picker_labels_preserve_owner_and_type_context() {
        let mut arena = Arena::default();
        arena
            .insert_ready(
                RuntimeValue::from_box(Box::new(PickerThing {
                    name: "Ada".to_owned(),
                }))
                .unwrap(),
            )
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let snapshot =
            BreadcrumbContextSnapshot::inspect(&source, Breadcrumbs::default(), 8, 8).unwrap();

        assert!(
            snapshot
                .fields()
                .iter()
                .any(|field| field.label() == "PickerThing.name (String)")
        );
    }
}
