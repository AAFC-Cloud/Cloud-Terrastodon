use crate::simple_css_warning::SimpleCssWarning;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::PoisonError;
use tracing::Event;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::Filter;

/// Admit each distinct simplecss warning to the terminal once per subscriber.
#[derive(Default)]
pub(crate) struct SimpleCssTerminalFilter {
    seen: Mutex<HashSet<String>>,
}

impl<S: Subscriber> Filter<S> for SimpleCssTerminalFilter {
    fn enabled(&self, _metadata: &Metadata<'_>, _context: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _context: &Context<'_, S>) -> bool {
        SimpleCssWarning::message(event).is_none_or(|message| {
            self.seen
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(message)
        })
    }
}
