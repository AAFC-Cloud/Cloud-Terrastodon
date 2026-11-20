use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::windows::resources_window::resources_ui;
use eframe::egui::Align;
use eframe::egui::Color32;
use eframe::egui::FontId;
use eframe::egui::Image;
use eframe::egui::Layout;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

/// Service shortcuts and the real resource inventory share the landing page.
pub fn portal_home(app: &mut MyApp, ui: &mut Ui) {
    const MAX_CONTENT_WIDTH: f32 = 1120.0;
    const SIDE_MARGIN: f32 = 24.0;

    let available = ui.available_rect_before_wrap();
    let margin = SIDE_MARGIN.min(available.width() / 4.0);
    let width = (available.width() - 2.0 * margin).min(MAX_CONTENT_WIDTH);
    let content = Rect::from_center_size(available.center(), Vec2::new(width, available.height()));
    ui.scope_builder(UiBuilder::new().max_rect(content), |ui| {
        ui.shrink_clip_rect(content);
        ui.set_min_width(width);
        ui.vertical_centered(|ui| azure_services(app, ui));
        ui.add_space(24.0);
        ui.label(RichText::new("Resources").strong().size(19.0));
        ui.add_space(12.0);
        resources_ui(app, ui);
    });
}

fn azure_services(app: &mut MyApp, ui: &mut Ui) {
    const TILE_WIDTH: f32 = 112.0;
    const TILE_HEIGHT: f32 = 100.0;
    const TILE_GAP: f32 = 8.0;
    const PAGES: &[PortalPage] = &[
        PortalPage::Subscriptions,
        PortalPage::Resources,
        PortalPage::ResourceGroups,
        PortalPage::ManagementGroups,
        PortalPage::StorageAccounts,
        PortalPage::ContainerRegistries,
        PortalPage::KeyVaults,
    ];

    ui.label(RichText::new("Azure services").strong().size(19.0));
    ui.add_space(16.0);
    ui.spacing_mut().item_spacing = Vec2::splat(TILE_GAP);
    let tile_width = TILE_WIDTH.min(ui.available_width());
    let tile_size = Vec2::new(tile_width, TILE_HEIGHT);
    let columns = ((ui.available_width() + TILE_GAP) / (tile_width + TILE_GAP)) as usize;
    for row in PAGES.chunks(columns.max(1)) {
        let row_width = row.len() as f32 * (tile_width + TILE_GAP) - TILE_GAP;
        ui.allocate_ui_with_layout(
            Vec2::new(row_width, TILE_HEIGHT),
            Layout::left_to_right(Align::Center),
            |ui| {
                for &page in row {
                    if service_tile(ui, page, tile_size) {
                        app.navigate(page);
                    }
                }
            },
        );
    }
}

fn service_tile(ui: &mut Ui, page: PortalPage, size: Vec2) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), page.title()));
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, 2.0, Color32::from_rgb(243, 248, 252));
    }
    Image::new(page_icon(page)).paint_at(
        ui,
        Rect::from_center_size(
            Pos2::new(rect.center().x, rect.top() + 25.0),
            Vec2::splat(34.0),
        ),
    );
    let title = ui.painter().layout(
        page.title().to_owned(),
        FontId::proportional(13.0),
        Color32::from_rgb(0, 95, 184),
        (size.x - 8.0).max(1.0),
    );
    ui.painter().galley(
        Pos2::new(rect.center().x - title.size().x / 2.0, rect.top() + 52.0),
        title,
        Color32::from_rgb(0, 95, 184),
    );
    response
        .on_hover_text(format!("Browse {}", page.title().to_lowercase()))
        .clicked()
}
