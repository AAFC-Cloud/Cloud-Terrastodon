#![cfg(test)]

// SDK discovery uses explicit credential selections without runtime contexts.

#[path = "support/discovery_requests.rs"]
mod discovery_requests;

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpointsListRequest;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinition;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildYamlFilename;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use cloud_terrastodon_registry::FunctionDescriptor;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry_execution::DiscoverySimulation;
use cloud_terrastodon_registry_execution::Function;
use cloud_terrastodon_registry_execution::known_functions;
use discovery_requests::ProjectFoldersRequest;
use facet::Facet;
use facet::Shape;
use std::sync::Arc;

fn plan(
    catalog: impl IntoIterator<Item = &'static FunctionDescriptor>,
    seeds: &[&'static Shape],
    targets: &[&'static Shape],
) -> DiscoveryPlan {
    CancellationToken::new()
        .with_scope(|cancellation| {
            DiscoveryPlan::build(
                catalog,
                seeds.iter().copied(),
                targets.iter().copied(),
                &cancellation,
            )
        })
        .expect("planning inspects metadata without resolving credentials or invoking requests")
}

#[test]
fn sdk_linked_catalog_connects_projects_to_service_connections_and_full_build_definitions() {
    let plan = plan(
        known_functions().iter().map(Function::descriptor),
        &[
            AzureDevOpsOrganizationUrl::SHAPE,
            AzureDevOpsAuthSelection::SHAPE,
        ],
        &[
            AzureDevOpsServiceEndpoint::SHAPE,
            AzureDevOpsBuildDefinition::SHAPE,
        ],
    );

    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(plan.nodes.iter().all(|node| node.reachable));
    for request in [
        AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST,
        AzureDevOpsServiceEndpointsListRequest::REGISTERED_EXECUTABLE_REQUEST,
        AzureDevOpsBuildDefinitionListRequest::REGISTERED_EXECUTABLE_REQUEST,
        AzureDevOpsBuildDefinitionGetRequest::REGISTERED_EXECUTABLE_REQUEST,
    ] {
        assert!(
            plan.nodes.iter().any(|node| {
                node.kind == DiscoveryPlanNodeKind::Request
                    && node.label == describe_shape(request.input_shape)
            }),
            "missing prerequisite {} from {plan:?}",
            describe_shape(request.input_shape),
        );
    }
    assert!(
        plan.edges
            .iter()
            .any(|edge| edge.label == "element (if present)"),
        "project and definition inventories may be empty",
    );
}

#[test]
fn sdk_an_organization_without_authentication_cannot_unlock_project_inventories() {
    let plan = plan(
        known_functions().iter().map(Function::descriptor),
        &[AzureDevOpsOrganizationUrl::SHAPE],
        &[
            AzureDevOpsServiceEndpoint::SHAPE,
            AzureDevOpsBuildDefinition::SHAPE,
        ],
    );

    assert!(
        plan.nodes
            .iter()
            .filter(|node| node.target)
            .all(|node| !node.reachable),
    );
    assert!(plan.nodes.iter().any(|node| {
        node.kind == DiscoveryPlanNodeKind::Request
            && node.label == describe_shape(AzureDevOpsProjectsListRequest::SHAPE)
            && node
                .missing_inputs
                .iter()
                .any(|input| input == "auth_selection: AzureDevOpsAuthSelection")
    }));
}

#[test]
fn sdk_project_argument_conversion_is_required_even_when_request_arbitrary_is_registered() {
    // The curated catalog intentionally omits the alternate project ID/name
    // conversions. Their registrations in the linked global registry cannot
    // fill a gap in the catalog supplied to this planner.
    let catalog = [
        AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
        AzureDevOpsServiceEndpointsListRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
    ];
    let seeds = [
        AzureDevOpsOrganizationUrl::SHAPE,
        AzureDevOpsAuthSelection::SHAPE,
    ];
    let targets = [AzureDevOpsServiceEndpoint::SHAPE];

    let blocked = plan(catalog, &seeds, &targets);
    assert!(
        !blocked
            .nodes
            .iter()
            .find(|node| node.target)
            .unwrap()
            .reachable,
    );
    assert!(blocked.nodes.iter().any(|node| {
        node.kind == DiscoveryPlanNodeKind::Request
            && node
                .missing_inputs
                .iter()
                .any(|input| input == "project: AzureDevOpsProjectArgument")
    }));

    let connected = plan(
        catalog
            .into_iter()
            .chain([AzureDevOpsProject::REGISTERED_INTO_PROJECT_ARGUMENT.descriptor()]),
        &seeds,
        &targets,
    );
    assert!(connected.nodes.iter().all(|node| node.reachable));
    assert!(connected.diagnostics.is_empty());
}

#[test]
fn sdk_yaml_filename_is_a_conditional_descendant_of_a_full_build_definition() {
    let plan = plan(
        [
            AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
            AzureDevOpsProject::REGISTERED_INTO_PROJECT_ARGUMENT.descriptor(),
            AzureDevOpsBuildDefinitionListRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
            AzureDevOpsBuildDefinitionGetRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
        ],
        &[
            AzureDevOpsOrganizationUrl::SHAPE,
            AzureDevOpsAuthSelection::SHAPE,
        ],
        &[AzureDevOpsBuildYamlFilename::SHAPE],
    );

    assert!(plan.nodes.iter().all(|node| node.reachable));
    assert!(plan.edges.iter().any(|edge| edge.label == "yaml_filename"));
    let target = plan.nodes.iter().find(|node| node.target).unwrap();
    assert!(
        plan.edges
            .iter()
            .any(|edge| { edge.target == target.id && edge.label == "Some (if present)" })
    );
}

#[test]
fn sdk_devops_selection_has_only_conditional_structural_bearer_reachability() {
    let plan = plan(
        known_functions().iter().map(Function::descriptor),
        &[
            AzureDevOpsOrganizationUrl::SHAPE,
            AzureDevOpsAuthSelection::SHAPE,
        ],
        &[AzureBearerAuthenticationSelection::SHAPE],
    );

    // The linked catalog converts DevOps selections to AuthenticationSelection.
    // Their bearer variants now expose the same explicit selection payload.
    // Type-only planning can describe that path, but cannot promise the enum
    // value is a bearer selection rather than a PAT.
    let target = plan.nodes.iter().find(|node| node.target).unwrap();
    assert!(target.reachable);
    assert!(
        plan.edges
            .iter()
            .any(|edge| { edge.target == target.id && edge.label.ends_with("(if active)") })
    );
}

#[test]
fn sdk_value_discovery_preserves_bearer_identity_and_cannot_invent_it_from_a_pat() {
    let catalog: Vec<_> = known_functions()
        .iter()
        .filter(|function| {
            (function.kind == FunctionKind::Conversion
                && function
                    .input_shape
                    .is_shape(AzureDevOpsAuthSelection::SHAPE)
                && function
                    .output_shape
                    .is_shape(AuthenticationSelection::SHAPE))
                || (function.kind == FunctionKind::Constructor
                    && function.trait_contract == FunctionTraitContract::Arbitrary
                    && function
                        .output_shape
                        .is_shape(AzureBearerAuthenticationSelection::SHAPE))
        })
        .collect();
    assert_eq!(
        catalog.len(),
        2,
        "the test requires the linked conversion and generator"
    );
    let bearer = AzureBearerAuthenticationSelection {
        account_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
        auth_source: AzureBearerAuthSource::Browser,
        tenant_id: "22222222-2222-2222-2222-222222222222".parse().unwrap(),
    };
    for (selection, expected_bearer) in [
        (
            AzureDevOpsAuthSelection::PersonalAccessToken {
                account_id: bearer.account_id,
            },
            None,
        ),
        (AzureDevOpsAuthSelection::Bearer(bearer), Some(bearer)),
    ] {
        let simulation = CancellationToken::new()
            .with_scope(|cancellation| {
                DiscoverySimulation::run(
                    vec![RuntimeValue::from_box(Box::new(selection))?],
                    // A linked generator must not manufacture a missing
                    // credential selection while exploring actual values.
                    catalog.iter().copied(),
                    7,
                    16,
                    64,
                    &cancellation,
                )
            })
            .unwrap();
        assert!(simulation.converged, "{:?}", simulation.diagnostics);
        assert!(simulation.simulated_requests.is_empty());
        let discovered = simulation
            .values
            .get(AzureBearerAuthenticationSelection::SHAPE)
            .and_then(|values| values.first())
            .map(|value| {
                *value
                    .value
                    .peek()
                    .get::<AzureBearerAuthenticationSelection>()
                    .unwrap()
            });
        assert_eq!(discovered, expected_bearer);
        let converted = simulation.values[AuthenticationSelection::SHAPE][0]
            .value
            .peek()
            .get::<AuthenticationSelection>()
            .unwrap();
        assert_eq!(*converted, AuthenticationSelection::from(selection));
    }
}

#[test]
fn structural_project_folder_plan_covers_the_arbitrary_simulation_requests() {
    let catalog = [
        AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST,
        Function::Pure(AzureDevOpsProject::REGISTERED_INTO_PROJECT_ARGUMENT),
        ProjectFoldersRequest::REGISTERED_EXECUTABLE_REQUEST,
    ]
    .into_iter()
    .chain(known_functions().iter().filter(|function| {
        function.kind == FunctionKind::Constructor
            && function.trait_contract == FunctionTraitContract::Arbitrary
            && function.effects.is_empty()
            && !function.is_async()
            && function.input_shape.is_shape(ArbitraryBytes::SHAPE)
            && (function
                .output_shape
                .is_shape(Arc::<Vec<Arc<AzureDevOpsProject>>>::SHAPE)
                || function
                    .output_shape
                    .is_shape(Arc::<Vec<Arc<AzureDevOpsBuildFolder>>>::SHAPE))
    }))
    .collect::<Vec<_>>();
    let plan = plan(
        catalog.iter().map(|function| function.descriptor()),
        &[
            AzureDevOpsOrganizationUrl::SHAPE,
            AzureDevOpsAuthSelection::SHAPE,
        ],
        &[AzureDevOpsBuildFolder::SHAPE],
    );
    let simulation = CancellationToken::new()
        .with_scope(|cancellation| {
            DiscoverySimulation::run(
                vec![
                    RuntimeValue::from_box(Box::new(
                        "example".parse::<AzureDevOpsOrganizationUrl>()?,
                    ))?,
                    RuntimeValue::from_box(Box::new(
                        AzureDevOpsAuthSelection::PersonalAccessToken {
                            account_id: "11111111-1111-1111-1111-111111111111".parse()?,
                        },
                    ))?,
                ],
                catalog.iter().copied(),
                7,
                32,
                256,
                &cancellation,
            )
        })
        .unwrap();

    assert!(simulation.converged, "{:?}", simulation.diagnostics);
    assert!(
        simulation
            .values
            .contains_key(AzureDevOpsBuildFolder::SHAPE)
    );
    assert_eq!(simulation.simulated_requests.len(), 2);
    assert!(plan.nodes.iter().all(|node| node.reachable));
    for request in simulation.simulated_requests {
        assert!(plan.nodes.iter().any(|node| {
            node.kind == DiscoveryPlanNodeKind::Request
                && node.label == describe_shape(request.input_shape)
        }));
    }
}

#[test]
fn executable_discovery_requires_credential_selection_and_traverses_shared_records() {
    let catalog = [
        AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
        AzureDevOpsProject::REGISTERED_INTO_PROJECT_ARGUMENT.descriptor(),
        ProjectFoldersRequest::REGISTERED_EXECUTABLE_REQUEST.descriptor(),
    ];
    let blocked = plan(
        catalog,
        &[AzureDevOpsOrganizationUrl::SHAPE],
        &[AzureDevOpsBuildFolder::SHAPE],
    );
    assert!(
        blocked
            .nodes
            .iter()
            .filter(|node| node.target)
            .all(|node| !node.reachable)
    );
    assert!(blocked.nodes.iter().any(|node| {
        node.missing_inputs
            .iter()
            .any(|input| input == "auth_selection: AzureDevOpsAuthSelection")
    }));

    let reachable = plan(
        catalog,
        &[
            AzureDevOpsOrganizationUrl::SHAPE,
            AzureDevOpsAuthSelection::SHAPE,
        ],
        &[AzureDevOpsBuildFolder::SHAPE],
    );
    assert!(reachable.nodes.iter().all(|node| node.reachable));
    assert!(reachable.edges.iter().any(|edge| edge.label == "pointee"));
    assert!(
        reachable
            .edges
            .iter()
            .any(|edge| edge.label == "element (if present)")
    );
    assert!(
        !reachable
            .nodes
            .iter()
            .any(|node| node.label.contains("AuthContext"))
    );
}
