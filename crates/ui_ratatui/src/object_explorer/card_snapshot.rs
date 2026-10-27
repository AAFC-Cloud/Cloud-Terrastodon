use super::arena_address_source::ArenaAddressSource;
use super::card_address::CardAddress;
use super::card_row_key::CardRowKey;
use super::card_row_snapshot::CardRowContent;
use super::card_row_snapshot::CardRowSnapshot;
use super::resolved_value::ResolvedValue;
use super::revision::RootRevision;
use super::root_action_snapshot::RootActionSnapshot;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use super::value_resolution_error::ValueResolutionError;
use cloud_terrastodon_registry::RegisteredShape;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use facet::Facet;
use facet::Type;

const MAX_SCALAR_SUMMARY_CHARS: usize = 256;

/// UI-neutral description of one reflected card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CardSnapshot {
    address: CardAddress,
    owned_slot: Option<SlotId>,
    shape: String,
    root_revision: RootRevision,
    rows: Vec<CardRowSnapshot>,
    relationships_complete: bool,
}

impl CardSnapshot {
    pub(crate) fn new_slot() -> Self {
        Self {
            address: CardAddress::NewSlot,
            owned_slot: None,
            shape: "new object".to_owned(),
            root_revision: RootRevision::default(),
            rows: Vec::new(),
            relationships_complete: true,
        }
    }

    pub(crate) fn owned(
        slot: SlotId,
        shape: impl Into<String>,
        root_revision: RootRevision,
        rows: Vec<CardRowSnapshot>,
        relationships_complete: bool,
    ) -> Self {
        Self {
            address: CardAddress::Value(ValueAddress::root(slot)),
            owned_slot: Some(slot),
            shape: shape.into(),
            root_revision,
            rows,
            relationships_complete,
        }
    }

    /// Observe one card and at most max_relationship_rows direct children.
    pub(crate) fn observe(
        source: &ArenaAddressSource<'_>,
        address: ValueAddress,
        max_relationship_rows: usize,
    ) -> Result<Self, ValueResolutionError> {
        Self::observe_rows(source, address, max_relationship_rows, true)
    }

    /// Observe data fields without spending the row budget on terminal actions.
    pub(crate) fn observe_fields(
        source: &ArenaAddressSource<'_>,
        address: ValueAddress,
        max_relationship_rows: usize,
    ) -> Result<Self, ValueResolutionError> {
        Self::observe_rows(source, address, max_relationship_rows, false)
    }

