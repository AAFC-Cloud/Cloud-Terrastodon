use super::arena::Arena;
use super::preorder_cursor::AddressSource;
use super::resolved_value::ResolvedValue;
use super::revision::RootRevision;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use super::value_resolution_error::ValueResolutionError;
use facet_reflect::HasFields;
use facet_reflect::Peek;

/// Facet-backed topology adapter over Arena roots.
///
/// Ordinary query/export sources expose only Ready roots. The object-pool
/// source additionally emits lifecycle roots as leaves so partially built and
/// pending owned values do not disappear from navigation.
pub(crate) struct ArenaAddressSource<'arena> {
    arena: &'arena Arena,
    include_lifecycle_roots: bool,
}

impl<'arena> ArenaAddressSource<'arena> {
    pub(crate) const fn new(arena: &'arena Arena) -> Self {
        Self {
            arena,
            include_lifecycle_roots: false,
        }
    }

    pub(crate) const fn object_pool(arena: &'arena Arena) -> Self {
        Self {
            arena,
            include_lifecycle_roots: true,
        }
    }

    pub(crate) fn resolve(
        &self,
        address: &ValueAddress,
    ) -> Result<ResolvedValue<'arena>, ValueResolutionError> {
        let root_revision = self
            .arena
            .root_revision(address.root_id())
            .ok_or(super::arena::ArenaError::UnknownSlot(address.root_id()))?;
        self.resolve_with_revision(address, root_revision)
    }

    pub(crate) fn resolve_at_revision(
        &self,
        address: &ValueAddress,
        expected: RootRevision,
    ) -> Result<ResolvedValue<'arena>, ValueResolutionError> {
        let actual = self
            .arena
            .root_revision(address.root_id())
            .ok_or(super::arena::ArenaError::UnknownSlot(address.root_id()))?;
        if actual != expected {
            return Err(ValueResolutionError::StaleRootRevision {
                root: address.root_id(),
                expected,
                actual,
            });
        }
        self.resolve_with_revision(address, actual)
    }

    pub(crate) fn reflected_children(
        &self,
        parent: &ValueAddress,
    ) -> Result<Option<Box<dyn Iterator<Item = ValueAddress> + 'arena>>, ValueResolutionError> {
        Ok(child_addresses(
            parent.clone(),
            self.resolve(parent)?.peek(),
        ))
    }

    fn resolve_with_revision(
        &self,
        address: &ValueAddress,
        root_revision: RootRevision,
    ) -> Result<ResolvedValue<'arena>, ValueResolutionError> {
        let mut current = self.arena.resolve_root(address.root_id())?.peek();
        let mut parent = ValueAddress::root(address.root_id());
        for segment in address.path().segments() {
            current = match segment {
                ValuePathSegment::Field(field_name) => field(current, &parent, field_name)?,
                ValuePathSegment::Index(index) => indexed_value(current, &parent, *index)?,
                ValuePathSegment::Key(key) => map_value(current, &parent, key)?,
            };
            parent = parent.child(segment.clone());
        }
        Ok(ResolvedValue::new(current, root_revision))
    }
}

impl AddressSource for ArenaAddressSource<'_> {
    fn roots(&self) -> Box<dyn Iterator<Item = SlotId> + '_> {
        if self.include_lifecycle_roots {
            Box::new(self.arena.object_pool_slot_ids())
        } else {
            Box::new(self.arena.ready_slot_ids())
        }
    }

    fn children(
        &self,
        parent: &ValueAddress,
    ) -> Option<Box<dyn Iterator<Item = ValueAddress> + '_>> {
        self.reflected_children(parent).ok().flatten()
    }
}

