use std::fmt::Debug;
use tracing::Event;
use tracing::Level;
use tracing::field::Field;
use tracing::field::Visit;

/// Extract the message and origin from native or log-bridged simplecss warnings.
pub(crate) struct SimpleCssWarning {
    from_simplecss: bool,
    message: Option<String>,
}

impl SimpleCssWarning {
    pub(crate) fn message(event: &Event<'_>) -> Option<String> {
        let metadata = event.metadata();
        let from_simplecss = Self::matches_target(metadata.target());
        if *metadata.level() != Level::WARN
            || (!from_simplecss && metadata.fields().field("log.target").is_none())
        {
            return None;
        }
        let mut warning = Self {
            from_simplecss,
            message: None,
        };
        event.record(&mut warning);
        if warning.from_simplecss {
            warning.message
        } else {
            None
        }
    }

    fn matches_target(target: &str) -> bool {
        target == "simplecss" || target.starts_with("simplecss::")
    }
}

impl Visit for SimpleCssWarning {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "log.target" => self.from_simplecss |= Self::matches_target(value),
            "message" => self.message = Some(value.to_owned()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }
}
