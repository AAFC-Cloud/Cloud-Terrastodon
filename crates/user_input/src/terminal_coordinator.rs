//! Cooperative, nested ownership of an exclusive terminal.
//!
//! The coordinator deliberately knows nothing about Ratatui or Crossterm. An
//! owner receives control messages and is responsible for suspending and
//! resuming its own backend before acknowledging them.

use eyre::Result;
use futures::Future;
use futures::FutureExt;
use std::any::Any;
use std::collections::VecDeque;
use std::panic::Location;
use std::sync::Arc;
use std::sync::Mutex;
#[cfg(feature = "terminal_coordinator_debug")]
use std::sync::OnceLock;
use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

tokio::task_local! {
    static CURRENT_TERMINAL_COORDINATOR: TerminalCoordinator;
}

#[cfg(feature = "terminal_coordinator_debug")]
static DEBUG_APPLICATION_ROOT: OnceLock<Mutex<Option<usize>>> = OnceLock::new();

#[derive(Clone)]
pub struct TerminalCoordinator {
    commands: Arc<CommandBus>,
    state: Arc<CoordinatorState>,
}

#[derive(Clone)]
pub struct TerminalActivity {
    active_frames: Arc<AtomicU64>,
}

impl TerminalActivity {
    pub fn new() -> Self {
        Self {
            active_frames: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_frames.load(Ordering::Acquire) != 0
    }
}

struct CoordinatorState {
    next_id: AtomicU64,
    poisoned: AtomicBool,
    activity: TerminalActivity,
    actor_panic: Mutex<Option<Box<dyn Any + Send + 'static>>>,
}

struct CommandBus {
    sender: mpsc::UnboundedSender<Command>,
}

impl TerminalCoordinator {
    /// Start a coordinator on the currently running Tokio runtime.
    #[track_caller]
    pub fn new() -> Self {
        Self::try_new().expect("TerminalCoordinator::new requires a Tokio runtime")
    }

    /// Fallible version for library and test callers.
    #[track_caller]
    pub fn try_new() -> Result<Self> {
        Self::try_new_with_activity(TerminalActivity::new())
    }

    #[track_caller]
    pub fn try_new_with_activity(activity: TerminalActivity) -> Result<Self> {
        tokio::runtime::Handle::try_current().map_err(|error| {
            eyre::eyre!("terminal coordinator requires a Tokio runtime: {error}")
        })?;

        let (sender, receiver) = mpsc::unbounded_channel();
        let commands = Arc::new(CommandBus { sender });
        let state = Arc::new(CoordinatorState {
            next_id: AtomicU64::new(1),
            poisoned: AtomicBool::new(false),
            activity,
            actor_panic: Mutex::new(None),
        });
        let actor_state = Arc::clone(&state);
        let actor_commands = Arc::downgrade(&commands);
        tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(
                Actor::new(receiver, actor_state.clone(), actor_commands).run(),
            )
            .catch_unwind()
            .await;
            if let Err(payload) = result {
                actor_state.poisoned.store(true, Ordering::Release);
                *actor_state
                    .actor_panic
                    .lock()
                    .expect("terminal coordinator actor panic slot poisoned") = Some(payload);
            }
        });

