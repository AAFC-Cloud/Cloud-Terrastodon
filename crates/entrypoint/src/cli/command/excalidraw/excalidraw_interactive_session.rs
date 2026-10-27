use super::excalidraw_target_picker::pick_planning_targets;
use crate::cli::command::dump::DumpPlanReport;
use crate::cli::command::dump::DumpPlanSession;
use cloud_terrastodon_app::CancellationReportExt;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_excalidraw::ExcalidrawInteractiveDocument;
use cloud_terrastodon_excalidraw::ExcalidrawLogLevel;
use cloud_terrastodon_excalidraw::ExcalidrawLogRecord;
use cloud_terrastodon_excalidraw::ExcalidrawSharedFile;
use cloud_terrastodon_user_input::InlineTerminal;
use cloud_terrastodon_user_input::PickerLogBufferHandle;
use cloud_terrastodon_user_input::PickerLogLevel;
use cloud_terrastodon_user_input::PickerLogRecord;
use cloud_terrastodon_user_input::TerminalCoordinator;
use cloud_terrastodon_user_input::TerminalGuard;
use cloud_terrastodon_user_input::TerminalTextExt;
use cloud_terrastodon_user_input::apply_terminal_control;
use cloud_terrastodon_user_input::try_current_picker_log_buffer;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use eyre::Context;
use eyre::Result;
use ratatui::Frame;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tokio::task::spawn_blocking;
use tokio::time::MissedTickBehavior;

/// Owns a file session's accepted snapshot, pending edits and bounded log history.
/// Keeping these private prevents a successful read from accidentally discarding
/// a pending write, or an invalid external document from replacing the last good view.
pub(in crate::cli::command) struct ExcalidrawInteractiveSession {
    file: ExcalidrawSharedFile,
    snapshot: String,
    questions: BTreeMap<String, String>,
    diagnostics: Vec<String>,
    log_level: Option<ExcalidrawLogLevel>,
    logs: VecDeque<PickerLogRecord>,
    logs_dirty: bool,
    pending_test_status: Option<String>,
    test_changes: u64,
    disk_changes: u64,
    io_error: Option<String>,
    planning: Option<DumpPlanSession>,
    // The picker edits a saved selection snapshot. Retain both halves until
    // publication succeeds, including while VS Code owns the shared file lock.
    pending_targets: Option<(Vec<String>, Vec<String>)>,
}

impl ExcalidrawInteractiveSession {
    pub(super) async fn open(path: PathBuf, cancellation: &CancellationScope) -> Result<Self> {
        Self::open_with_plan(path, None, cancellation).await
    }

    pub(in crate::cli::command) async fn open_plan(
        path: PathBuf,
        planning: DumpPlanSession,
        cancellation: &CancellationScope,
    ) -> Result<Self> {
        Self::open_with_plan(path, Some(planning), cancellation).await
    }

