use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, BG_BASE, BORDER, TEXT_DIM, TEXT_MUTED};
use super::{LABEL_W, RULER_H};

pub fn draw_ruler(ui: &mut egui::Ui, painter: &egui::Painter, vm: &mut EditorViewModel, origin: Pos2, tracks_rect: Rect) {
    let ruler_rect = Rect::from_min_size(
        Pos2::new(origin.x + LABEL_W, origin.y),
        Vec2::new(tracks_rect.width() - LABEL_W, RULER_H),
    );
    painter.rect_filled(ruler_rect, 0.0, BG_BASE);

    // Interactive ruler scrubbing
    let ruler_resp = ui.interact(ruler_rect, ui.id().with("timeline_ruler_bar"), egui::Sense::click_and_drag());
    
    if ruler_resp.drag_started() {
        vm.is_scrubbing = true;
    }

    if ruler_resp.clicked() || ruler_resp.dragged() {
        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) {
            let click_x = pos.x - (origin.x + LABEL_W);
            let t = vm.px_to_time(click_x).clamp(0.0, vm.project.duration);
            vm.seek_to(t);
        }
    }

    let visible_secs = ((tracks_rect.width() - LABEL_W) / vm.timeline_zoom) as i32 + 2;
    let first_sec    = vm.timeline_scroll.floor() as i32;
    let max_sec      = vm.project.duration.ceil() as i32;

    let tick_interval = if vm.timeline_zoom >= 200.0 { 1 }
                        else if vm.timeline_zoom >= 50.0 { 5 }
                        else { 10 };

    for s in first_sec.max(0)..=(first_sec + visible_secs).min(max_sec + 2) {
        let px     = vm.time_to_px(s as f64);
        let tick_x = origin.x + LABEL_W + px;
        if tick_x < origin.x + LABEL_W || tick_x > origin.x + tracks_rect.width() { continue; }

        let is_major = s % tick_interval == 0;
        let tick_h   = if is_major { 7.0 } else { 3.0 };

        painter.line_segment([
                Pos2::new(tick_x, ruler_rect.max.y - tick_h),
                Pos2::new(tick_x, ruler_rect.max.y),
            ],
            Stroke::new(1.0, if is_major { TEXT_MUTED } else { BORDER }),
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
    
    let (mut exp_start, mut exp_end) = vm.project.export_range.unwrap_or((0.0, vm.project.duration));
    
    let start_px = origin.x + LABEL_W + vm.time_to_px(exp_start);
    let end_px = origin.x + LABEL_W + vm.time_to_px(exp_end);

    let marker_y = origin.y + 4.0;
    let marker_h = RULER_H - 8.0;

    if end_px > start_px {
        painter.line_segment(
            [Pos2::new(start_px.max(origin.x + LABEL_W), marker_y + marker_h / 2.0),
             Pos2::new(end_px.min(origin.x + tracks_rect.width()), marker_y + marker_h / 2.0)],
            Stroke::new(2.0, ACCENT_CYAN.linear_multiply(0.4))
        );
    }

    let head_size = Vec2::new(8.0, marker_h);
    let start_rect = Rect::from_min_size(Pos2::new(start_px - head_size.x/2.0, marker_y), head_size);
    let end_rect = Rect::from_min_size(Pos2::new(end_px - head_size.x/2.0, marker_y), head_size);

    let mut changed = false;

    let start_resp = ui.interact(start_rect, ui.id().with("exp_start"), egui::Sense::click_and_drag());
    if start_resp.dragged() {
        exp_start = (exp_start + start_resp.drag_delta().x as f64 / vm.timeline_zoom as f64).max(0.0).min(exp_end - 0.1);
        changed = true;
    }

    let end_resp = ui.interact(end_rect, ui.id().with("exp_end"), egui::Sense::click_and_drag());
    if end_resp.dragged() {
        exp_end = (exp_end + end_resp.drag_delta().x as f64 / vm.timeline_zoom as f64).max(exp_start + 0.1).min(vm.project.duration);
        changed = true;
    }

    if start_resp.double_clicked() || end_resp.double_clicked() {
        vm.project.export_range = None;
    } else if changed {
        vm.project.export_range = Some((exp_start, exp_end));
    }

    let marker_color = Color32::from_rgb(160, 200, 255);
    let draw_head = |rect: Rect, hovered: bool| {
        if rect.max.x < origin.x + LABEL_W || rect.min.x > origin.x + tracks_rect.width() { return; }
        painter.rect_filled(rect, 2.0, if hovered { Color32::WHITE } else { marker_color });
    };

    draw_head(start_rect, start_resp.hovered() || start_resp.dragged());
    draw_head(end_rect, end_resp.hovered() || end_resp.dragged());

    painter.line_segment([
            Pos2::new(origin.x, origin.y + RULER_H),
            Pos2::new(origin.x + tracks_rect.width(), origin.y + RULER_H),
        ],
        Stroke::new(1.0, BORDER),
    );
}