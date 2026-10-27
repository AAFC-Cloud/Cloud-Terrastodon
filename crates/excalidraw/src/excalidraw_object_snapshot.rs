use crate::ExcalidrawObjectField;
use cloud_terrastodon_registry::DiscoveryPlan;

/// An owned, bounded observation suitable for drawing after the source arena is released.
/// No reflected pointers, executable operations, or borrowed values cross this boundary.
#[derive(Debug, Clone)]
pub struct ExcalidrawObjectSnapshot {
    pub address: String,
    pub shape: String,
    pub fields: Vec<ExcalidrawObjectField>,
    /// False means the observer's field limit was reached; the drawing says so explicitly.
    pub fields_complete: bool,
    /// Discovery plans additionally show their dependency graph below the object's fields.
    pub plan: Option<DiscoveryPlan>,
}