    fn observe_rows(
        source: &ArenaAddressSource<'_>,
        address: ValueAddress,
        max_relationship_rows: usize,
        include_actions: bool,
    ) -> Result<Self, ValueResolutionError> {
        let resolved = source.resolve(&address)?;
        let shape = cloud_terrastodon_registry::describe_shape(resolved.shape()).to_owned();
        let mut rows = vec![CardRowSnapshot::new(
            CardRowKey::Shape,
            "shape",
            CardRowContent::Text(shape.clone()),
        )];
        let variant = resolved
            .peek()
            .innermost_peek()
            .into_enum()
            .ok()
            .and_then(|object| object.active_variant().ok())
            .map(|variant| variant.effective_name());
        if let Some(variant) = variant {
            // A unit variant has no child fields. Its discriminant still tells
            // users whether a value is unresolved, disabled, or another state.
            rows.push(CardRowSnapshot::new(
                CardRowKey::Variant,
                "variant",
                CardRowContent::Text(variant.to_owned()),
            ));
        }

        let mut relationships_complete = true;
        let mut admitted_relationships = 0;
        // RegisteredShape's opaque metadata pointer is an implementation detail.
        // Its selected type identity is the scalar value users need to inspect.
        if resolved.shape().is_shape(RegisteredShape::SHAPE) {
            rows.push(CardRowSnapshot::new(
                CardRowKey::Value,
                "value",
                CardRowContent::Text(scalar_summary(resolved).expect("registered shape display")),
            ));
        } else {
            match source.reflected_children(&address)? {
                Some(mut children) => {
                    for child in children.by_ref().take(max_relationship_rows) {
                        rows.push(relationship_row(source, child)?);
                        admitted_relationships += 1;
                    }
                    relationships_complete = children.next().is_none();
                }
                None => {
                    if let Some(summary) = scalar_summary(resolved)
                        && Some(summary.as_str()) != variant
                    {
                        rows.push(CardRowSnapshot::new(
                            CardRowKey::Value,
                            "value",
                            CardRowContent::Text(summary),
                        ));
                    }
                }
            }
        }

        let owned_slot = address
            .path()
            .segments()
            .is_empty()
            .then_some(address.root_id());
        if include_actions && owned_slot.is_some() {
            let can_clone = RuntimeValue::can_clone_shape(resolved.shape());
            let actions = cloud_terrastodon_registry::functions_from(resolved.shape())
                .into_iter()
                .flat_map(|function| {
                    [
                        Some(RootActionSnapshot::Invoke {
                            function,
                            mode: super::invocation_mode::InvocationMode::Consume,
                        }),
                        can_clone.then_some(RootActionSnapshot::Invoke {
                            function,
                            mode: super::invocation_mode::InvocationMode::Retain,
                        }),
                        super::production_controller::arbitrary_constructor_for(
                            function.output_shape,
                        )
                        .map(|constructor| {
                            RootActionSnapshot::InvokeArbitrary {
                                request_function: function,
                                constructor,
                            }
                        }),
                    ]
                    .into_iter()
                    .flatten()
                });
            let mut actions = actions.peekable();
            for action in actions
                .by_ref()
                .take(max_relationship_rows.saturating_sub(admitted_relationships))
            {
                rows.push(CardRowSnapshot::new(
                    CardRowKey::Action(action.id()),
                    "action",
                    super::card_row_snapshot::CardRowContent::RootAction(action),
                ));
            }
            relationships_complete &= actions.next().is_none();
        }
        Ok(Self {
            address: CardAddress::Value(address),
            owned_slot,
            shape,
            root_revision: resolved.root_revision(),
            rows,
            relationships_complete,
        })
    }

    pub(crate) const fn address(&self) -> &CardAddress {
        &self.address
    }

    pub(crate) const fn owned_slot(&self) -> Option<SlotId> {
        self.owned_slot
    }

    pub(crate) fn shape(&self) -> &str {
        &self.shape
    }

    pub(crate) const fn root_revision(&self) -> RootRevision {
        self.root_revision
    }

    pub(crate) fn rows(&self) -> &[CardRowSnapshot] {
        &self.rows
    }

    pub(crate) const fn relationships_complete(&self) -> bool {
        self.relationships_complete
    }
}

fn relationship_row(
    source: &ArenaAddressSource<'_>,
    address: ValueAddress,
) -> Result<CardRowSnapshot, ValueResolutionError> {
    let resolved = source.resolve(&address)?;
    let type_name = cloud_terrastodon_registry::describe_shape(resolved.shape()).to_owned();
    let segment = address
        .path()
        .segments()
        .last()
        .expect("a reflected child has a non-empty path");
    let (key, label) = match segment {
        ValuePathSegment::Field(field) => (CardRowKey::Field(field.clone()), field.clone()),
        ValuePathSegment::Index(index) => (CardRowKey::Element(*index), format!("[{index}]")),
        ValuePathSegment::Key(key) => (CardRowKey::MapValue(key.clone()), format!("[{key:?}]")),
    };
    let row = CardRowSnapshot::new(key, label, CardRowContent::Address(address))
        .with_type_name(type_name);
    Ok(match scalar_summary(resolved) {
        Some(value) => row.with_value_display(value),
        None => row,
    })
}

