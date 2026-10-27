use super::DumpPlanner;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Effect;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::RegistrationSite;
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
    #[derive(Clone, facet::Facet)]
    pub struct DuplicateName;
}

mod second {
    #[derive(Clone, facet::Facet)]
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
const fn request<I: Facet<'static>, O: Facet<'static>>(effects: &'static [Effect]) -> Function {
    Function::async_value(
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
) -> Function {
    Function::sync_value(
        I::SHAPE,
        O::SHAPE,
        kind,
        "fixture pure operation",
        trait_contract,
        &[],
        |_| panic!("offline planning must not execute pure operations"),
        |_| panic!("offline planning must not materialize values"),
        RegistrationSite::new(file!(), line!()),
    )
}

static INVENTORY_REQUEST: Function = request::<InventoryRequest, Vec<Inventory>>(&[Effect::Read]);
static WRITE_REQUEST: Function = request::<WriteRequest, WriteOnly>(&[Effect::Write]);
static INTERACTIVE_REQUEST: Function =
    request::<InteractiveRequest, InteractiveOnly>(&[Effect::Read, Effect::Solicit]);
static FIRST_REQUEST: Function = request::<FirstRequest, first::DuplicateName>(&[Effect::Read]);
static SECOND_REQUEST: Function = request::<SecondRequest, second::DuplicateName>(&[Effect::Read]);
static INVENTORY_CONVERSION: Function =
    pure::<Inventory, ConvertedInventory>(FunctionKind::Conversion, FunctionTraitContract::From);
static UNRELATED_CONVERSION: Function =
    pure::<UnrelatedInput, UnrelatedOutput>(FunctionKind::Conversion, FunctionTraitContract::From);
static ARBITRARY_CONSTRUCTOR: Function = pure::<ArbitraryBytes, ArbitraryOnly>(
    FunctionKind::Constructor,
    FunctionTraitContract::Arbitrary,
);

const CATALOG: [&Function; 6] = [
    &INVENTORY_REQUEST,
    &WRITE_REQUEST,
    &INTERACTIVE_REQUEST,
    &INVENTORY_CONVERSION,
    &UNRELATED_CONVERSION,
    &ARBITRARY_CONSTRUCTOR,
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
        [&FIRST_REQUEST, &SECOND_REQUEST],
        [],
        vec![Organization::SHAPE],
    );
    assert!(planner.target_names().is_empty());
    let error = CancellationToken::new()
        .with_scope(|scope| planner.plan(&["DuplicateName".to_owned()], &scope))
        .unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
}
