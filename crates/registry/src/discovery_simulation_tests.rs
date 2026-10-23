use crate::ArbitraryBytes;
use crate::CancellationScope;
use crate::DiscoverySimulation;
use crate::Effect;
use crate::Function;
use crate::FunctionExecutor;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::InvocationFuture;
use crate::RegistrationSite;
use crate::RuntimeValue;
use crate::runtime_from_boxed;
use arbitrary::Arbitrary;
use facet::Facet;
use std::any::Any;
use std::cell::Cell;
use std::cell::RefCell;
use std::future::IntoFuture;
use std::future::Ready;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationToken;

thread_local! {
    static RESPONSE_CANCELLATION: RefCell<Option<CancellationScope>> = const { RefCell::new(None) };
    static CONVERSION_INVOCATIONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Debug, Facet)]
struct CancelledResponse;

fn cancel_response_generation(_: &mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
    RESPONSE_CANCELLATION.with(|scope| {
        scope
            .borrow()
            .as_ref()
            .expect("generation runs within the test's bounded scope")
            .request_cancel("cancelled during synthetic response generation");
    });
    eyre::bail!("synthetic response generation failed after requesting cancellation");
}

#[derive(Clone, Debug, Facet)]
struct CatalogSeed {
    marker: u8,
}

#[derive(Clone, Debug, Facet)]
struct LateDependency;

#[derive(Clone, Debug, Facet)]
struct RepeatedFieldsSeed {
    first: String,
    second: String,
    dependency: LateDependency,
}

#[derive(Clone, Debug, Facet)]
struct CatalogRequest {
    seed: CatalogSeed,
}

impl<'a> Arbitrary<'a> for CatalogRequest {
    fn arbitrary(_: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        panic!("request Arbitrary must not bypass its missing dependencies")
    }
}

#[derive(Arbitrary, Clone, Debug, Facet)]
struct CatalogItem {
    marker: u8,
}

#[derive(Arbitrary, Clone, Debug, Facet)]
struct ItemArgument {
    marker: u8,
}

impl From<CatalogItem> for ItemArgument {
    fn from(value: CatalogItem) -> Self {
        Self {
            marker: value.marker.wrapping_add(1),
        }
    }
}

impl From<ItemArgument> for CatalogItem {
    fn from(value: ItemArgument) -> Self {
        Self {
            marker: value.marker.wrapping_sub(1),
        }
    }
}

#[derive(Clone, Debug, Facet)]
struct ItemDetailsRequest {
    argument: ItemArgument,
}

#[derive(Arbitrary, Clone, Debug, Facet)]
struct ItemDetails {
    marker: u8,
}

#[derive(Arbitrary, Clone, Debug, Facet)]
struct CatalogRequestEnvelope {
    request: CatalogRequest,
}

#[derive(Clone, Debug, Facet)]
struct CatalogRequestEnvelopeRequest {
    seed: CatalogSeed,
}

impl IntoFuture for CatalogRequest {
    type Output = eyre::Result<CatalogItem>;
    type IntoFuture = Ready<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        panic!("discovery must fabricate the response without invoking the request")
    }
}

impl IntoFuture for ItemDetailsRequest {
    type Output = eyre::Result<ItemDetails>;
    type IntoFuture = Ready<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        panic!("discovery must never execute downstream requests")
    }
}

impl IntoFuture for CatalogRequestEnvelopeRequest {
    type Output = eyre::Result<CatalogRequestEnvelope>;
    type IntoFuture = Ready<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        panic!("nested request response simulation must never invoke a request")
    }
}

crate::register_arbitrary!(CatalogRequest, associated = REGISTERED_ARBITRARY);
crate::register_thing!(CatalogItem);
crate::register_arbitrary!(CatalogItem, associated = REGISTERED_ARBITRARY);
crate::register_thing!(ItemArgument);
crate::register_arbitrary!(ItemArgument, associated = REGISTERED_ARBITRARY);
crate::register_arbitrary!(ItemDetails, associated = REGISTERED_ARBITRARY);
crate::register_arbitrary!(CatalogRequestEnvelope, associated = REGISTERED_ARBITRARY);
crate::register_into_future!(CatalogRequest => CatalogItem, effects = [Read]);
crate::register_into_future!(ItemDetailsRequest => ItemDetails, effects = [Read]);
crate::register_into_future!(CatalogRequestEnvelopeRequest => CatalogRequestEnvelope, effects = [Read]);
crate::register_from!(CatalogItem => ItemArgument, associated = REGISTERED_INTO_ITEM_ARGUMENT);
crate::register_from!(ItemArgument => CatalogItem, associated = REGISTERED_INTO_CATALOG_ITEM);

