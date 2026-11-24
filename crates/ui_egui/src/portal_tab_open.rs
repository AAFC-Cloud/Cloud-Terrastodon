use crate::app::MyApp;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::PointerButton;
use eframe::egui::Response;

/// Open a destination using the current tab policy or in a separate background tab.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PortalTabOpen {
    Current(PortalTabTarget),
    Background(PortalTabTarget),
}

impl PortalTabOpen {
    pub fn opens_in_background(response: &Response) -> bool {
        response.middle_clicked()
            || (response.clicked_by(PointerButton::Primary)
                && response
                    .ctx
                    .input(|input| input.modifiers.ctrl || input.modifiers.command))
    }

    pub fn from_response(
        response: &Response,
        target: impl FnOnce() -> PortalTabTarget,
    ) -> Option<Self> {
        if Self::opens_in_background(response) {
            Some(Self::Background(target()))
        } else if response.clicked() {
            Some(Self::Current(target()))
        } else {
            None
        }
    }

    /// Apply the navigation and report whether the current view should stop drawing.
    pub fn apply(self, app: &mut MyApp) -> bool {
        match self {
            Self::Current(target) => {
                app.navigate_tab(target);
                true
            }
            Self::Background(target) => {
                app.open_background_tab(target);
                false
            }
        }
    }
}
