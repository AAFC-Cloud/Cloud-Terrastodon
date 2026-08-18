use crate::object_explorer::BreadcrumbContextSnapshot;
use crate::object_explorer::ValueFilterOperator;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) enum BreadcrumbFilterFieldPickerOutcome {
    Selected {
        field_shape: String,
        field_name: String,
        operator: ValueFilterOperator,
    },
    Cancelled,
    Failed(String),
}

pub(super) struct BreadcrumbFilterFieldPickerTask {
    handle: JoinHandle<BreadcrumbFilterFieldPickerOutcome>,
}

impl BreadcrumbFilterFieldPickerTask {
    pub(super) fn spawn(snapshot: &BreadcrumbContextSnapshot) -> Result<Self, String> {
        let choices = snapshot
            .fields()
            .iter()
            .flat_map(|field| {
                [
                    ("equals", ValueFilterOperator::Equals),
                    ("contains", ValueFilterOperator::Contains),
                    ("does not equal", ValueFilterOperator::NotEquals),
                ]
                .into_iter()
                .map(|(label, operator)| Choice {
                    key: format!("filter {} {label} …", field.label()),
                    value: (
                        field.field_shape().to_owned(),
                        field.selection().field_name().to_owned(),
                        operator,
                    ),
                })
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("the breadcrumb prefix has no reflected fields".to_owned());
        }
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header("Filter fields")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok((field_shape, field_name, operator)) => {
                    BreadcrumbFilterFieldPickerOutcome::Selected {
                        field_shape,
                        field_name,
                        operator,
                    }
                }
                Err(PickError::Cancelled) => BreadcrumbFilterFieldPickerOutcome::Cancelled,
                Err(error) => BreadcrumbFilterFieldPickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<BreadcrumbFilterFieldPickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("breadcrumb filter field PickerTui task failed: {error}"))
    }
}
