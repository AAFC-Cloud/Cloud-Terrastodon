//! Lifetime regressions for engine destruction, using real borrowed Cows.
//!
//! Drop probes own their event log independently of the reflected source. They
//! never inspect a Cow pointee during destruction, so a wrong drop order produces
//! an ordinary assertion failure rather than a dangling-pointer dereference.

use super::*;
use crate::object_explorer::explorer_command::OwnedValuePacket;
use crate::object_explorer::field_binding::FieldBinding;
use crate::object_explorer::invocation_controller::InvocationStart;
use crate::object_explorer::invocation_host::FakeInvocationHost;
use crate::object_explorer::invocation_host::InvocationId;
use crate::object_explorer::invocation_mode::InvocationMode;
use crate::object_explorer::slot_id::SlotId;
use crate::object_explorer::value_address::ValueAddress;
use crate::object_explorer::value_builder::BuilderTransition;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::InvocationFuture;
use cloud_terrastodon_registry::RegistrationSite;
use cloud_terrastodon_registry::Thing;
use cloud_terrastodon_registry::runtime_from_boxed;
use cloud_terrastodon_registry::runtime_into_boxed;
use std::any::Any;
use std::borrow::Cow;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::Mutex;

type Events = Arc<Mutex<Vec<&'static str>>>;

#[derive(Clone, Debug, Facet)]
#[facet(opaque)]
struct DropProbe {
    name: &'static str,
    events: Events,
    panic_on_drop: bool,
}

impl DropProbe {
    fn new(events: &Events, name: &'static str) -> Self {
        Self {
            name,
            events: Arc::clone(events),
            panic_on_drop: false,
        }
    }
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        // Release the mutex before the deliberate panic; the event log must
        // remain usable by the outer catch_unwind assertion.
        self.events.lock().unwrap().push(self.name);
        if self.panic_on_drop {
            panic!("deliberate lifetime regression destructor panic");
        }
    }
}

#[derive(Clone, Debug, Facet)]
#[facet(traits(Clone))]
struct Source {
    probe: DropProbe,
}

#[derive(Clone, Debug, Facet)]
#[facet(traits(Clone))]
struct BorrowRecord {
    source: Cow<'static, Source>,
    probe: DropProbe,
}

#[derive(Clone, Debug, Facet)]
struct OuterRecord {
    middle: Cow<'static, BorrowRecord>,
    probe: DropProbe,
}

#[derive(Debug, Facet)]
struct IncompleteRecord {
    cloned: BorrowRecord,
    source: Cow<'static, Source>,
    unfinished: String,
}

fn events() -> Events {
    Arc::new(Mutex::new(Vec::new()))
}

fn observed(events: &Events) -> Vec<&'static str> {
    events.lock().unwrap().clone()
}

fn runtime<T: Facet<'static> + Send + 'static>(value: T) -> RuntimeValue {
    RuntimeValue::from_box(Box::new(value)).expect("fixture has a reflected runtime representation")
}

fn reserve<T: Facet<'static>>(engine: &mut ExplorerEngine) -> SlotId {
    let (slot, transition) = engine
        .builders
        .create_and_finalize(&mut engine.arena, &mut engine.borrow_graph, T::SHAPE)
        .unwrap();
    assert_eq!(transition, BuilderTransition::Building);
    slot
}

fn set_field(
    engine: &mut ExplorerEngine,
    slot: SlotId,
    index: usize,
    binding: FieldBinding,
) -> BuilderTransition {
    engine
        .builders
        .set_field_and_finalize(
            &mut engine.arena,
            &mut engine.borrow_graph,
            slot,
            index,
            binding,
        )
        .unwrap()
}

fn insert_source(engine: &mut ExplorerEngine, events: &Events) -> SlotId {
    engine
        .arena
        .insert_ready(runtime(Source {
            probe: DropProbe::new(events, "source"),
        }))
        .unwrap()
}

fn finish_borrower(
    engine: &mut ExplorerEngine,
    borrower: SlotId,
    source: SlotId,
    probe: DropProbe,
) {
    assert_eq!(
        set_field(
            engine,
            borrower,
            0,
            FieldBinding::BorrowFrom(ValueAddress::root(source)),
        ),
        BuilderTransition::Building
    );
    assert_eq!(
        set_field(
            engine,
            borrower,
            1,
            FieldBinding::InlineOwned(runtime(probe))
        ),
        BuilderTransition::Ready
    );
}

