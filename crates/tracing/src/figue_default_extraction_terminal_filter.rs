use std::fmt::Debug;
use tracing::Event;
use tracing::Level;
use tracing::Metadata;
use tracing::Subscriber;
use tracing::field::Field;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::Filter;

/// Suppress Figue's successful default extraction events on the terminal.
pub(crate) struct FigueDefaultExtractionTerminalFilter;

impl<S: Subscriber> Filter<S> for FigueDefaultExtractionTerminalFilter {
    fn enabled(&self, _metadata: &Metadata<'_>, _context: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _context: &Context<'_, S>) -> bool {
        let metadata = event.metadata();
        if metadata.target() != "figue::schema::from_schema" || *metadata.level() != Level::DEBUG {
            return true;
        }

        let mut successful_extraction = false;
        event.record(&mut |field: &Field, value: &dyn Debug| {
            if field.name() == "message" {
                successful_extraction =
                    format!("{value:?}") == "extract_field_default: successfully extracted default";
            }
        });
        !successful_extraction
    }
}
