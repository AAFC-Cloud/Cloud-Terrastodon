use super::DumpPlanner;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::PureFunction;
use cloud_terrastodon_registry::RegistrationSite;
use cloud_terrastodon_registry_execution::Function;
use cloud_terrastodon_registry_execution::RuntimeFunction;
use facet::Facet;
use teamy_cancellation::CancellationToken;

#[derive(Clone, Facet)]
struct Organization;

#[derive(Clone, Facet)]
struct NestedRecord;

#[derive(Clone, Facet)]
struct CliSchema {
    #[facet(figue::named)]
    flag: bool,
}

#[derive(Clone, Facet)]
struct Inventory {
    nested: Option<Vec<NestedRecord>>,
    description: String,
    cli_schema: CliSchema,
}

#[derive(Clone, Facet)]
struct ConvertedInventory;

#[derive(Clone, Facet)]
struct WriteOnly;

#[derive(Clone, Facet)]
struct InteractiveOnly;

#[derive(Clone, Facet)]
struct ArbitraryOnly;

#[derive(Clone, Facet)]
struct UnrelatedInput;

#[derive(Clone, Facet)]
struct UnrelatedOutput;

#[derive(Facet)]
struct InventoryRequest {
    organization: Organization,
}

#[derive(Facet)]
struct WriteRequest {
    organization: Organization,
}

#[derive(Facet)]
struct InteractiveRequest {
    organization: Organization,
}

mod first {
    use facet::Facet;
    #[derive(Clone, Facet)]
    pub struct DuplicateName;
}

mod second {
    use facet::Facet;
    #[derive(Clone, Facet)]
    pub struct DuplicateName;
}

#[derive(Facet)]
struct FirstRequest {
    organization: Organization,
}

#[derive(Facet)]
struct SecondRequest {
    organization: Organization,
}

// These descriptors intentionally stay outside linkme. A planner-only fixture
// must not expand the production registry or its response-generator obligations.
const fn request<I: Facet<'static>, O: Facet<'static>>(
    effects: &'static [Effect],
) -> RuntimeFunction {
    RuntimeFunction::async_value(
        I::SHAPE,
        O::SHAPE,
        FunctionKind::AsyncInvoke,
        "fixture request",
        FunctionTraitContract::IntoFuture,
        effects,
        |_| panic!("offline planning must not execute requests"),
        |_| panic!("offline planning must not materialize values"),
        RegistrationSite::new(file!(), line!()),
    )
}

const fn pure<I: Facet<'static>, O: Facet<'static>>(
    kind: FunctionKind,
    trait_contract: FunctionTraitContract,
) -> PureFunction {
    PureFunction::sync_value(
        I::SHAPE,
        O::SHAPE,
        kind,
        "fixture pure operation",
        trait_contract,
        |_| panic!("offline planning must not execute pure operations"),
        |_| panic!("offline planning must not materialize values"),
        RegistrationSite::new(file!(), line!()),
    )
}

static INVENTORY_REQUEST: RuntimeFunction =
    request::<InventoryRequest, Vec<Inventory>>(&[Effect::Read]);
static WRITE_REQUEST: RuntimeFunction = request::<WriteRequest, WriteOnly>(&[Effect::Write]);
static INTERACTIVE_REQUEST: RuntimeFunction =
    request::<InteractiveRequest, InteractiveOnly>(&[Effect::Read, Effect::Solicit]);
static FIRST_REQUEST: RuntimeFunction =
    request::<FirstRequest, first::DuplicateName>(&[Effect::Read]);
static SECOND_REQUEST: RuntimeFunction =
    request::<SecondRequest, second::DuplicateName>(&[Effect::Read]);
static INVENTORY_CONVERSION: PureFunction =
    pure::<Inventory, ConvertedInventory>(FunctionKind::Conversion, FunctionTraitContract::From);
static UNRELATED_CONVERSION: PureFunction =
    pure::<UnrelatedInput, UnrelatedOutput>(FunctionKind::Conversion, FunctionTraitContract::From);
static ARBITRARY_CONSTRUCTOR: PureFunction = pure::<ArbitraryBytes, ArbitraryOnly>(
    FunctionKind::Constructor,
    FunctionTraitContract::Arbitrary,
);

const CATALOG: [Function; 6] = [
    Function::Runtime(&INVENTORY_REQUEST),
    Function::Runtime(&WRITE_REQUEST),
    Function::Runtime(&INTERACTIVE_REQUEST),
    Function::Pure(&INVENTORY_CONVERSION),
    Function::Pure(&UNRELATED_CONVERSION),
    Function::Pure(&ARBITRARY_CONSTRUCTOR),
];

#[test]
fn palette_offers_read_descendants_and_related_conversions_even_without_scope() {
    let planner = DumpPlanner::new(CATALOG, [], Vec::new());
    assert_eq!(
        planner.target_names(),
        ["ConvertedInventory", "Inventory", "NestedRecord"]
    );
    let plan = CancellationToken::new()
        .with_scope(|scope| planner.plan(&["NestedRecord".to_owned()], &scope))
        .unwrap();
    assert!(plan.nodes.iter().any(|node| node.target && !node.reachable));
    assert!(
        plan.diagnostics
            .iter()
            .any(|message| message.contains("unreachable"))
    );
}

#[test]
fn selected_conversion_can_be_planned_without_running_catalog_code() {
    let planner = DumpPlanner::new(CATALOG, [], vec![Organization::SHAPE]);
    let plan = CancellationToken::new()
        .with_scope(|scope| planner.plan(&["ConvertedInventory".to_owned()], &scope))
        .unwrap();
    assert!(plan.nodes.iter().all(|node| node.reachable));
    assert!(
        plan.nodes
            .iter()
            .any(|node| node.label == "InventoryRequest")
    );
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn unknown_target_is_preserved_with_a_diagnostic_and_duplicates_are_collapsed() {
    let planner = DumpPlanner::new(CATALOG, [], vec![Organization::SHAPE]);
    let plan = CancellationToken::new()
        .with_scope(|scope| {
            planner.plan(
                &["MissingInventory".to_owned(), "MissingInventory".to_owned()],
                &scope,
            )
        })
        .unwrap();
    assert_eq!(plan.nodes.len(), 1);
    assert_eq!(plan.nodes[0].label, "MissingInventory");
    assert!(plan.nodes[0].target);
    assert!(!plan.nodes[0].reachable);
    assert_eq!(plan.diagnostics.len(), 1);
    assert!(plan.diagnostics[0].contains("no linked type registration"));
}

#[test]
fn removing_all_targets_clears_the_graph() {
    let planner = DumpPlanner::new(CATALOG, [], vec![Organization::SHAPE]);
    let plan = CancellationToken::new()
        .with_scope(|scope| planner.plan(&[], &scope))
        .unwrap();
    assert!(plan.nodes.is_empty());
    assert!(plan.edges.is_empty());
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn ambiguous_names_are_not_offered_and_cannot_silently_select_a_type() {
    let planner = DumpPlanner::new(
        [
            Function::Runtime(&FIRST_REQUEST),
            Function::Runtime(&SECOND_REQUEST),
        ],
        [],
        vec![Organization::SHAPE],
    );
    assert!(planner.target_names().is_empty());
    let error = CancellationToken::new()
        .with_scope(|scope| planner.plan(&["DuplicateName".to_owned()], &scope))
        .unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
}
