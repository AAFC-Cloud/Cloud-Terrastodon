use std::time::Duration;
use std::time::Instant;
use teamy_tracing::capture::LogLevel;
use teamy_tracing::capture::LogRecord;
use teamy_tracing::gui::escape_display;

/// A timed notification pointing to an absolute position in the captured log buffer.
pub(crate) struct PortalLogToast {
    pub record_index: usize,
    pub level: LogLevel,
    pub text: String,
    remaining: Duration,
    last_update: Instant,
    hovered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::SystemTime;

    #[test]
    fn toast_countdown_resumes_from_the_remaining_time_after_hover() {
        let record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Info,
            target: Arc::from("test"),
            message: Arc::from("Example notification"),
            fields: Vec::new(),
            spans: Vec::new(),
            file: None,
            line: None,
        };
        let now = Instant::now();
        let mut toast = PortalLogToast::from_record(7, &record, now).unwrap();

        toast.tick(now + Duration::from_secs(2), false);
        let progress = toast.progress();
        assert!(progress < 1.0);
        toast.tick(now + Duration::from_secs(2), true);
        toast.tick(now + Duration::from_secs(8), true);
        toast.tick(now + Duration::from_secs(12), false);
        assert_eq!(toast.progress(), progress);
        assert!(toast.is_visible());

        toast.tick(now + Duration::from_secs(15), false);
        assert!(!toast.is_visible());
    }
}

impl PortalLogToast {
    const LIFETIME: Duration = Duration::from_secs(5);

    pub fn from_record(record_index: usize, record: &LogRecord, now: Instant) -> Option<Self> {
        if !matches!(record.level, LogLevel::Info | LogLevel::Error)
            || record.message.trim().is_empty()
        {
            return None;
        }
        let line = escape_display(record.message.lines().next().unwrap_or_default());
        let mut text: String = line.chars().take(160).collect();
        if text.len() < line.len() {
            text.push('…');
        }
        Some(Self {
            record_index,
            level: record.level,
            text,
            remaining: Self::LIFETIME,
            last_update: now,
            hovered: false,
        })
    }

    pub fn tick(&mut self, now: Instant, hovered: bool) {
        // A frame ending a hover can follow a long idle interval.
        if !self.hovered && !hovered {
            self.remaining = self
                .remaining
                .saturating_sub(now.saturating_duration_since(self.last_update));
        }
        self.last_update = now;
        self.hovered = hovered;
    }

    pub fn progress(&self) -> f32 {
        self.remaining.as_secs_f32() / Self::LIFETIME.as_secs_f32()
    }

    pub fn is_visible(&self) -> bool {
        !self.remaining.is_zero()
    }

    pub fn dismiss(&mut self) {
        self.remaining = Duration::ZERO;
    }
}