fn child_addresses<'mem>(
    parent: ValueAddress,
    value: Peek<'mem, 'static>,
) -> Option<Box<dyn Iterator<Item = ValueAddress> + 'mem>> {
    if value.shape().proxy.is_some() || !value.shape().format_proxies.is_empty() {
        return None;
    }

    let value = value.innermost_peek();
    if let Ok(list) = value.into_list_like() {
        if list.is_empty() {
            return None;
        }
        return Some(Box::new(list.iter().enumerate().map(move |(index, _)| {
            parent.child(ValuePathSegment::Index(index))
        })));
    }
    if let Ok(set) = value.into_set() {
        if set.is_empty() {
            return None;
        }
        return Some(Box::new(set.iter().enumerate().map(move |(index, _)| {
            parent.child(ValuePathSegment::Index(index))
        })));
    }
    if let Ok(object) = value.into_struct() {
        if object.field_count() == 0 {
            return None;
        }
        return Some(Box::new(object.fields().map(move |(field, _)| {
            parent.child(ValuePathSegment::Field(field.effective_name().to_owned()))
        })));
    }
    if let Ok(object) = value.into_enum() {
        let variant = object.active_variant().ok()?;
        if variant.data.fields.is_empty() {
            return None;
        }
        return Some(Box::new(variant.data.fields.iter().map(move |field| {
            parent.child(ValuePathSegment::Field(field.effective_name().to_owned()))
        })));
    }
    if let Ok(tuple) = value.into_tuple() {
        if tuple.len() == 0 {
            return None;
        }
        return Some(Box::new(tuple.fields().enumerate().map(
            move |(index, _)| parent.child(ValuePathSegment::Index(index)),
        )));
    }
    if let Ok(map) = value.into_map() {
        if map.is_empty() {
            return None;
        }
        return Some(Box::new(map.iter().map(move |(key, _)| {
            let key = key
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| key.to_string());
            parent.child(ValuePathSegment::Key(key))
        })));
    }
    None
}

fn field<'mem>(
    value: Peek<'mem, 'static>,
    parent: &ValueAddress,
    name: &str,
) -> Result<Peek<'mem, 'static>, ValueResolutionError> {
    let value = value.innermost_peek();
    if let Ok(object) = value.into_struct() {
        return object
            .fields()
            .find_map(|(field, child)| (field.effective_name() == name).then_some(child))
            .ok_or_else(|| ValueResolutionError::MissingField {
                parent: parent.clone(),
                field: name.to_owned(),
                shape: shape_name(value),
            });
    }
    if let Ok(object) = value.into_enum() {
        let variant =
            object
                .active_variant()
                .map_err(|error| ValueResolutionError::Reflection {
                    parent: parent.clone(),
                    operation: "read the active enum variant",
                    message: error.to_string(),
                })?;
        let index = variant
            .data
            .fields
            .iter()
            .position(|field| field.effective_name() == name)
            .ok_or_else(|| ValueResolutionError::MissingField {
                parent: parent.clone(),
                field: name.to_owned(),
                shape: shape_name(value),
            })?;
        return object
            .field(index)
            .map_err(|error| ValueResolutionError::Reflection {
                parent: parent.clone(),
                operation: "read an enum field",
                message: error.to_string(),
            })?
            .ok_or_else(|| ValueResolutionError::MissingField {
                parent: parent.clone(),
                field: name.to_owned(),
                shape: shape_name(value),
            });
    }
    Err(unsupported(
        value,
        parent,
        ValuePathSegment::Field(name.to_owned()),
    ))
}

fn indexed_value<'mem>(
    value: Peek<'mem, 'static>,
    parent: &ValueAddress,
    index: usize,
) -> Result<Peek<'mem, 'static>, ValueResolutionError> {
    let value = value.innermost_peek();
    if let Ok(list) = value.into_list_like() {
        return list
            .get(index)
            .ok_or_else(|| index_error(value, parent, index, list.len()));
    }
    if let Ok(set) = value.into_set() {
        return set
            .iter()
            .nth(index)
            .ok_or_else(|| index_error(value, parent, index, set.len()));
    }
    if let Ok(tuple) = value.into_tuple() {
        return tuple
            .field(index)
            .ok_or_else(|| index_error(value, parent, index, tuple.len()));
    }
    Err(unsupported(value, parent, ValuePathSegment::Index(index)))
}

