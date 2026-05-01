use egui::{Pos2, Rect, Stroke, Vec2};
use crate::views::theme::{BG_PANEL, BORDER, TEXT_DIM};
use super::{LABEL_W, RULER_H, TRACK_H, TRACKS};

// ── Labels ─────────────────────────────────────────────────────────────────────

pub fn draw_track_labels(painter: &egui::Painter, origin: Pos2, tracks_rect: Rect) {
painter.rect_filled(
Rect::from_min_size(origin, Vec2::new(LABEL_W, tracks_rect.height())),
0.0,
BG_PANEL,
);
painter.line_segment([
Pos2::new(origin.x + LABEL_W, origin.y),
Pos2::new(origin.x + LABEL_W, origin.y + tracks_rect.height()),
],
Stroke::new(1.0, BORDER),
);
}

// ── Track separators ───────────────────────────────────────────────────────────

pub fn draw_track_lines(painter: &egui::Painter, origin: Pos2, tracks_rect: Rect) {
for (i, name) in TRACKS.iter().enumerate() {
let row_y = origin.y + RULER_H + (i as f32 * TRACK_H);
if i % 2 == 0 {
painter.rect_filled(
Rect::from_min_size(
Pos2::new(origin.x + LABEL_W, row_y),
Vec2::new(tracks_rect.width() - LABEL_W, TRACK_H),
),
0.0,
egui::Color32::from_rgba_unmultiplied(255, 255, 255, 3),
);
}
painter.text(
Pos2::new(origin.x + 8.0, row_y + TRACK_H / 2.0),
egui::Align2::LEFT_CENTER,
*name,
egui::FontId::proportional(9.5),
TEXT_DIM,
);
painter.line_segment([
Pos2::new(origin.x, row_y + TRACK_H),
Pos2::new(origin.x + tracks_rect.width(), row_y + TRACK_H),
],
Stroke::new(1.0, BORDER),
);
}
}