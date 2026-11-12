#![cfg(test)]

use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectPickRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::FunctionTraitContract;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry_execution::DiscoverySimulation;
use cloud_terrastodon_registry_execution::Function;
use cloud_terrastodon_registry_execution::known_functions;
use discovery_requests::ProjectFoldersRequest;
use facet::Facet;
use std::sync::Arc;

#[path = "support/discovery_requests.rs"]
mod discovery_requests;

fn fabricated_seed(seed: u64) -> AzureDevOpsProjectsListRequest<'static> {
    let bytes = vec![seed as u8; 16_384];
    let mut request =
        AzureDevOpsProjectsListRequest::arbitrary(&mut Unstructured::new(&bytes)).unwrap();
    // Discovery uses serializable credential selection without resolving credentials
    // or invoking requests.
    request.auth_selection = AzureDevOpsAuthSelection::PersonalAccessToken {
        account_id: "11111111-1111-1111-1111-111111111111".parse().unwrap(),
    };
    request
}

fn discovered<T: Facet<'static>>(simulation: &DiscoverySimulation) -> Option<&T> {
    simulation
        .values
        .get(T::SHAPE)
        .and_then(|values| values.first())
        .map(|value| value.value.peek().get::<T>().unwrap())
}

#[test]
fn fabricated_project_inventory_unlocks_project_scoped_folder_discovery() {
    CancellationToken::new()
        .with_scope(|cancellation| {
            for seed in [0, 1, 7, 42] {
                let request = fabricated_seed(seed);
                let organization = request.org_url.clone();
                let simulation = DiscoverySimulation::run(
                    vec![RuntimeValue::from_box(Box::new(request)).unwrap()],
                    [
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
                    })),
                    seed,
                    32,
                    256,
                    &cancellation,
                )
                .unwrap();
                assert!(
                    simulation.converged,
                    "seed {seed}: {:?}",
                    simulation.diagnostics
                );
                assert!(
                    simulation.rounds > 1,
                    "discovery must cross the project output boundary"
                );

                let project = discovered::<AzureDevOpsProject>(&simulation).unwrap_or_else(|| {
                    panic!("seed {seed}: no project; {:?}", simulation.diagnostics)
                });
                let argument = discovered::<AzureDevOpsProjectArgument<'static>>(&simulation)
                    .unwrap_or_else(|| {
                        panic!(
                            "seed {seed}: no project argument; {:?}",
                            simulation.diagnostics
                        )
                    });
                assert_eq!(argument, &AzureDevOpsProjectArgument::from(project.id));

                let folders_request = discovered::<ProjectFoldersRequest>(&simulation)
                    .unwrap_or_else(|| {
                        panic!(
                            "seed {seed}: no folder request; {:?}",
                            simulation.diagnostics
                        )
                    });
                assert_eq!(&folders_request.org_url, organization.as_ref());
                assert_eq!(&folders_request.project, argument);
                assert!(matches!(
                    folders_request.auth_selection,
                    AzureDevOpsAuthSelection::PersonalAccessToken { .. }
                ));
                assert_eq!(folders_request.path, None);
                assert!(
                    discovered::<AzureDevOpsBuildFolder>(&simulation).is_some(),
                    "seed {seed}: no folder record; {:?}",
                    simulation.diagnostics
                );
                assert_eq!(simulation.simulated_requests.len(), 2);
                assert!(simulation.simulated_requests.iter().any(|function| {
                    function
                        .input_shape
                        .is_shape(AzureDevOpsProjectsListRequest::SHAPE)
                }));
                assert!(simulation.simulated_requests.iter().any(|function| {
                    function.input_shape.is_shape(ProjectFoldersRequest::SHAPE)
                }));
            }
            eyre::Ok(())
        })
        .unwrap();
}

#[test]
fn missing_project_conversion_blocks_folder_discovery_without_arbitrary_shortcuts() {
    CancellationToken::new()
        .with_scope(|cancellation| {
            let simulation = DiscoverySimulation::run(
                vec![RuntimeValue::from_box(Box::new(fabricated_seed(7))).unwrap()],
                [
                    AzureDevOpsProjectsListRequest::REGISTERED_EXECUTABLE_REQUEST,
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
                })),
                7,
                32,
                256,
                &cancellation,
            )
            .unwrap();

            assert!(simulation.converged, "{:?}", simulation.diagnostics);
            assert!(discovered::<AzureDevOpsProject>(&simulation).is_some());
            assert!(discovered::<AzureDevOpsProjectArgument<'static>>(&simulation).is_none());
            assert!(discovered::<ProjectFoldersRequest>(&simulation).is_none());
            assert!(discovered::<AzureDevOpsBuildFolder>(&simulation).is_none());
            assert_eq!(simulation.simulated_requests.len(), 1);
            eyre::Ok(())
        })
        .unwrap();
}

#[test]
fn bounded_simulation_reports_why_discovery_stopped_before_convergence() {
    CancellationToken::new()
        .with_scope(|cancellation| {
            let simulation = DiscoverySimulation::run(
                vec![RuntimeValue::from_box(Box::new(fabricated_seed(7))).unwrap()],
                [
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
                })),
                7,
                1,
                256,
                &cancellation,
            )
            .unwrap();

            assert!(!simulation.converged);
            assert_eq!(simulation.rounds, 1);
            assert!(!simulation.diagnostics.is_empty());
            eyre::Ok(())
        })
        .unwrap();
}

#[test]
fn full_linked_catalog_reaches_project_scoped_folders_and_reports_type_coverage() {
    CancellationToken::new()
        .with_scope(|cancellation| {
            let simulation = DiscoverySimulation::run(
                vec![RuntimeValue::from_box(Box::new(fabricated_seed(7))).unwrap()],
                known_functions(),
                7,
                64,
                4096,
                &cancellation,
            )?;

            assert!(simulation.converged, "{:?}", simulation.diagnostics);
            assert!(
                !simulation.simulated_requests.iter().any(|function| {
                    function.input_shape.is_shape(AzureDevOpsProjectPickRequest::SHAPE)
                }),
                "unattended discovery must obtain projects without soliciting a selection",
            );
            assert!(
                discovered::<AzureDevOpsBuildFolder>(&simulation).is_some(),
                "{:?}",
                simulation.diagnostics
            );

            let maximum_depth = simulation
                .values
                .values()
                .flatten()
                .map(|value| value.depth)
                .max()
                .unwrap_or(0);
            println!(
                "Offline discovery: {} simulated requests, {} values, {} rounds, maximum depth {}, {} diagnostics",
                simulation.simulated_requests.len(),
                simulation.values.values().map(Vec::len).sum::<usize>(),
                simulation.rounds,
                maximum_depth,
                simulation.diagnostics.len()
            );
            let mut reached_shapes = simulation
                .values
                .values()
                .flatten()
                .map(|value| (describe_shape(value.value.peek().shape()), value.depth))
                .collect::<Vec<_>>();
            reached_shapes.sort_by(|left, right| left.0.cmp(&right.0));
            for (shape, depth) in reached_shapes {
                println!("{shape} (depth {depth})");
            }
            for diagnostic in &simulation.diagnostics {
                println!("Diagnostic: {diagnostic}");
            }
            eyre::Ok(())
        })
        .unwrap();
}
