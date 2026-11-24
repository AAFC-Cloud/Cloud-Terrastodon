use crate::icons::ACCESS_CONTROL_ICON;
use crate::icons::ACTIVITY_LOG_ICON;
use crate::icons::STORAGE_EXPLORER_ICON;
use crate::icons::TAG_ICON;
use crate::icons::networking_icon;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_tags_browser::supports_tags;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use eframe::egui::Align2;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::ImageSource;
use eframe::egui::Panel;
use eframe::egui::Rect;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

pub(crate) fn scope_navigation(
    ui: &mut Ui,
    scope: &PortalResourceScope,
    current: Option<PortalScopeView>,
    expanded: &mut bool,
    overview_icon: ImageSource<'static>,
) -> Option<PortalTabOpen> {
    let show_labels = *expanded;
    Panel::left(ui.make_persistent_id("scope_details_navigation"))
        .exact_size(if show_labels { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::new().fill(Color32::WHITE))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(ui, expanded, "resource navigation");
            let mut destination = navigation_tile(
                ui,
                "Overview",
                overview_icon,
                current.is_none(),
                show_labels,
                || scope.overview(),
            );
            for (view, icon) in [
                (PortalScopeView::ActivityLog, ACTIVITY_LOG_ICON),
                (PortalScopeView::AccessControl, ACCESS_CONTROL_ICON),
                (PortalScopeView::Tags, TAG_ICON),
                (PortalScopeView::StorageExplorer, STORAGE_EXPLORER_ICON),
                (PortalScopeView::Networking, networking_icon()),
            ] {
                match view {
                    PortalScopeView::Tags if !supports_tags(scope) => continue,
                    PortalScopeView::StorageExplorer | PortalScopeView::Networking
                        if !matches!(scope, PortalResourceScope::StorageAccount(_)) =>
                    {
                        continue;
                    }
                    _ => {}
                }
                if let Some(open) = navigation_tile(
                    ui,
                    view.label(),
                    icon,
                    current == Some(view),
                    show_labels,
                    || PortalTabTarget::ScopeView {
                        scope: scope.clone(),
                        view,
                    },
                ) {
                    destination = Some(open);
                }
            }
            destination
        })
        .inner
}

pub(crate) fn navigation_tile(
    ui: &mut Ui,
    title: &str,
    icon: ImageSource<'static>,
    selected: bool,
    show_label: bool,
    target: impl FnOnce() -> PortalTabTarget,
) -> Option<PortalTabOpen> {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.0), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            title,
        )
    });

    let painter = ui.painter().with_clip_rect(rect);
    let background = match (
        selected,
        response.hovered() || response.is_pointer_button_down_on(),
    ) {
        (false, false) => None,
        (false, true) => Some(240),
        (true, false) => Some(228),
        (true, true) => Some(218),
    };
    if let Some(gray) = background {
        painter.rect_filled(rect, 0.0, Color32::from_gray(gray));
    }
    let accent = Color32::from_rgb(0, 120, 212);
    if selected {
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            0.0,
            accent,
        );
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            0.0,
            Stroke::new(1.0, accent),
            StrokeKind::Inside,
        );
    }

    let icon_center = if show_label {
        rect.left_center() + Vec2::new(19.0, 0.0)
    } else {
        rect.center()
    };
    Image::new(icon).paint_at(ui, Rect::from_center_size(icon_center, Vec2::splat(18.0)));
    if show_label {
        painter.text(
            rect.left_center() + Vec2::new(36.0, 0.0),
            Align2::LEFT_CENTER,
            title,
            TextStyle::Body.resolve(ui.style()),
            ui.visuals().text_color(),
        );
    }
    let response = response.on_hover_text(title);
    PortalTabOpen::from_response(&response, target)
        .filter(|open| !selected || matches!(open, PortalTabOpen::Background(_)))
}
