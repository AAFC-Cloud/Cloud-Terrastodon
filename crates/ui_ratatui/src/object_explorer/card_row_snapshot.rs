use super::card_row_key::CardRowKey;
use super::field_binding_snapshot::FieldBindingSnapshot;
use super::root_action_snapshot::RootActionSnapshot;
use super::value_address::ValueAddress;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CardRowContent {
    Text(String),
    Address(ValueAddress),
    Binding(FieldBindingSnapshot),
    RootAction(RootActionSnapshot),
}

/// One semantic row, independent of terminal width or visual wrapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CardRowSnapshot {
    key: CardRowKey,
    label: String,
    type_name: Option<String>,
    value_display: Option<String>,
    content: CardRowContent,
}

impl CardRowSnapshot {
    pub(crate) fn new(key: CardRowKey, label: impl Into<String>, content: CardRowContent) -> Self {
        Self {
            key,
            label: label.into(),
            type_name: None,
            value_display: None,
            content,
        }
    }

    pub(crate) fn with_type_name(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    pub(crate) fn with_value_display(mut self, value: impl Into<String>) -> Self {
        self.value_display = Some(value.into());
        self
    }

    pub(crate) const fn key(&self) -> &CardRowKey {
        &self.key
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    pub(crate) fn value_display(&self) -> Option<&str> {
        self.value_display.as_deref()
    }

    pub(crate) const fn content(&self) -> &CardRowContent {
        &self.content
    }
}