fn map_value<'mem>(
    value: Peek<'mem, 'static>,
    parent: &ValueAddress,
    key: &str,
) -> Result<Peek<'mem, 'static>, ValueResolutionError> {
    let value = value.innermost_peek();
    let map = value
        .into_map()
        .map_err(|_| unsupported(value, parent, ValuePathSegment::Key(key.to_owned())))?;
    map.iter()
        .find_map(|(candidate, value)| {
            let matches =
                candidate.as_str().is_some_and(|text| text == key) || candidate.to_string() == key;
            matches.then_some(value)
        })
        .ok_or_else(|| ValueResolutionError::MissingKey {
            parent: parent.clone(),
            key: key.to_owned(),
            shape: shape_name(value),
        })
}

fn index_error(
    value: Peek<'_, 'static>,
    parent: &ValueAddress,
    index: usize,
    len: usize,
) -> ValueResolutionError {
    ValueResolutionError::IndexOutOfRange {
        parent: parent.clone(),
        index,
        len,
        shape: shape_name(value),
    }
}

fn unsupported(
    value: Peek<'_, 'static>,
    parent: &ValueAddress,
    segment: ValuePathSegment,
) -> ValueResolutionError {
    ValueResolutionError::SegmentNotSupported {
        parent: parent.clone(),
        segment,
        shape: shape_name(value),
    }
}

