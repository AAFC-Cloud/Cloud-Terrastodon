use cloud_terrastodon_azure_devops::{
    AzureDevOpsDescriptor, AzureDevOpsProjectMember, AzureDevOpsProjectMemberListRequest,
    AzureDevOpsProjectPermissionObject,
};
use cloud_terrastodon_registry::{
    ArbitraryBytes, RuntimeValue, functions_from, known_thing_for_shape,
};
use facet::Facet;
use tokio::io::{AsyncReadExt, duplex};

use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::arena_query_context::ArenaQueryContext;
use super::borrow_graph::BorrowGraph;
use super::breadcrumb::{Breadcrumb, ValueFilterOperator};
use super::breadcrumbs::Breadcrumbs;
use super::card_address::CardAddress;
use super::explorer_engine::ExplorerEngine;
use super::field_binding::FieldBinding;
use super::field_candidate_action::{FieldCandidateAction, FieldCandidateActions};
use super::invocation_controller::{InvocationController, InvocationEventState, InvocationStart};
use super::invocation_host::FakeInvocationHost;
use super::invocation_mode::InvocationMode;
use super::preorder_cursor::PreorderCursor;
use super::produce_json_request::ProduceJsonRequest;
use super::production_controller::arbitrary_constructor_for;
use super::query_cursor::QueryCursor;
use super::query_plan::QueryPlan;
use super::query_progress::QueryProgressState;
use super::revision::{QueryRevision, ScanRevisionStamp};
use super::selection::CardSelection;
use super::tab::Tab;
use super::value_address::ValueAddress;
use super::value_builder::{BuilderStore, BuilderTransition, ValueBuilder};
use super::value_candidate::scan_value_candidates;
use super::value_path::ValuePathSegment;
use super::work_budget::WorkBudget;

fn runtime<T>(value: T) -> RuntimeValue
where
    T: Facet<'static> + Send + 'static,
{
    RuntimeValue::from_box(Box::new(value)).expect("acceptance fixture is representable")
}

fn permission(
    member: &str,
    index: usize,
    display_name: &str,
) -> AzureDevOpsProjectPermissionObject {
    AzureDevOpsProjectPermissionObject {
        descriptor: AzureDevOpsDescriptor::Other(format!("{member}-permission-{index}")),
        display_name: display_name.to_owned(),
        principal_name: format!("{member}-permission-{index}@example.invalid"),
        origin: "acceptance".to_owned(),
        origin_id: format!("{member}-permission-origin-{index}"),
        subject_kind: "group".to_owned(),
    }
}

fn member(name: &str, roles: &[&str]) -> AzureDevOpsProjectMember {
    let slug = name.to_ascii_lowercase();
    AzureDevOpsProjectMember {
        descriptor: AzureDevOpsDescriptor::Other(format!("{slug}-descriptor")),
        display_name: name.to_owned(),
        principal_name: format!("{slug}@example.invalid"),
        mail_address: Some(format!("{slug}@example.invalid")),
        origin: "acceptance".to_owned(),
        origin_id: format!("{slug}-origin"),
        subject_kind: "user".to_owned(),
        permission_objects: roles
            .iter()
            .enumerate()
            .map(|(index, role)| permission(&slug, index, role))
            .collect(),
    }
}