        Ok(Self { commands, state })
    }

    pub fn activity(&self) -> TerminalActivity {
        self.state.activity.clone()
    }

    /// Take an internal actor panic so the application supervisor can resume it
    /// on its normal panic-reporting thread.
    pub fn take_actor_panic(&self) -> Option<Box<dyn Any + Send + 'static>> {
        self.state
            .actor_panic
            .lock()
            .expect("terminal coordinator actor panic slot poisoned")
            .take()
    }

    #[cfg(feature = "terminal_coordinator_debug")]
    pub fn debug_register_as_application_root(&self) -> Result<DebugApplicationRoot> {
        let registry = DEBUG_APPLICATION_ROOT.get_or_init(|| Mutex::new(None));
        let mut registered = registry
            .lock()
            .expect("terminal coordinator debug registry poisoned");
        if registered.is_some() {
            eyre::bail!("a terminal coordinator application root is already registered");
        }
        let identity = Arc::as_ptr(&self.state) as usize;
        *registered = Some(identity);
        Ok(DebugApplicationRoot {
            coordinator: self.clone(),
            identity,
        })
    }

    #[cfg(feature = "terminal_coordinator_debug")]
    pub fn debug_assert_matches_current(&self, current: &Self) -> Result<()> {
        if !Arc::ptr_eq(&self.state, &current.state) {
            eyre::bail!("explicit terminal coordinator does not match the current task scope");
        }
        Ok(())
    }

    #[cfg(feature = "terminal_coordinator_debug")]
    pub fn debug_assert_matches_registered_application_root(&self) -> Result<()> {
        let registry = DEBUG_APPLICATION_ROOT.get_or_init(|| Mutex::new(None));
        let registered = registry
            .lock()
            .expect("terminal coordinator debug registry poisoned");
        if let Some(identity) = *registered
            && identity != Arc::as_ptr(&self.state) as usize
        {
            eyre::bail!(
                "terminal coordinator does not match the registered application root; the current scope may be missing or a private coordinator may have been created"
            );
        }
        Ok(())
    }

    /// Acquire the terminal, waiting for the current owner to suspend first.
    #[track_caller]
    pub fn acquire(&self) -> impl Future<Output = Result<TerminalGuard>> + Send + 'static {
        self.acquire_from(std::panic::Location::caller())
    }

    #[track_caller]
    fn acquire_from(
        &self,
        _caller: &'static std::panic::Location<'static>,
    ) -> impl Future<Output = Result<TerminalGuard>> + Send + 'static {
        let coordinator = self.clone();
        async move {
            if coordinator.state.poisoned.load(Ordering::Acquire) {
                eyre::bail!("terminal coordinator is poisoned");
            }

            let (response_sender, response_receiver) = oneshot::channel();
            coordinator
                .commands
                .sender
                .send(Command::Acquire {
                    response: response_sender,
                    caller: _caller,
                })
                .map_err(|_| eyre::eyre!("terminal coordinator actor has stopped"))?;

            response_receiver
                .await
                .map_err(|_| eyre::eyre!("terminal coordinator actor stopped while acquiring"))?
                .map_err(|message| eyre::eyre!(message))
        }
    }

    pub fn try_current() -> Option<Self> {
        CURRENT_TERMINAL_COORDINATOR.try_with(Clone::clone).ok()
    }

    /// Run a future with this coordinator visible to APIs that cannot accept
    /// context parameters, such as registry IntoFuture implementations.
    pub fn scope<'a, F>(&'a self, future: F) -> impl Future<Output = F::Output> + 'a
    where
        F: Future + 'a,
    {
        CURRENT_TERMINAL_COORDINATOR.scope(self.clone(), future)
    }

    fn acknowledge(&self, frame_id: u64, request_id: u64) -> Result<()> {
        self.commands
            .sender
            .send(Command::Acknowledged {
                frame_id,
                request_id,
            })
            .map_err(|_| eyre::eyre!("terminal coordinator actor has stopped"))
    }

    fn release(&self, frame_id: u64) -> impl Future<Output = Result<()>> + Send + 'static {
        let coordinator = self.clone();
        async move {
            let (response_sender, response_receiver) = oneshot::channel();
            coordinator
                .commands
                .sender
                .send(Command::Release {
                    frame_id,
                    response: Some(response_sender),
                })
                .map_err(|_| eyre::eyre!("terminal coordinator actor has stopped"))?;
            response_receiver
                .await
                .map_err(|_| eyre::eyre!("terminal coordinator actor stopped while releasing"))?
                .map_err(|message| eyre::eyre!(message))
        }
    }

    fn emergency_release(&self, frame_id: u64) {
        let _ = self.commands.sender.send(Command::Release {
            frame_id,
            response: None,
        });
    }

    /// Mark terminal ownership as unsafe to continue and notify every owner.
    ///
    /// Owners should call this when their terminal backend cannot be restored or
    /// reinitialized. The coordinator remains fail-closed after poisoning.
    pub fn poison(&self, message: impl Into<String>) {
        self.state.poisoned.store(true, Ordering::Release);
        let _ = self.commands.sender.send(Command::Poison {
            message: message.into(),
        });
    }
}

#[cfg(feature = "terminal_coordinator_debug")]
pub struct DebugApplicationRoot {
    coordinator: TerminalCoordinator,
    identity: usize,
}

