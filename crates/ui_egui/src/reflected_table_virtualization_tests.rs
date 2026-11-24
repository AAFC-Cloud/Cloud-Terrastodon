use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Id;
use eframe::egui::ImageSource;
use eframe::egui::Modifiers;
use eframe::egui::MouseWheelUnit;
use eframe::egui::Pos2;
use eframe::egui::RawInput;
use eframe::egui::Rect;
use eframe::egui::TouchPhase;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use facet::Facet;
use std::cell::Cell;
use std::collections::HashMap;

const DRAWN_ROWS: &str = "virtualization-test-drawn-rows";

#[derive(Facet)]
struct InventoryRow {
    index: usize,
}

fn record_draw(ui: &Ui, value: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    let index = *value.column("index").unwrap().get::<usize>().unwrap();
    ui.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<usize>>(Id::new(DRAWN_ROWS))
            .push(index);
    });
    None
}

#[test]
fn cached_inventory_draws_only_viewport_rows_as_it_scrolls() {
    let context = Context::default();
    let columns = [ReflectedTableColumn::new("Name", "index").with_icon(record_draw)];
    let table = ReflectedTable::new("virtual-inventory", &columns);
    let rows: Vec<_> = (0..10_000).map(|index| InventoryRow { index }).collect();
    let mut order = None;
    let mut checked_rows = HashMap::new();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 240.0));
    let mut frame = |events| {
        let prepared = Cell::new(0);
        let mut output = context.run_ui(
            RawInput {
                events,
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                ui.data_mut(|data| data.insert_temp(Id::new(DRAWN_ROWS), Vec::<usize>::new()));
                table.show_cached(
                    ui,
                    &rows,
                    &mut checked_rows,
                    &mut order,
                    |row| {
                        prepared.set(prepared.get() + 1);
                        Id::new(row.index)
                    },
                    None,
                );
            },
        );
        output.textures_delta.clear();
        let drawn = context.data(|data| data.get_temp::<Vec<usize>>(Id::new(DRAWN_ROWS)).unwrap());
        (drawn, prepared.get())
    };

    let (initial, _) = frame(vec![]);
    assert_eq!(initial.first(), Some(&0));
    assert!(initial.len() <= 16, "drawn on initial frame: {initial:?}");

    frame(vec![
        Event::PointerMoved(Pos2::new(80.0, 120.0)),
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::ZERO,
            phase: TouchPhase::Start,
            modifiers: Modifiers::NONE,
        },
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0.0, -3_000.0),
            phase: TouchPhase::Move,
            modifiers: Modifiers::NONE,
        },
    ]);
    let (scrolled, prepared) = frame(vec![]);
    assert!(scrolled.first().is_some_and(|index| *index > 80));
    assert!(scrolled.len() <= 16, "drawn after scrolling: {scrolled:?}");
    assert_eq!(prepared, 0);
}
