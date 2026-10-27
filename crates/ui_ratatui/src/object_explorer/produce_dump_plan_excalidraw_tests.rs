use super::arena_query_context::ArenaQueryContext;
use super::arena_slot_state::ArenaSlotState;
use super::breadcrumbs::Breadcrumbs;
use super::explorer_command::FieldBindingPacket;
use super::explorer_command::OwnedValuePacket;
use super::explorer_engine::ExplorerEngine;
use super::field_candidate_action::FieldCandidateAction;
use super::invocation_controller::InvocationEventState;
use super::invocation_mode::InvocationMode;
use super::query_progress::QueryProgressState;
use super::value_address::ValueAddress;
use super::value_builder::BuilderTransition;
use super::value_candidate_window::ValueCandidateWindowBudget;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_excalidraw::ProduceExcalidrawDumpPlanRequest;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::known_thing_for_shape;
use facet::Facet;
use std::future::Ready;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Facet)]
struct DiagramInput;

#[derive(Facet)]
struct DiagramInventory;

#[derive(Facet)]
struct DiagramInventoryRequest {
    input: DiagramInput,
}

impl IntoFuture for DiagramInventoryRequest {
    type Output = eyre::Result<DiagramInventory>;
    type IntoFuture = Ready<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        panic!("exporting the plan must never execute a planned operation")
    }
}

cloud_terrastodon_registry::register_into_future!(DiagramInventoryRequest => DiagramInventory, effects = [Read]);

// Observe only the native scene fields this integration produces. The typed
// writer's tagged/flattened element enum is not a general-purpose scene decoder.
#[derive(Facet)]
struct DrawingObservation {
    elements: Vec<ElementObservation>,
}

#[derive(Facet)]
struct ElementObservation {
    #[facet(rename = "type")]
    kind: String,
    #[facet(default)]
    text: Option<String>,
    #[facet(rename = "groupIds")]
    group_ids: Vec<String>,
}

#[tokio::test]
async fn explorer_picker_builds_and_invokes_plan_export_with_a_typed_filename() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(export_plan_through_explorer)
        .await
}

