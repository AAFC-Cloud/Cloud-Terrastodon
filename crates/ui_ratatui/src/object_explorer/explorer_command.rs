use std::error::Error;
use std::fmt;

use std::any::Any;

use cloud_terrastodon_registry::{
    Function, RuntimeFromBoxedFn, RuntimeValue, Thing, runtime_from_boxed,
};
use facet::{Facet, Shape};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use super::arena_query_command::ArenaQueryCommand;
use super::breadcrumb_context_snapshot::{
    BreadcrumbContextSnapshot, BreadcrumbContextValueSnapshot,
};
use super::breadcrumbs::Breadcrumbs;
use super::browse_command::BrowseCommand;
use super::field_binding::FieldBinding;
use super::field_binding_error::FieldBindingError;
use super::field_candidate_action::FieldCandidateActions;
use super::invocation_controller::{ArbitraryInvocationStart, InvocationEvent, InvocationStart};
use super::invocation_host::InvocationId;
use super::invocation_mode::InvocationMode;
use super::production_job::{ProductionBatch, ProductionStrategy};
use super::root_snapshot::RootSnapshot;
use super::slot_id::SlotId;
use super::tab::Tab;
use super::tab_update::TabUpdate;
use super::value_address::ValueAddress;
use super::value_builder::BuilderTransition;

pub(crate) type ExplorerCommandResponse<T> = Result<T, String>;

/// Send-safe output waiting to be ingested by the single-owner engine.
///
/// RuntimeValue itself contains engine-local reflected pointers and is not
/// Send. Registered invocations already return Box<dyn Any + Send>; retaining
/// the matching conversion function lets the engine construct RuntimeValue at
/// the exact linearization point.
pub(crate) struct OwnedValuePacket {
    value: Box<dyn Any + Send>,
    into_runtime: RuntimeFromBoxedFn,
}

impl OwnedValuePacket {
    pub(crate) fn new<T>(value: T) -> Self
    where
        T: Facet<'static> + Any + Send + 'static,
    {
        Self {
            value: Box::new(value),
            into_runtime: runtime_from_boxed::<T>,
        }
    }

    pub(crate) fn from_invocation(
        value: Box<dyn Any + Send>,
        into_runtime: RuntimeFromBoxedFn,
    ) -> Self {
        Self {
            value,
            into_runtime,
        }
    }

    pub(crate) fn into_runtime(self) -> eyre::Result<RuntimeValue> {
        (self.into_runtime)(self.value)
    }
}

/// Send-safe field intent converted to engine-local FieldBinding at ingestion.
pub(crate) enum FieldBindingPacket {
    Unset,
    Default,
    InlineOwned(OwnedValuePacket),
    CloneFrom(ValueAddress),
    MoveFrom(SlotId),
    BorrowFrom(ValueAddress),
    PendingProducer,
}

impl FieldBindingPacket {
    pub(crate) fn move_from_address(address: ValueAddress) -> Result<Self, FieldBindingError> {
        if !address.path().segments().is_empty() {
            return Err(FieldBindingError::NestedMoveSource(address));
        }
        Ok(Self::MoveFrom(address.root_id()))
    }

    pub(crate) fn into_binding(self) -> eyre::Result<FieldBinding> {
        Ok(match self {
            Self::Unset => FieldBinding::Unset,
            Self::Default => FieldBinding::Default,
            Self::InlineOwned(value) => FieldBinding::InlineOwned(value.into_runtime()?),
            Self::CloneFrom(address) => FieldBinding::CloneFrom(address),
            Self::MoveFrom(slot) => FieldBinding::MoveFrom(slot),
            Self::BorrowFrom(address) => FieldBinding::BorrowFrom(address),
            Self::PendingProducer => FieldBinding::PendingProducer,
        })
    }
}

/// One FIFO command stream is the engine's linearization order.
///
/// Query commands are value-free. Mutation commands may carry an owned
/// RuntimeValue because accepting one is precisely the engine's ingestion
/// boundary; no borrowed Facet value crosses this channel.
pub(crate) enum ExplorerCommand {
    Query(ArenaQueryCommand),
    Browse(BrowseCommand),
    Mutation(ArenaMutationCommand),
    Read(ArenaReadCommand),
}