#[tokio::test]
async fn project_admin_query_composes_and_exports_without_azure() {
    type MemberRequest = AzureDevOpsProjectMemberListRequest<'static>;

    let mut arena = Arena::default();
    let mut builders = BuilderStore::default();
    let mut borrows = BorrowGraph::default();
    let mut invocations = InvocationController::default();
    let mut host = FakeInvocationHost::default();

    // Construct the real request through its registered ArbitraryBytes
    // constructor. Both the bytes and produced request receive ordinary arena
    // identities, matching the production picker pipeline.
    let bytes = (0_u8..=255).cycle().take(4096).collect::<Vec<_>>();
    let bytes_slot = arena
        .insert_ready(runtime(ArbitraryBytes::new(bytes)))
        .unwrap();
    let constructor = arbitrary_constructor_for(MemberRequest::SHAPE)
        .expect("Azure request exposes registered ArbitraryBytes production");
    let bytes_thing =
        known_thing_for_shape(ArbitraryBytes::SHAPE).expect("ArbitraryBytes is registered");
    let request_slot = match invocations
        .invoke(
            &mut arena,
            &mut builders,
            &mut borrows,
            &mut host,
            bytes_slot,
            bytes_thing,
            constructor,
            InvocationMode::Consume,
        )
        .unwrap()
    {
        InvocationStart::Ready { output } => output,
        state => panic!("arbitrary construction must complete synchronously, got {state:?}"),
    };

    let fixture = vec![
        member("Ada", &["Project Administrators", "Project Administrators"]),
        member("Ben", &["Readers"]),
        member("Cy", &["Contributors", "Project Administrators"]),
    ];
    let request_thing =
        known_thing_for_shape(MemberRequest::SHAPE).expect("member request is registered");
    let member_function = functions_from(MemberRequest::SHAPE)
        .into_iter()
        .find(|function| {
            function
                .output_shape
                .is_shape(Vec::<AzureDevOpsProjectMember>::SHAPE)
        })
        .expect("member request IntoFuture is registered");
    let (invocation, members_slot) = match invocations
        .invoke(
            &mut arena,
            &mut builders,
            &mut borrows,
            &mut host,
            request_slot,
            request_thing,
            member_function,
            InvocationMode::Consume,
        )
        .unwrap()
    {
        InvocationStart::Pending { invocation, output } => (invocation, output),
        state => panic!("fake Azure invocation should be pending, got {state:?}"),
    };
    host.complete(invocation, fixture);
    let events = invocations.poll(&mut arena, &mut builders, &mut borrows, &mut host);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].state, InvocationEventState::Ready);
    assert_eq!(events[0].output, members_slot);

    let breadcrumbs = Breadcrumbs::new(vec![
        Breadcrumb::ShapeFilter {
            included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                AzureDevOpsProjectPermissionObject::SHAPE,
            )],
        },
        Breadcrumb::ValueFilter {
            field_shape: cloud_terrastodon_registry::describe_shape(String::SHAPE),
            field_name: "displayName".to_owned(),
            operator: ValueFilterOperator::Equals,
            value: "Project Administrators".to_owned(),
        },
        Breadcrumb::Pop,
        Breadcrumb::Pop,
    ]);
    let expected_addresses = vec![
        ValueAddress::root(members_slot).child(ValuePathSegment::Index(0)),
        ValueAddress::root(members_slot).child(ValuePathSegment::Index(2)),
    ];
    let source = ArenaAddressSource::new(&arena);
    let mut evaluated = QueryPlan::new(breadcrumbs.clone()).evaluate(&source);
    assert_eq!(evaluated.by_ref().collect::<Vec<_>>(), expected_addresses);
    assert!(
        evaluated.inspected() < 64,
        "the cursor retains and emits addresses instead of cloning members"
    );
    drop(evaluated);
    drop(source);

    let tab_slot = arena
        .insert_ready(runtime(Tab::new("project admins", breadcrumbs)))
        .unwrap();
    let tab_breadcrumbs =
        ValueAddress::root(tab_slot).child(ValuePathSegment::Field("breadcrumbs".to_owned()));
    let allocated_before_picker = arena.allocated_slot_count();
    let candidate = {
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = PreorderCursor::new(&source);
        let mut found = None;
        while found.is_none() {
            let batch =
                scan_value_candidates(&mut cursor, &source, Breadcrumbs::SHAPE, WorkBudget::new(8));
            found = batch
                .candidates
                .into_iter()
                .find(|candidate| candidate.address() == &tab_breadcrumbs);
            assert!(
                found.is_some() || !batch.complete,
                "the generic picker must discover Tab.breadcrumbs"
            );
        }
        found.unwrap()
    };
    assert!(
        candidate
            .display_label()
            .contains("field breadcrumbs of slot")
    );
    assert!(candidate.display_label().contains("(Tab)"));
    assert_eq!(
        arena.allocated_slot_count(),
        allocated_before_picker,
        "projected picker candidates never allocate view slots"
    );

    let export_slot = arena.reserve_builder().unwrap();
    assert_eq!(
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                export_slot,
                ValueBuilder::new(ProduceJsonRequest::SHAPE),
            )
            .unwrap(),
        BuilderTransition::Building
    );
    let actions = FieldCandidateActions::inspect(
        &arena,
        &builders,
        &borrows,
        export_slot,
        0,
        candidate.address().clone(),
    )
    .unwrap();
    assert_eq!(actions.candidate().address(), &tab_breadcrumbs);
    assert_eq!(
        actions
            .consequences()
            .iter()
            .map(|consequence| consequence.action())
            .collect::<Vec<_>>(),
        [FieldCandidateAction::Clone]
    );
    assert_eq!(
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                export_slot,
                0,
                FieldBinding::CloneFrom(tab_breadcrumbs),
            )
            .unwrap(),
        BuilderTransition::Building
    );
    assert_eq!(
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                export_slot,
                1,
                FieldBinding::InlineOwned(runtime(String::from("project-admins.json"))),
            )
            .unwrap(),
        BuilderTransition::Ready
    );
    let export = arena
        .ready_value(export_slot)
        .unwrap()
        .try_clone()
        .unwrap()
        .into_box::<ProduceJsonRequest>()
        .unwrap()
        .downcast::<ProduceJsonRequest>()
        .unwrap();

    let engine = ExplorerEngine::new(arena);
    let (context, inbox) = ArenaQueryContext::channel(8);
    let client = async move {
        let (mut writer, mut reader) = duplex(64 * 1024);
        let write = async move {
            let path = export
                .write_to_sink(context, &mut writer)
                .await
                .expect("coherent in-memory export succeeds");
            drop(writer);
            path
        };
        let read = async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.unwrap();
            String::from_utf8(bytes).unwrap()
        };
        let (path, json) = tokio::join!(write, read);
        assert_eq!(path, "project-admins.json");
        let decoded: Vec<AzureDevOpsProjectMember> =
            facet_json::from_str(&json).expect("export is one valid JSON array");
        assert_eq!(
            decoded
                .iter()
                .map(|member| member.display_name.as_str())
                .collect::<Vec<_>>(),
            ["Ada", "Cy"]
        );
        json
    };
    let (engine, json) = tokio::join!(engine.run(inbox), client);

    assert!(json.contains("\"Ada\""));
    assert!(json.contains("\"Cy\""));
    assert!(!json.contains("\"Ben\""));
    assert_eq!(engine.json_serialization_count(), 2);
    assert_eq!(engine.arena().allocated_slot_count(), 5);
}

