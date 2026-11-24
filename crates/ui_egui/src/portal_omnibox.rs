use eframe::egui::Align;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Margin;
use eframe::egui::Modifiers;
use eframe::egui::TextEdit;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::text::CCursor;
use eframe::egui::text::CCursorRange;

pub(crate) const OMNIBOX_CONTENT_HEIGHT: f32 = 26.0;

/// Display the current address and use edits in the same field to search resources.
pub(crate) fn portal_omnibox(
    ui: &mut Ui,
    address: &str,
    address_tooltip: &str,
    monospace: bool,
    search: &mut String,
    focus_search: bool,
) -> bool {
    // The field keeps focus when entering a query changes the active destination.
    let id = Id::new("portal-omnibox");
    let search_mode = id.with("search-mode");
    let focused = ui.memory(|memory| memory.has_focus(id));
    let escaped = (focused || ui.memory(|memory| memory.had_focus_last_frame(id)))
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
    if escaped {
        search.clear();
        ui.memory_mut(|memory| memory.surrender_focus(id));
    } else if focus_search {
        ui.memory_mut(|memory| memory.request_focus(id));
        ui.data_mut(|data| data.insert_temp(search_mode, true));
    }

    let searching = !escaped
        && (!search.is_empty()
            || focus_search
            || (focused && ui.data(|data| data.get_temp(search_mode).unwrap_or(false))));
    let mut text = if searching {
        search.clone()
    } else {
        address.to_owned()
    };
    let edit = TextEdit::singleline(&mut text)
        .id(id)
        .hint_text("Search resources (Ctrl+K)")
        .font(if monospace && !searching {
            TextStyle::Monospace
        } else {
            TextStyle::Body
        })
        .frame(Frame::NONE)
        .margin(Margin::symmetric(2, 0))
        .vertical_align(Align::Center)
        .desired_width(ui.available_width())
        .min_size(Vec2::new(0.0, OMNIBOX_CONTENT_HEIGHT));
    let mut output = edit.show(ui);
    let changed = output.response.changed();
    if changed {
        *search = text;
        ui.data_mut(|data| data.insert_temp(search_mode, true));
    } else if focus_search || (output.response.gained_focus() && !searching) {
        // Opening the field selects the address so typing starts a fresh search.
        output.state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(0),
            CCursor::new(text.chars().count()),
        )));
        output.state.store(ui.ctx(), id);
        ui.ctx().request_repaint();
    }
    if escaped || output.response.lost_focus() {
        ui.data_mut(|data| data.remove::<bool>(search_mode));
    }
    if !output.response.has_focus() {
        output.response.response.on_hover_text(if searching {
            "Search resources (Ctrl+K)"
        } else {
            address_tooltip
        });
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;

    #[test]
    fn search_replaces_the_address_and_escape_clears_the_query() {
        let context = Context::default();
        let address = "ct az resource list";
        let frame = |search: &mut String, events, focus_search| {
            let mut changed = false;
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 80.0))),
                    events,
                    ..Default::default()
                },
                |ui| changed = portal_omnibox(ui, address, address, true, search, focus_search),
            );
            output.textures_delta.clear();
            changed
        };
        let mut search = String::new();
        assert!(!frame(&mut search, vec![], true));
        assert!(frame(
            &mut search,
            vec![Event::Text("storage".into())],
            false
        ));
        assert_eq!(search, "storage");

        frame(
            &mut search,
            vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            false,
        );
        assert!(search.is_empty());

        let pos = Pos2::new(65.0, 15.0);
        for pressed in [true, false] {
            frame(
                &mut search,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                false,
            );
        }
        assert!(frame(
            &mut search,
            vec![Event::Text("network".into())],
            false
        ));
        assert_eq!(search, "network");
    }
}