const GRAPH: [&'static Function; 6] = [
    CatalogRequest::REGISTERED_INTO_FUTURE,
    ItemDetailsRequest::REGISTERED_INTO_FUTURE,
    CatalogItem::REGISTERED_INTO_ITEM_ARGUMENT,
    CatalogItem::REGISTERED_ARBITRARY,
    ItemDetails::REGISTERED_ARBITRARY,
    CatalogRequest::REGISTERED_ARBITRARY,
];

fn runtime<T: Facet<'static> + Send + 'static>(value: T) -> RuntimeValue {
    RuntimeValue::from_box(Box::new(value)).expect("fixture supports reflection")
}

fn simulate(
    seeds: Vec<RuntimeValue>,
    functions: &[&'static Function],
    max_rounds: usize,
    max_values: usize,
) -> DiscoverySimulation {
    CancellationToken::new()
        .with_scope(|scope| {
            DiscoverySimulation::run(
                seeds,
                functions.iter().copied(),
                42,
                max_rounds,
                max_values,
                &scope,
            )
        })
        .expect("offline simulation succeeds")
}

fn has<T: Facet<'static>>(simulation: &DiscoverySimulation) -> bool {
    simulation.values.contains_key(T::SHAPE)
}

#[test]
fn discovery_fabricates_reads_and_runs_real_conversions_to_unlock_downstream_requests() {
    let simulation = simulate(vec![runtime(CatalogSeed { marker: 7 })], &GRAPH, 32, 128);

    assert!(simulation.converged, "{:?}", simulation.diagnostics);
    assert_eq!(simulation.simulated_requests.len(), 2);
    assert!(has::<ItemDetails>(&simulation));
    let item = simulation
        .values
        .get(CatalogItem::SHAPE)
        .and_then(|values| values.first())
        .map(|value| value.value.peek().get::<CatalogItem>().unwrap())
        .unwrap();
    let argument = simulation
        .values
        .get(ItemArgument::SHAPE)
        .and_then(|values| values.first())
        .map(|value| value.value.peek().get::<ItemArgument>().unwrap())
        .unwrap();
    assert_eq!(argument.marker, item.marker.wrapping_add(1));
    let details = simulation
        .values
        .get(ItemDetails::SHAPE)
        .and_then(|values| values.first())
        .unwrap();
    assert!(details.depth > 0);
    assert!(details.producer.is_some());

    let repeated = simulate(vec![runtime(CatalogSeed { marker: 7 })], &GRAPH, 32, 128);
    let repeated_details = repeated
        .values
        .get(ItemDetails::SHAPE)
        .and_then(|values| values.first())
        .map(|value| value.value.peek().get::<ItemDetails>().unwrap())
        .unwrap();
    assert_eq!(
        details.value.peek().get::<ItemDetails>().unwrap().marker,
        repeated_details.marker,
        "the same seed must reproduce fabricated outputs",
    );
}

#[test]
fn response_generators_outside_the_catalog_do_not_unlock_discovery() {
    let mut functions = GRAPH.to_vec();
    functions.retain(|function| {
        function.kind != FunctionKind::Constructor
            || !function.output_shape.is_shape(CatalogItem::SHAPE)
    });
    let simulation = simulate(
        vec![runtime(CatalogSeed { marker: 7 })],
        &functions,
        32,
        128,
    );

    assert!(simulation.converged, "{:?}", simulation.diagnostics);
    assert_eq!(simulation.simulated_requests.len(), 1);
    assert!(!has::<CatalogItem>(&simulation));
    assert!(!has::<ItemDetails>(&simulation));
    assert!(
        simulation.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains(
                "no exact Arbitrary response generator for CatalogItem in the supplied catalog",
            )
        }),
        "{:?}",
        simulation.diagnostics
    );
}

#[test]
fn custom_trait_contract_text_cannot_claim_the_arbitrary_generator_contract() {
    let mut constructor = *CatalogItem::REGISTERED_ARBITRARY;
    constructor.trait_contract = FunctionTraitContract::Other("Arbitrary");
    let simulation = simulate(
        vec![runtime(CatalogSeed { marker: 7 })],
        &[
            CatalogRequest::REGISTERED_INTO_FUTURE,
            Box::leak(Box::new(constructor)),
        ],
        32,
        128,
    );

    assert_eq!(simulation.simulated_requests.len(), 1);
    assert!(!has::<CatalogItem>(&simulation));
    assert!(
        simulation.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains(
                "no exact Arbitrary response generator for CatalogItem in the supplied catalog",
            )
        }),
        "{:?}",
        simulation.diagnostics
    );
}

