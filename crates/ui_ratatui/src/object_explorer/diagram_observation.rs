use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::card_row_key::CardRowKey;
use super::card_row_snapshot::CardRowContent;
use super::card_snapshot::CardSnapshot;
use super::field_binding_snapshot::FieldBindingSnapshot;
use super::root_snapshot::RootSnapshot;
use super::value_address::ValueAddress;
use super::value_builder::BuilderStore;
use cloud_terrastodon_excalidraw::ExcalidrawObjectField;
use cloud_terrastodon_excalidraw::ExcalidrawObjectSnapshot;
use cloud_terrastodon_registry::DiscoveryPlan;

/// Observe lifecycle roots as well as ready values without attempting construction.
pub(crate) fn observe(
    arena: &Arena,
    builders: &BuilderStore,
    source: &ArenaAddressSource<'_>,
    address: ValueAddress,
    max_rows: usize,
) -> Result<ExcalidrawObjectSnapshot, String> {
    let card = if address.path().segments().is_empty() && source.resolve(&address).is_err() {
        RootSnapshot::observe(arena, builders, address.root_id(), max_rows)
            .map_err(|error| error.to_string())?
            .card()
            .clone()
    } else {
        CardSnapshot::observe_fields(source, address.clone(), max_rows)
            .map_err(|error| error.to_string())?
    };
    let fields = card
        .rows()
        .iter()
        .filter(|row| !matches!(row.key(), CardRowKey::Shape | CardRowKey::Action(_)))
        .map(|row| {
            let value =
                row.value_display()
                    .map(str::to_owned)
                    .unwrap_or_else(|| match row.content() {
                        CardRowContent::Text(text) => text.clone(),
                        CardRowContent::Address(address) => address.to_string(),
                        CardRowContent::Binding(binding) => match binding {
                            FieldBindingSnapshot::Unset => "unset".into(),
                            FieldBindingSnapshot::Default => "default".into(),
                            FieldBindingSnapshot::InlineOwned { shape } => {
                                format!("supplied {shape}")
                            }
                            FieldBindingSnapshot::CloneFrom(address) => {
                                format!("clone from {address}")
                            }
                            FieldBindingSnapshot::MoveFrom(slot) => {
                                format!("move from slot {slot}")
                            }
                            FieldBindingSnapshot::BorrowFrom(address) => {
                                format!("borrow from {address}")
                            }
                            FieldBindingSnapshot::PendingProducer => "waiting for producer".into(),
                        },
                        CardRowContent::RootAction(_) => {
                            unreachable!("actions are excluded from object fields")
                        }
                    });
            ExcalidrawObjectField {
                name: row.label().to_owned(),
                type_name: row.type_name().map(str::to_owned),
                value,
            }
        })
        .collect();
    let plan = source
        .resolve(&address)
        .ok()
        .and_then(|value| value.peek().get::<DiscoveryPlan>().ok().cloned());
    Ok(ExcalidrawObjectSnapshot {
        address: address.to_string(),
        shape: card.shape().to_owned(),
        fields,
        fields_complete: card.relationships_complete(),
        plan,
    })
}
