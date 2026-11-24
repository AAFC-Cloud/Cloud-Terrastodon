use teamy_tracing::capture::LogRecord;
use teamy_tracing::capture::LogValue;
use teamy_tracing::gui::LogView;

pub(crate) const DEFAULT_LOG_EXCLUSIONS: &str =
    "simplecss,figue::schema::from_schema[extract_field_default: successfully extracted default]";

/// A rule for hiding captured records in the Logs view.
pub(crate) enum PortalLogExclusion<'a> {
    Text(LogView),
    MessageAtTarget { target: &'a str, message: &'a str },
}

impl<'a> PortalLogExclusion<'a> {
    /// Parse rules separated by commas outside square brackets. Plain text uses
    /// the log view's case-insensitive search. `target[message]` matches a complete
    /// target and message exactly. A message can contain commas and balanced brackets.
    pub fn parse_list(input: &'a str) -> Vec<Self> {
        let mut bracket_depth = 0_usize;
        input
            .split(move |character| {
                match character {
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth = bracket_depth.saturating_sub(1),
                    _ => {}
                }
                character == ',' && bracket_depth == 0
            })
            .map(str::trim)
            .filter(|rule| !rule.is_empty())
            .map(|rule| {
                if let Some((target, message)) = rule.split_once('[')
                    && !target.trim_end().is_empty()
                    && let Some(message) = message.strip_suffix(']')
                {
                    Self::MessageAtTarget {
                        target: target.trim_end(),
                        message,
                    }
                } else {
                    Self::Text(LogView {
                        filter: rule.to_owned(),
                        ..LogView::default()
                    })
                }
            })
            .collect()
    }

    pub fn matches(&self, record: &LogRecord) -> bool {
        match *self {
            Self::Text(ref view) => view.matches(record),
            Self::MessageAtTarget { target, message } => record.message.as_ref() == message
                && (record.target.as_ref() == target
                    || record.fields.iter().any(|(name, value)| {
                        name == "log.target"
                            && matches!(value, LogValue::Text(value) if value.as_str() == target)
                    })),
        }
    }
}
