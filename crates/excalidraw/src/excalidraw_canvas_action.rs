/// An explicit action requested through a saved, visible canvas control.
///
/// The caption identifies the action; Excalidraw element IDs are used only to
/// maintain generated presentation, so copying a control keeps its meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcalidrawCanvasAction {
    AddTarget,
}

impl ExcalidrawCanvasAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::AddTarget => "Add Target",
        }
    }

    pub(crate) fn from_label(label: &str) -> Option<Self> {
        label
            .trim()
            .eq_ignore_ascii_case(Self::AddTarget.label())
            .then_some(Self::AddTarget)
    }
}
