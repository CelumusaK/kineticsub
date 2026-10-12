use egui::{Pos2, Rect, Stroke, Vec2};
use crate::models::types::TransitionType;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL, BORDER, TEXT_DIM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER};
use super::{LABEL_W, RULER_H, TRACK_H};

pub fn draw_track_labels(painter: &egui::Painter, origin: Pos2, tracks_rect: Rect, _vm: &EditorViewModel, _ui: &mut egui::Ui) {
    painter.rect_filled(
        Rect::from_min_size(origin, Vec2::new(LABEL_W, tracks_rect.height())),
        0.0,
        BG_PANEL,
    );
    painter.line_segment([
        Pos2::new(origin.x + LABEL_W, origin.y),
        Pos2::new(origin.x + LABEL_W, origin.y + tracks_rect.height()),
    ],
    Stroke::new(1.0_f32, BORDER));
}

pub fn draw_track_lines(painter: &egui::Painter, origin: Pos2, tracks_rect: Rect, vm: &mut EditorViewModel, ui: &mut egui::Ui) {
    let num_tracks = vm.total_tracks();
    
    let mut swap_action = None;
    let mut delete_action = None;
    let mut add_track_action = None;
    let mut select_all_track = None;
    let mut add_transition_action = None;

    for row in 0..num_tracks {
        let row_y = origin.y + RULER_H + (row as f32 * TRACK_H);
        
        let header_rect = Rect::from_min_size(
            Pos2::new(origin.x, row_y),
            Vec2::new(LABEL_W, TRACK_H),
        );
        let track_body_rect = Rect::from_min_size(
            Pos2::new(origin.x + LABEL_W, row_y),
            Vec2::new(tracks_rect.width() - LABEL_W, TRACK_H),
        );

        let is_selected = vm.selected_track == row;
        
        let resp = ui.interact(header_rect, ui.id().with(format!("track_hdr_{}", row)), egui::Sense::click_and_drag());
        let body_resp = ui.interact(track_body_rect, ui.id().with(format!("track_bdy_{}", row)), egui::Sense::click());
        
        if body_resp.clicked() {
            vm.selected_track = row;
            if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) {
                let click_x = pos.x - (origin.x + LABEL_W);
                let t = vm.px_to_time(click_x).clamp(0.0, vm.project.duration);
                vm.seek_to(t);
            }
        }

        body_resp.context_menu(|ui| {
            let click_pos = ui.ctx().input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()));
            let click_time = click_pos.map(|pos| {
                vm.px_to_time(pos.x - (origin.x + LABEL_W)).clamp(0.0, vm.project.duration)
            }).unwrap_or(vm.current_time());

            ui.label("✨ Transitions");
            ui.separator();
            for t in TransitionType::all() {
                if ui.button(format!("Add {}", t.label())).clicked() {
                    add_transition_action = Some((click_time, row, t.clone()));
                    ui.close_menu();
                }
            }
        });

        if is_selected {
            painter.rect_filled(header_rect, 0.0, ACCENT_CYAN.linear_multiply(0.2));
            painter.rect_filled(track_body_rect, 0.0, egui::Color32::from_white_alpha(5));
        } else if resp.hovered() || resp.dragged() {
            painter.rect_filled(header_rect, 0.0, egui::Color32::from_white_alpha(10));
        } else if row % 2 == 0 {
            painter.rect_filled(
                track_body_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 3),
            );
        }
        
        if resp.double_clicked() {
            select_all_track = Some(row);
        } else if resp.clicked() {
            vm.selected_track = row;
        }

        if resp.dragged() && row > 0 {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let target_row = ((pos.y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let target_row = target_row.clamp(1, vm.total_tracks() as isize - 1) as usize;
                
                let drop_y = origin.y + RULER_H + (target_row as f32 * TRACK_H);
                painter.line_segment(
                    [Pos2::new(origin.x, drop_y), Pos2::new(origin.x + tracks_rect.width(), drop_y)],
                    Stroke::new(3.0_f32, ACCENT_CYAN)
                );
            }
        }
        if resp.drag_stopped() && row > 0 {
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let target_row = ((pos.y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let target_row = target_row.clamp(1, vm.total_tracks() as isize - 1) as usize;
                if target_row != row {
                    swap_action = Some((row, target_row));
                }
            }
        }

        let name = vm.get_track_name(row);
        let (idx, is_audio) = vm.get_row_track(row);

        resp.context_menu(|ui| {
            ui.label(egui::RichText::new(&name).color(TEXT_BRIGHT).strong());
            ui.separator();

            if ui.button("Select All on Layer").clicked() {
                select_all_track = Some(row);
                ui.close_menu();
            }
            ui.separator();
            
            if ui.button(egui::RichText::new(if is_audio { "➕ Add Audio Track Above" } else { "➕ Add Video Track Above" })).clicked() {
                add_track_action = Some((row, true, is_audio));
                ui.close_menu();
            }
            if ui.button(egui::RichText::new(if is_audio { "➕ Add Audio Track Below" } else { "➕ Add Video Track Below" })).clicked() {
                add_track_action = Some((row, false, is_audio));
                ui.close_menu();
            }
            if idx == 0 {
                if ui.button(egui::RichText::new("➕ Add Video Track Below")).clicked() {
                    add_track_action = Some((row, false, false));
                    ui.close_menu();
                }
            }
            
            ui.separator();

            if idx == 0 {
                ui.label(egui::RichText::new("The Subtitle track cannot be deleted.").color(TEXT_DIM).size(10.0));
            } else {
                if ui.button(egui::RichText::new("🗑 Delete Track").color(ACCENT_AMBER)).clicked() {
                    delete_action = Some(row);
                    ui.close_menu();
                }
            }
        });

        painter.text(
            Pos2::new(origin.x + 12.0, row_y + TRACK_H / 2.0),
            egui::Align2::LEFT_CENTER,
            name,
            egui::FontId::proportional(10.5),
            if is_selected { ACCENT_CYAN } else { TEXT_DIM },
        );
        
        painter.line_segment([
            Pos2::new(origin.x, row_y + TRACK_H),
            Pos2::new(origin.x + tracks_rect.width(), row_y + TRACK_H),
        ],
        Stroke::new(1.0_f32, BORDER));
    }

    if let Some((time, row, t_type)) = add_transition_action {
        let (track_id, _) = vm.get_row_track(row);
        vm.add_transition_at(time, track_id, t_type);
    }
    if let Some(row) = select_all_track {
        vm.select_all_on_track(row);
    }
    if let Some((r, above, is_audio)) = add_track_action {
        vm.add_track_row(r, above, Some(is_audio));
    }
    if let Some((r1, r2)) = swap_action {
        if r2 > 0 && r2 < num_tracks {
            vm.move_track_row(r1, r2);
        }
    }
    if let Some(row) = delete_action {
        vm.delete_track(row);
    }
}