fn scalar_summary(value: ResolvedValue<'_>) -> Option<String> {
    if let Ok(shape) = value.peek().get::<RegisteredShape>() {
        return Some(truncate(
            &format!("{} · {shape}", describe_shape(shape.shape())),
            MAX_SCALAR_SUMMARY_CHARS,
        ));
    }
    let value = value.peek().innermost_peek();
    if let Some(text) = value.as_str() {
        return Some(truncate(text, MAX_SCALAR_SUMMARY_CHARS));
    }
    // Domain scalars such as validated URLs may be reflected as structs while
    // their proxy hides the fields. Use their public Display representation.
    if matches!(value.shape().ty, Type::Primitive(_)) || value.shape().vtable.has_display() {
        return Some(truncate(&value.to_string(), MAX_SCALAR_SUMMARY_CHARS));
    }
    if let Ok(object) = value.into_enum()
        && let Ok(variant) = object.active_variant()
    {
        // Summarize only the discriminant, never Debug-format enum payloads:
        // authentication states can contain sensitive credentials and caches.
        return Some(truncate(variant.effective_name(), MAX_SCALAR_SUMMARY_CHARS));
    }
    None
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
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::card_window::CardWindow;
    use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AuthSource;
    use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
    use cloud_terrastodon_credentials::WorkloadIdentityConfig;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;
    use std::num::NonZeroUsize;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct MyThing {
        age: usize,
        name: String,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn auth_context_variants_are_visible_in_owned_cards_projections_and_field_previews() {
        for (context, expected_variant) in [
            (AuthContext::Unresolved, "Unresolved"),
            (AuthContext::None, "None"),
            (AuthContext::explicit(AuthSource::AzureCli), "Resolved"),
        ] {
            let mut arena = Arena::default();
            let owned = arena.insert_ready(runtime(context.clone())).unwrap();
            let wrapper = arena
                .insert_ready(runtime(AzureDevOpsAuthContext::AzureCli(context)))
                .unwrap();
            let source = ArenaAddressSource::new(&arena);
            let projected =
                ValueAddress::root(wrapper).child(ValuePathSegment::Field("0".to_owned()));
            for address in [ValueAddress::root(owned), projected] {
                // Variant metadata remains visible even with no field budget.
                let card = CardSnapshot::observe_fields(&source, address, 0).unwrap();
                assert!(card.rows().iter().any(|row| {
                    row.key() == &CardRowKey::Variant
                        && row.content() == &CardRowContent::Text(expected_variant.to_owned())
                }));
                assert!(
                    card.rows()
                        .iter()
                        .all(|row| row.key() != &CardRowKey::Value)
                );
            }
            let card =
                CardSnapshot::observe_fields(&source, ValueAddress::root(wrapper), 1).unwrap();
            assert!(card.rows().iter().any(|row| {
                row.key() == &CardRowKey::Variant
                    && row.content() == &CardRowContent::Text("AzureCli".to_owned())
            }));
            let field = card
                .rows()
                .iter()
                .find(|row| row.key() == &CardRowKey::Field("0".to_owned()))
                .unwrap();
            assert_eq!(field.value_display(), Some(expected_variant));
        }
    }

    #[test]
    fn resolved_auth_cards_show_public_policy_and_tenant_without_credential_fields() {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity)
            .with_workload_identity(WorkloadIdentityConfig {
                tenant_id,
                client_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
                federated_token: "synthetic-hidden-credential".to_owned(),
            })
            .unwrap();
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(
                AzureDevOpsAuthContext::for_tenant(&context, tenant_id).unwrap(),
            ))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let tenant_context =
            ValueAddress::root(root).child(ValuePathSegment::Field("0".to_owned()));
        let policy = tenant_context.child(ValuePathSegment::Field("auth_context".to_owned()));
        let card = CardSnapshot::observe_fields(&source, policy, 8).unwrap();
        let fields = card
            .rows()
            .iter()
            .filter_map(|row| match row.key() {
                CardRowKey::Field(name) => Some((name.as_str(), row.value_display())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            [
                ("requested_source", Some("workload-identity")),
                ("source", Some("workload-identity")),
                ("headless", Some("false")),
            ]
        );
        assert!(card.relationships_complete());
        let card = CardSnapshot::observe_fields(&source, tenant_context, 8).unwrap();
        let tenant = card
            .rows()
            .iter()
            .find(|row| row.key() == &CardRowKey::Field("tenant_id".to_owned()))
            .unwrap();
        assert_eq!(
            tenant.value_display(),
            Some("11111111-1111-1111-1111-111111111111")
        );
    }

    #[test]
    fn my_thing_is_one_owned_card_plus_addressed_field_cards() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(MyThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 8).unwrap();
        let age = ValueAddress::root(root).child(ValuePathSegment::Field("age".to_owned()));
        let name = ValueAddress::root(root).child(ValuePathSegment::Field("name".to_owned()));

        assert_eq!(window.cards().len(), 3);
        assert!(!window.has_before());
        assert!(!window.has_after());

        let root_card = &window.cards()[0];
        assert_eq!(
            root_card.address(),
            &CardAddress::Value(ValueAddress::root(root))
        );
        assert_eq!(root_card.owned_slot(), Some(root));
        assert_eq!(root_card.shape(), "MyThing");
        assert!(root_card.relationships_complete());
        assert!(root_card.rows().iter().any(|row| {
            row.key() == &CardRowKey::Field("age".to_owned())
                && row.content() == &CardRowContent::Address(age.clone())
        }));
        assert!(root_card.rows().iter().any(|row| {
            row.key() == &CardRowKey::Field("name".to_owned())
                && row.content() == &CardRowContent::Address(name.clone())
        }));

        assert_eq!(window.cards()[1].address(), &CardAddress::Value(age));
        assert_eq!(window.cards()[2].address(), &CardAddress::Value(name));
        assert_eq!(window.cards()[1].owned_slot(), None);
        assert_eq!(window.cards()[2].owned_slot(), None);
        assert!(window.cards()[2].rows().iter().any(|row| {
            row.key() == &CardRowKey::Value
                && row.content() == &CardRowContent::Text("Ada".to_owned())
        }));
        assert_eq!(
            arena.allocated_slot_count(),
            1,
            "cards and rows are projections, not ownership identities"
        );
    }

    #[test]
    fn card_and_relationship_budgets_bound_large_sequence_observation() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let window = CardWindow::first(&source, NonZeroUsize::new(8).unwrap(), 3).unwrap();

        assert_eq!(window.cards().len(), 8);
        assert!(window.has_after());
        assert_eq!(window.cards()[0].rows().len(), 4);
        assert!(!window.cards()[0].relationships_complete());
        assert!(
            window
                .cards()
                .iter()
                .skip(1)
                .all(|card| card.owned_slot().is_none())
        );
    }

    #[test]
    fn selected_target_types_display_their_identity_in_list_rows_and_value_cards() {
        let selected = RegisteredShape::from_shape(MyThing::SHAPE);
        let display = format!("MyThing · {selected}");
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(vec![selected])).unwrap();
        let source = ArenaAddressSource::new(&arena);
        let list = CardSnapshot::observe_fields(&source, ValueAddress::root(root), 8).unwrap();
        assert_eq!(list.rows()[1].value_display(), Some(display.as_str()));
        let value = CardSnapshot::observe_fields(
            &source,
            ValueAddress::root(root).child(ValuePathSegment::Index(0)),
            8,
        )
        .unwrap();
        assert!(
            value
                .rows()
                .iter()
                .any(|row| row.content() == &CardRowContent::Text(display.clone()))
        );
        assert_eq!(
            value.rows().len(),
            2,
            "type metadata pointers are not object fields"
        );
    }

    #[test]
    fn organization_url_fields_and_projected_cards_show_the_expanded_url() {
        #[derive(Facet)]
        struct Scope {
            org_url: AzureDevOpsOrganizationUrl,
        }

        for url in [
            "https://dev.azure.com/example-org",
            "https://example-org.visualstudio.com/DefaultCollection",
            "http://server.example.invalid:8080/tfs/My%20Collection",
        ] {
            let mut arena = Arena::default();
            arena
                .insert_ready(runtime(Scope {
                    org_url: url.parse().unwrap(),
                }))
                .unwrap();
            let source = ArenaAddressSource::new(&arena);
            let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 8).unwrap();

            assert_eq!(window.cards().len(), 2, "URL internals remain opaque");
            let field = window.cards()[0]
                .rows()
                .iter()
                .find(|row| row.key() == &CardRowKey::Field("org_url".to_owned()))
                .unwrap();
            assert_eq!(field.value_display(), Some(url));
            let projected = &window.cards()[1];
            assert_eq!(projected.owned_slot(), None);
            assert!(projected.rows().iter().any(|row| {
                row.key() == &CardRowKey::Value
                    && row.content() == &CardRowContent::Text(url.to_owned())
            }));
        }
    }
}
