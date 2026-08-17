use std::collections::BTreeSet;

use nucleo::Matcher;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};

use crate::object_explorer::{CardRowContent, CardRowKey, CardSnapshot, FieldBindingSnapshot};

pub(super) struct RowSearchState {
    query: String,
    matches: Vec<CardRowKey>,
    selected: usize,
}

impl RowSearchState {
    pub(super) fn new(card: &CardSnapshot, query: impl Into<String>) -> Self {
        let query = query.into();
        let matches = matching_rows(card, &query);
        Self {
            query,
            matches,
            selected: 0,
        }
    }

    pub(super) fn refresh(&mut self, card: &CardSnapshot) {
        self.matches = matching_rows(card, &self.query);
        self.selected = 0;
    }

    pub(super) fn push(&mut self, card: &CardSnapshot, character: char) {
        self.query.push(character);
        self.refresh(card);
    }

    pub(super) fn pop(&mut self, card: &CardSnapshot) {
        self.query.pop();
        self.refresh(card);
    }

    pub(super) fn move_by(&mut self, delta: isize) {
        if self.matches.is_empty() {
            self.selected = 0;
            return;
        }
        self.selected = self
            .selected
            .saturating_add_signed(delta)
            .min(self.matches.len() - 1);
    }

    pub(super) fn move_to_edge(&mut self, first: bool) {
        self.selected = if first {
            0
        } else {
            self.matches.len().saturating_sub(1)
        };
    }

    pub(super) fn query(&self) -> &str {
        &self.query
    }

    pub(super) fn matches(&self) -> &[CardRowKey] {
        &self.matches
    }

    pub(super) fn selected(&self) -> Option<&CardRowKey> {
        self.matches.get(self.selected)
    }
}

fn matching_rows(card: &CardSnapshot, query: &str) -> Vec<CardRowKey> {
    let labels = card.rows().iter().map(row_search_label).collect::<Vec<_>>();
    ranked_indices(query, &labels)
        .into_iter()
        .filter_map(|index| card.rows().get(index).map(|row| row.key().clone()))
        .collect()
}

fn ranked_indices(query: &str, labels: &[String]) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..labels.len()).collect();
    }

    let query_lower = query.to_lowercase();
    let mut ranked = Vec::new();
    let mut taken = BTreeSet::new();
    for (index, label) in labels.iter().enumerate() {
        if label.to_lowercase().starts_with(&query_lower) {
            taken.insert(index);
            ranked.push(index);
        }
    }
    for (index, label) in labels.iter().enumerate() {
        if !taken.contains(&index) && label.to_lowercase().contains(&query_lower) {
            taken.insert(index);
            ranked.push(index);
        }
    }

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo::Config::DEFAULT);
    for (matched, _) in pattern.match_list(labels, &mut matcher) {
        if let Some(index) = labels
            .iter()
            .enumerate()
            .find_map(|(index, label)| (label == matched && taken.insert(index)).then_some(index))
        {
            ranked.push(index);
        }
    }
    ranked
}

fn row_search_label(row: &crate::object_explorer::CardRowSnapshot) -> String {
    let mut label = String::new();
    if let Some(type_name) = row.type_name() {
        label.push_str("type ");
        label.push_str(type_name);
        label.push(' ');
    }
    label.push_str(row.label());
    label.push(' ');
    label.push_str(&match row.content() {
        CardRowContent::Text(text) => text.clone(),
        CardRowContent::Address(address) => address.to_string(),
        CardRowContent::Binding(binding) => match binding {
            FieldBindingSnapshot::Unset => "unset".to_owned(),
            FieldBindingSnapshot::Default => "default".to_owned(),
            FieldBindingSnapshot::InlineOwned { shape } => format!("inline {shape}"),
            FieldBindingSnapshot::CloneFrom(address) => format!("clone {address}"),
            FieldBindingSnapshot::MoveFrom(slot) => format!("move slot {slot}"),
            FieldBindingSnapshot::BorrowFrom(address) => format!("borrow {address}"),
            FieldBindingSnapshot::PendingProducer => "pending producer".to_owned(),
        },
        CardRowContent::RootAction(action) => action.label(),
    });
    label
}

#[cfg(test)]
mod tests {
    use crate::object_explorer::{
        CardAddress, CardRowSnapshot, CardSnapshot, RootRevision, SlotId,
    };

    use super::*;

    #[test]
    fn nucleo_row_search_prioritizes_plain_substrings_then_fuzzy_matches() {
        let card = CardSnapshot::owned(
            SlotId::new(4),
            "Request",
            RootRevision::default(),
            vec![
                CardRowSnapshot::new(
                    CardRowKey::Field("organization".to_owned()),
                    "organization",
                    CardRowContent::Text("unset".to_owned()),
                ),
                CardRowSnapshot::new(
                    CardRowKey::Action("clone-invoke".to_owned()),
                    "action",
                    CardRowContent::Text("clone and invoke".to_owned()),
                ),
            ],
            true,
        );

        let search = RowSearchState::new(&card, "clone invoke");

        assert_eq!(
            search.selected(),
            Some(&CardRowKey::Action("clone-invoke".to_owned()))
        );
        assert_eq!(
            card.address(),
            &CardAddress::Value(crate::object_explorer::ValueAddress::root(SlotId::new(4)))
        );
    }
}
