use cloud_terrastodon_user_input::InlineTerminal;
use cloud_terrastodon_user_input::PickerLogBuffer;
use cloud_terrastodon_user_input::PickerLogBufferHandle;
use cloud_terrastodon_user_input::TerminalBackend;
use cloud_terrastodon_user_input::TerminalControl;
use cloud_terrastodon_user_input::TerminalGuard;
use cloud_terrastodon_user_input::apply_terminal_control;
use eyre::Result;
use ratatui::Frame;
use std::io::Write;

/// Owns the explorer's inline viewport and delivery of captured tracing output.
/// Nested pickers suspend this viewport; their logs stay buffered until it owns
/// the terminal again. Only successfully printed records are acknowledged.
pub(crate) struct ObjectBrowserTerminal {
    screen: InlineTerminal,
    logs: PickerLogBufferHandle,
    log_cursor: usize,
    height: u16,
    // Closing raw mode for cleanup is different from yielding to a child picker.
    // Only the current owner may print directly after closing its viewport.
    suspended: bool,
}

impl ObjectBrowserTerminal {
    pub(crate) fn new(logs: PickerLogBufferHandle) -> Result<Self> {
        let height = viewport_height()?;
        Ok(Self {
            screen: InlineTerminal::new(height)?,
            logs,
            log_cursor: 0,
            height,
            suspended: false,
        })
    }

    pub(crate) fn draw(&mut self, draw: impl FnOnce(&mut Frame<'_>)) -> Result<()> {
        if !self.is_active() {
            return Ok(());
        }
        let height = viewport_height()?;
        if self.height != height {
            // Ratatui clips an inline viewport on shrink, but its requested
            // height is fixed. Recreate only on height changes so it also grows.
            self.screen.close()?;
            self.screen = InlineTerminal::new(height)?;
            self.height = height;
        }
        self.flush_logs()?;
        self.screen.draw(draw)
    }

    fn flush_logs(&mut self) -> Result<()> {
        publish_logs(self.logs.as_ref(), &mut self.log_cursor, |records| {
            self.screen.print_logs(records)
        })
    }

    pub(crate) fn apply_control(
        &mut self,
        control: TerminalControl,
        guard: &mut TerminalGuard,
    ) -> Result<()> {
        let suspended = matches!(control, TerminalControl::Suspend { .. });
        apply_terminal_control(control, guard, self)?;
        // A closed shutdown viewport needs no suspend backend operation, but
        // the acknowledged handoff still prevents direct writes until Resume.
        self.suspended = suspended;
        Ok(())
    }

    pub(crate) fn close(&mut self) -> Result<()> {
        let printed = if self.is_active() {
            self.flush_logs()
        } else {
            Ok(())
        };
        // Restore input even when printing fails.
        let restored = self.screen.close();
        printed.and(restored)
    }

    /// Call after closing the viewport, while still holding the terminal guard.
    /// Cleanup futures may have logged while the guard captured stderr. Normal
    /// console mode is restored, and no parent owner has resumed yet, so these
    /// final events can be printed directly before releasing ownership.
    pub(crate) fn flush_after_close(&mut self) -> Result<()> {
        if self.suspended {
            return Ok(());
        }
        eyre::ensure!(!self.is_active(), "the explorer viewport is still active");
        publish_logs(self.logs.as_ref(), &mut self.log_cursor, |records| {
            let mut stderr = std::io::stderr().lock();
            for record in records {
                writeln!(stderr, "{record}")?;
            }
            stderr.flush()?;
            Ok(())
        })
    }
}

impl TerminalBackend for ObjectBrowserTerminal {
    fn is_active(&self) -> bool {
        self.screen.is_active()
    }

    fn suspend(&mut self) -> Result<()> {
        self.close()?;
        self.suspended = true;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if !self.is_active() {
            let height = viewport_height()?;
            self.screen = InlineTerminal::new(height)?;
            self.height = height;
            self.suspended = false;
            self.flush_logs()?;
        }
        Ok(())
    }
}

fn viewport_height() -> Result<u16> {
    let (_, rows) = crossterm::terminal::size()?;
    // Keep recent native log lines visible above the browser on normal-sized
    // terminals. Small terminals use their available height for navigation.
    Ok(rows.saturating_sub(4).max(6).min(rows.max(1)))
}

fn publish_logs(
    logs: &dyn PickerLogBuffer,
    cursor: &mut usize,
    write: impl FnOnce(&[String]) -> Result<()>,
) -> Result<()> {
    let mut through = *cursor;
    let records = logs.formatted_records_since(&mut through);
    if !records.is_empty() {
        write(&records)?;
    }
    logs.acknowledge_through(through);
    *cursor = through;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_user_input::PickerLogRecord;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    #[derive(Default)]
    struct BufferedLogs {
        records: Mutex<Vec<String>>,
        acknowledged: AtomicUsize,
    }

    impl PickerLogBuffer for BufferedLogs {
        fn records_since(&self, _: &mut usize) -> Vec<PickerLogRecord> {
            panic!("terminal output must preserve the subscriber's formatting");
        }

        fn formatted_records_since(&self, cursor: &mut usize) -> Vec<String> {
            let records = self.records.lock().unwrap();
            let start = (*cursor).max(self.acknowledged.load(Ordering::SeqCst));
            *cursor = records.len();
            records.iter().skip(start).cloned().collect()
        }

        fn acknowledge_through(&self, cursor: usize) {
            self.acknowledged.fetch_max(cursor, Ordering::SeqCst);
        }
    }

    #[test]
    fn publishing_retries_failed_output_and_retains_logs_arriving_during_printing() {
        let logs = BufferedLogs::default();
        let first = "\x1b[31mERROR\x1b[0m request failed\n  caused by: détail 界".to_owned();
        logs.records.lock().unwrap().push(first.clone());
        let mut cursor = 0;

        let error = publish_logs(&logs, &mut cursor, |_| {
            eyre::bail!("terminal write failed");
        });
        assert!(error.is_err());
        assert_eq!(cursor, 0);
        assert_eq!(logs.acknowledged.load(Ordering::SeqCst), 0);

        let mut printed = Vec::new();
        publish_logs(&logs, &mut cursor, |records| {
            printed.extend_from_slice(records);
            logs.records
                .lock()
                .unwrap()
                .push("cleanup completed".to_owned());
            Ok(())
        })
        .unwrap();
        assert_eq!(printed, std::slice::from_ref(&first));
        assert_eq!(cursor, 1);

        publish_logs(&logs, &mut cursor, |records| {
            printed.extend_from_slice(records);
            Ok(())
        })
        .unwrap();
        assert_eq!(printed, [first, "cleanup completed".to_owned()]);
        assert_eq!(logs.acknowledged.load(Ordering::SeqCst), 2);
        publish_logs(&logs, &mut cursor, |_| {
            panic!("printed logs must not replay")
        })
        .unwrap();
    }
}
