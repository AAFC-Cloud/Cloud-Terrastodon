use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::borrow_graph::BorrowGraph;
use super::field_binding::FieldBinding;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_builder::BuilderStore;
use super::value_candidate::ValueCandidate;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldCandidateAction {
    Borrow,
    Move,
    Clone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldCandidateConsequence {
    action: FieldCandidateAction,
    description: String,
}

impl FieldCandidateConsequence {
    pub(crate) const fn action(&self) -> FieldCandidateAction {
        self.action
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }
}

/// Valid transfer choices for one picker candidate, including generic owner
/// provenance. No candidate value is cloned while this preview is produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldCandidateActions {
    candidate: ValueCandidate,
    consequences: Vec<FieldCandidateConsequence>,
}

impl FieldCandidateActions {
    pub(crate) fn inspect(
        arena: &Arena,
        builders: &BuilderStore,
        borrow_graph: &BorrowGraph,
        destination: SlotId,
        field: usize,
        source: ValueAddress,
    ) -> Result<Self, String> {
        let address_source = ArenaAddressSource::new(arena);
        let candidate = ValueCandidate::resolve(&address_source, source.clone())
            .ok_or_else(|| format!("candidate {source} does not resolve"))?;
        let field_name = builders
            .builder(destination)
            .and_then(|builder| builder.field_name(field))
            .ok_or_else(|| format!("slot {destination} has no builder field {field}"))?;
        let destination_label = format!("slot {destination}.{field_name}");
        let candidate_label = candidate.display_label();
        let mut consequences = Vec::new();

        if builders
            .validate_field_binding(
                arena,
                borrow_graph,
                destination,
                field,
                &FieldBinding::BorrowFrom(source.clone()),
            )
            .is_ok()
        {
            consequences.push(FieldCandidateConsequence {
                action: FieldCandidateAction::Borrow,
                description: format!(
                    "{candidate_label} will be borrowed by {destination_label}. The owning root stays in place and is protected from mutation, move, and deletion while borrowed."
                ),
            });
        }

        if source.path().segments().is_empty()
            && builders
                .validate_field_binding(
                    arena,
                    borrow_graph,
                    destination,
                    field,
                    &FieldBinding::MoveFrom(source.root_id()),
                )
                .is_ok()
        {
            consequences.push(FieldCandidateConsequence {
                action: FieldCandidateAction::Move,
                description: format!(
                    "{candidate_label} will move into {destination_label}. Its arena root becomes Consumed."
                ),
            });
        }

        if builders
            .validate_field_binding(
                arena,
                borrow_graph,
                destination,
                field,
                &FieldBinding::CloneFrom(source),
            )
            .is_ok()
        {
            consequences.push(FieldCandidateConsequence {
                action: FieldCandidateAction::Clone,
                description: format!(
                    "{candidate_label} will be cloned into {destination_label}. Its owning root and containing field remain unchanged."
                ),
            });
        }

        if consequences.is_empty() {
            return Err(format!(
                "{} cannot populate {destination_label}",
                candidate.display_label()
            ));
        }
        Ok(Self {
            candidate,
            consequences,
        })
    }

    pub(crate) const fn candidate(&self) -> &ValueCandidate {
        &self.candidate
    }

    pub(crate) fn consequences(&self) -> &[FieldCandidateConsequence] {
        &self.consequences
    }
}