async fn export_plan_through_explorer(cancellation: CancellationScope) -> eyre::Result<()> {
    let directory = tempfile::tempdir()?;
    let filename = directory.path().join("chosen plan drawing.excalidraw");
    let plan = DiscoveryPlan::build(
        [DiagramInventoryRequest::REGISTERED_INTO_FUTURE],
        [DiagramInput::SHAPE],
        [DiagramInventory::SHAPE],
        &cancellation,
    )?;
    assert!(
        !plan.edges.is_empty(),
        "the fixture includes a real dependency chain"
    );
    let engine = ExplorerEngine::empty_with_tokio_invocation_host(|future| future, &cancellation);
    let (context, inbox) = ArenaQueryContext::channel(8);
    let client = async {
        let handle = context.engine_handle();
        let source_plan = handle.insert_ready(OwnedValuePacket::new(plan)).await?;
        let (request, state) = handle
            .create_builder(ProduceExcalidrawDumpPlanRequest::SHAPE)
            .await?;
        assert_eq!(state, BuilderTransition::Building);
        let snapshot = handle.inspect_root(request, 16).await?;
        let fields = snapshot
            .builder()
            .expect("request is still being constructed")
            .fields();
        let plan_field = fields
            .iter()
            .find(|field| field.name() == "plan")
            .unwrap()
            .index();
        let filename_field = fields
            .iter()
            .find(|field| field.name() == "filename")
            .unwrap()
            .index();
        assert_eq!(
            fields.len(),
            2,
            "cancellation is supplied by the invocation host"
        );

        let picker = context.open_browse(Breadcrumbs::default()).await?;
        picker.set_candidate_shape(DiscoveryPlan::SHAPE).await?;
        let candidate = loop {
            let progress = picker
                .fill_value_candidates(
                    None,
                    ValueCandidateWindowBudget::new(32, NonZeroUsize::new(4).unwrap()),
                )
                .await?;
            match progress.into_state() {
                QueryProgressState::Ready(window) => {
                    break window
                        .candidates()
                        .iter()
                        .find(|candidate| candidate.address() == &ValueAddress::root(source_plan))
                        .expect("the ordinary picker discovers the ready plan")
                        .address()
                        .clone();
                }
                QueryProgressState::Pending => {}
                state => panic!("plan picker did not produce a candidate: {state:?}"),
            }
        };
        let choices = handle
            .inspect_field_candidate(request, plan_field, candidate.clone())
            .await?;
        assert!(
            choices
                .consequences()
                .iter()
                .any(|choice| choice.action() == FieldCandidateAction::Clone)
        );
        assert_eq!(
            handle
                .set_builder_field(
                    request,
                    plan_field,
                    FieldBindingPacket::CloneFrom(candidate)
                )
                .await?,
            BuilderTransition::Building
        );

        // Exercise the same scalar-text command used by the terminal editor.
        // PathBuf's Facet parser accepts the path without JSON quoting.
        let (path_slot, state) = handle.create_builder(PathBuf::SHAPE).await?;
        assert_eq!(state, BuilderTransition::Building);
        assert_eq!(
            handle
                .set_builder_scalar_text(path_slot, filename.to_string_lossy().into_owned())
                .await?,
            BuilderTransition::Ready
        );
        let choices = handle
            .inspect_field_candidate(request, filename_field, ValueAddress::root(path_slot))
            .await?;
        assert!(
            choices
                .consequences()
                .iter()
                .any(|choice| choice.action() == FieldCandidateAction::Move)
        );
        assert_eq!(
            handle
                .set_builder_field(
                    request,
                    filename_field,
                    FieldBindingPacket::MoveFrom(path_slot)
                )
                .await?,
            BuilderTransition::Ready
        );
        picker.close().await?;

        let started = handle
            .invoke(
                request,
                known_thing_for_shape(ProduceExcalidrawDumpPlanRequest::SHAPE)
                    .expect("export request is registered"),
                ProduceExcalidrawDumpPlanRequest::REGISTERED_INTO_FUTURE,
                InvocationMode::Consume,
            )
            .await?;
        let output = started.output();
        loop {
            let events = handle.poll_invocations().await?;
            if let Some(event) = events.iter().find(|event| event.output == output) {
                assert_eq!(event.state, InvocationEventState::Ready, "{event:?}");
                break;
            }
            tokio::task::yield_now().await;
        }
        drop(handle);
        drop(context);
        Ok::<_, eyre::Report>((source_plan, path_slot, request, output))
    };
    let (engine, result) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::join!(engine.run(inbox), client)
    })
    .await
    .expect("the local export invocation must complete");
    let (source_plan, path_slot, request, output) = result?;
    assert!(
        engine.arena().ready_value(source_plan).is_some(),
        "the chosen clone action preserves the source plan"
    );
    assert!(matches!(
        engine.arena().slot(path_slot).unwrap().state(),
        ArenaSlotState::Consumed
    ));
    assert!(matches!(
        engine.arena().slot(request).unwrap().state(),
        ArenaSlotState::Consumed
    ));
    let result = engine
        .arena()
        .ready_value(output)
        .unwrap()
        .peek()
        .get::<PathBuf>()?;
    assert_eq!(
        std::fs::canonicalize(result)?,
        std::fs::canonicalize(&filename)?
    );
    let scene: DrawingObservation = facet_json::from_str(&std::fs::read_to_string(&filename)?)?;
    assert!(
        scene.elements.iter().any(|element| element.kind == "arrow"),
        "the registered invocation exports the dependency graph"
    );
    for expected in [
        "DiagramInput",
        "DiagramInventoryRequest",
        "DiagramInventory",
    ] {
        assert!(
            scene.elements.iter().any(|element| {
                element.kind == "text"
                    && element
                        .text
                        .as_ref()
                        .is_some_and(|text| text.contains(expected))
            }),
            "the graph includes {expected}"
        );
    }
    assert!(
        scene
            .elements
            .iter()
            .all(|element| element.group_ids.is_empty())
    );
    Ok(())
}
