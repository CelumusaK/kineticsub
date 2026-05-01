use egui::{Pos2, Rect, Stroke};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN};
use super::{LABEL_W};

// ── Playhead ───────────────────────────────────────────────────────────────────

pub fn draw_playhead(painter: &egui::Painter, vm: &EditorViewModel, origin: Pos2, tracks_rect: Rect) {
let ph_px = vm.time_to_px(vm.current_time());
let ph_x = origin.x + LABEL_W + ph_px;

if ph_x >= origin.x + LABEL_W && ph_x <= origin.x + tracks_rect.width() {
    painter.line_segment([
            Pos2::new(ph_x, origin.y),
            Pos2::new(ph_x, origin.y + tracks_rect.height()),
        ],
        Stroke::new(1.5, ACCENT_CYAN),
    );
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(ph_x - 6.0, origin.y),
            Pos2::new(ph_x + 6.0, origin.y),
            Pos2::new(ph_x,       origin.y + 11.0),
        ],
        ACCENT_CYAN,
        Stroke::NONE,
    ));
}

}