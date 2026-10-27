use crate::ExcalidrawLogLevel;

/// A structured canvas log event. Message and field values remain intact so
/// Excalidraw can wrap each native text element using its actual font metrics.
#[derive(Debug, Clone)]
pub struct ExcalidrawLogRecord {
    pub level: ExcalidrawLogLevel,
    pub message: String,
    pub fields: Vec<(String, String)>,
}
