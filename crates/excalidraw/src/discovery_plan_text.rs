//! Text measurements shared by layout occupancy and Excalidraw rendering.

pub(crate) const EDGE_FONT_SIZE: f64 = 14.0;
pub(crate) const LINE_HEIGHT: f64 = 1.25;
pub(crate) const EDGE_LABEL_WIDTH: f64 = 260.0;

pub(crate) fn edge_label_size(label: &str) -> (f64, f64) {
    if label.is_empty() {
        return (0.0, 0.0);
    }
    (
        EDGE_LABEL_WIDTH,
        wrap(label, 30).lines().count() as f64 * EDGE_FONT_SIZE * LINE_HEIGHT,
    )
}

pub(crate) fn wrap(text: &str, columns: usize) -> String {
    text.lines()
        .map(|line| {
            line.chars()
                .collect::<Vec<_>>()
                .chunks(columns)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