#[test]
fn arbitrary_request_constructor_cannot_create_a_missing_dependency() {
    let simulation = simulate(Vec::new(), &GRAPH, 32, 128);
    assert!(simulation.converged);
    assert!(simulation.simulated_requests.is_empty());
    assert!(!has::<CatalogItem>(&simulation));
    assert!(!has::<ItemDetails>(&simulation));
}

#[test]
fn write_prompting_mixed_and_unclassified_requests_are_never_simulated() {
    for effects in [
        &[Effect::Write][..],
        &[Effect::Read, Effect::Write][..],
        &[Effect::Solicit][..],
        &[Effect::Read, Effect::Solicit][..],
        &[Effect::Read, Effect::Write, Effect::Solicit][..],
        &[][..],
    ] {
        // Mutating a copy models external custom registration bypassing the
        // standard constructor's nonempty-effects invariant.
        let mut request = *CatalogRequest::REGISTERED_INTO_FUTURE;
        request.effects = effects;
        let request = Box::leak(Box::new(request));
        let simulation = simulate(
            vec![runtime(CatalogSeed { marker: 7 })],
            &[request, CatalogItem::REGISTERED_ARBITRARY],
            32,
            128,
        );
        assert!(simulation.simulated_requests.is_empty(), "{effects:?}");
        assert!(!has::<CatalogItem>(&simulation), "{effects:?}");
    }
}

fn forbidden_conversion(_: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
    panic!("an effectful conversion must never run during discovery")
}

fn forbidden_async_conversion(_: Box<dyn Any + Send>) -> InvocationFuture {
    panic!("an async executor must never run during discovery")
}

#[test]
fn effectful_conversions_cannot_unlock_downstream_reads() {
    for effects in [
        &[Effect::Read][..],
        &[Effect::Write][..],
        &[Effect::Solicit][..],
    ] {
        let mut conversion = *CatalogItem::REGISTERED_INTO_ITEM_ARGUMENT;
        conversion.effects = effects;
        conversion.executor = FunctionExecutor::SyncValue(forbidden_conversion);
        let simulation = simulate(
            vec![runtime(CatalogItem { marker: 11 })],
            &[
                Box::leak(Box::new(conversion)),
                ItemDetailsRequest::REGISTERED_INTO_FUTURE,
                ItemDetails::REGISTERED_ARBITRARY,
            ],
            32,
            128,
        );
        assert!(simulation.simulated_requests.is_empty(), "{effects:?}");
        assert!(!has::<ItemArgument>(&simulation), "{effects:?}");
        assert!(!has::<ItemDetails>(&simulation), "{effects:?}");
    }
}

#[test]
fn async_conversion_metadata_does_not_permit_executing_its_future() {
    let mut conversion = *CatalogItem::REGISTERED_INTO_ITEM_ARGUMENT;
    conversion.executor = FunctionExecutor::AsyncValue(forbidden_async_conversion);
    let simulation = simulate(
        vec![runtime(CatalogItem { marker: 11 })],
        &[
            Box::leak(Box::new(conversion)),
            ItemDetailsRequest::REGISTERED_INTO_FUTURE,
            ItemDetails::REGISTERED_ARBITRARY,
        ],
        32,
        128,
    );
    assert!(simulation.simulated_requests.is_empty());
    assert!(!has::<ItemArgument>(&simulation));
}

#[test]
fn conversion_cycle_reaches_a_fixed_point_without_repeating_requests() {
    let mut functions = GRAPH.to_vec();
    functions.push(ItemArgument::REGISTERED_INTO_CATALOG_ITEM);
    let simulation = simulate(
        vec![runtime(CatalogSeed { marker: 7 })],
        &functions,
        32,
        128,
    );
    assert!(simulation.converged, "{:?}", simulation.diagnostics);
    assert_eq!(simulation.simulated_requests.len(), 2);
    assert!(simulation.rounds < 32);
}

fn count_failed_conversion(_: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
    CONVERSION_INVOCATIONS.with(|count| count.set(count.get() + 1));
    eyre::bail!("this fixture conversion deliberately fails");
}