fn shape_name(value: Peek<'_, 'static>) -> String {
    cloud_terrastodon_registry::describe_shape(value.shape()).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::preorder_cursor::PreorderCursor;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static BREADCRUMB_EVALUATIONS: AtomicUsize = AtomicUsize::new(0);
    static BREADCRUMB_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    enum TestBreadcrumb {
        Pop,
    }

    #[derive(Clone, Debug, Default, facet::Facet)]
    #[repr(C)]
    struct TestBreadcrumbs {
        operations: Vec<TestBreadcrumb>,
    }

    impl TestBreadcrumbs {
        fn evaluate(&self) {
            BREADCRUMB_EVALUATIONS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct TestTab {
        name: String,
        breadcrumbs: TestBreadcrumbs,
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct MyThing {
        age: usize,
        name: String,
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct TestPermission {
        display_name: String,
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct TestMember {
        permission_objects: Vec<TestPermission>,
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[facet(transparent)]
    struct TransparentNames(Vec<String>);

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct AddressFixture {
        members: Vec<TestMember>,
        pair: (usize, String),
        fixed: [u16; 2],
        scores: BTreeMap<String, usize>,
        names: TransparentNames,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: facet::Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn test_tab() -> TestTab {
        TestTab {
            name: "admins".to_owned(),
            breadcrumbs: TestBreadcrumbs {
                operations: vec![TestBreadcrumb::Pop],
            },
        }
    }

    fn address_fixture() -> AddressFixture {
        AddressFixture {
            members: vec![TestMember {
                permission_objects: vec![
                    TestPermission {
                        display_name: "Readers".to_owned(),
                    },
                    TestPermission {
                        display_name: "Project Administrators".to_owned(),
                    },
                ],
            }],
            pair: (7, "tuple value".to_owned()),
            fixed: [11, 13],
            scores: BTreeMap::from([("admins".to_owned(), 42)]),
            names: TransparentNames(vec!["Ada".to_owned(), "Grace".to_owned()]),
        }
    }

    #[test]
    fn my_thing_fields_are_addresses_not_arena_slots() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(MyThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let allocated = arena.allocated_slot_count();
        let source = ArenaAddressSource::new(&arena);
        let age = ValueAddress::root(root).child(ValuePathSegment::Field("age".to_owned()));
        let name = ValueAddress::root(root).child(ValuePathSegment::Field("name".to_owned()));

        assert_eq!(
            *source.resolve(&age).unwrap().peek().get::<usize>().unwrap(),
            42
        );
        assert_eq!(source.resolve(&name).unwrap().peek().as_str(), Some("Ada"));
        assert_eq!(arena.allocated_slot_count(), allocated);
        assert_eq!(allocated, 1, "reflected children never allocate SlotIds");
    }

    #[test]
    fn typed_resolution_handles_nested_collections_and_transparent_wrappers() {
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(address_fixture())).unwrap();
        let source = ArenaAddressSource::new(&arena);
        let field = |name: &str| ValuePathSegment::Field(name.to_owned());

        let permission_name = ValueAddress::root(root)
            .child(field("members"))
            .child(ValuePathSegment::Index(0))
            .child(field("permission_objects"))
            .child(ValuePathSegment::Index(1))
            .child(field("display_name"));
        assert_eq!(
            source.resolve(&permission_name).unwrap().peek().as_str(),
            Some("Project Administrators")
        );

        let tuple_value = ValueAddress::root(root)
            .child(field("pair"))
            .child(ValuePathSegment::Index(1));
        assert_eq!(
            source.resolve(&tuple_value).unwrap().peek().as_str(),
            Some("tuple value")
        );

        let array_value = ValueAddress::root(root)
            .child(field("fixed"))
            .child(ValuePathSegment::Index(1));
        assert_eq!(
            *source
                .resolve(&array_value)
                .unwrap()
                .peek()
                .get::<u16>()
                .unwrap(),
            13
        );

        let map_value = ValueAddress::root(root)
            .child(field("scores"))
            .child(ValuePathSegment::Key("admins".to_owned()));
        assert_eq!(
            *source
                .resolve(&map_value)
                .unwrap()
                .peek()
                .get::<usize>()
                .unwrap(),
            42
        );

        let transparent_value = ValueAddress::root(root)
            .child(field("names"))
            .child(ValuePathSegment::Index(1));
        assert_eq!(
            source.resolve(&transparent_value).unwrap().peek().as_str(),
            Some("Grace")
        );
    }

    #[test]
    fn resolution_failures_preserve_the_specific_address_cause() {
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(address_fixture())).unwrap();
        let building = arena.reserve_builder().unwrap();
        let source = ArenaAddressSource::new(&arena);
        let field = |name: &str| ValuePathSegment::Field(name.to_owned());

        assert!(matches!(
            source.resolve(&ValueAddress::root(SlotId::new(10_000))),
            Err(ValueResolutionError::Root(
                super::super::arena::ArenaError::UnknownSlot(_)
            ))
        ));
        assert!(matches!(
            source.resolve(&ValueAddress::root(building)),
            Err(ValueResolutionError::Root(
                super::super::arena::ArenaError::RootNotReady {
                    state: "Building",
                    ..
                }
            ))
        ));

        let missing_field = ValueAddress::root(root).child(field("unknown"));
        assert!(matches!(
            source.resolve(&missing_field),
            Err(ValueResolutionError::MissingField {
                field,
                ..
            }) if field == "unknown"
        ));

        let out_of_range = ValueAddress::root(root)
            .child(field("fixed"))
            .child(ValuePathSegment::Index(9));
        assert!(matches!(
            source.resolve(&out_of_range),
            Err(ValueResolutionError::IndexOutOfRange {
                index: 9,
                len: 2,
                ..
            })
        ));

        let missing_key = ValueAddress::root(root)
            .child(field("scores"))
            .child(ValuePathSegment::Key("missing".to_owned()));
        assert!(matches!(
            source.resolve(&missing_key),
            Err(ValueResolutionError::MissingKey {
                key,
                ..
            }) if key == "missing"
        ));

        let scalar_child = ValueAddress::root(root)
            .child(field("pair"))
            .child(ValuePathSegment::Index(0))
            .child(ValuePathSegment::Index(0));
        assert!(matches!(
            source.resolve(&scalar_child),
            Err(ValueResolutionError::SegmentNotSupported {
                segment: ValuePathSegment::Index(0),
                ..
            })
        ));
    }

    #[test]
    fn observed_root_revision_detects_stale_reads_without_changing_address_identity() {
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(address_fixture())).unwrap();
        let address = ValueAddress::root(root).child(ValuePathSegment::Field("members".to_owned()));
        let observed = ArenaAddressSource::new(&arena)
            .resolve(&address)
            .unwrap()
            .root_revision();

        arena.delete(root).unwrap();
        let source = ArenaAddressSource::new(&arena);
        assert!(matches!(
            source.resolve_at_revision(&address, observed),
            Err(ValueResolutionError::StaleRootRevision {
                root: stale_root,
                expected,
                actual,
            }) if stale_root == root && expected == observed && actual != observed
        ));
        assert!(matches!(
            source.resolve(&address),
            Err(ValueResolutionError::Root(
                super::super::arena::ArenaError::Tombstone { slot, .. }
            )) if slot == root
        ));
        assert_eq!(
            address,
            ValueAddress::root(root).child(ValuePathSegment::Field("members".to_owned()))
        );
    }

    #[test]
    fn everything_includes_tab_and_breadcrumb_projections() {
        let mut arena = Arena::default();
        let tab = arena.insert_ready(runtime(test_tab())).unwrap();
        let string = arena
            .insert_ready(runtime(String::from("ordinary value")))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let addresses = PreorderCursor::new(&source).collect::<Vec<_>>();

        assert_eq!(addresses[0], ValueAddress::root(tab));
        assert!(
            addresses.contains(
                &ValueAddress::root(tab).child(ValuePathSegment::Field("name".to_owned()))
            )
        );
        assert!(
            addresses.contains(
                &ValueAddress::root(tab)
                    .child(ValuePathSegment::Field("breadcrumbs".to_owned()))
                    .child(ValuePathSegment::Field("operations".to_owned()))
                    .child(ValuePathSegment::Index(0))
            )
        );
        assert!(addresses.contains(&ValueAddress::root(string)));
    }

    #[test]
    fn encountering_breadcrumbs_does_not_evaluate_them() {
        let _guard = BREADCRUMB_TEST_LOCK.lock().unwrap();
        BREADCRUMB_EVALUATIONS.store(0, Ordering::SeqCst);
        let mut arena = Arena::default();
        arena.insert_ready(runtime(test_tab())).unwrap();
        let source = ArenaAddressSource::new(&arena);

        let _observed = PreorderCursor::new(&source).collect::<Vec<_>>();

        assert_eq!(BREADCRUMB_EVALUATIONS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn tab_query_can_select_its_own_tab_without_recursion() {
        let _guard = BREADCRUMB_TEST_LOCK.lock().unwrap();
        BREADCRUMB_EVALUATIONS.store(0, Ordering::SeqCst);
        let mut arena = Arena::default();
        let tab = arena.insert_ready(runtime(test_tab())).unwrap();
        let source = ArenaAddressSource::new(&arena);

        let tab_matches = PreorderCursor::new(&source)
            .filter(|address| {
                source
                    .resolve(address)
                    .is_ok_and(|value| value.shape().is_shape(TestTab::SHAPE))
            })
            .collect::<Vec<_>>();

        assert_eq!(tab_matches, [ValueAddress::root(tab)]);
        assert_eq!(BREADCRUMB_EVALUATIONS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn actual_hashmap_order_matches_reflected_iteration_for_root_revision() {
        let value = HashMap::from([
            ("zeta".to_owned(), 1_usize),
            ("alpha".to_owned(), 2_usize),
            ("middle".to_owned(), 3_usize),
        ]);
        let mut arena = Arena::default();
        let root = arena.insert_ready(runtime(value)).unwrap();
        let expected = arena
            .ready_value(root)
            .unwrap()
            .peek()
            .into_map()
            .unwrap()
            .iter()
            .map(|(key, _)| key.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let source = ArenaAddressSource::new(&arena);

        let actual = PreorderCursor::new(&source)
            .skip(1)
            .map(|address| match address.path().segments() {
                [ValuePathSegment::Key(key)] => key.clone(),
                path => panic!("expected one map key segment, got {path:?}"),
            })
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
    }

    #[test]
    fn million_value_first_window_does_not_materialize_all_paths() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);

        let first_window = PreorderCursor::new(&source).take(9).collect::<Vec<_>>();

        assert_eq!(first_window[0], ValueAddress::root(root));
        assert_eq!(first_window.len(), 9);
        for (index, address) in first_window[1..].iter().enumerate() {
            assert_eq!(address.path().segments(), [ValuePathSegment::Index(index)]);
        }
    }

    #[test]
    fn breadcrumb_evaluation_is_explicit() {
        let _guard = BREADCRUMB_TEST_LOCK.lock().unwrap();
        BREADCRUMB_EVALUATIONS.store(0, Ordering::SeqCst);
        let breadcrumbs = TestBreadcrumbs::default();

        breadcrumbs.evaluate();

        assert_eq!(BREADCRUMB_EVALUATIONS.load(Ordering::SeqCst), 1);
    }
}
