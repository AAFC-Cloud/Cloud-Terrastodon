use super::arena_query_context::ArenaQueryContext;
use super::breadcrumb::Breadcrumb;
use super::breadcrumb::ProjectFieldsMode;
use super::breadcrumb::ValueFilterOperator;
use super::breadcrumbs::Breadcrumbs;
use super::explorer_command::FieldBindingPacket;
use super::explorer_command::OwnedValuePacket;
use super::explorer_engine::ExplorerEngine;
use super::field_candidate_action::FieldCandidateAction;
use super::projected_field::ProjectedField;
use super::tab::Tab;
use super::value_address::ValueAddress;
use super::value_builder::BuilderTransition;
use super::value_path::ValuePathSegment;
use facet::Facet;
use std::time::Duration;

#[tokio::test]
async fn cloning_a_tabs_projected_operations_builds_an_independent_populated_query()
-> eyre::Result<()> {
    let expected = Breadcrumbs::new(vec![
        Breadcrumb::projection(
            17,
            vec![
                ValuePathSegment::Field("projects".into()),
                ValuePathSegment::Index(2),
                ValuePathSegment::Key("project with spaces".into()),
            ],
        ),
        Breadcrumb::ShapeFilter {
            included_shapes: vec!["ExampleProject".into(), "ExampleRepository".into()],
        },
        Breadcrumb::ValueFilter {
            field_shape: "String".into(),
            field_name: "displayName".into(),
            operator: ValueFilterOperator::Contains,
            value: "Équipe example".into(),
        },
        Breadcrumb::ProjectFields {
            mode: ProjectFieldsMode::Extend,
            included_fields: vec![
                ProjectedField::new("ExampleProject", "name"),
                ProjectedField::new("ExampleProject", "description"),
            ],
        },
    ]);
    let engine = ExplorerEngine::empty();
    let (context, inbox) = ArenaQueryContext::channel(4);
    let client = async {
        let handle = context.engine_handle();
        let source = handle
            .insert_ready(OwnedValuePacket::new(Tab::new(
                "source query",
                expected.clone(),
            )))
            .await?;
        let (destination, state) = handle.create_builder(Breadcrumbs::SHAPE).await?;
        assert_eq!(state, BuilderTransition::Building);
        let snapshot = handle.inspect_root(destination, 8).await?;
        let operations_field = snapshot
            .builder()
            .expect("operations has not been supplied")
            .fields()
            .iter()
            .find(|field| field.name() == "operations")
            .unwrap()
            .index();
        let projected_operations = ValueAddress::root(source)
            .child(ValuePathSegment::Field("breadcrumbs".into()))
            .child(ValuePathSegment::Field("operations".into()));
        let actions = handle
            .inspect_field_candidate(destination, operations_field, projected_operations.clone())
            .await?;
        assert!(
            actions
                .consequences()
                .iter()
                .any(|choice| { choice.action() == FieldCandidateAction::Clone })
        );
        assert!(
            !actions
                .consequences()
                .iter()
                .any(|choice| { choice.action() == FieldCandidateAction::Move }),
            "a projected list has no independent root to move"
        );
        assert_eq!(
            handle
                .set_builder_field(
                    destination,
                    operations_field,
                    FieldBindingPacket::CloneFrom(projected_operations),
                )
                .await?,
            BuilderTransition::Ready,
            "the ordinary Clone field action must materialize the reflected list"
        );
        drop(handle);
        drop(context);
        Ok::<_, eyre::Report>((source, destination))
    };
    let (engine, result) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        )
    })
    .await
    .expect("cloning a local projected list must finish");
    let (source, destination) = result?;
    let retained = engine
        .arena()
        .ready_value(source)
        .unwrap()
        .peek()
        .get::<Tab>()?;
    assert_eq!(retained.name(), "source query");
    assert_eq!(
        retained.breadcrumbs(),
        &expected,
        "Clone retains the source and every nested field"
    );
    let cloned = engine
        .arena()
        .ready_value(destination)
        .unwrap()
        .peek()
        .get::<Breadcrumbs>()?;
    assert_eq!(cloned, &expected);
    assert_ne!(
        retained.breadcrumbs().operations().as_ptr(),
        cloned.operations().as_ptr(),
        "the destination owns a separate list allocation"
    );
    Ok(())
}
