use egui::{Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_BASE, BORDER, TEXT_DIM};
use super::{LABEL_W, RULER_H};

// ── Ruler ──────────────────────────────────────────────────────────────────────

pub fn draw_ruler(painter: &egui::Painter, vm: &EditorViewModel, origin: Pos2, tracks_rect: Rect) {
let ruler_rect = Rect::from_min_size(
Pos2::new(origin.x + LABEL_W, origin.y),
Vec2::new(tracks_rect.width() - LABEL_W, RULER_H),
);
painter.rect_filled(ruler_rect, 0.0, BG_BASE);

let visible_secs = ((tracks_rect.width() - LABEL_W) / vm.timeline_zoom) as i32 + 2;
let first_sec    = vm.timeline_scroll as i32;
let max_sec      = vm.project.duration.ceil() as i32;

let tick_interval = if vm.timeline_zoom >= 200.0 { 1 }
                    else if vm.timeline_zoom >= 50.0 { 5 }
                    else { 10 };

for s in first_sec..=(first_sec + visible_secs).min(max_sec + 2) {
    let px     = vm.time_to_px(s as f64);
    let tick_x = origin.x + LABEL_W + px;
    if tick_x < origin.x + LABEL_W || tick_x > origin.x + tracks_rect.width() { continue; }

    let is_major = s % tick_interval == 0;
    let tick_h   = if is_major { 7.0 } else { 3.0 };

    painter.line_segment([
            Pos2::new(tick_x, ruler_rect.max.y - tick_h),
            Pos2::new(tick_x, ruler_rect.max.y),
        ],
        Stroke::new(1.0, if is_major { TEXT_DIM } else { BORDER }),
    );
    if is_major {
        painter.text(
            Pos2::new(tick_x + 2.0, ruler_rect.min.y + 3.0),
            egui::Align2::LEFT_TOP,
            format!("{:02}:{:02}", s / 60, s % 60),
            egui::FontId::proportional(9.0),
            TEXT_DIM,
        );
    }
}
painter.line_segment([
        Pos2::new(origin.x, origin.y + RULER_H),
        Pos2::new(origin.x + tracks_rect.width(), origin.y + RULER_H),
    ],
    Stroke::new(1.0, BORDER),
);

}