#[cfg(feature = "terminal_coordinator_debug")]
impl Drop for DebugApplicationRoot {
    fn drop(&mut self) {
        let registry = DEBUG_APPLICATION_ROOT
            .get()
            .expect("terminal coordinator debug registry was not initialized");
        let mut registered = registry
            .lock()
            .expect("terminal coordinator debug registry poisoned");
        assert_eq!(
            *registered,
            Some(self.identity),
            "terminal coordinator application root registration was dropped out of order"
        );
        *registered = None;
        drop(registered);

        if self.coordinator.activity().is_active() {
            self.coordinator
                .poison("terminal coordinator application root was dropped with active frames");
            panic!("terminal coordinator application root was dropped with active frames");
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TerminalControl {
    Suspend { request_id: u64 },
    Resume { request_id: u64 },
    Poisoned { message: Arc<str> },
}

impl TerminalControl {
    pub fn request_id(&self) -> Option<u64> {
        match self {
            Self::Suspend { request_id } | Self::Resume { request_id } => Some(*request_id),
            Self::Poisoned { .. } => None,
        }
    }
}

/// The terminal operations owned by a coordinator consumer.
///
/// The coordinator itself remains backend-agnostic. Consumers implement this
/// small interface for their Ratatui/Crossterm (or test) backend and use
/// [`apply_terminal_control`] before acknowledging ownership handoffs.
pub trait TerminalBackend {
    fn is_active(&self) -> bool;
    fn suspend(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
}

/// Perform a backend transition and acknowledge the coordinator only after it
/// succeeds. Any backend or acknowledgement failure poisons the coordinator.
pub fn apply_terminal_control<B: TerminalBackend>(
    control: TerminalControl,
    guard: &mut TerminalGuard,
    backend: &mut B,
) -> Result<()> {
    match &control {
        TerminalControl::Suspend { .. } => {
            if backend.is_active() {
                if let Err(error) = backend.suspend() {
                    guard.poison(format!("terminal suspension failed: {error}"));
                    return Err(error);
                }
            }
            guard.acknowledge(&control).map_err(|error| {
                guard.poison(format!(
                    "terminal suspension acknowledgement failed: {error}"
                ));
                error
            })?;
        }
        TerminalControl::Resume { .. } => {
            if !backend.is_active() {
                if let Err(error) = backend.resume() {
                    guard.poison(format!("terminal resume failed: {error}"));
                    return Err(error);
                }
            }
            guard.acknowledge(&control).map_err(|error| {
                guard.poison(format!("terminal resume acknowledgement failed: {error}"));
                error
            })?;
        }
        TerminalControl::Poisoned { message } => {
            eyre::bail!("terminal coordinator poisoned: {message}");
        }
    }
    Ok(())
}

pub struct TerminalGuard {
    coordinator: TerminalCoordinator,
    frame_id: u64,
    caller: &'static Location<'static>,
    drop_allowed: Arc<AtomicBool>,
    release_started: bool,
    controls: mpsc::UnboundedReceiver<TerminalControl>,
}

impl TerminalGuard {
    pub async fn next_control(&mut self) -> Option<TerminalControl> {
        self.controls.recv().await
    }

    pub fn try_next_control(&mut self) -> Result<Option<TerminalControl>> {
        match self.controls.try_recv() {
            Ok(control) => Ok(Some(control)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                eyre::bail!("terminal owner control channel closed")
            }
        }
    }

    /// Acknowledge a control after the owner's backend transition completes.
    pub fn acknowledge(&self, control: &TerminalControl) -> Result<()> {
        let request_id = control
            .request_id()
            .ok_or_else(|| eyre::eyre!("poisoned terminal control cannot be acknowledged"))?;
        self.coordinator.acknowledge(self.frame_id, request_id)
    }

    /// Release this frame and wait until any suspended parent has resumed.
    pub async fn release(mut self) -> Result<()> {
        if self.release_started {
            return Ok(());
        }
        self.release_started = true;
        self.coordinator.release(self.frame_id).await
    }

    pub fn frame_id(&self) -> u64 {
        self.frame_id
    }

    pub fn caller(&self) -> &'static Location<'static> {
        self.caller
    }

    /// Poison the coordinator when the owner's backend cannot safely continue.
    pub fn poison(&self, message: impl Into<String>) {
        self.coordinator.poison(message);
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.release_started {
            return;
        }

        let panicking = std::thread::panicking();
        if panicking {
            self.coordinator
                .poison("terminal owner panicked while holding a terminal frame");
        }

        if !self.drop_allowed.load(Ordering::Acquire) {
            self.coordinator.poison(format!(
                "terminal frame {} acquired at {}:{}:{} was dropped out of LIFO order",
                self.frame_id,
                self.caller.file(),
                self.caller.line(),
                self.caller.column(),
            ));
            if !panicking {
                panic!(
                    "terminal frame {} acquired at {}:{}:{} was dropped while a nested terminal frame was active",
                    self.frame_id,
                    self.caller.file(),
                    self.caller.line(),
                    self.caller.column(),
                );
            }
            return;
        }

        self.release_started = true;
        self.coordinator.emergency_release(self.frame_id);
    }
}

/// Attach a coordinator to a future at a Tokio spawn boundary.
pub trait TerminalCoordinatorFutureExt: Future + Sized {
    fn with_terminal_coordinator(
        self,
        coordinator: TerminalCoordinator,
    ) -> impl Future<Output = Self::Output>;
}

impl<F> TerminalCoordinatorFutureExt for F
where
    F: Future + Sized,
{
    fn with_terminal_coordinator(
        self,
        coordinator: TerminalCoordinator,
    ) -> impl Future<Output = Self::Output> {
        CURRENT_TERMINAL_COORDINATOR.scope(coordinator, self)
    }
}

struct AcquireRequest {
    response: oneshot::Sender<std::result::Result<TerminalGuard, String>>,
    caller: &'static Location<'static>,
}

struct PendingHandoff {
    request_id: u64,
    deadline: tokio::time::Instant,
    kind: HandoffKind,
}

enum HandoffKind {
    Suspend {
        parent_id: u64,
        child: AcquireRequest,
    },
    Resume {
        child_id: u64,
        parent_id: u64,
        response: Option<oneshot::Sender<std::result::Result<(), String>>>,
    },
}

enum Command {
    Acquire {
        response: oneshot::Sender<std::result::Result<TerminalGuard, String>>,
        caller: &'static Location<'static>,
    },
    Acknowledged {
        frame_id: u64,
        request_id: u64,
    },
    Release {
        frame_id: u64,
        response: Option<oneshot::Sender<std::result::Result<(), String>>>,
    },
    Poison {
        message: String,
    },
    #[cfg(test)]
    Panic,
}

struct Frame {
    id: u64,
    owner: mpsc::UnboundedSender<TerminalControl>,
    drop_allowed: Arc<AtomicBool>,
    caller: &'static Location<'static>,
}

struct Actor {
    commands: mpsc::UnboundedReceiver<Command>,
    sender: Weak<CommandBus>,
    state: Arc<CoordinatorState>,
    frames: Vec<Frame>,
    queued_acquires: VecDeque<AcquireRequest>,
    handoff: Option<PendingHandoff>,
}

impl Actor {
    fn new(
        commands: mpsc::UnboundedReceiver<Command>,
        state: Arc<CoordinatorState>,
        sender: Weak<CommandBus>,
    ) -> Self {
        Self {
            commands,
            sender,
            state,
            frames: Vec::new(),
            queued_acquires: VecDeque::new(),
            handoff: None,
        }
    }

    async fn run(mut self) {
        loop {
            let timeout = self
                .handoff
                .as_ref()
                .map(|handoff| tokio::time::sleep_until(handoff.deadline))
                .unwrap_or_else(|| tokio::time::sleep(Duration::from_secs(86_400)));
            tokio::pin!(timeout);
            tokio::select! {
                command = self.commands.recv() => {
                    let Some(command) = command else {
                        break;
                    };
                    self.handle(command);
                    self.advance();
                }
                _ = &mut timeout, if self.handoff.is_some() => {
                    self.poison("terminal handoff acknowledgement timed out");
                    break;
                }
            }
        }

        for request in std::mem::take(&mut self.queued_acquires) {
            let _ = request
                .response
                .send(Err("terminal coordinator actor has stopped".to_string()));
        }
        if let Some(PendingHandoff { kind, .. }) = self.handoff.take() {
            match kind {
                HandoffKind::Suspend { child, .. } => {
                    let _ = child
                        .response
                        .send(Err("terminal coordinator actor has stopped".to_string()));
                }
                HandoffKind::Resume { response, .. } => {
                    if let Some(response) = response {
                        let _ = response
                            .send(Err("terminal coordinator actor has stopped".to_string()));
                    }
                }
            }
        }
    }

    fn handle(&mut self, command: Command) {
        match command {
            Command::Acquire { response, caller } => {
                if self.state.poisoned.load(Ordering::Acquire) {
                    let _ = response.send(Err("terminal coordinator is poisoned".to_string()));
                } else {
                    self.queued_acquires
                        .push_back(AcquireRequest { response, caller });
                }
            }
            Command::Acknowledged {
                frame_id,
                request_id,
            } => self.handle_acknowledgement(frame_id, request_id),
            Command::Release { frame_id, response } => self.handle_release(frame_id, response),
            Command::Poison { message } => self.poison(message),
            #[cfg(test)]
            Command::Panic => panic!("test terminal coordinator actor panic"),
        }
    }

    fn handle_acknowledgement(&mut self, frame_id: u64, request_id: u64) {
        let Some(handoff) = self.handoff.take() else {
            self.poison("received an unexpected terminal acknowledgement");
            return;
        };

        if handoff.request_id != request_id {
            self.handoff = Some(handoff);
            self.poison("terminal acknowledgement had the wrong request id");
            return;
        }

        match handoff.kind {
            HandoffKind::Suspend { parent_id, child } => {
                if frame_id != parent_id
                    || self.frames.last().is_none_or(|frame| frame.id != parent_id)
                {
                    self.poison("terminal suspension acknowledgement came from the wrong owner");
                    let _ = child.response.send(Err(
                        "terminal suspension acknowledgement came from the wrong owner".to_string(),
                    ));
                    return;
                }

                if child.response.is_closed() {
                    if let Some(parent) = self.frames.last() {
                        parent.drop_allowed.store(true, Ordering::Release);
                    }
                    return;
                }
                self.grant(child);
            }
            HandoffKind::Resume {
                child_id,
                parent_id,
                response,
            } => {
                if frame_id != parent_id
                    || self.frames.len() < 2
                    || self.frames[self.frames.len() - 2].id != parent_id
                    || self.frames.last().is_none_or(|frame| frame.id != child_id)
                {
                    self.poison("terminal resume acknowledgement came from the wrong owner");
                    if let Some(response) = response {
                        let _ = response.send(Err(
                            "terminal resume acknowledgement came from the wrong owner".to_string(),
                        ));
                    }
                    return;
                }

                self.frames.pop();
                self.state
                    .activity
                    .active_frames
                    .fetch_sub(1, Ordering::AcqRel);
                if let Some(parent) = self.frames.last() {
                    parent.drop_allowed.store(true, Ordering::Release);
                }
                if let Some(response) = response {
                    let _ = response.send(Ok(()));
                }
            }
        }
    }

    fn handle_release(
        &mut self,
        frame_id: u64,
        response: Option<oneshot::Sender<std::result::Result<(), String>>>,
    ) {
        let Some(frame) = self.frames.last() else {
            if let Some(response) = response {
                let _ = response.send(Err("terminal frame was not active".to_string()));
            }
            return;
        };
        if frame.id != frame_id || self.handoff.is_some() {
            self.poison(format!(
                "terminal frame {frame_id} acquired at {}:{}:{} attempted an out-of-order release",
                frame.caller.file(),
                frame.caller.line(),
                frame.caller.column(),
            ));
            if let Some(response) = response {
                let _ = response.send(Err(
                    "terminal frame release violated LIFO ownership".to_string()
                ));
            }
            return;
        }

        if self.frames.len() == 1 {
            self.frames.pop();
            self.state
                .activity
                .active_frames
                .fetch_sub(1, Ordering::AcqRel);
            if let Some(response) = response {
                let _ = response.send(Ok(()));
            }
            return;
        }

        let parent_id = self.frames[self.frames.len() - 2].id;
        let request_id = self.next_id();
        let Some(parent) = self.frames.get(self.frames.len() - 2) else {
            unreachable!("the frame length was checked above");
        };
        if parent
            .owner
            .send(TerminalControl::Resume { request_id })
            .is_err()
        {
            parent.drop_allowed.store(true, Ordering::Release);
            self.poison("parent terminal owner closed during resume");
            self.frames.pop();
            self.state
                .activity
                .active_frames
                .fetch_sub(1, Ordering::AcqRel);
            if let Some(response) = response {
                let _ =
                    response.send(Err("parent terminal owner closed during resume".to_string()));
            }
            return;
        }
        self.handoff = Some(PendingHandoff {
            request_id,
            deadline: tokio::time::Instant::now() + Duration::from_secs(30),
            kind: HandoffKind::Resume {
                child_id: frame_id,
                parent_id,
                response,
            },
        });
    }

    fn advance(&mut self) {
        if self.handoff.is_some() || self.state.poisoned.load(Ordering::Acquire) {
            return;
        }
        let Some(request) = self.queued_acquires.pop_front() else {
            return;
        };

        if let Some(parent) = self.frames.last() {
            let request_id = self.next_id();
            parent.drop_allowed.store(false, Ordering::Release);
            if parent
                .owner
                .send(TerminalControl::Suspend { request_id })
                .is_err()
            {
                parent.drop_allowed.store(true, Ordering::Release);
                self.poison("active terminal owner closed during suspension");
                let _ = request.response.send(Err(
                    "active terminal owner closed during suspension".to_string(),
                ));
                return;
            }
            self.handoff = Some(PendingHandoff {
                request_id,
                deadline: tokio::time::Instant::now() + Duration::from_secs(30),
                kind: HandoffKind::Suspend {
                    parent_id: parent.id,
                    child: request,
                },
            });
        } else {
            self.grant(request);
        }
    }

    fn grant(&mut self, request: AcquireRequest) {
        if request.response.is_closed() {
            if let Some(parent) = self.frames.last() {
                parent.drop_allowed.store(true, Ordering::Release);
            }
            return;
        }

        let id = self.next_id();
        let (owner, controls) = mpsc::unbounded_channel();
        let drop_allowed = Arc::new(AtomicBool::new(true));
        if let Some(parent) = self.frames.last() {
            parent.drop_allowed.store(false, Ordering::Release);
        }
        self.frames.push(Frame {
            id,
            owner,
            drop_allowed: Arc::clone(&drop_allowed),
            caller: request.caller,
        });
        self.state
            .activity
            .active_frames
            .fetch_add(1, Ordering::AcqRel);
        let Some(commands) = self.sender.upgrade() else {
            self.frames.pop();
            self.state
                .activity
                .active_frames
                .fetch_sub(1, Ordering::AcqRel);
            if let Some(parent) = self.frames.last() {
                parent.drop_allowed.store(true, Ordering::Release);
            }
            let _ = request
                .response
                .send(Err("coordinator command bus has stopped".to_string()));
            return;
        };
        let guard = TerminalGuard {
            coordinator: TerminalCoordinator {
                commands,
                state: Arc::clone(&self.state),
            },
            frame_id: id,
            caller: request.caller,
            drop_allowed,
            release_started: false,
            controls,
        };
        if request.response.send(Ok(guard)).is_err() {
            self.frames.pop();
            self.state
                .activity
                .active_frames
                .fetch_sub(1, Ordering::AcqRel);
            if let Some(parent) = self.frames.last() {
                parent.drop_allowed.store(true, Ordering::Release);
            }
        }
    }

    fn next_id(&self) -> u64 {
        self.state.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn poison(&mut self, message: impl Into<String>) {
        let message: Arc<str> = Arc::from(message.into());
        self.state.poisoned.store(true, Ordering::Release);
        if let Some(handoff) = self.handoff.take() {
            match handoff.kind {
                HandoffKind::Suspend { child, .. } => {
                    let _ = child.response.send(Err(message.to_string()));
                }
                HandoffKind::Resume {
                    child_id, response, ..
                } => {
                    // The child guard has already been consumed by release(). If the
                    // handoff is poisoned before the parent acknowledges Resume, retire
                    // that child frame so the parent can still perform its own cleanup.
                    if self.frames.last().is_some_and(|frame| frame.id == child_id) {
                        self.frames.pop();
                        self.state
                            .activity
                            .active_frames
                            .fetch_sub(1, Ordering::AcqRel);
                        if let Some(parent) = self.frames.last() {
                            parent.drop_allowed.store(true, Ordering::Release);
                        }
                    }
                    if let Some(response) = response {
                        let _ = response.send(Err(message.to_string()));
                    }
                }
            }
        }
        for request in self.queued_acquires.drain(..) {
            let _ = request.response.send(Err(message.to_string()));
        }
        for frame in &self.frames {
            let _ = frame.owner.send(TerminalControl::Poisoned {
                message: Arc::clone(&message),
            });
        }
    }
}

impl Drop for Actor {
    fn drop(&mut self) {
        for request in self.queued_acquires.drain(..) {
            let _ = request.response.send(Err(
                "terminal coordinator actor stopped unexpectedly".to_string()
            ));
        }
        if let Some(PendingHandoff { kind, .. }) = self.handoff.take() {
            match kind {
                HandoffKind::Suspend { child, .. } => {
                    let _ = child.response.send(Err(
                        "terminal coordinator actor stopped unexpectedly".to_string(),
                    ));
                }
                HandoffKind::Resume { response, .. } => {
                    if let Some(response) = response {
                        let _ = response.send(Err(
                            "terminal coordinator actor stopped unexpectedly".to_string(),
                        ));
                    }
                }
            }
        }
        if self.frames.is_empty() {
            return;
        }

        self.state.poisoned.store(true, Ordering::Release);
        let message: Arc<str> = Arc::from("terminal coordinator actor stopped unexpectedly");
        for frame in &self.frames {
            frame.drop_allowed.store(true, Ordering::Release);
            let _ = frame.owner.send(TerminalControl::Poisoned {
                message: Arc::clone(&message),
            });
        }
        self.state
            .activity
            .active_frames
            .fetch_sub(self.frames.len() as u64, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "terminal_coordinator_debug")]
    static DEBUG_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct MockTerminalBackend {
        active: bool,
        suspend_error: Option<&'static str>,
        resume_error: Option<&'static str>,
    }

    impl TerminalBackend for MockTerminalBackend {
        fn is_active(&self) -> bool {
            self.active
        }

        fn suspend(&mut self) -> Result<()> {
            if let Some(message) = self.suspend_error.take() {
                eyre::bail!(message);
            }
            self.active = false;
            Ok(())
        }

        fn resume(&mut self) -> Result<()> {
            if let Some(message) = self.resume_error.take() {
                eyre::bail!(message);
            }
            self.active = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn nested_acquisition_is_strictly_lifo() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        assert!(coordinator.activity().is_active());

        let child_acquire = coordinator.acquire();
        tokio::pin!(child_acquire);
        let control = tokio::select! {
            control = root.next_control() => {
                control.expect("child acquisition should suspend root")
            }
            child = &mut child_acquire => {
                child?;
                panic!("child acquisition completed before root was suspended")
            }
        };
        assert!(matches!(control, TerminalControl::Suspend { .. }));
        root.acknowledge(&control)?;
        let child = child_acquire.await?;

        let release = child.release();
        tokio::pin!(release);
        let control = tokio::select! {
            control = root.next_control() => {
                control.expect("child release should resume root")
            }
            result = &mut release => {
                result?;
                panic!("child release completed before root was asked to resume")
            }
        };
        assert!(matches!(control, TerminalControl::Resume { .. }));
        root.acknowledge(&control)?;
        release.await?;
        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn three_level_nested_release_resumes_each_parent_in_lifo_order() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;

        let mut child_acquire = Box::pin(coordinator.acquire());
        let root_suspend = tokio::select! {
            control = root.next_control() => control.expect("child acquisition should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("child acquisition completed before root suspension acknowledgement")
            }
        };
        root.acknowledge(&root_suspend)?;
        let mut child = child_acquire.await?;

        let mut grandchild_acquire = Box::pin(coordinator.acquire());
        let child_suspend = tokio::select! {
            control = child.next_control() => control.expect("grandchild acquisition should suspend child"),
            grandchild = &mut grandchild_acquire => {
                grandchild?;
                panic!("grandchild acquisition completed before child suspension acknowledgement")
            }
        };
        child.acknowledge(&child_suspend)?;
        let grandchild = grandchild_acquire.await?;

        let mut grandchild_release = Box::pin(grandchild.release());
        let child_resume = tokio::select! {
            control = child.next_control() => control.expect("grandchild release should resume child"),
            result = &mut grandchild_release => {
                result?;
                panic!("grandchild release completed before child resume acknowledgement")
            }
        };
        assert!(matches!(child_resume, TerminalControl::Resume { .. }));
        child.acknowledge(&child_resume)?;
        grandchild_release.await?;

        let mut child_release = Box::pin(child.release());
        let root_resume = tokio::select! {
            control = root.next_control() => control.expect("child release should resume root"),
            result = &mut child_release => {
                result?;
                panic!("child release completed before root resume acknowledgement")
            }
        };
        assert!(matches!(root_resume, TerminalControl::Resume { .. }));
        root.acknowledge(&root_resume)?;
        child_release.await?;

        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn coordinator_scope_is_nested_and_restored() -> Result<()> {
        let outer = TerminalCoordinator::try_new()?;
        let inner = TerminalCoordinator::try_new()?;
        assert!(TerminalCoordinator::try_current().is_none());

        let outer_for_scope = outer.clone();
        let inner_for_scope = inner.clone();
        let outer_state = Arc::clone(&outer.state);
        let inner_state = Arc::clone(&inner.state);
        outer_for_scope
            .scope(async move {
                assert_eq!(
                    TerminalCoordinator::try_current()
                        .expect("outer scope")
                        .state
                        .as_ref() as *const _,
                    outer_state.as_ref() as *const _
                );
                inner_for_scope
                    .scope(async move {
                        assert_eq!(
                            TerminalCoordinator::try_current()
                                .expect("inner scope")
                                .state
                                .as_ref() as *const _,
                            inner_state.as_ref() as *const _
                        );
                    })
                    .await;
                assert_eq!(
                    TerminalCoordinator::try_current()
                        .expect("outer scope restored")
                        .state
                        .as_ref() as *const _,
                    outer_state.as_ref() as *const _
                );
            })
            .await;
        assert!(TerminalCoordinator::try_current().is_none());
        Ok(())
    }

    #[tokio::test]
    async fn cancelled_acquire_before_grant_does_not_leak_a_frame() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut acquire = Box::pin(coordinator.acquire());
        let waker = futures::task::noop_waker_ref();
        let mut context = std::task::Context::from_waker(waker);
        assert!(std::future::Future::poll(acquire.as_mut(), &mut context).is_pending());
        drop(acquire);

        tokio::task::yield_now().await;
        let guard = coordinator.acquire().await?;
        assert!(coordinator.activity().is_active());
        guard.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn actor_panics_are_retained_for_the_supervisor() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        coordinator
            .commands
            .sender
            .send(Command::Panic)
            .expect("actor should still be listening");

        let payload = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(payload) = coordinator.take_actor_panic() {
                    break payload;
                }
                tokio::task::yield_now().await;
            }
        })
        .await?;

        assert_eq!(
            payload.downcast_ref::<&str>(),
            Some(&"test terminal coordinator actor panic")
        );
        assert!(coordinator.state.poisoned.load(Ordering::Acquire));
        Ok(())
    }

    #[tokio::test]
    async fn actor_panics_notify_owners_and_release_frame_accounting() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        coordinator
            .commands
            .sender
            .send(Command::Panic)
            .expect("actor should still be listening");

        let control = tokio::time::timeout(Duration::from_secs(1), root.next_control())
            .await?
            .expect("actor panic should notify the owner");
        assert!(matches!(control, TerminalControl::Poisoned { .. }));
        assert!(!coordinator.activity().is_active());
        assert!(coordinator.take_actor_panic().is_some());
        assert!(root.release().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn actor_panics_fail_pending_handoffs_and_wake_all_owners() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let _suspend = tokio::select! {
            control = root.next_control() => control.expect("child acquisition should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("child acquisition completed before suspension acknowledgement")
            }
        };
        coordinator
            .commands
            .sender
            .send(Command::Panic)
            .expect("actor should still be listening");

        assert!(child_acquire.await.is_err());
        let control = tokio::time::timeout(Duration::from_secs(1), root.next_control())
            .await?
            .expect("actor panic should notify the root owner");
        assert!(matches!(control, TerminalControl::Poisoned { .. }));
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn actor_panics_fail_pending_resume_and_wake_all_owners() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let suspend = tokio::select! {
            control = root.next_control() => control.expect("child acquisition should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("child acquisition completed before suspension acknowledgement")
            }
        };
        root.acknowledge(&suspend)?;
        let child = child_acquire.await?;

        let mut child_release = Box::pin(child.release());
        let resume = tokio::select! {
            control = root.next_control() => control.expect("child release should resume root"),
            release = &mut child_release => {
                release?;
                panic!("child release completed before resume acknowledgement")
            }
        };
        assert!(matches!(resume, TerminalControl::Resume { .. }));
        coordinator
            .commands
            .sender
            .send(Command::Panic)
            .expect("actor should still be listening");

        assert!(child_release.await.is_err());
        let control = tokio::time::timeout(Duration::from_secs(1), root.next_control())
            .await?
            .expect("actor panic should notify the root owner");
        assert!(matches!(control, TerminalControl::Poisoned { .. }));
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn poisoning_fails_pending_resume_without_leaking_the_release() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let suspend = tokio::select! {
            control = root.next_control() => control.expect("child acquisition should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("child acquisition completed before suspension acknowledgement")
            }
        };
        root.acknowledge(&suspend)?;
        let child = child_acquire.await?;

        let mut child_release = Box::pin(child.release());
        let resume = tokio::select! {
            control = root.next_control() => control.expect("child release should resume root"),
            release = &mut child_release => {
                release?;
                panic!("child release completed before resume acknowledgement")
            }
        };
        assert!(matches!(resume, TerminalControl::Resume { .. }));
        coordinator.poison("test backend failure during resume");

        assert!(child_release.await.is_err());
        let control = tokio::time::timeout(Duration::from_secs(1), root.next_control())
            .await?
            .expect("poison should notify the root owner");
        assert!(matches!(control, TerminalControl::Poisoned { .. }));
        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn backend_failure_poison_is_fail_closed_and_owner_can_cleanup() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut guard = coordinator.acquire().await?;

        guard.poison("test terminal backend failure");
        let control = tokio::time::timeout(Duration::from_secs(1), guard.next_control())
            .await?
            .expect("poison should notify the active owner");
        assert!(matches!(control, TerminalControl::Poisoned { .. }));
        assert!(coordinator.acquire().await.is_err());

        guard.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn backend_setup_failure_is_injected_before_resume_acknowledgement() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut guard = coordinator.acquire().await?;
        let mut backend = MockTerminalBackend {
            active: false,
            suspend_error: None,
            resume_error: Some("injected setup failure"),
        };

        let error = apply_terminal_control(
            TerminalControl::Resume { request_id: 1 },
            &mut guard,
            &mut backend,
        )
        .expect_err("resume setup failure should be returned");
        assert!(error.to_string().contains("injected setup failure"));
        assert!(!backend.is_active());
        assert!(coordinator.acquire().await.is_err());
        guard.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn backend_teardown_failure_is_injected_before_suspend_acknowledgement() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut guard = coordinator.acquire().await?;
        let mut backend = MockTerminalBackend {
            active: true,
            suspend_error: Some("injected teardown failure"),
            resume_error: None,
        };

        let error = apply_terminal_control(
            TerminalControl::Suspend { request_id: 1 },
            &mut guard,
            &mut backend,
        )
        .expect_err("suspend teardown failure should be returned");
        assert!(error.to_string().contains("injected teardown failure"));
        assert!(backend.is_active());
        assert!(coordinator.acquire().await.is_err());
        guard.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn owner_panic_poisoned_coordinator_during_guard_drop() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let guard = coordinator.acquire().await?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = guard;
            panic!("test terminal owner panic");
        }));
        assert!(panic.is_err());
        tokio::task::yield_now().await;
        assert!(coordinator.acquire().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn cancelled_nested_acquire_restores_parent_drop_invariant() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let control = tokio::select! {
            control = root.next_control() => control.expect("nested acquire should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("nested acquire completed before suspension acknowledgement")
            }
        };
        assert!(matches!(control, TerminalControl::Suspend { .. }));
        drop(child_acquire);
        root.acknowledge(&control)?;
        tokio::task::yield_now().await;

        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn owner_control_channel_closure_poisoned_coordinator_and_allows_cleanup() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let (_, closed_controls) = mpsc::unbounded_channel();
        let original_controls = std::mem::replace(&mut root.controls, closed_controls);
        drop(original_controls);

        assert!(coordinator.acquire().await.is_err());
        assert!(coordinator.state.poisoned.load(Ordering::Acquire));
        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn parent_control_closure_during_child_release_fails_closed() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let suspend = tokio::select! {
            control = root.next_control() => control.expect("child acquisition should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("nested acquire completed before suspension acknowledgement")
            }
        };
        root.acknowledge(&suspend)?;
        let child = child_acquire.await?;

        root.controls.close();

        let child_release = tokio::time::timeout(Duration::from_secs(1), child.release())
            .await
            .expect("child release should fail after the parent channel closes");
        assert!(child_release.is_err());
        assert!(coordinator.state.poisoned.load(Ordering::Acquire));
        root.release().await?;
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn wrong_acknowledgement_poisoned_all_owners() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let control = tokio::select! {
            control = root.next_control() => control.expect("nested acquire should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("nested acquire completed before suspension acknowledgement")
            }
        };
        assert!(matches!(control, TerminalControl::Suspend { .. }));
        let request_id = control.request_id().expect("suspend has a request id");
        root.acknowledge(&TerminalControl::Suspend {
            request_id: request_id.wrapping_add(1),
        })?;

        let child_result = child_acquire.await;
        assert!(child_result.is_err());
        assert!(matches!(
            root.next_control().await,
            Some(TerminalControl::Poisoned { .. })
        ));
        assert!(coordinator.acquire().await.is_err());
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(root)));
        assert!(panic.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn unacknowledged_handoff_times_out_and_poisoned_state_is_fail_closed() -> Result<()> {
        tokio::time::pause();
        let coordinator = TerminalCoordinator::try_new()?;
        let mut root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let control = tokio::select! {
            control = root.next_control() => control.expect("nested acquire should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("nested acquire completed before suspension acknowledgement")
            }
        };
        assert!(matches!(control, TerminalControl::Suspend { .. }));

        tokio::time::advance(Duration::from_secs(31)).await;
        tokio::task::yield_now().await;
        assert!(child_acquire.await.is_err());
        assert!(matches!(
            root.next_control().await,
            Some(TerminalControl::Poisoned { .. })
        ));
        assert!(coordinator.acquire().await.is_err());
        drop(root);
        assert!(!coordinator.activity().is_active());
        Ok(())
    }

    #[tokio::test]
    async fn out_of_order_drop_panics_and_poisoned_state_is_observable() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let root = coordinator.acquire().await?;
        let mut child_acquire = Box::pin(coordinator.acquire());
        let mut root = root;
        let control = tokio::select! {
            control = root.next_control() => control.expect("nested acquire should suspend root"),
            child = &mut child_acquire => {
                child?;
                panic!("nested acquire completed before suspension acknowledgement")
            }
        };
        root.acknowledge(&control)?;
        let child = child_acquire.await?;

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(root)));
        assert!(panic.is_err());
        drop(child);
        tokio::task::yield_now().await;
        assert!(coordinator.acquire().await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn acquire_retains_original_callsite() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let expected_line = line!() + 1;
        let guard = coordinator.acquire().await?;
        assert_eq!(guard.caller().file(), file!());
        assert_eq!(guard.caller().line(), expected_line);
        guard.release().await?;
        Ok(())
    }

    #[tokio::test]
    async fn task_locals_do_not_cross_raw_spawn_but_adapters_restore_them() -> Result<()> {
        let coordinator = TerminalCoordinator::try_new()?;
        let raw_spawn = coordinator
            .scope(async {
                tokio::spawn(async { TerminalCoordinator::try_current().is_none() })
                    .await
                    .expect("raw task should complete")
            })
            .await;
        assert!(raw_spawn);

        let attached = tokio::spawn(
            async { TerminalCoordinator::try_current().is_some() }
                .with_terminal_coordinator(coordinator.clone()),
        )
        .await
        .expect("attached task should complete");
        assert!(attached);
        Ok(())
    }

    #[cfg(feature = "terminal_coordinator_debug")]
    #[tokio::test]
    async fn debug_root_registry_rejects_duplicates_and_mismatches() -> Result<()> {
        let _lock = DEBUG_TEST_LOCK.lock().expect("debug test lock poisoned");
        let coordinator = TerminalCoordinator::try_new()?;
        let other = TerminalCoordinator::try_new()?;
        let registration = coordinator.debug_register_as_application_root()?;
        assert!(coordinator.debug_register_as_application_root().is_err());
        assert!(other.debug_register_as_application_root().is_err());
        assert!(
            coordinator
                .debug_assert_matches_current(&coordinator)
                .is_ok()
        );
        assert!(coordinator.debug_assert_matches_current(&other).is_err());
        assert!(
            coordinator
                .debug_assert_matches_registered_application_root()
                .is_ok()
        );
        assert!(
            other
                .debug_assert_matches_registered_application_root()
                .is_err()
        );
        drop(registration);
        Ok(())
    }

    #[cfg(feature = "terminal_coordinator_debug")]
    #[tokio::test]
    async fn debug_root_drop_with_active_frame_panics_and_clears_registration() -> Result<()> {
        let _lock = DEBUG_TEST_LOCK.lock().expect("debug test lock poisoned");
        let coordinator = TerminalCoordinator::try_new()?;
        let registration = coordinator.debug_register_as_application_root()?;
        let guard = coordinator.acquire().await?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(registration)));
        assert!(panic.is_err());
        assert!(coordinator.state.poisoned.load(Ordering::Acquire));
        drop(guard);
        tokio::task::yield_now().await;

        let registration = coordinator.debug_register_as_application_root()?;
        drop(registration);
        Ok(())
    }
}