fn borrower_fixture(engine: &mut ExplorerEngine, events: &Events) -> (SlotId, SlotId) {
    let source = insert_source(engine, events);
    let borrower = reserve::<BorrowRecord>(engine);
    finish_borrower(engine, borrower, source, DropProbe::new(events, "request"));
    (source, borrower)
}

fn hold_request(input: Box<dyn Any + Send>) -> InvocationFuture {
    Box::pin(async move {
        let request = input.downcast::<BorrowRecord>().unwrap();
        request.probe.events.lock().unwrap().push("polled");
        std::future::pending::<()>().await;
        Ok(request as Box<dyn Any + Send>)
    })
}

fn return_request(input: Box<dyn Any + Send>) -> InvocationFuture {
    Box::pin(async move { Ok(input) })
}

static BORROW_THING: Thing = Thing::value(
    BorrowRecord::SHAPE,
    runtime_from_boxed::<BorrowRecord>,
    runtime_into_boxed::<BorrowRecord>,
    RegistrationSite::new(file!(), line!()),
);

static HOLD_REQUEST: Function = Function::async_value(
    BorrowRecord::SHAPE,
    BorrowRecord::SHAPE,
    FunctionKind::AsyncInvoke,
    "hold",
    "lifetime regression",
    &[],
    hold_request,
    runtime_from_boxed::<BorrowRecord>,
    RegistrationSite::new(file!(), line!()),
);

static RETURN_REQUEST: Function = Function::async_value(
    BorrowRecord::SHAPE,
    BorrowRecord::SHAPE,
    FunctionKind::AsyncInvoke,
    "return",
    "lifetime regression",
    &[],
    return_request,
    runtime_from_boxed::<BorrowRecord>,
    RegistrationSite::new(file!(), line!()),
);

fn invoke(
    engine: &mut ExplorerEngine,
    request: SlotId,
    function: &'static Function,
) -> InvocationId {
    let InvocationStart::Pending { invocation, .. } = engine
        .invocations
        .invoke(
            &mut engine.arena,
            &mut engine.builders,
            &mut engine.borrow_graph,
            engine.invocation_host.as_mut(),
            request,
            &BORROW_THING,
            function,
            InvocationMode::Consume,
        )
        .unwrap()
    else {
        panic!("fixture should start a host-owned asynchronous request");
    };
    invocation
}

#[test]
fn shutdown_drops_nested_borrowers_topologically_not_by_slot_id() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    // Reserve the middle before inserting its source. Neither ascending nor
    // descending slot order is a valid destruction order for this graph.
    let middle = reserve::<BorrowRecord>(&mut engine);
    let source = insert_source(&mut engine, &events);
    let outer = reserve::<OuterRecord>(&mut engine);
    assert!(middle < source && source < outer);
    finish_borrower(
        &mut engine,
        middle,
        source,
        DropProbe::new(&events, "middle"),
    );
    finish_borrower(&mut engine, outer, middle, DropProbe::new(&events, "outer"));
    assert!(engine.borrow_graph.protects_root(source));
    assert!(engine.borrow_graph.protects_root(middle));
    assert!(observed(&events).is_empty());

    drop(engine);

    assert_eq!(observed(&events), ["outer", "middle", "source"]);
}

