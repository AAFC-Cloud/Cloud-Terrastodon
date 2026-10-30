use super::arena_query_context::ArenaQueryContext;
use super::card_row_snapshot::CardRowContent;
use super::explorer_command::OwnedValuePacket;
use super::explorer_engine::ExplorerEngine;
use super::field_candidate_action::FieldCandidateAction;
use super::invocation_mode::InvocationMode;
use super::value_address::ValueAddress;
use super::value_builder::BuilderTransition;
use facet::Facet;
use std::time::Duration;

#[derive(Facet)]
#[repr(C)]
struct NonCloneMenuInput {
    value: String,
}

impl From<NonCloneMenuInput> for usize {
    fn from(_: NonCloneMenuInput) -> Self {
        panic!("observing invocation choices must not execute a conversion")
    }
}

cloud_terrastodon_registry::register_from!(NonCloneMenuInput => usize);

#[derive(Facet)]
#[facet(traits(Clone))]
#[repr(C)]
struct PanickingCloneMenuInput {
    value: String,
}

impl Clone for PanickingCloneMenuInput {
    fn clone(&self) -> Self {
        panic!("menu previews must inspect clone metadata without calling user Clone")
    }
}

impl From<PanickingCloneMenuInput> for usize {
    fn from(_: PanickingCloneMenuInput) -> Self {
        panic!("observing invocation choices must not execute a conversion")
    }
}

cloud_terrastodon_registry::register_from!(PanickingCloneMenuInput => usize);

#[derive(Facet)]
#[repr(C)]
struct MenuDestination<T> {
    source: T,
}

async fn preview_actions<T: Facet<'static> + Send + 'static>(
    value: T,
) -> eyre::Result<(Vec<FieldCandidateAction>, Vec<InvocationMode>)> {
    let engine = ExplorerEngine::empty();
    let (context, inbox) = ArenaQueryContext::channel(4);
    let client = async {
        let handle = context.engine_handle();
        let source = handle.insert_ready(OwnedValuePacket::new(value)).await?;
        let (destination, state) = handle.create_builder(MenuDestination::<T>::SHAPE).await?;
        assert_eq!(state, BuilderTransition::Building);
        let actions = handle
            .inspect_field_candidate(destination, 0, ValueAddress::root(source))
            .await?
            .consequences()
            .iter()
            .map(|choice| choice.action())
            .collect::<Vec<_>>();
        let modes = handle
            .inspect_root(source, 16)
            .await?
            .card()
            .rows()
            .iter()
            .filter_map(|row| match row.content() {
                CardRowContent::RootAction(action) => action.invocation().map(|(_, mode)| mode),
                _ => None,
            })
            .collect::<Vec<_>>();
        drop(handle);
        drop(context);
        Ok::<_, eyre::Report>((source, destination, actions, modes))
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
    .expect("local action previews must finish without invoking user operations");
    let (source, destination, actions, modes) = result?;
    assert!(engine.arena().ready_value(source).is_some());
    assert!(engine.builders().builder(destination).is_some());
    assert_eq!(
        engine.arena().allocated_slot_count(),
        2,
        "previews allocate no value copies"
    );
    assert_eq!(engine.borrow_graph().edge_count(), 0);
    Ok((actions, modes))
}

#[tokio::test]
async fn noncloneable_values_keep_move_and_consume_without_offering_clone_or_retain()
-> eyre::Result<()> {
    let (actions, modes) = preview_actions(NonCloneMenuInput {
        value: "owned".into(),
    })
    .await?;
    assert_eq!(actions, [FieldCandidateAction::Move]);
    assert_eq!(modes, [InvocationMode::Consume]);
    Ok(())
}

#[tokio::test]
async fn clone_and_retain_previews_never_call_user_clone() -> eyre::Result<()> {
    let (actions, modes) = preview_actions(PanickingCloneMenuInput {
        value: "owned".into(),
    })
    .await?;
    assert_eq!(
        actions,
        [FieldCandidateAction::Move, FieldCandidateAction::Clone]
    );
    assert_eq!(modes, [InvocationMode::Consume, InvocationMode::Retain]);
    Ok(())
}
