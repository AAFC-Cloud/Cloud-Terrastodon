use eframe::egui::Context;
use eframe::egui::FontData;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use eframe::egui::FontTweak;
use eframe::egui::epaint::text::VariationCoords;
use std::sync::Arc;

/// The bundled font's copyright notices and SIL Open Font License 1.1.
pub const LICENSE: &str = include_str!("../assets/fonts/allo/OFL.txt");

/// Install Allo's variable font while preserving egui's existing glyph fallbacks.
///
/// Interface text uses weight 400, Text optical size 14, and Friendly roundness 50.
/// Individual `RichText` values can select other `wght`, `opsz`, and `ROND` values
/// from the same embedded bytes. Monospaced diagnostics keep their existing
/// primary font, with Allo available for otherwise missing symbols.
///
/// See egui's [`FontTweak`] for defaults and
/// [`RichText::variation`](eframe::egui::RichText::variation) for per-run overrides.
pub fn install(context: &Context) {
    let mut definitions = FontDefinitions::default();
    definitions.font_data.insert(
        "Allo Variable".to_owned(),
        Arc::new(
            FontData::from_static(include_bytes!("../assets/fonts/allo/AlloVariable.ttf")).tweak(
                FontTweak {
                    coords: VariationCoords::new([("wght", 400.0), ("opsz", 14.0), ("ROND", 50.0)]),
                    ..Default::default()
                },
            ),
        ),
    );
    definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Allo Variable".to_owned());
    definitions
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("Allo Variable".to_owned());
    context.set_fonts(definitions);
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Color32;
    use eframe::egui::FontId;
    use eframe::egui::RawInput;
    use eframe::egui::RichText;

    #[test]
    fn bundled_variable_font_renders_branding_and_navigation_symbols_without_a_window() {
        let context = Context::default();
        install(&context);
        let mut output = context.run_ui(RawInput::default(), |ui| {
            ui.fonts_mut(|fonts| {
                assert!(fonts.has_glyphs(&FontId::proportional(17.0), "Cloud Terrastodon"));
                let missing: Vec<_> = "←→↑↓↔↕↖↗↘↙⇐⇒⇔↩↵↻↺«»▼▶◀▲"
                    .chars()
                    .filter(|character| !fonts.has_glyph(&FontId::proportional(14.0), *character))
                    .collect();
                assert!(missing.is_empty(), "Missing navigation glyphs: {missing:?}");
            });
            ui.label(
                RichText::new("Cloud Terrastodon")
                    .color(Color32::WHITE)
                    .size(17.0)
                    .variation("wght", 600.0)
                    .variation("opsz", 17.0),
            );
        });
        // No renderer exists in this headless test to apply the texture uploads.
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }
}
