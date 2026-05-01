use egui::{Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL, BORDER, TEXT_DIM};
use super::{HEADER_H};

// ── Header ─────────────────────────────────────────────────────────────────────

pub fn draw_header(ui: &mut egui::Ui, vm: &EditorViewModel, panel_rect: Rect) {
let hdr_rect = Rect::from_min_size(panel_rect.min, Vec2::new(panel_rect.width(), HEADER_H));
let (_, p) = ui.allocate_painter(Vec2::new(panel_rect.width(), HEADER_H), egui::Sense::hover());
p.rect_filled(hdr_rect, 0.0, BG_PANEL);
p.rect_stroke(hdr_rect, 0.0, Stroke::new(1.0, BORDER));
p.text(
Pos2::new(hdr_rect.min.x + 14.0, hdr_rect.center().y),
egui::Align2::LEFT_CENTER,
"TIMELINE",
egui::FontId::proportional(11.0),
TEXT_DIM,
);
p.text(
Pos2::new(hdr_rect.max.x - 12.0, hdr_rect.center().y),
egui::Align2::RIGHT_CENTER,
format!("{:.0} px/s", vm.timeline_zoom),
egui::FontId::proportional(10.0),
TEXT_DIM,
);
}