pub(crate) enum ArenaMutationCommand {
    ReserveBuilder {
        response: oneshot::Sender<ExplorerCommandResponse<SlotId>>,
    },
    SetBuilderShape {
        slot: SlotId,
        shape: &'static Shape,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    CreateBuilder {
        shape: &'static Shape,
        response: oneshot::Sender<ExplorerCommandResponse<(SlotId, BuilderTransition)>>,
    },
    SetBuilderField {
        slot: SlotId,
        field: usize,
        binding: FieldBindingPacket,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    UnsetBuilderField {
        slot: SlotId,
        field: usize,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    CompleteBuilderField {
        slot: SlotId,
        field: usize,
        binding: FieldBindingPacket,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    SelectBuilderVariant {
        slot: SlotId,
        variant: usize,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    SetBuilderScalar {
        slot: SlotId,
        value: OwnedValuePacket,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    SetBuilderScalarText {
        slot: SlotId,
        text: String,
        response: oneshot::Sender<ExplorerCommandResponse<BuilderTransition>>,
    },
    Invoke {
        input: SlotId,
        input_thing: &'static Thing,
        function: &'static Function,
        mode: InvocationMode,
        response: oneshot::Sender<ExplorerCommandResponse<InvocationStart>>,
    },
    InvokeArbitrary {
        request: SlotId,
        request_function: &'static Function,
        constructor: &'static Function,
        bytes: Vec<u8>,
        response: oneshot::Sender<ExplorerCommandResponse<ArbitraryInvocationStart>>,
    },
    PollInvocations {
        response: oneshot::Sender<ExplorerCommandResponse<Vec<InvocationEvent>>>,
    },
    StartProduction {
        destination: SlotId,
        field: usize,
        function: &'static Function,
        strategy: ProductionStrategy,
        max_work: usize,
        response: oneshot::Sender<ExplorerCommandResponse<ProductionBatch>>,
    },
    AdvanceProductions {
        max_work: usize,
        response: oneshot::Sender<ExplorerCommandResponse<ProductionBatch>>,
    },
    UpdateTab {
        slot: SlotId,
        update: TabUpdate,
        response: oneshot::Sender<ExplorerCommandResponse<Tab>>,
    },
    CancelInvocation {
        invocation: InvocationId,
        response: oneshot::Sender<ExplorerCommandResponse<InvocationEvent>>,
    },
    InsertReady {
        value: OwnedValuePacket,
        response: oneshot::Sender<ExplorerCommandResponse<SlotId>>,
    },
    SetReady {
        slot: SlotId,
        value: OwnedValuePacket,
        response: oneshot::Sender<ExplorerCommandResponse<()>>,
    },
    Delete {
        slot: SlotId,
        response: oneshot::Sender<ExplorerCommandResponse<()>>,
    },
}

pub(crate) enum ArenaReadCommand {
    ResolveJson {
        address: ValueAddress,
        response: oneshot::Sender<ExplorerCommandResponse<String>>,
    },
    InspectFieldCandidate {
        destination: SlotId,
        field: usize,
        source: ValueAddress,
        response: oneshot::Sender<ExplorerCommandResponse<FieldCandidateActions>>,
    },
    InspectRoot {
        slot: SlotId,
        max_relationship_rows: usize,
        response: oneshot::Sender<ExplorerCommandResponse<RootSnapshot>>,
    },
    InspectTab {
        slot: SlotId,
        response: oneshot::Sender<ExplorerCommandResponse<Tab>>,
    },
    InspectBreadcrumbContext {
        breadcrumbs: Breadcrumbs,
        max_work: usize,
        max_choices: usize,
        response: oneshot::Sender<ExplorerCommandResponse<BreadcrumbContextSnapshot>>,
    },
    InspectBreadcrumbValues {
        breadcrumbs: Breadcrumbs,
        field_shape: String,
        field_name: String,
        max_work: usize,
        max_choices: usize,
        response: oneshot::Sender<ExplorerCommandResponse<BreadcrumbContextValueSnapshot>>,
    },
}

pub(crate) struct ExplorerInbox {
    commands: mpsc::Receiver<ExplorerCommand>,
}

impl ExplorerInbox {
    pub(crate) async fn recv(&mut self) -> Option<ExplorerCommand> {
        self.commands.recv().await
    }
}

#[derive(Clone)]
pub(crate) struct ExplorerHandle {
    commands: mpsc::Sender<ExplorerCommand>,
}

pub(crate) struct MutationReceipt<T> {
    response: oneshot::Receiver<ExplorerCommandResponse<T>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExplorerHandleError {
    EngineStopped,
    ResponseDropped,
    Rejected(String),
}

impl fmt::Display for ExplorerHandleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EngineStopped => write!(formatter, "the explorer engine has stopped"),
            Self::ResponseDropped => write!(formatter, "the explorer engine dropped a response"),
            Self::Rejected(message) => write!(
                formatter,
                "the explorer engine rejected the command: {message}"
            ),
        }
    }
}

impl Error for ExplorerHandleError {}

pub(crate) fn explorer_channel(capacity: usize) -> (mpsc::Sender<ExplorerCommand>, ExplorerInbox) {
    assert!(capacity > 0, "Explorer command channel must be bounded");
    let (commands, receiver) = mpsc::channel(capacity);
    (commands, ExplorerInbox { commands: receiver })
}

impl ExplorerHandle {
    pub(crate) async fn reserve_builder(&self) -> Result<SlotId, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::ReserveBuilder { response },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn set_builder_shape(
        &self,
        slot: SlotId,
        shape: &'static Shape,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::SetBuilderShape {
                slot,
                shape,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn create_builder(
        &self,
        shape: &'static Shape,
    ) -> Result<(SlotId, BuilderTransition), ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::CreateBuilder { shape, response },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn set_builder_field(
        &self,
        slot: SlotId,
        field: usize,
        binding: FieldBindingPacket,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::SetBuilderField {
                slot,
                field,
                binding,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn unset_builder_field(
        &self,
        slot: SlotId,
        field: usize,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::UnsetBuilderField {
                slot,
                field,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn complete_builder_field(
        &self,
        slot: SlotId,
        field: usize,
        binding: FieldBindingPacket,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::CompleteBuilderField {
                slot,
                field,
                binding,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn select_builder_variant(
        &self,
        slot: SlotId,
        variant: usize,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::SelectBuilderVariant {
                slot,
                variant,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn set_builder_scalar(
        &self,
        slot: SlotId,
        value: OwnedValuePacket,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::SetBuilderScalar {
                slot,
                value,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn set_builder_scalar_text(
        &self,
        slot: SlotId,
        text: String,
    ) -> Result<BuilderTransition, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::SetBuilderScalarText {
                slot,
                text,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn invoke(
        &self,
        input: SlotId,
        input_thing: &'static Thing,
        function: &'static Function,
        mode: InvocationMode,
    ) -> Result<InvocationStart, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(ArenaMutationCommand::Invoke {
            input,
            input_thing,
            function,
            mode,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn invoke_arbitrary(
        &self,
        request: SlotId,
        request_function: &'static Function,
        constructor: &'static Function,
        bytes: Vec<u8>,
    ) -> Result<ArbitraryInvocationStart, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::InvokeArbitrary {
                request,
                request_function,
                constructor,
                bytes,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn poll_invocations(
        &self,
    ) -> Result<Vec<InvocationEvent>, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::PollInvocations { response },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn start_production(
        &self,
        destination: SlotId,
        field: usize,
        function: &'static Function,
        strategy: ProductionStrategy,
        max_work: usize,
    ) -> Result<ProductionBatch, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::StartProduction {
                destination,
                field,
                function,
                strategy,
                max_work,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn advance_productions(
        &self,
        max_work: usize,
    ) -> Result<ProductionBatch, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::AdvanceProductions { max_work, response },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn update_tab(
        &self,
        slot: SlotId,
        update: TabUpdate,
    ) -> Result<Tab, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(ArenaMutationCommand::UpdateTab {
            slot,
            update,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn cancel_invocation(
        &self,
        invocation: InvocationId,
    ) -> Result<InvocationEvent, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::CancelInvocation {
                invocation,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) fn from_sender(commands: mpsc::Sender<ExplorerCommand>) -> Self {
        Self { commands }
    }

    pub(crate) async fn insert_ready(
        &self,
        value: OwnedValuePacket,
    ) -> Result<SlotId, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(
            ArenaMutationCommand::InsertReady { value, response },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn set_ready(
        &self,
        slot: SlotId,
        value: OwnedValuePacket,
    ) -> Result<(), ExplorerHandleError> {
        self.submit_set_ready(slot, value).await?.wait().await
    }

    pub(crate) async fn submit_set_ready(
        &self,
        slot: SlotId,
        value: OwnedValuePacket,
    ) -> Result<MutationReceipt<()>, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(ArenaMutationCommand::SetReady {
            slot,
            value,
            response,
        }))
        .await?;
        Ok(MutationReceipt { response: receiver })
    }

    pub(crate) async fn delete(&self, slot: SlotId) -> Result<(), ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Mutation(ArenaMutationCommand::Delete {
            slot,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn resolve_json(
        &self,
        address: ValueAddress,
    ) -> Result<String, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(ArenaReadCommand::ResolveJson {
            address,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn inspect_field_candidate(
        &self,
        destination: SlotId,
        field: usize,
        source: ValueAddress,
    ) -> Result<FieldCandidateActions, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(
            ArenaReadCommand::InspectFieldCandidate {
                destination,
                field,
                source,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn inspect_root(
        &self,
        slot: SlotId,
        max_relationship_rows: usize,
    ) -> Result<RootSnapshot, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(ArenaReadCommand::InspectRoot {
            slot,
            max_relationship_rows,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn inspect_tab(&self, slot: SlotId) -> Result<Tab, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(ArenaReadCommand::InspectTab {
            slot,
            response,
        }))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn inspect_breadcrumb_context(
        &self,
        breadcrumbs: Breadcrumbs,
        max_work: usize,
        max_choices: usize,
    ) -> Result<BreadcrumbContextSnapshot, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(
            ArenaReadCommand::InspectBreadcrumbContext {
                breadcrumbs,
                max_work,
                max_choices,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    pub(crate) async fn inspect_breadcrumb_values(
        &self,
        breadcrumbs: Breadcrumbs,
        field_shape: String,
        field_name: String,
        max_work: usize,
        max_choices: usize,
    ) -> Result<BreadcrumbContextValueSnapshot, ExplorerHandleError> {
        let (response, receiver) = oneshot::channel();
        self.send(ExplorerCommand::Read(
            ArenaReadCommand::InspectBreadcrumbValues {
                breadcrumbs,
                field_shape,
                field_name,
                max_work,
                max_choices,
                response,
            },
        ))
        .await?;
        Self::receive(receiver).await
    }

    async fn send(&self, command: ExplorerCommand) -> Result<(), ExplorerHandleError> {
        self.commands
            .send(command)
            .await
            .map_err(|_| ExplorerHandleError::EngineStopped)
    }

    async fn receive<T>(
        receiver: oneshot::Receiver<ExplorerCommandResponse<T>>,
    ) -> Result<T, ExplorerHandleError> {
        receiver
            .await
            .map_err(|_| ExplorerHandleError::ResponseDropped)?
            .map_err(ExplorerHandleError::Rejected)
    }
}

impl<T> MutationReceipt<T> {
    pub(crate) fn try_result(&mut self) -> Result<Option<T>, ExplorerHandleError> {
        match self.response.try_recv() {
            Ok(result) => result.map(Some).map_err(ExplorerHandleError::Rejected),
            Err(oneshot::error::TryRecvError::Empty) => Ok(None),
            Err(oneshot::error::TryRecvError::Closed) => Err(ExplorerHandleError::ResponseDropped),
        }
    }

    pub(crate) async fn wait(self) -> Result<T, ExplorerHandleError> {
        ExplorerHandle::receive(self.response).await
    }
}