#[test]
fn shutdown_drops_fake_host_held_request_before_its_source() {
    let events = events();
    let mut engine =
        ExplorerEngine::empty_with_invocation_host(Box::new(FakeInvocationHost::default()));
    let (source, request) = borrower_fixture(&mut engine, &events);
    invoke(&mut engine, request, &HOLD_REQUEST);
    assert!(engine.borrow_graph.protects_root(source));
    assert!(observed(&events).is_empty());

    drop(engine);

    assert_eq!(observed(&events), ["request", "source"]);
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_synchronously_drops_tokio_request_before_first_poll() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let (_, request) = borrower_fixture(&mut engine, &events);
    invoke(&mut engine, request, &HOLD_REQUEST);

    // No yield: aborting a Tokio task does not synchronously destroy this
    // future. Engine destruction must not rely on another scheduler turn.
    drop(engine);

    assert_eq!(observed(&events), ["request", "source"]);
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_synchronously_drops_tokio_request_after_pending_poll() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let (_, request) = borrower_fixture(&mut engine, &events);
    invoke(&mut engine, request, &HOLD_REQUEST);
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while !observed(&events).contains(&"polled") {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the host should poll the held request");

    drop(engine);

    assert_eq!(observed(&events), ["polled", "request", "source"]);
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_drops_completed_unclaimed_cow_output_before_source() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let (source, request) = borrower_fixture(&mut engine, &events);
    let invocation = invoke(&mut engine, request, &RETURN_REQUEST);
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while !engine.invocation_host.is_ready(invocation) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the returned borrowed record should be waiting in the host");
    assert_eq!(engine.invocations.pending_count(), 1);
    assert!(engine.borrow_graph.protects_root(source));
    assert!(observed(&events).is_empty());

    // Deliberately do not poll InvocationController: the host still owns the
    // output, which contains exactly the same borrowed Cow as the input.
    drop(engine);

    assert_eq!(observed(&events), ["request", "source"]);
}

#[test]
fn shutdown_drops_incomplete_builder_borrowed_clone_before_ready_sources() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let (source, request) = borrower_fixture(&mut engine, &events);
    let mut cloned = engine
        .arena
        .ready_value(request)
        .unwrap()
        .peek()
        .get::<BorrowRecord>()
        .unwrap()
        .clone();
    cloned.probe.name = "builder_clone";
    assert!(matches!(cloned.source, Cow::Borrowed(_)));
    let incomplete = reserve::<IncompleteRecord>(&mut engine);
    // Establish independent builder protection for the source before storing
    // the shallow clone. The third field stays unset, so InlineOwned really
    // resides in BuilderStore rather than being finalized into the arena.
    assert_eq!(
        set_field(
            &mut engine,
            incomplete,
            1,
            FieldBinding::BorrowFrom(ValueAddress::root(source)),
        ),
        BuilderTransition::Building
    );
    assert_eq!(
        set_field(
            &mut engine,
            incomplete,
            0,
            FieldBinding::InlineOwned(runtime(cloned)),
        ),
        BuilderTransition::Building
    );
    assert!(observed(&events).is_empty());

    drop(engine);

    assert_eq!(observed(&events), ["builder_clone", "request", "source"]);
}

#[test]
fn shutdown_discards_deferred_packets_before_ready_borrow_sources() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let (_, request) = borrower_fixture(&mut engine, &events);
    let mut queued = engine
        .arena
        .ready_value(request)
        .unwrap()
        .peek()
        .get::<BorrowRecord>()
        .unwrap()
        .clone();
    queued.probe.name = "queued";
    assert!(matches!(queued.source, Cow::Borrowed(_)));

    // This is a defense-in-depth fixture, not a claim that current production
    // ingestion permits escaping arena borrows into packets. Deliberately put
    // an internal shallow clone in the deferred queue while the original ready
    // request retains its source lease. Shutdown must discard the packet before
    // releasing that original owner and source, even though the queue is not a
    // BuilderStore or InvocationHost.
    engine.export_barrier.begin(QuerySessionId::new(1)).unwrap();
    let (response, _receiver) = oneshot::channel();
    assert!(matches!(
        engine
            .export_barrier
            .submit(ArenaMutationCommand::InsertReady {
                value: OwnedValuePacket::new(queued),
                response,
            }),
        MutationSubmission::Deferred
    ));
    assert!(observed(&events).is_empty());

    drop(engine);

    assert_eq!(observed(&events), ["queued", "request", "source"]);
}

#[test]
fn shutdown_destructor_panic_retains_remaining_borrow_sources() {
    let events = events();
    let mut engine = ExplorerEngine::empty();
    let source = insert_source(&mut engine, &events);
    let borrower = reserve::<BorrowRecord>(&mut engine);
    let mut probe = DropProbe::new(&events, "panic_borrower");
    probe.panic_on_drop = true;
    finish_borrower(&mut engine, borrower, source, probe);
    assert!(observed(&events).is_empty());

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| drop(engine)));

    assert!(
        result.is_err(),
        "the deliberately panicking destructor propagates"
    );
    assert_eq!(
        observed(&events),
        ["panic_borrower"],
        "remaining source ownership must fail closed instead of unwinding in arena-map order"
    );
}
