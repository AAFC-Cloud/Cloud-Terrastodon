use super::arena::Arena;
use super::borrow_graph::BorrowGraph;
use super::invocation_host::InvocationHost;
use super::invocation_host::InvocationHostPoll;
use super::invocation_host::InvocationId;
use super::invocation_mode::InvocationMode;
use super::invocation_plan::InvocationPlan;
use super::invocation_plan::InvocationPlanId;
use super::invocation_plan::PlannedInvocation;
use super::slot_id::SlotId;
use super::value_builder::BuilderStore;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionInvocation;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::RuntimeFromBoxedFn;
use cloud_terrastodon_registry::RuntimeValue;
use cloud_terrastodon_registry::Thing;
use cloud_terrastodon_registry::known_thing_for_shape;
use facet::Facet;
use std::any::Any;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

/// Dispose arbitrary user panic payloads before source leases can be released.
/// A replacement payload raised by a panicking destructor has the same lifetime
/// obligation. Only copied, independently owned diagnostic text may escape.
fn consume_invocation_panic(mut payload: Box<dyn Any + Send>) -> String {
    let message = if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload".to_owned()
    };
    loop {
        match catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            Ok(()) => return message,
            Err(next) => payload = next,
        }
    }
}

/// Run user conversion/invocation/drop code while the caller still protects its
/// sources. Errors can themselves contain borrowed values, and their Display
/// and Drop implementations can panic, so normalize and dispose those here too.
fn guarded_user_operation<T>(
    context: &str,
    operation: impl FnOnce() -> eyre::Result<T>,
) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => {
            let message = match catch_unwind(AssertUnwindSafe(|| error.to_string())) {
                Ok(message) => format!("{context}: {message}"),
                Err(payload) => format!(
                    "{context}: error formatting panicked: {}",
                    consume_invocation_panic(payload)
                ),
            };
            match catch_unwind(AssertUnwindSafe(|| drop(error))) {
                Ok(()) => Err(message),
                Err(payload) => Err(format!(
                    "{context}: error destruction panicked: {}",
                    consume_invocation_panic(payload)
                )),
            }
        }
        Err(payload) => Err(format!(
            "{context} panicked: {}",
            consume_invocation_panic(payload)
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvocationControllerError {
    message: String,
}

impl InvocationControllerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for InvocationControllerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for InvocationControllerError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InvocationStart {
    Pending {
        invocation: InvocationId,
        output: SlotId,
    },
    Ready {
        output: SlotId,
    },
    Failed {
        output: SlotId,
        message: String,
    },
}

impl InvocationStart {
    pub(crate) const fn output(&self) -> SlotId {
        match *self {
            Self::Pending { output, .. } | Self::Ready { output } | Self::Failed { output, .. } => {
                output
            }
        }
    }
}

/// Provenance for a fake request response produced through the ordinary
/// invocation-plan machinery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArbitraryInvocationStart {
    request: SlotId,
    source: SlotId,
    plan: InvocationPlanId,
    invocation: InvocationStart,
}

impl ArbitraryInvocationStart {
    pub(crate) const fn request(&self) -> SlotId {
        self.request
    }

    pub(crate) const fn source(&self) -> SlotId {
        self.source
    }

    pub(crate) const fn plan_id(&self) -> u64 {
        self.plan.get()
    }

    pub(crate) const fn invocation(&self) -> &InvocationStart {
        &self.invocation
    }

    pub(crate) const fn output(&self) -> SlotId {
        self.invocation.output()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InvocationEventState {
    Ready,
    Failed(String),
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvocationEvent {
    pub(crate) invocation: InvocationId,
    pub(crate) input: SlotId,
    pub(crate) output: SlotId,
    pub(crate) state: InvocationEventState,
}

#[derive(Clone, Copy)]
struct PlanStepLink {
    plan: InvocationPlanId,
    step: usize,
}

struct PendingInvocation {
    input: SlotId,
    output: SlotId,
    output_to_runtime: RuntimeFromBoxedFn,
    plan_step: Option<PlanStepLink>,
}

#[derive(Default)]
pub(crate) struct InvocationController {
    next_invocation: u64,
    pending: BTreeMap<InvocationId, PendingInvocation>,
    next_plan: u64,
    plans: BTreeMap<InvocationPlanId, InvocationPlan>,
}

impl InvocationController {
    #[expect(
        clippy::too_many_arguments,
        reason = "invocation requires the arena, borrow graph, host, and operation metadata"
    )]
    pub(crate) fn invoke<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        input: SlotId,
        input_thing: &'static Thing,
        function: &'static Function,
        mode: InvocationMode,
    ) -> Result<InvocationStart, InvocationControllerError> {
        self.invoke_linked(
            arena,
            builders,
            borrow_graph,
            host,
            PlannedInvocation {
                input,
                input_thing,
                function,
                mode,
            },
            None,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "arbitrary invocation requires the arena, borrow graph, host, and operation metadata"
    )]
    pub(crate) fn invoke_arbitrary<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        request: SlotId,
        request_function: &'static Function,
        constructor: &'static Function,
        bytes: Vec<u8>,
    ) -> Result<ArbitraryInvocationStart, InvocationControllerError> {
        let request_value = arena.ready_value(request).ok_or_else(|| {
            InvocationControllerError::new(format!("request slot {request} is not Ready"))
        })?;
        if !request_value.shape().is_shape(request_function.input_shape) {
            return Err(InvocationControllerError::new(format!(
                "slot {request} does not contain the request function's input shape"
            )));
        }
        if !constructor.input_shape.is_shape(ArbitraryBytes::SHAPE)
            || constructor.production_kind(request_function.output_shape)
                != Some(ProductionKind::Exact)
        {
            return Err(InvocationControllerError::new(format!(
                "{} is not an exact ArbitraryBytes constructor for {}",
                constructor.label,
                cloud_terrastodon_registry::describe_shape(request_function.output_shape)
            )));
        }

        let source_value =
            RuntimeValue::from_box(Box::new(ArbitraryBytes::new(bytes))).map_err(|error| {
                InvocationControllerError::new(format!(
                    "could not create the arbitrary byte source: {error}"
                ))
            })?;
        let source = arena
            .insert_ready(source_value)
            .map_err(|error| InvocationControllerError::new(error.to_string()))?;
        let input_thing = known_thing_for_shape(ArbitraryBytes::SHAPE).ok_or_else(|| {
            InvocationControllerError::new("ArbitraryBytes has no registered runtime Thing")
        })?;
        let operation = PlannedInvocation {
            input: source,
            input_thing,
            function: constructor,
            mode: InvocationMode::Retain,
        };
        let plan = self.create_plan(
            format!(
                "invoke arbitrary {} for request slot {request}",
                cloud_terrastodon_registry::describe_shape(request_function.output_shape)
            ),
            [operation],
        );
        let start = match self.invoke_linked(
            arena,
            builders,
            borrow_graph,
            host,
            operation,
            Some(PlanStepLink { plan, step: 0 }),
        ) {
            Ok(start) => start,
            Err(error) => {
                self.plans
                    .get_mut(&plan)
                    .expect("new arbitrary plan remains indexed")
                    .mark_failed(0, error.to_string());
                return Err(error);
            }
        };
        match &start {
            InvocationStart::Pending { invocation, output } => self
                .plans
                .get_mut(&plan)
                .expect("new arbitrary plan remains indexed")
                .mark_waiting(0, *invocation, *output),
            InvocationStart::Ready { output } => self
                .plans
                .get_mut(&plan)
                .expect("new arbitrary plan remains indexed")
                .mark_complete(0, *output),
            InvocationStart::Failed { message, .. } => self
                .plans
                .get_mut(&plan)
                .expect("new arbitrary plan remains indexed")
                .mark_failed(0, message.clone()),
        }

        Ok(ArbitraryInvocationStart {
            request,
            source,
            plan,
            invocation: start,
        })
    }

    pub(crate) fn poll<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
    ) -> Vec<InvocationEvent> {
        let ids = self.pending.keys().copied().collect::<Vec<_>>();
        let mut events = Vec::new();
        for id in ids {
            let host_result = host.poll(id);
            if matches!(host_result, InvocationHostPoll::Pending) {
                continue;
            }
            let Some(pending) = self.pending.remove(&id) else {
                continue;
            };
            let state = match host_result {
                InvocationHostPoll::Pending => unreachable!(),
                InvocationHostPoll::Ready(output) => {
                    match guarded_user_operation("could not store invocation output", || {
                        (pending.output_to_runtime)(output)
                    }) {
                        Ok(value) => {
                            match guarded_user_operation(
                                "could not complete invocation output",
                                || {
                                    builders
                                        .complete_pending(
                                            arena,
                                            borrow_graph,
                                            pending.output,
                                            value,
                                        )
                                        .map_err(Into::into)
                                },
                            ) {
                                Ok(()) => InvocationEventState::Ready,
                                Err(message) => InvocationEventState::Failed(message),
                            }
                        }
                        Err(message) => {
                            let _ = builders.fail_pending(
                                arena,
                                borrow_graph,
                                pending.output,
                                message.clone(),
                            );
                            InvocationEventState::Failed(message)
                        }
                    }
                }
                InvocationHostPoll::Failed(message) => {
                    let _ =
                        builders.fail_pending(arena, borrow_graph, pending.output, message.clone());
                    InvocationEventState::Failed(message)
                }
                InvocationHostPoll::Cancelled => {
                    let _ = builders.cancel_pending(arena, borrow_graph, pending.output);
                    InvocationEventState::Cancelled
                }
            };
            self.settle_plan_step(pending.plan_step, pending.output, &state);
            events.push(InvocationEvent {
                invocation: id,
                input: pending.input,
                output: pending.output,
                state,
            });
        }
        events
    }

    pub(crate) fn cancel<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        invocation: InvocationId,
    ) -> Result<InvocationEvent, InvocationControllerError> {
        let pending = self
            .pending
            .remove(&invocation)
            .ok_or_else(|| InvocationControllerError::new("unknown pending invocation"))?;
        // The host must synchronously destroy its future and any unclaimed
        // result before cancel_pending releases the source leases below.
        host.cancel(invocation);
        builders
            .cancel_pending(arena, borrow_graph, pending.output)
            .map_err(|error| InvocationControllerError::new(error.to_string()))?;
        self.settle_plan_step(
            pending.plan_step,
            pending.output,
            &InvocationEventState::Cancelled,
        );
        Ok(InvocationEvent {
            invocation,
            input: pending.input,
            output: pending.output,
            state: InvocationEventState::Cancelled,
        })
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub(crate) fn has_ready<H: InvocationHost + ?Sized>(&self, host: &H) -> bool {
        self.pending.keys().any(|id| host.is_ready(*id))
    }

    pub(crate) fn create_plan(
        &mut self,
        title: impl Into<String>,
        operations: impl IntoIterator<Item = PlannedInvocation>,
    ) -> InvocationPlanId {
        let id = InvocationPlanId::new(self.next_plan);
        self.next_plan = self
            .next_plan
            .checked_add(1)
            .expect("invocation plan identity space exhausted");
        self.plans
            .insert(id, InvocationPlan::new(id, title, operations));
        id
    }

    pub(crate) fn plan(&self, id: InvocationPlanId) -> Option<&InvocationPlan> {
        self.plans.get(&id)
    }

    pub(crate) fn plan_count(&self) -> usize {
        self.plans.len()
    }

    pub(crate) fn advance_plan<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        plan: InvocationPlanId,
    ) -> Result<(), InvocationControllerError> {
        loop {
            let Some((step, operation)) = self
                .plans
                .get(&plan)
                .ok_or_else(|| InvocationControllerError::new("unknown invocation plan"))?
                .next_planned()
            else {
                return Ok(());
            };
            match self.invoke_linked(
                arena,
                builders,
                borrow_graph,
                host,
                operation,
                Some(PlanStepLink { plan, step }),
            ) {
                Ok(InvocationStart::Pending { invocation, output }) => {
                    self.plans
                        .get_mut(&plan)
                        .expect("plan remains indexed")
                        .mark_waiting(step, invocation, output);
                    return Ok(());
                }
                Ok(InvocationStart::Ready { output }) => {
                    self.plans
                        .get_mut(&plan)
                        .expect("plan remains indexed")
                        .mark_complete(step, output);
                }
                Ok(InvocationStart::Failed { message, .. })
                | Err(InvocationControllerError { message }) => {
                    self.plans
                        .get_mut(&plan)
                        .expect("plan remains indexed")
                        .mark_failed(step, message);
                    return Ok(());
                }
            }
        }
    }

    pub(crate) fn cancel_plan<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        plan: InvocationPlanId,
    ) -> Result<(), InvocationControllerError> {
        if !self.plans.contains_key(&plan) {
            return Err(InvocationControllerError::new("unknown invocation plan"));
        }
        let active = self
            .pending
            .iter()
            .filter_map(|(id, pending)| {
                pending
                    .plan_step
                    .is_some_and(|link| link.plan == plan)
                    .then_some(*id)
            })
            .collect::<Vec<_>>();
        for invocation in active {
            self.cancel(arena, builders, borrow_graph, host, invocation)?;
        }
        self.plans
            .get_mut(&plan)
            .expect("plan remains indexed")
            .cancel();
        Ok(())
    }

    fn invoke_linked<H: InvocationHost + ?Sized>(
        &mut self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        host: &mut H,
        operation: PlannedInvocation,
        plan_step: Option<PlanStepLink>,
    ) -> Result<InvocationStart, InvocationControllerError> {
        let PlannedInvocation {
            input,
            input_thing,
            function,
            mode,
        } = operation;
        if !input_thing.shape.is_shape(function.input_shape) {
            return Err(InvocationControllerError::new(
                "registered function input does not match the supplied Thing",
            ));
        }
        let ready = arena
            .ready_value(input)
            .ok_or_else(|| InvocationControllerError::new(format!("slot {input} is not Ready")))?;
        if !ready.shape().is_shape(input_thing.shape) {
            return Err(InvocationControllerError::new(format!(
                "slot {input} does not contain the function's input shape"
            )));
        }
        if mode == InvocationMode::Consume || function.receiver_mode == ReceiverMode::ByMut {
            borrow_graph
                .ensure_root_unprotected(input)
                .map_err(|error| InvocationControllerError::new(error.to_string()))?;
        }
        let runtime_input = match mode {
            InvocationMode::Retain => {
                guarded_user_operation("could not clone invocation input", || ready.try_clone())
            }
            InvocationMode::Consume => {
                guarded_user_operation("could not consume invocation input", || {
                    arena.consume(input).map_err(Into::into)
                })
            }
        }
        .map_err(InvocationControllerError::new)?;
        let (runtime_input, output) =
            guarded_user_operation("could not reserve invocation output", || {
                // On an allocation/state error the captured input is destroyed
                // inside the guard, while its original source leases are live.
                let output = arena.insert_pending()?;
                Ok((runtime_input, output))
            })
            .map_err(InvocationControllerError::new)?;
        let lease_result = match mode {
            InvocationMode::Retain => {
                builders.clone_ready_leases_to_pending(arena, borrow_graph, input, output)
            }
            InvocationMode::Consume => {
                builders.transfer_ready_leases_to_pending(arena, borrow_graph, input, output)
            }
        };
        if let Err(error) = lease_result {
            let message =
                guarded_user_operation("could not destroy rejected invocation input", || {
                    drop(runtime_input);
                    Ok(())
                })
                .err()
                .unwrap_or_else(|| error.to_string());
            let _ = builders.fail_pending(arena, borrow_graph, output, message.clone());
            return Err(InvocationControllerError::new(message));
        }

        // This guard owns the input and every intermediate result. In particular,
        // ByRef/ByMut input destructors and discarded outputs finish before an
        // error can reach failed_start and release the pending source leases.
        let invocation = guarded_user_operation("invocation dispatch", || {
            use eyre::WrapErr;

            let mut boxed_input = input_thing
                .runtime_into_boxed(runtime_input)
                .wrap_err("could not convert invocation input")?;
            match function.receiver_mode {
                ReceiverMode::ByValue => function.invoke_value_boxed(boxed_input),
                ReceiverMode::ByRef => function
                    .invoke_ref_boxed(boxed_input.as_ref())
                    .map(FunctionInvocation::Ready),
                ReceiverMode::ByMut => {
                    let result = function
                        .invoke_mut_boxed(boxed_input.as_mut())
                        .map(FunctionInvocation::Ready);
                    if result.is_ok() && mode == InvocationMode::Retain {
                        let updated = input_thing
                            .runtime_from_boxed(boxed_input)
                            .wrap_err("could not convert mutated input")?;
                        let previous = arena
                            .replace_ready(input, updated)
                            .wrap_err("could not store mutated input")?;
                        drop(previous);
                    }
                    result
                }
            }
        });
        let invocation = match invocation {
            Ok(invocation) => invocation,
            Err(message) => {
                return Ok(self.failed_start(arena, builders, borrow_graph, output, message));
            }
        };
        match invocation {
            FunctionInvocation::Ready(value) => {
                match guarded_user_operation("could not store invocation output", || {
                    (function.output_to_runtime)(value)
                }) {
                    Ok(value) => {
                        guarded_user_operation("could not complete invocation output", || {
                            builders
                                .complete_pending(arena, borrow_graph, output, value)
                                .map_err(Into::into)
                        })
                        .map_err(InvocationControllerError::new)?;
                        Ok(InvocationStart::Ready { output })
                    }
                    Err(message) => {
                        Ok(self.failed_start(arena, builders, borrow_graph, output, message))
                    }
                }
            }
            FunctionInvocation::Pending(future) => {
                let invocation = self.allocate_invocation();
                host.start(invocation, future);
                self.pending.insert(
                    invocation,
                    PendingInvocation {
                        input,
                        output,
                        output_to_runtime: function.output_to_runtime,
                        plan_step,
                    },
                );
                Ok(InvocationStart::Pending { invocation, output })
            }
        }
    }

    fn failed_start(
        &self,
        arena: &mut Arena,
        builders: &mut BuilderStore,
        borrow_graph: &mut BorrowGraph,
        output: SlotId,
        message: String,
    ) -> InvocationStart {
        let _ = builders.fail_pending(arena, borrow_graph, output, message.clone());
        InvocationStart::Failed { output, message }
    }

    fn allocate_invocation(&mut self) -> InvocationId {
        let id = InvocationId::new(self.next_invocation);
        self.next_invocation = self
            .next_invocation
            .checked_add(1)
            .expect("invocation identity space exhausted");
        id
    }

    fn settle_plan_step(
        &mut self,
        plan_step: Option<PlanStepLink>,
        output: SlotId,
        state: &InvocationEventState,
    ) {
        let Some(link) = plan_step else {
            return;
        };
        let Some(plan) = self.plans.get_mut(&link.plan) else {
            return;
        };
        match state {
            InvocationEventState::Ready => plan.mark_complete(link.step, output),
            InvocationEventState::Failed(message) => plan.mark_failed(link.step, message.clone()),
            InvocationEventState::Cancelled => plan.mark_cancelled(link.step),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena_slot_state::ArenaSlotState;
    use crate::object_explorer::field_binding::FieldBinding;
    use crate::object_explorer::invocation_host::FakeInvocationHost;
    use crate::object_explorer::invocation_plan::InvocationPlanCompletion;
    use crate::object_explorer::invocation_plan::InvocationPlanStepStatus;
    use crate::object_explorer::tokio_invocation_host::TokioInvocationHost;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_builder::ValueBuilder;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::InvocationFuture;
    use cloud_terrastodon_registry::RegistrationSite;
    use cloud_terrastodon_registry::RuntimeValue;
    use cloud_terrastodon_registry::runtime_from_boxed;
    use cloud_terrastodon_registry::runtime_into_boxed;
    use facet::Facet;
    use std::any::Any;
    use std::borrow::Cow;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct TestRequest {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct BorrowSource {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[facet(traits(Clone))]
    #[repr(C)]
    struct BorrowRequest<'a> {
        source: Cow<'a, BorrowSource>,
    }

    fn invoke_test_request(input: Box<dyn Any + Send>) -> InvocationFuture {
        Box::pin(async move {
            let request = input
                .downcast::<TestRequest>()
                .map_err(|_| eyre::eyre!("wrong test request input"))?;
            Ok(Box::new(format!("result: {}", request.value)) as Box<dyn Any + Send>)
        })
    }

    fn invoke_borrow_request(input: Box<dyn Any + Send>) -> InvocationFuture {
        Box::pin(async move {
            let request = input
                .downcast::<BorrowRequest<'static>>()
                .map_err(|_| eyre::eyre!("wrong borrow request input"))?;
            Ok(Box::new(request.source.value.clone()) as Box<dyn Any + Send>)
        })
    }

    static TEST_REQUEST_THING: Thing = Thing::value(
        TestRequest::SHAPE,
        runtime_from_boxed::<TestRequest>,
        runtime_into_boxed::<TestRequest>,
        RegistrationSite::new(file!(), line!()),
    );

    static TEST_FUNCTION: Function = Function::async_value(
        TestRequest::SHAPE,
        String::SHAPE,
        FunctionKind::AsyncInvoke,
        "invoke",
        "test",
        &[],
        invoke_test_request,
        runtime_from_boxed::<String>,
        RegistrationSite::new(file!(), line!()),
    );

    static BORROW_REQUEST_THING: Thing = Thing::value(
        <BorrowRequest<'static>>::SHAPE,
        runtime_from_boxed::<BorrowRequest<'static>>,
        runtime_into_boxed::<BorrowRequest<'static>>,
        RegistrationSite::new(file!(), line!()),
    );

    static BORROW_FUNCTION: Function = Function::async_value(
        <BorrowRequest<'static>>::SHAPE,
        String::SHAPE,
        FunctionKind::AsyncInvoke,
        "invoke",
        "test",
        &[],
        invoke_borrow_request,
        runtime_from_boxed::<String>,
        RegistrationSite::new(file!(), line!()),
    );

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    fn state(arena: &Arena, slot: SlotId) -> &ArenaSlotState {
        arena.slot(slot).expect("slot exists").state()
    }

    fn attach_nothing(future: InvocationFuture) -> InvocationFuture {
        future
    }

    #[tokio::test]
    async fn headless_controller_executes_the_ordinary_registry_future() {
        let mut arena = Arena::default();
        let input = arena
            .insert_ready(runtime(TestRequest {
                value: "ordinary registry".to_owned(),
            }))
            .unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut host = TokioInvocationHost::new(attach_nothing);
        let mut controller = InvocationController::default();
        let InvocationStart::Pending { output, .. } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                input,
                &TEST_REQUEST_THING,
                &TEST_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("registered async Function must return Pending")
        };

        let events = loop {
            let events = controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
            if !events.is_empty() {
                break events;
            }
            tokio::task::yield_now().await;
        };

        assert_eq!(events[0].state, InvocationEventState::Ready);
        let value = arena
            .ready_value(output)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<String>()
            .unwrap()
            .downcast::<String>()
            .unwrap();
        assert_eq!(value.as_str(), "result: ordinary registry");
    }

    #[test]
    fn fake_host_holds_completes_fails_and_cancels_headless_invocations() {
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(TestRequest {
                value: "first".to_owned(),
            }))
            .unwrap();
        let second = arena
            .insert_ready(runtime(TestRequest {
                value: "second".to_owned(),
            }))
            .unwrap();
        let third = arena
            .insert_ready(runtime(TestRequest {
                value: "third".to_owned(),
            }))
            .unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut host = FakeInvocationHost::default();
        let mut controller = InvocationController::default();

        let InvocationStart::Pending {
            invocation: complete,
            output: complete_output,
        } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                first,
                &TEST_REQUEST_THING,
                &TEST_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        let InvocationStart::Pending {
            invocation: fail,
            output: fail_output,
        } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                second,
                &TEST_REQUEST_THING,
                &TEST_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        let InvocationStart::Pending {
            invocation: cancel,
            output: cancel_output,
        } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                third,
                &TEST_REQUEST_THING,
                &TEST_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        assert_eq!(
            controller.poll(&mut arena, &mut builders, &mut borrows, &mut host),
            []
        );
        host.complete(complete, String::from("completed by fake host"));
        host.fail(fail, "failed by fake host");
        host.finish_cancelled(cancel);

        let events = controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
        assert_eq!(events.len(), 3);
        assert!(matches!(
            state(&arena, complete_output),
            ArenaSlotState::Ready(_)
        ));
        assert!(
            matches!(state(&arena, fail_output), ArenaSlotState::Failed(message) if message == "failed by fake host")
        );
        assert!(matches!(
            state(&arena, cancel_output),
            ArenaSlotState::Cancelled
        ));
        assert_eq!(controller.pending_count(), 0);
    }

    #[test]
    fn invocation_host_preserves_borrow_leases() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(BorrowSource {
                value: "borrowed".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<BorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        let mut host = FakeInvocationHost::default();
        let mut controller = InvocationController::default();

        let InvocationStart::Pending { invocation, output } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                request,
                &BORROW_REQUEST_THING,
                &BORROW_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        assert_eq!(builders.pending_lease_count(output), 1);
        assert_eq!(borrows.edge_count(), 2);
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());

        host.complete(invocation, String::from("done"));
        let events = controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
        assert_eq!(events[0].state, InvocationEventState::Ready);
        assert_eq!(builders.pending_lease_count(output), 0);
        assert_eq!(borrows.edge_count(), 2);
        builders.delete(&mut arena, &mut borrows, request).unwrap();
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());
        builders.delete(&mut arena, &mut borrows, output).unwrap();
        builders.delete(&mut arena, &mut borrows, source).unwrap();

        let moved_source = arena
            .insert_ready(runtime(BorrowSource {
                value: "moved borrow".to_owned(),
            }))
            .unwrap();
        let moved_request = arena.reserve_builder().unwrap();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                moved_request,
                ValueBuilder::new(<BorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                moved_request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(moved_source)),
            )
            .unwrap();
        let InvocationStart::Pending { invocation, output } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                moved_request,
                &BORROW_REQUEST_THING,
                &BORROW_FUNCTION,
                InvocationMode::Consume,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        assert!(matches!(
            state(&arena, moved_request),
            ArenaSlotState::Consumed
        ));
        assert_eq!(builders.pending_lease_count(output), 1);
        assert_eq!(borrows.edge_count(), 1);

        host.complete(invocation, String::from("moved done"));
        controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
        assert_eq!(borrows.edge_count(), 1);
        assert!(
            builders
                .delete(&mut arena, &mut borrows, moved_source)
                .is_err()
        );
        builders.delete(&mut arena, &mut borrows, output).unwrap();
        assert_eq!(borrows.edge_count(), 0);
        builders
            .delete(&mut arena, &mut borrows, moved_source)
            .unwrap();
    }

    #[test]
    fn plans_are_controller_metadata_and_progress_without_synthetic_slots() {
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(TestRequest {
                value: "first".to_owned(),
            }))
            .unwrap();
        let second = arena
            .insert_ready(runtime(TestRequest {
                value: "second".to_owned(),
            }))
            .unwrap();
        let roots_before_plan = arena.allocated_slot_count();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut host = FakeInvocationHost::default();
        let mut controller = InvocationController::default();
        let plan = controller.create_plan(
            "two ordinary invocations",
            [first, second].map(|input| PlannedInvocation {
                input,
                input_thing: &TEST_REQUEST_THING,
                function: &TEST_FUNCTION,
                mode: InvocationMode::Retain,
            }),
        );
        assert_eq!(arena.allocated_slot_count(), roots_before_plan);
        assert_eq!(controller.plan_count(), 1);
        assert_eq!(
            controller.plan(plan).unwrap().title(),
            "two ordinary invocations"
        );

        controller
            .advance_plan(&mut arena, &mut builders, &mut borrows, &mut host, plan)
            .unwrap();
        assert_eq!(arena.allocated_slot_count(), roots_before_plan + 1);
        let (first_invocation, first_output) =
            match controller.plan(plan).unwrap().steps()[0].status() {
                InvocationPlanStepStatus::Waiting { invocation, output } => (*invocation, *output),
                status => panic!("expected first waiting step, got {status:?}"),
            };
        host.complete(first_invocation, String::from("first output"));
        controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
        assert!(
            matches!(controller.plan(plan).unwrap().steps()[0].status(), InvocationPlanStepStatus::Complete { output } if *output == first_output)
        );

        controller
            .advance_plan(&mut arena, &mut builders, &mut borrows, &mut host, plan)
            .unwrap();
        let second_invocation = match controller.plan(plan).unwrap().steps()[1].status() {
            InvocationPlanStepStatus::Waiting { invocation, .. } => *invocation,
            status => panic!("expected second waiting step, got {status:?}"),
        };
        host.complete(second_invocation, String::from("second output"));
        controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);

        assert_eq!(
            controller.plan(plan).unwrap().completion(),
            InvocationPlanCompletion::Complete
        );
        assert_eq!(
            arena.allocated_slot_count(),
            roots_before_plan + 2,
            "only actual invocation outputs receive arena slots"
        );
    }

    fn return_borrowed_source(input: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        let request = input
            .downcast::<BorrowRequest<'static>>()
            .map_err(|_| eyre::eyre!("wrong borrowed request"))?;
        Ok(Box::new(request.source))
    }

    fn return_borrowed_source_async(input: Box<dyn Any + Send>) -> InvocationFuture {
        Box::pin(async move { return_borrowed_source(input) })
    }

    static RETURN_BORROWED_SOURCE: Function = Function::sync_value(
        <BorrowRequest<'static>>::SHAPE,
        <Cow<'static, BorrowSource>>::SHAPE,
        FunctionKind::Conversion,
        "return source",
        "test",
        &[],
        return_borrowed_source,
        runtime_from_boxed::<Cow<'static, BorrowSource>>,
        RegistrationSite::new(file!(), line!()),
    );

    static RETURN_BORROWED_SOURCE_ASYNC: Function = Function::async_value(
        <BorrowRequest<'static>>::SHAPE,
        <Cow<'static, BorrowSource>>::SHAPE,
        FunctionKind::AsyncInvoke,
        "return source asynchronously",
        "test",
        &[],
        return_borrowed_source_async,
        runtime_from_boxed::<Cow<'static, BorrowSource>>,
        RegistrationSite::new(file!(), line!()),
    );

    #[tokio::test]
    async fn sync_and_async_results_retain_borrowed_sources_in_both_invocation_modes() {
        for function in [&RETURN_BORROWED_SOURCE, &RETURN_BORROWED_SOURCE_ASYNC] {
            for mode in [InvocationMode::Retain, InvocationMode::Consume] {
                let mut arena = Arena::default();
                let source = arena
                    .insert_ready(runtime(BorrowSource {
                        value: "retained source".into(),
                    }))
                    .unwrap();
                let request = arena.reserve_builder().unwrap();
                let mut builders = BuilderStore::default();
                let mut borrows = BorrowGraph::default();
                builders
                    .insert_and_finalize(
                        &mut arena,
                        &mut borrows,
                        request,
                        ValueBuilder::new(<BorrowRequest<'static>>::SHAPE),
                    )
                    .unwrap();
                builders
                    .set_field_and_finalize(
                        &mut arena,
                        &mut borrows,
                        request,
                        0,
                        FieldBinding::BorrowFrom(ValueAddress::root(source)),
                    )
                    .unwrap();
                let mut host = TokioInvocationHost::new(attach_nothing);
                let mut controller = InvocationController::default();
                let start = controller
                    .invoke(
                        &mut arena,
                        &mut builders,
                        &mut borrows,
                        &mut host,
                        request,
                        &BORROW_REQUEST_THING,
                        function,
                        mode,
                    )
                    .unwrap();
                let output = start.output();
                if matches!(start, InvocationStart::Pending { .. }) {
                    tokio::time::timeout(std::time::Duration::from_secs(5), async {
                        loop {
                            let events =
                                controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
                            if let Some(event) = events.first() {
                                assert_eq!(event.state, InvocationEventState::Ready);
                                break;
                            }
                            tokio::task::yield_now().await;
                        }
                    })
                    .await
                    .expect("finite invocation completes");
                }
                if mode == InvocationMode::Retain {
                    builders.delete(&mut arena, &mut borrows, request).unwrap();
                }
                assert!(builders.delete(&mut arena, &mut borrows, source).is_err());
                assert_eq!(builders.leases(output).len(), 1);
                let result = arena
                    .ready_value(output)
                    .unwrap()
                    .peek()
                    .get::<Cow<'static, BorrowSource>>()
                    .unwrap();
                assert!(matches!(result, Cow::Borrowed(_)));
                assert_eq!(result.value, "retained source");
                builders.delete(&mut arena, &mut borrows, output).unwrap();
                builders.delete(&mut arena, &mut borrows, source).unwrap();
                assert_eq!(borrows.edge_count(), 0);
            }
        }
    }

    #[derive(Clone, Debug, Facet)]
    #[facet(opaque)]
    struct GuardDropProbe(std::sync::Arc<std::sync::atomic::AtomicUsize>);

    impl Drop for GuardDropProbe {
        fn drop(&mut self) {
            // The probe never dereferences the borrowed source. A missing guard
            // is detected by the count, not by exercising a dangling pointer.
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    #[derive(Clone, Debug, Facet)]
    #[facet(traits(Clone))]
    struct GuardRequest {
        source: Cow<'static, BorrowSource>,
        probe: GuardDropProbe,
    }

    #[derive(Debug)]
    struct BorrowingError(GuardRequest);

    impl fmt::Display for BorrowingError {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            // Both the error and this replacement panic payload retain a real
            // shallow Cow borrow. Both must die before the controller releases it.
            std::panic::panic_any(self.0.clone());
        }
    }

    impl Error for BorrowingError {}

    fn guarded_return(input: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        Ok(input)
    }

    fn guarded_return_async(input: Box<dyn Any + Send>) -> InvocationFuture {
        Box::pin(async move { Ok(input) })
    }

    fn guarded_ref_error(input: &(dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        Err(eyre::Report::new(BorrowingError(
            input.downcast_ref::<GuardRequest>().unwrap().clone(),
        )))
    }

    fn guarded_mut_error(input: &mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        guarded_ref_error(input)
    }

    fn guarded_mut_return(input: &mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>> {
        Ok(Box::new(
            input.downcast_ref::<GuardRequest>().unwrap().clone(),
        ))
    }

    fn guarded_panic(input: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        std::panic::panic_any(input);
    }

    fn guarded_error(input: Box<dyn Any + Send>) -> eyre::Result<Box<dyn Any + Send>> {
        Err(eyre::Report::new(BorrowingError(
            *input.downcast::<GuardRequest>().unwrap(),
        )))
    }

    fn guarded_converter_panic(input: Box<dyn Any + Send>) -> eyre::Result<RuntimeValue> {
        std::panic::panic_any(input);
    }

    fn guarded_converter_error(input: Box<dyn Any + Send>) -> eyre::Result<RuntimeValue> {
        Err(eyre::Report::new(BorrowingError(
            *input.downcast::<GuardRequest>().unwrap(),
        )))
    }

    fn guarded_input_converter_panic(input: RuntimeValue) -> eyre::Result<Box<dyn Any + Send>> {
        std::panic::panic_any(runtime_into_boxed::<GuardRequest>(input)?);
    }

    static GUARD_THING: Thing = Thing::value(
        GuardRequest::SHAPE,
        runtime_from_boxed::<GuardRequest>,
        runtime_into_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );

    static GUARD_PANICKING_INPUT_THING: Thing = Thing::value(
        GuardRequest::SHAPE,
        runtime_from_boxed::<GuardRequest>,
        guarded_input_converter_panic,
        RegistrationSite::new(file!(), line!()),
    );

    static GUARD_MUTATED_INPUT_ERROR_THING: Thing = Thing::value(
        GuardRequest::SHAPE,
        guarded_converter_error,
        runtime_into_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );

    static GUARD_SYNC_PANIC: Function = Function::sync_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "panic with borrowed input",
        "test",
        &[],
        guarded_panic,
        runtime_from_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_SYNC_ERROR: Function = Function::sync_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "error retaining borrowed input",
        "test",
        &[],
        guarded_error,
        runtime_from_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_CONVERTER_PANIC: Function = Function::sync_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "converter panic with borrowed output",
        "test",
        &[],
        guarded_return,
        guarded_converter_panic,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_CONVERTER_ERROR: Function = Function::sync_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "converter error retaining borrowed output",
        "test",
        &[],
        guarded_return,
        guarded_converter_error,
        RegistrationSite::new(file!(), line!()),
    );

    static GUARD_ASYNC_CONVERTER_PANIC: Function = Function::async_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::AsyncInvoke,
        "async converter panic",
        "test",
        &[],
        guarded_return_async,
        guarded_converter_panic,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_ASYNC_CONVERTER_ERROR: Function = Function::async_value(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::AsyncInvoke,
        "async converter error",
        "test",
        &[],
        guarded_return_async,
        guarded_converter_error,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_REF_ERROR: Function = Function::sync_ref(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "ByRef error",
        "test",
        &[],
        guarded_ref_error,
        runtime_from_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_MUT_ERROR: Function = Function::sync_mut(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "ByMut error",
        "test",
        &[],
        guarded_mut_error,
        runtime_from_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );
    static GUARD_MUT_RETURN: Function = Function::sync_mut(
        GuardRequest::SHAPE,
        GuardRequest::SHAPE,
        FunctionKind::Conversion,
        "ByMut conversion error discards successful output",
        "test",
        &[],
        guarded_mut_return,
        runtime_from_boxed::<GuardRequest>,
        RegistrationSite::new(file!(), line!()),
    );

    #[tokio::test]
    async fn sync_panics_and_conversion_errors_dispose_borrowed_payloads_before_releasing_leases() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        for (thing, function, mode, expected_drops) in [
            (&GUARD_THING, &GUARD_SYNC_PANIC, InvocationMode::Consume, 1),
            (&GUARD_THING, &GUARD_SYNC_ERROR, InvocationMode::Consume, 2),
            (
                &GUARD_THING,
                &GUARD_CONVERTER_PANIC,
                InvocationMode::Consume,
                1,
            ),
            (
                &GUARD_THING,
                &GUARD_CONVERTER_ERROR,
                InvocationMode::Consume,
                2,
            ),
            (
                &GUARD_PANICKING_INPUT_THING,
                &GUARD_SYNC_PANIC,
                InvocationMode::Consume,
                1,
            ),
            (
                &GUARD_THING,
                &GUARD_ASYNC_CONVERTER_PANIC,
                InvocationMode::Consume,
                1,
            ),
            (
                &GUARD_THING,
                &GUARD_ASYNC_CONVERTER_ERROR,
                InvocationMode::Consume,
                2,
            ),
            (&GUARD_THING, &GUARD_REF_ERROR, InvocationMode::Consume, 3),
            (&GUARD_THING, &GUARD_MUT_ERROR, InvocationMode::Consume, 3),
            (
                &GUARD_MUTATED_INPUT_ERROR_THING,
                &GUARD_MUT_RETURN,
                InvocationMode::Retain,
                2,
            ),
        ] {
            let drops = Arc::new(AtomicUsize::new(0));
            let mut arena = Arena::default();
            let source = arena
                .insert_ready(runtime(BorrowSource {
                    value: "source".into(),
                }))
                .unwrap();
            let request = arena.reserve_builder().unwrap();
            let mut builders = BuilderStore::default();
            let mut borrows = BorrowGraph::default();
            builders
                .insert_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    ValueBuilder::new(GuardRequest::SHAPE),
                )
                .unwrap();
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::BorrowFrom(ValueAddress::root(source)),
                )
                .unwrap();
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    1,
                    FieldBinding::InlineOwned(runtime(GuardDropProbe(drops.clone()))),
                )
                .unwrap();
            assert!(matches!(
                arena
                    .ready_value(request)
                    .unwrap()
                    .peek()
                    .get::<GuardRequest>()
                    .unwrap()
                    .source,
                Cow::Borrowed(_)
            ));
            let mut controller = InvocationController::default();
            let mut host = TokioInvocationHost::new(attach_nothing);
            let start = controller
                .invoke(
                    &mut arena,
                    &mut builders,
                    &mut borrows,
                    &mut host,
                    request,
                    thing,
                    function,
                    mode,
                )
                .expect("user panics become an owned failure rather than escaping the controller");
            let output = start.output();
            let message = match start {
                InvocationStart::Failed { message, .. } => message,
                InvocationStart::Pending { .. } => {
                    tokio::time::timeout(std::time::Duration::from_secs(5), async {
                        loop {
                            let events =
                                controller.poll(&mut arena, &mut builders, &mut borrows, &mut host);
                            if let Some(event) = events.into_iter().next() {
                                let InvocationEventState::Failed(message) = event.state else {
                                    panic!(
                                        "expected guarded asynchronous output-conversion failure"
                                    );
                                };
                                break message;
                            }
                            tokio::task::yield_now().await;
                        }
                    })
                    .await
                    .expect("finite invocation completes")
                }
                InvocationStart::Ready { .. } => panic!("expected guarded user failure"),
            };
            assert!(
                message.contains("panicked") || message.contains("could not convert mutated input"),
                "{message}"
            );
            assert_eq!(
                drops.load(Ordering::SeqCst),
                expected_drops,
                "{}",
                function.label
            );
            assert_eq!(builders.pending_lease_count(output), 0);
            let final_drops = if mode == InvocationMode::Retain {
                assert!(builders.delete(&mut arena, &mut borrows, source).is_err());
                builders.delete(&mut arena, &mut borrows, request).unwrap();
                expected_drops + 1
            } else {
                expected_drops
            };
            assert_eq!(borrows.edge_count(), 0);
            builders.delete(&mut arena, &mut borrows, source).unwrap();
            assert_eq!(drops.load(Ordering::SeqCst), final_drops);
        }
    }

    #[test]
    fn cancelling_controller_job_cancels_output_and_releases_host_job() {
        let mut arena = Arena::default();
        let input = arena
            .insert_ready(runtime(TestRequest {
                value: "cancel".to_owned(),
            }))
            .unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        let mut host = FakeInvocationHost::default();
        let mut controller = InvocationController::default();
        let InvocationStart::Pending { invocation, output } = controller
            .invoke(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                input,
                &TEST_REQUEST_THING,
                &TEST_FUNCTION,
                InvocationMode::Retain,
            )
            .unwrap()
        else {
            panic!("async function must be pending")
        };
        assert!(host.contains(invocation));

        let event = controller
            .cancel(
                &mut arena,
                &mut builders,
                &mut borrows,
                &mut host,
                invocation,
            )
            .unwrap();

        assert_eq!(event.state, InvocationEventState::Cancelled);
        assert!(!host.contains(invocation));
        assert!(matches!(state(&arena, output), ArenaSlotState::Cancelled));
    }
}