#[test]
fn repeated_catalog_handles_complete_each_distinct_registration_once_even_on_failure() {
    CONVERSION_INVOCATIONS.with(|count| count.set(0));
    let mut conversion = *CatalogItem::REGISTERED_INTO_ITEM_ARGUMENT;
    conversion.executor = FunctionExecutor::SyncValue(count_failed_conversion);
    // These entries deliberately share all metadata and the executor. Their
    // registration addresses, rather than their signatures, distinguish them.
    let first: &'static Function = Box::leak(Box::new(conversion));
    let second: &'static Function = Box::leak(Box::new(conversion));
    let simulation = simulate(
        vec![runtime(CatalogItem { marker: 7 })],
        &[first, first, second, second],
        32,
        128,
    );

    assert!(simulation.converged, "{:?}", simulation.diagnostics);
    CONVERSION_INVOCATIONS.with(|count| {
        assert_eq!(
            count.get(),
            2,
            "duplicate handles and later rounds must not retry failed conversions",
        );
    });
}

#[test]
fn round_limit_reports_incomplete_discovery() {
    let simulation = simulate(vec![runtime(CatalogSeed { marker: 7 })], &GRAPH, 1, 128);
    assert!(!simulation.converged);
    assert!(!simulation.diagnostics.is_empty());
}

#[test]
fn value_limit_reports_incomplete_discovery() {
    let simulation = simulate(vec![runtime(CatalogSeed { marker: 7 })], &GRAPH, 32, 1);
    assert!(!simulation.converged);
    assert!(simulation.values.values().map(Vec::len).sum::<usize>() <= 1);
    assert!(!simulation.diagnostics.is_empty());
}

#[test]
fn duplicate_early_fields_do_not_hide_a_later_dependency() {
    let simulation = simulate(
        vec![runtime(RepeatedFieldsSeed {
            first: "first".to_owned(),
            second: "second".to_owned(),
            dependency: LateDependency,
        })],
        &[],
        32,
        3,
    );
    assert!(
        has::<LateDependency>(&simulation),
        "{:?}",
        simulation.diagnostics
    );
    assert!(simulation.values.values().map(Vec::len).sum::<usize>() <= 3);
}

#[test]
fn arbitrary_response_containing_a_request_is_rejected_before_generation() {
    let simulation = simulate(
        vec![runtime(CatalogSeed { marker: 7 })],
        &[
            CatalogRequestEnvelopeRequest::REGISTERED_INTO_FUTURE,
            CatalogRequestEnvelope::REGISTERED_ARBITRARY,
            CatalogRequest::REGISTERED_INTO_FUTURE,
        ],
        32,
        128,
    );
    assert!(!has::<CatalogRequestEnvelope>(&simulation));
    assert!(
        simulation
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.contains("contains an executable request") }),
        "{:?}",
        simulation.diagnostics
    );
}

#[test]
fn cancellation_during_response_generation_is_not_reported_as_a_generation_failure()
-> eyre::Result<()> {
    struct RestoreResponseCancellation(Option<CancellationScope>);

    impl Drop for RestoreResponseCancellation {
        fn drop(&mut self) {
            RESPONSE_CANCELLATION.with(|scope| scope.replace(self.0.take()));
        }
    }

    CancellationToken::new().with_scope(|scope| {
        let previous = RESPONSE_CANCELLATION.with(|current| current.replace(Some(scope.clone())));
        let _restore = RestoreResponseCancellation(previous);
        let mut request = *CatalogRequest::REGISTERED_INTO_FUTURE;
        request.output_shape = CancelledResponse::SHAPE;
        request.output_to_runtime = runtime_from_boxed::<CancelledResponse>;
        let generator = Function::sync_mut(
            ArbitraryBytes::SHAPE,
            CancelledResponse::SHAPE,
            FunctionKind::Constructor,
            "arbitrary",
            FunctionTraitContract::Arbitrary,
            &[],
            cancel_response_generation,
            runtime_from_boxed::<CancelledResponse>,
            RegistrationSite::new(file!(), line!()),
        );
        let functions: [&'static Function; 2] =
            [Box::leak(Box::new(request)), Box::leak(Box::new(generator))];
        let result = DiscoverySimulation::run(
            vec![runtime(CatalogSeed { marker: 7 })],
            functions,
            42,
            1,
            128,
            &scope,
        );
        let error = result
            .err()
            .expect("cancellation must escape the final round");
        assert!(
            error.downcast_ref::<CancellationError>().is_some(),
            "expected caller cancellation, received {error:#}",
        );
        Ok(())
    })
}

#[test]
#[should_panic(expected = "effects")]
fn custom_async_request_constructor_rejects_empty_effects() {
    let _ = Function::async_value(
        CatalogRequest::SHAPE,
        CatalogItem::SHAPE,
        FunctionKind::AsyncInvoke,
        "invoke",
        FunctionTraitContract::Other("policy test"),
        &[],
        forbidden_async_conversion,
        runtime_from_boxed::<CatalogItem>,
        RegistrationSite::new(file!(), line!()),
    );
}
