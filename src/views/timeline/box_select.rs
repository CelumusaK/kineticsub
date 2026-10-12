use egui::{Context, Pos2, Rect, Stroke};
use crate::models::types::MediaType;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::ACCENT_CYAN;
use super::{LABEL_W, RULER_H, TRACK_H};

pub fn draw_box_select(
    _ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    _tracks_rect: Rect,
    bg_resp: &egui::Response,
    ctx: &Context,
) {
    let shift = ctx.input(|i| i.modifiers.shift);

    if bg_resp.drag_started() && shift {
        if let Some(pos) = bg_resp.interact_pointer_pos() {
            vm.box_select_start = Some(pos);
            vm.box_select_end   = Some(pos);
        }
    }

    if bg_resp.dragged() {
        if vm.box_select_start.is_some() {
            vm.box_select_end = bg_resp.interact_pointer_pos();
        }
    }

    if bg_resp.drag_stopped() {
        if let (Some(start), Some(end)) = (vm.box_select_start, vm.box_select_end) {
            let min_x = start.x.min(end.x) - (origin.x + LABEL_W);
            let max_x = start.x.max(end.x) - (origin.x + LABEL_W);
            let t_min = vm.px_to_time(min_x);
            let t_max = vm.px_to_time(max_x);

            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);

            vm.selected_ids.clear();

            // Subtitle track (Row 0) vertical bounds
            let sub_y_min = origin.y + RULER_H;
            let sub_y_max = sub_y_min + TRACK_H;
            if max_y > sub_y_min && min_y < sub_y_max {
                for sub in &vm.project.subtitles {
                    if sub.timeline_end > t_min && sub.timeline_start < t_max {
                        vm.selected_ids.insert(sub.id.clone());
                    }
                }
            }

            // Media tracks vertical bounds
            for m in &vm.project.media_files {
                if m.on_timeline {
                    let row_idx = vm.get_track_row(m.track_index, m.media_type == MediaType::Audio);
                    let m_y_min = origin.y + RULER_H + (row_idx as f32 * TRACK_H);
                    let m_y_max = m_y_min + TRACK_H;

                    if max_y > m_y_min && min_y < m_y_max {
                        if m.timeline_offset + m.duration > t_min && m.timeline_offset < t_max {
                            vm.selected_ids.insert(m.id.clone());
                        }
                    }
                }
            }

            vm.selected_id = vm.selected_ids.iter().next().cloned();
        }
        vm.box_select_start = None;
        vm.box_select_end   = None;
    }

    if let (Some(start), Some(end)) = (vm.box_select_start, vm.box_select_end) {
        let sel_rect = Rect::from_two_pos(start, end);
        painter.rect_filled(sel_rect, 0.0, egui::Color32::from_rgba_unmultiplied(34, 211, 238, 20));
        painter.rect_stroke(sel_rect, 0.0, Stroke::new(1.0_f32, ACCENT_CYAN));
    }
}