#[test]
fn million_value_navigation_filter_and_selection_are_counter_bounded() {
    let mut arena = Arena::default();
    let large = arena
        .insert_ready(runtime((0_u32..1_000_000).collect::<Vec<_>>()))
        .unwrap();
    let selected = arena
        .insert_ready(runtime(String::from("stable selection")))
        .unwrap();
    let stamp = ScanRevisionStamp {
        arena: arena.arena_revision(),
        query: QueryRevision::default(),
    };
    let source = ArenaAddressSource::new(&arena);

    let mut first_window = QueryCursor::new(
        &source,
        QueryPlan::new(Breadcrumbs::default()),
        stamp,
        NonZeroUsize::new(9).unwrap(),
    );
    let mut first_budget = WorkBudget::new(16);
    let first = first_window
        .fill_window(
            None,
            NonZeroUsize::new(8).unwrap(),
            stamp,
            &mut first_budget,
        )
        .unwrap();
    assert_eq!(first.work_spent(), 9, "eight cards plus one lookahead");
    assert_eq!(first.instrumentation().cached, 9);
    assert!(matches!(first.state(), QueryProgressState::Ready(_)));

    let no_match = Breadcrumbs::new(vec![
        Breadcrumb::ShapeFilter {
            included_shapes: vec![cloud_terrastodon_registry::describe_shape(u32::SHAPE)],
        },
        Breadcrumb::ValueFilter {
            field_shape: "*".to_owned(),
            field_name: "field_that_does_not_exist".to_owned(),
            operator: ValueFilterOperator::Equals,
            value: "never".to_owned(),
        },
    ]);
    let mut filtered = QueryCursor::new(
        &source,
        QueryPlan::new(no_match),
        stamp,
        NonZeroUsize::new(16).unwrap(),
    );
    let mut seven = WorkBudget::new(7);
    let first_scan = filtered.next(stamp, &mut seven);
    assert_eq!(first_scan.state(), &QueryProgressState::Pending);
    assert_eq!(first_scan.work_spent(), 7);
    let mut eleven = WorkBudget::new(11);
    let second_scan = filtered.next(stamp, &mut eleven);
    assert_eq!(second_scan.state(), &QueryProgressState::Pending);
    assert_eq!(second_scan.work_spent(), 11);
    assert_eq!(second_scan.instrumentation().addressed, 18);

    let selected_address = ValueAddress::root(selected);
    let mut selection = CardSelection::new(CardAddress::Value(selected_address.clone()));
    selection.reconcile(&source);
    assert_eq!(
        selection.selected(),
        &CardAddress::Value(selected_address.clone())
    );
    drop(filtered);
    drop(first_window);
    drop(source);

    // Grow the complete projection set before the selected root. The logical
    // address remains stable even though any flattened card ordinal would
    // shift.
    let previous = arena
        .replace_ready(large, runtime((0_u32..1_000_001).collect::<Vec<_>>()))
        .unwrap();
    drop(previous);
    let source = ArenaAddressSource::new(&arena);
    selection.reconcile(&source);
    assert_eq!(selection.selected(), &CardAddress::Value(selected_address));
    assert_eq!(arena.allocated_slot_count(), 2);
}
use std::num::NonZeroUsize;
