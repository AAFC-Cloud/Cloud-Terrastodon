use crate::DiscoveryPlan;
use crate::DiscoveryPlanNodeKind;
use crate::Function;
use crate::describe_shape;
use arbitrary::Arbitrary;
use facet::Facet;
use facet::OpaqueBorrow;
use facet::Shape;
use std::borrow::Cow;
use std::future::IntoFuture;
use std::future::Ready;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

#[derive(Clone, Facet)]
struct Organization;

#[derive(Clone, Facet)]
struct Auth;

#[derive(Clone, Facet)]
struct OtherAuth;

#[derive(Clone, Facet)]
#[repr(u8)]
enum AuthChoice {
    Other(OtherAuth),
    Current(Auth),
}

#[derive(Arbitrary, Clone, Facet)]
struct Project;

#[derive(Clone, Facet)]
struct ProjectArgument;

impl From<Project> for ProjectArgument {
    fn from(_: Project) -> Self {
        panic!("planning must not execute conversions")
    }
}

impl From<ProjectArgument> for Project {
    fn from(_: ProjectArgument) -> Self {
        panic!("planning must not execute reverse conversions")
    }
}

#[derive(Clone, Facet)]
struct ServiceConnection;

#[derive(Clone, Facet)]
struct Unrelated;

#[derive(Facet)]
struct ProjectsRequest {
    org: Organization,
    auth: Auth,
    cancellation: OpaqueBorrow<'static, CancellationScope>,
}

#[derive(Facet)]
struct ConnectionsRequest {
    org: Organization,
    auth: Auth,
    project: Cow<'static, ProjectArgument>,
    filter: Option<String>,
    cancellation: OpaqueBorrow<'static, CancellationScope>,
}

#[derive(Clone, Facet)]
struct UnrelatedRequest {
    org: Organization,
}

#[derive(Clone, Facet)]
struct MutationRequest {
    org: Organization,
}

#[derive(Clone, Facet)]
struct SolicitingRequest {
    org: Organization,
}

#[derive(Facet)]
struct AmbientProjectRequest {
    cancellation: OpaqueBorrow<'static, CancellationScope>,
    filter: Option<String>,
}

macro_rules! nonexecuting_request {
    ($request:ty => $output:ty) => {
        impl IntoFuture for $request {
            type Output = eyre::Result<$output>;
            type IntoFuture = Ready<Self::Output>;

            fn into_future(self) -> Self::IntoFuture {
                panic!("planning must not invoke requests")
            }
        }
    };
}

nonexecuting_request!(ProjectsRequest => Vec<Project>);
nonexecuting_request!(ConnectionsRequest => Vec<ServiceConnection>);
nonexecuting_request!(UnrelatedRequest => Unrelated);
nonexecuting_request!(MutationRequest => ServiceConnection);
nonexecuting_request!(SolicitingRequest => ServiceConnection);
nonexecuting_request!(AmbientProjectRequest => Project);
crate::register_into_future!(ProjectsRequest => Vec<Project>, effects = [Read]);
crate::register_into_future!(ConnectionsRequest => Vec<ServiceConnection>, effects = [Read]);
crate::register_into_future!(UnrelatedRequest => Unrelated, effects = [Read]);
crate::register_into_future!(MutationRequest => ServiceConnection, effects = [Write]);
crate::register_into_future!(SolicitingRequest => ServiceConnection, effects = [Read, Solicit]);
crate::register_into_future!(AmbientProjectRequest => Project, effects = [Read]);
crate::register_from!(Project => ProjectArgument, associated = REGISTERED_ARGUMENT);
crate::register_from!(ProjectArgument => Project, associated = REGISTERED_PROJECT);
crate::register_arbitrary!(Project, associated = REGISTERED_ARBITRARY);