    async fn open_with_plan(
        path: PathBuf,
        mut planning: Option<DumpPlanSession>,
        cancellation: &CancellationScope,
    ) -> Result<Self> {
        eyre::ensure!(
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("excalidraw")),
            "interactive sessions require a plain .excalidraw JSON file"
        );
        let file = spawn_blocking(move || ExcalidrawSharedFile::new(path))
            .await
            .wrap_err("opening Excalidraw file worker")??;
        let initial_file = file.clone();
        // Creation is a required-completion unit: once the worker starts, await
        // its atomic publication and lock release even if cancellation arrives.
        let (snapshot, planning) = cancellation
            .deferred("initializing Excalidraw drawing", |protected| async move {
                let protected = protected.clone();
                spawn_blocking(move || -> Result<_> {
                    if let Some(planning) = &mut planning {
                        initial_file.update(|current| planning.refresh(current, &protected))?;
                    } else if initial_file.read()?.is_none() {
                        initial_file.update(|current| {
                            if current.is_some() {
                                Ok(None)
                            } else {
                                Ok(Some(ExcalidrawInteractiveDocument::demo()?.to_json()?))
                            }
                        })?;
                    }
                    let snapshot = initial_file
                        .read()?
                        .ok_or_else(|| eyre::eyre!("drawing was removed during initialization"))?;
                    Ok((snapshot, planning))
                })
                .await
                .wrap_err("initializing Excalidraw file worker")?
            })
            .await?;
        let document = ExcalidrawInteractiveDocument::from_json(&snapshot)?;
        let session = Self {
            file,
            snapshot,
            questions: if planning.is_none() {
                document.questions()
            } else {
                BTreeMap::new()
            },
            diagnostics: if planning.is_none() {
                document.diagnostics()
            } else {
                Vec::new()
            },
            log_level: document.selected_log_level(),
            logs: VecDeque::new(),
            logs_dirty: true,
            pending_test_status: None,
            test_changes: 0,
            disk_changes: 0,
            io_error: None,
            planning,
            pending_targets: None,
        };
        tracing::info!(
            path = %format_args!("\"{}\"", session.file.path().to_string_lossy().sanitized_for_terminal()),
            "Watching Excalidraw drawing; save canvas edits to publish them to CT"
        );
        for (question, answer) in &session.questions {
            tracing::info!(question = %question.sanitized_for_terminal(), answer = %answer.sanitized_for_terminal(), "Canvas answer");
        }
        for diagnostic in &session.diagnostics {
            tracing::warn!("{}", diagnostic.sanitized_for_terminal());
        }
        Ok(session)
    }

    pub(in crate::cli::command) async fn run(
        &mut self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let logs = try_current_picker_log_buffer().ok_or_else(|| {
            eyre::eyre!("interactive Excalidraw requires application log capture")
        })?;
        let mut terminal = if InlineTerminal::supported() {
            let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
                eyre::eyre!("interactive Excalidraw requires the terminal coordinator")
            })?;
            let guard = coordinator.acquire().await?;
            match InlineTerminal::new(8) {
                Ok(terminal) => Some((terminal, guard)),
                Err(error) => {
                    guard.release().await?;
                    return Err(error);
                }
            }
        } else {
            tracing::info!("No interactive terminal; watching the file until Ctrl-C");
            None
        };
        let result = self.run_loop(terminal.as_mut(), &logs, cancellation).await;
        if result.is_ok()
            && (self.planning.is_some() || self.pending_test_status.is_some())
            && !cancellation.is_cancelled()
        {
            // A terminal selection or Space immediately followed by Q still
            // owes publication. Ctrl-C stops admitting new file updates.
            self.poll(cancellation).await;
            if self.pending_test_status.is_some() {
                tracing::warn!(
                    "Closing with an unpublished reload-test update; the pending change was not saved"
                );
            }
            if self.pending_targets.is_some() {
                tracing::warn!(
                    "Closing with an unpublished target selection; the report contains the last saved selection"
                );
            }
        }
        if let Some((mut terminal, guard)) = terminal {
            // The last poll may have emitted a message just before Q/Ctrl-C.
            // Print remaining terminal-filtered events before releasing the
            // renderer; normal application shutdown need not replay them.
            let printed = if terminal.is_active() {
                let mut cursor = 0;
                let pending = logs.formatted_records_since(&mut cursor);
                terminal
                    .print_logs(&pending)
                    .map(|()| logs.acknowledge_through(cursor))
            } else {
                Ok(())
            };
            let restored = terminal.close();
            if let Err(error) = &restored {
                guard.poison(format!(
                    "restoring inline Excalidraw terminal failed: {error}"
                ));
            }
            let released = guard.release().await;
            result?;
            printed?;
            restored?;
            released?;
        } else {
            result?;
        }
        tracing::info!("Excalidraw file session stopped");
        Ok(())
    }

    pub(in crate::cli::command) fn into_plan_report(self) -> Option<DumpPlanReport> {
        self.planning.map(DumpPlanSession::into_report)
    }

    async fn run_loop(
        &mut self,
        mut terminal: Option<&mut (InlineTerminal, TerminalGuard)>,
        logs: &PickerLogBufferHandle,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let mut ticks = tokio::time::interval(Duration::from_millis(100));
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut last_read = Instant::now() - Duration::from_secs(1);
        let mut terminal_cursor = 0;
        let mut canvas_cursor = 0;
        loop {
            tokio::select! {
                () = cancellation.cancelled() => break,
                _ = ticks.tick() => {}
            }
            let mut edit_targets = false;
            if let Some((screen, guard)) = terminal.as_deref_mut() {
                while let Some(control) = guard.try_next_control()? {
                    apply_terminal_control(control, guard, screen)?;
                }
                if screen.is_active() {
                    // Nonblocking polling avoids an input worker surviving the
                    // session or competing with a nested terminal owner.
                    for _ in 0..32 {
                        if !event::poll(Duration::ZERO)? {
                            break;
                        }
                        if let Event::Key(key) = event::read()?
                            && key.kind == KeyEventKind::Press
                        {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                KeyCode::Char('c')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    cancellation.request_cancel("Excalidraw session interrupted");
                                    return Ok(());
                                }
                                KeyCode::Char(' ') if self.planning.is_none() => {
                                    self.change_test_status()
                                }
                                KeyCode::Char('a' | 'A') if self.planning.is_some() => {
                                    edit_targets = true;
                                    // Leave following buffered keystrokes for
                                    // the picker once it acquires the terminal.
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    let pending = logs.formatted_records_since(&mut terminal_cursor);
                    screen.print_logs(&pending)?;
                    logs.acknowledge_through(terminal_cursor);
                }
            }
            let captured = logs.captured_records_since(&mut canvas_cursor);
            if !captured.is_empty() {
                self.logs.extend(captured);
                while self.logs.len() > 512 {
                    self.logs.pop_front();
                }
                self.logs_dirty = true;
            }
            if edit_targets || last_read.elapsed() >= Duration::from_millis(250) {
                self.poll(cancellation).await;
                last_read = Instant::now();
            }
            if edit_targets && !cancellation.is_cancelled() {
                if self.io_error.is_some() {
                    tracing::warn!("Fix the drawing error before editing targets");
                } else if let Some((screen, guard)) = terminal.as_deref_mut() {
                    self.edit_targets(screen, guard, cancellation).await?;
                } else {
                    tracing::warn!("Editing targets needs an interactive terminal");
                }
            }
            if let Some((screen, _)) = terminal.as_deref_mut()
                && screen.is_active()
            {
                screen.draw(|frame| self.draw(frame))?;
            }
        }
        Ok(())
    }

    async fn edit_targets(
        &mut self,
        screen: &mut InlineTerminal,
        guard: &mut TerminalGuard,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let Some(planning) = &self.planning else {
            return Ok(());
        };
        let expected = planning.targets().to_vec();
        let selected = pick_planning_targets(
            planning.target_choices(),
            &expected,
            screen,
            guard,
            cancellation,
        )
        .await?;
        let Some(mut selected) = selected else {
            return Ok(());
        };
        selected.sort();
        selected.dedup();
        if selected == expected || cancellation.is_cancelled() {
            return Ok(());
        }
        self.pending_targets = Some((expected, selected));
        self.poll(cancellation).await;
        Ok(())
    }

    fn change_test_status(&mut self) {
        self.test_changes += 1;
        self.pending_test_status = Some(format!(
            "Space pressed {} — {}",
            self.test_changes,
            chrono::Utc::now().format("%H:%M:%S UTC"),
        ));
        tracing::debug!(change = self.test_changes, "Preparing live reload test");
        tracing::info!(
            change = self.test_changes,
            "Spacebar requested a drawing update"
        );
    }

    async fn poll(&mut self, cancellation: &CancellationScope) {
        let file = self.file.clone();
        let read = async {
            spawn_blocking(move || file.read())
                .await
                .wrap_err("reading Excalidraw file worker")?
                .wrap_err("reading Excalidraw drawing")?
                .ok_or_else(|| eyre::eyre!("drawing is missing; waiting for it to be restored"))
        }
        .await;
        let snapshot = match read {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.report_io_error(error);
                return;
            }
        };
        if snapshot != self.snapshot {
            match ExcalidrawInteractiveDocument::from_json(&snapshot) {
                Ok(document) => self.accept_document(snapshot, document),
                Err(error) => {
                    self.report_io_error(
                        error.wrap_err("drawing is invalid; retaining the last valid view"),
                    );
                    return;
                }
            }
        }
        if !self.logs_dirty && self.pending_test_status.is_none() && self.pending_targets.is_none()
        {
            self.io_error = None;
            return;
        }
        let file = self.file.clone();
        let logs: Vec<_> = self.logs.iter().map(canvas_log_record).collect();
        let status = self.pending_test_status.clone();
        let mut planning = self.planning.clone();
        let target_edit = self.pending_targets.clone();
        // The read/merge/replace runs as one bounded blocking operation. Never
        // detach it during cancellation: it owns a shared lock and cleanup.
        let written = cancellation
            .deferred("publishing Excalidraw update", |protected| async move {
                let protected = protected.clone();
                spawn_blocking(move || {
                    let changed = file.update(|current| {
                        let current = current
                            .ok_or_else(|| eyre::eyre!("drawing was removed before publication"))?;
                        if let Some(planning) = &mut planning {
                            if let Some((expected, selected)) = &target_edit {
                                return planning
                                    .replace_targets(current, expected, selected, &protected);
                            }
                            return planning.refresh(Some(current), &protected);
                        }
                        let mut document = ExcalidrawInteractiveDocument::from_json(current)?;
                        let mut changed = false;
                        if let Some(status) = &status {
                            changed |= document.set_test_status(status)?;
                        }
                        if let Some(level) = document.selected_log_level() {
                            let visible_records = logs
                                .iter()
                                .filter(|record| admits(level, record.level))
                                .cloned()
                                .collect::<Vec<_>>();
                            changed |= document.set_logs(&visible_records)?;
                        }
                        if changed {
                            Ok(Some(document.to_json()?))
                        } else {
                            Ok(None)
                        }
                    })?;
                    Ok::<_, eyre::Report>((changed, planning))
                })
                .await
                .wrap_err("publishing Excalidraw file worker")?
            })
            .await;
        match written {
            Ok((_, planning)) => {
                if let Some(planning) = &planning
                    && self.planning.as_ref().map(DumpPlanSession::targets)
                        != Some(planning.targets())
                {
                    tracing::info!(targets = planning.targets().len(), "Plan targets updated");
                }
                self.planning = planning;
                self.io_error = None;
                self.logs_dirty = false;
                self.pending_test_status = None;
                self.pending_targets = None;
            }
            Err(error) => {
                if !error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|error| error.kind() == ErrorKind::WouldBlock)
                {
                    // A changed terminal selection rejects this stale edit. Busy
                    // locks are retried, but policy/validation failures need a
                    // fresh picker instead of silently overwriting newer input.
                    self.pending_targets = None;
                }
                self.report_io_error(error);
            }
        }
    }

    fn accept_document(&mut self, snapshot: String, document: ExcalidrawInteractiveDocument) {
        if self.planning.is_some() {
            self.snapshot = snapshot;
            self.disk_changes += 1;
            self.logs_dirty = true;
            return;
        }
        let questions = document.questions();
        if questions != self.questions {
            for (question, answer) in &questions {
                if self.questions.get(question) != Some(answer) {
                    tracing::info!(question = %question.sanitized_for_terminal(), answer = %answer.sanitized_for_terminal(), "Canvas answer changed");
                }
            }
            for question in self
                .questions
                .keys()
                .filter(|question| !questions.contains_key(*question))
            {
                tracing::info!(question = %question.sanitized_for_terminal(), "Canvas answer is no longer unambiguous");
            }
            self.questions = questions;
        }
        let level = document.selected_log_level();
        if level != self.log_level {
            tracing::info!(?level, "Canvas log level changed");
            self.log_level = level;
        }
        let diagnostics = document.diagnostics();
        if diagnostics != self.diagnostics {
            for diagnostic in &diagnostics {
                tracing::warn!("{}", diagnostic.sanitized_for_terminal());
            }
            self.diagnostics = diagnostics;
        }
        self.snapshot = snapshot;
        self.disk_changes += 1;
        // Frame geometry may have changed even when the semantic controls did not.
        self.logs_dirty = true;
    }

    fn report_io_error(&mut self, error: eyre::Report) {
        if error.is_cancellation() {
            return;
        }
        let message = format!("{error:#}");
        if self.io_error.as_ref() != Some(&message) {
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == ErrorKind::WouldBlock)
            {
                tracing::debug!("Drawing is busy; pending edits will retry");
            } else {
                tracing::warn!("{}", message.sanitized_for_terminal());
            }
            self.io_error = Some(message);
        }
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let mut lines = vec![
            Line::from(format!(
                "\"{}\"",
                self.file.path().to_string_lossy().sanitized_for_terminal()
            )),
            Line::from(if self.planning.is_some() {
                "A: edit targets • Canvas: diagram only • Q / Esc: finish • Ctrl-C: stop"
            } else {
                "Space: change document to test VS Code reloading • Q / Esc: quit • Ctrl-C: cancel"
            }),
        ];
        if let Some(planning) = &self.planning {
            lines.push(Line::from(format!(
                "Disk updates: {}   Selected targets: {}   Offline planning",
                self.disk_changes,
                planning.targets().len(),
            )));
            lines.push(Line::from(
                planning.targets().join(", ").sanitized_for_terminal(),
            ));
        } else {
            lines.push(Line::from(format!(
                "Disk updates: {}   Canvas log filter: {}",
                self.disk_changes,
                self.log_level
                    .map(|level| format!("{level:?}"))
                    .unwrap_or_else(|| "unselected".into()),
            )));
        }
        lines.extend(self.questions.iter().map(|(question, answer)| {
            Line::from(format!("{question} -> {answer}").sanitized_for_terminal())
        }));
        if let Some(error) = &self.io_error {
            lines.push(Line::from(error.sanitized_for_terminal()));
        } else if let Some(diagnostic) = self.diagnostics.first() {
            lines.push(Line::from(diagnostic.sanitized_for_terminal()));
        }
        frame.render_widget(
            Paragraph::new(lines).block(Block::bordered().title("Excalidraw file session")),
            frame.area(),
        );
    }
}

fn admits(level: ExcalidrawLogLevel, record: ExcalidrawLogLevel) -> bool {
    match level {
        ExcalidrawLogLevel::Debug => true,
        ExcalidrawLogLevel::Info => !matches!(record, ExcalidrawLogLevel::Debug),
        ExcalidrawLogLevel::Warn => {
            matches!(record, ExcalidrawLogLevel::Warn | ExcalidrawLogLevel::Error)
        }
        ExcalidrawLogLevel::Error => matches!(record, ExcalidrawLogLevel::Error),
        ExcalidrawLogLevel::Off => false,
    }
}

fn canvas_log_record(record: &PickerLogRecord) -> ExcalidrawLogRecord {
    ExcalidrawLogRecord {
        level: match record.level {
            PickerLogLevel::Debug => ExcalidrawLogLevel::Debug,
            PickerLogLevel::Info => ExcalidrawLogLevel::Info,
            PickerLogLevel::Warn => ExcalidrawLogLevel::Warn,
            PickerLogLevel::Error => ExcalidrawLogLevel::Error,
        },
        message: record.message.to_string(),
        fields: record
            .fields
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
    }
}

#[cfg(test)]
#[path = "excalidraw_interactive_session_tests.rs"]
mod tests;
