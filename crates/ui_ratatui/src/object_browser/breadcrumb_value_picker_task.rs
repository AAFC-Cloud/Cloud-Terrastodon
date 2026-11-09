use crate::object_explorer::BreadcrumbContextValueSnapshot;
use crate::object_explorer::ValueFilterOperator;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) enum BreadcrumbValuePickerOutcome {
    Selected(String),
    Cancelled,
    Failed(String),
}

pub(super) struct BreadcrumbValuePickerTask {
    pub(super) edit_index: Option<usize>,
    pub(super) field_shape: String,
    pub(super) field_name: String,
    pub(super) operator: ValueFilterOperator,
    pub(super) context_complete: bool,
    handle: JoinHandle<eyre::Result<BreadcrumbValuePickerOutcome>>,
    cancellation: CancellationToken,
}

impl BreadcrumbValuePickerTask {
    pub(super) fn spawn(
        edit_index: Option<usize>,
        field_shape: String,
        field_name: String,
        operator: ValueFilterOperator,
        snapshot: BreadcrumbContextValueSnapshot,
        initial_value: Option<String>,
        cancellation: &CancellationScope,
    ) -> Result<Self, String> {
        let mut values = snapshot.values().to_vec();
        if let Some(initial) = &initial_value
            && !values.iter().any(|value| value == initial)
        {
            values.push(initial.clone());
            values.sort();
        }
        if values.is_empty() {
            return Err("the breadcrumb prefix has no scalar values for this field".to_owned());
        }
        let choices = values
            .into_iter()
            .map(|value| Choice {
                key: value.clone(),
                value,
            })
            .collect::<Vec<_>>();
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let header = format!(
            "Filter {} {} ({})",
            field_name,
            match operator {
                ValueFilterOperator::Equals => "equals",
                ValueFilterOperator::NotEquals => "does not equal",
                ValueFilterOperator::Contains => "contains",
            },
            field_shape
        );
        let query = initial_value.unwrap_or_default();
        let context_complete = snapshot.complete();
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            eyre::Ok(
                match PickerTui::new()
                    .set_header(header)
                    .set_query(query)
                    .set_auto_accept(false)
                    .terminal_coordinator(coordinator)
                    .pick_one(choices, &scope)
                    .await
                {
                    Ok(value) => BreadcrumbValuePickerOutcome::Selected(value),
                    Err(PickError::Cancelled) => BreadcrumbValuePickerOutcome::Cancelled,
                    Err(error) => BreadcrumbValuePickerOutcome::Failed(error.to_string()),
                },
            )
        });
        let handle = tokio::spawn(future);
        Ok(Self {
            edit_index,
            field_shape,
            field_name,
            operator,
            context_complete,
            handle,
            cancellation,
        })
    }

    pub(super) async fn close(self) {
        self.cancellation.request_cancel("object browser closed");
        match self.finish().await {
            Ok(BreadcrumbValuePickerOutcome::Failed(error)) | Err(error) => {
                tracing::warn!(%error, "Nested picker failed during shutdown");
            }
            Ok(_) => {}
        }
    }
    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<BreadcrumbValuePickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("breadcrumb value PickerTui task failed: {error}"))?
            .map_err(|error| error.to_string())
    }
}