const CATALOG: [&Function; 8] = [
    ConnectionsRequest::REGISTERED_INTO_FUTURE,
    ProjectsRequest::REGISTERED_INTO_FUTURE,
    UnrelatedRequest::REGISTERED_INTO_FUTURE,
    MutationRequest::REGISTERED_INTO_FUTURE,
    SolicitingRequest::REGISTERED_INTO_FUTURE,
    Project::REGISTERED_ARGUMENT,
    ProjectArgument::REGISTERED_PROJECT,
    Project::REGISTERED_ARBITRARY,
];

fn plan(
    catalog: impl IntoIterator<Item = &'static Function>,
    seeds: &[&'static Shape],
    targets: &[&'static Shape],
) -> DiscoveryPlan {
    CancellationToken::new()
        .with_scope(|scope| {
            DiscoveryPlan::build(
                catalog,
                seeds.iter().copied(),
                targets.iter().copied(),
                &scope,
            )
        })
        .expect("planning succeeds without running any operation")
}

#[test]
fn includes_every_required_input_in_a_target_directed_chain() {
    let plan = plan(
        CATALOG,
        &[Organization::SHAPE, Auth::SHAPE],
        &[ServiceConnection::SHAPE],
    );
    assert!(plan.diagnostics.is_empty());
    assert!(plan.nodes.iter().all(|node| node.reachable));
    let requests: Vec<_> = plan
        .nodes
        .iter()
        .filter(|node| node.kind == DiscoveryPlanNodeKind::Request)
        .map(|node| node.label.as_str())
        .collect();
    assert_eq!(requests.len(), 2);
    assert!(requests.contains(&"ProjectsRequest"));
    assert!(requests.contains(&"ConnectionsRequest"));
    for request in ["ProjectsRequest", "ConnectionsRequest"] {
        let node = plan
            .nodes
            .iter()
            .find(|node| node.label == request)
            .unwrap();
        let inputs: Vec<_> = plan
            .edges
            .iter()
            .filter(|edge| edge.target == node.id)
            .map(|edge| edge.label.as_str())
            .collect();
        assert!(inputs.contains(&"org"));
        assert!(inputs.contains(&"auth"));
        if request == "ConnectionsRequest" {
            assert!(inputs.contains(&"project"));
        }
        assert!(!inputs.contains(&"filter"));
        assert!(!inputs.contains(&"cancellation"));
    }
    assert!(plan.edges.iter().all(|edge| {
        edge.source < plan.nodes.len()
            && edge.target < plan.nodes.len()
            && plan.nodes[edge.source].level < plan.nodes[edge.target].level
    }));
}

#[test]
fn missing_auth_blocks_the_chain_and_is_explained() {
    let plan = plan(CATALOG, &[Organization::SHAPE], &[ServiceConnection::SHAPE]);
    assert!(
        !plan
            .nodes
            .iter()
            .find(|node| node.target)
            .unwrap()
            .reachable
    );
    assert!(plan.nodes.iter().any(|node| {
        node.kind == DiscoveryPlanNodeKind::Request
            && node
                .missing_inputs
                .iter()
                .any(|input| input == "auth: Auth")
    }));
    assert_eq!(plan.diagnostics.len(), 1);
}

#[test]
fn writes_solicitation_and_arbitrary_cannot_supply_a_missing_target() {
    let plan = plan(
        [
            MutationRequest::REGISTERED_INTO_FUTURE,
            SolicitingRequest::REGISTERED_INTO_FUTURE,
            Project::REGISTERED_ARBITRARY,
        ],
        &[Organization::SHAPE],
        &[ServiceConnection::SHAPE, Project::SHAPE],
    );
    assert_eq!(plan.nodes.len(), 2);
    assert!(plan.nodes.iter().all(|node| node.target && !node.reachable));
    assert!(plan.edges.is_empty());
}

#[test]
fn unseeded_conversion_cycle_cannot_invent_values() {
    let catalog = [
        Project::REGISTERED_ARGUMENT,
        ProjectArgument::REGISTERED_PROJECT,
    ];
    let blocked = plan(catalog, &[], &[ProjectArgument::SHAPE]);
    assert!(blocked.nodes.iter().all(|node| !node.reachable));
    let reachable = plan(catalog, &[Project::SHAPE], &[ProjectArgument::SHAPE]);
    assert!(reachable.nodes.iter().all(|node| node.reachable));
    assert_eq!(reachable.nodes.len(), 3);
    assert_eq!(reachable.edges.len(), 2);
}

#[test]
fn a_supplied_catalog_is_the_only_source_of_operations() {
    let plan = plan(
        [ConnectionsRequest::REGISTERED_INTO_FUTURE],
        &[Organization::SHAPE, Auth::SHAPE],
        &[ServiceConnection::SHAPE],
    );
    assert!(plan.nodes.iter().any(|node| {
        node.missing_inputs
            .iter()
            .any(|input| input == "project: ProjectArgument")
    }));
    assert!(
        !plan
            .nodes
            .iter()
            .find(|node| node.target)
            .unwrap()
            .reachable
    );
}

#[test]
fn seed_enum_variants_do_not_authorize_additional_auth_contexts() {
    let plan = plan(
        CATALOG,
        &[Organization::SHAPE, AuthChoice::SHAPE],
        &[ServiceConnection::SHAPE, OtherAuth::SHAPE],
    );
    assert!(
        plan.nodes
            .iter()
            .filter(|node| node.target)
            .all(|node| !node.reachable)
    );
    assert!(!plan.nodes.iter().any(|node| node.label == "AuthChoice"));
}

#[test]
fn collection_elements_are_planned_without_fabricating_or_executing() {
    let plan = plan(
        [ProjectsRequest::REGISTERED_INTO_FUTURE],
        &[Organization::SHAPE, Auth::SHAPE],
        &[Project::SHAPE],
    );
    assert!(
        plan.nodes
            .iter()
            .find(|node| node.target)
            .unwrap()
            .reachable
    );
    assert!(
        plan.edges
            .iter()
            .any(|edge| edge.label == "element (if present)")
    );
    // There is no Arbitrary constructor in this catalog and its request executor
    // panics. Empty live results would still have this same structural plan.
}

#[test]
fn explicit_request_seed_is_an_alternative_to_assembling_its_fields() {
    let plan = plan(
        [ProjectsRequest::REGISTERED_INTO_FUTURE],
        &[ProjectsRequest::SHAPE],
        &[Project::SHAPE],
    );
    assert!(plan.nodes.iter().all(|node| node.reachable));
    assert!(
        !plan
            .nodes
            .iter()
            .any(|node| node.label == describe_shape(Auth::SHAPE))
    );
}

#[test]
fn input_order_and_duplicate_catalog_handles_do_not_change_the_plan() {
    let first = plan(
        CATALOG,
        &[Auth::SHAPE, Organization::SHAPE],
        &[ServiceConnection::SHAPE],
    );
    let second = plan(
        CATALOG.into_iter().rev().chain(CATALOG),
        &[Organization::SHAPE, Auth::SHAPE],
        &[ServiceConnection::SHAPE],
    );
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
}

#[test]
fn requests_without_required_domain_inputs_cannot_bootstrap_ambient_defaults() {
    let catalog = [AmbientProjectRequest::REGISTERED_INTO_FUTURE];
    let blocked = plan(catalog, &[Organization::SHAPE], &[Project::SHAPE]);
    assert!(blocked.nodes.iter().all(|node| !node.reachable));
    assert!(blocked.nodes.iter().any(|node| {
        node.kind == DiscoveryPlanNodeKind::Request
            && node.missing_inputs == ["input: AmbientProjectRequest"]
    }));
    let explicit = plan(catalog, &[AmbientProjectRequest::SHAPE], &[Project::SHAPE]);
    assert!(explicit.nodes.iter().all(|node| node.reachable));
    assert!(explicit.diagnostics.is_empty());
}
