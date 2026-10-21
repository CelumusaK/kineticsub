use super::{LABEL_W, RULER_H, TRACK_H};
use crate::models::types::{TrackType, TransitionType};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_AMBER, ACCENT_CYAN, BG_PANEL, BORDER, TEXT_BRIGHT, TEXT_DIM};
use egui::{Pos2, Rect, Stroke, Vec2};

pub fn draw_track_labels(
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    _vm: &EditorViewModel,
    _ui: &mut egui::Ui,
) {
    let rect = Rect::from_min_size(origin, Vec2::new(LABEL_W, tracks_rect.height()));
    painter.rect_filled(rect, 0.0, BG_PANEL);
    painter.line_segment(
        [
            Pos2::new(rect.max.x, rect.min.y),
            Pos2::new(rect.max.x, rect.max.y),
        ],
        Stroke::new(1.0_f32, BORDER),
    );
}

pub fn draw_track_lines(
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    vm: &mut EditorViewModel,
    ui: &mut egui::Ui,
) {
    let num_tracks = vm.total_tracks();

    // The painter is clipped to the visible track area (below the ruler). Use the
    // same rect to clip hit-testing so scrolled-away rows never steal input from
    // the ruler / toolbar.
    let visible = painter.clip_rect();

    let mut swap_action = None;
    let mut delete_action = None;
    let mut add_track_action = None;
    let mut select_all_track = None;
    let mut add_transition_action = None;

    for row in 0..num_tracks {
        let row_y = origin.y + RULER_H + (row as f32 * TRACK_H);

        // Skip rows that are scrolled completely out of view.
        if row_y + TRACK_H < visible.min.y || row_y > visible.max.y {
            continue;
        }

        let header_rect =
            Rect::from_min_size(Pos2::new(origin.x, row_y), Vec2::new(LABEL_W, TRACK_H));
        let track_body_rect = Rect::from_min_size(
            Pos2::new(origin.x + LABEL_W, row_y),
            Vec2::new(tracks_rect.width() - LABEL_W, TRACK_H),
        );

        let is_selected = vm.selected_track == row;

        let header_hit = header_rect.intersect(visible);
        let body_hit = track_body_rect.intersect(visible);
        if !header_hit.is_positive() || !body_hit.is_positive() {
            continue;
        }

        let resp = ui.interact(
            header_hit,
            ui.id().with(format!("track_hdr_{}", row)),
            egui::Sense::click_and_drag(),
        );
        let body_resp = ui.interact(
            body_hit,
            ui.id().with(format!("track_bdy_{}", row)),
            egui::Sense::click(),
        );

        let (_, track_type) = vm.get_row_track(row);

        if body_resp.clicked() {
            vm.selected_track = row;
            if let Some(pos) = ui
                .ctx()
                .input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()))
            {
                let click_x = pos.x - (origin.x + LABEL_W);
                let t = vm.px_to_time(click_x).clamp(0.0, vm.project.duration);
                vm.seek_to(t);
            }
        }

        body_resp.context_menu(|ui| {
            let click_pos = ui
                .ctx()
                .input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()));
            let click_time = click_pos
                .map(|pos| {
                    vm.px_to_time(pos.x - (origin.x + LABEL_W))
                        .clamp(0.0, vm.project.duration)
                })
                .unwrap_or(vm.current_time());

            if track_type != TrackType::Subtitle {
                ui.label("✨ Transitions");
                ui.separator();
                for t in TransitionType::all() {
                    if ui.button(format!("Add {}", t.label())).clicked() {
                        add_transition_action = Some((click_time, row, t.clone()));
                        ui.close_menu();
                    }
                }
            } else {
                ui.label("Transitions cannot be added to subtitle tracks.");
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

        let is_first_of_kind = match track_type {
            TrackType::Subtitle => row == 0,
            TrackType::Video => row == vm.project.subtitle_track_order.len(),
            TrackType::Audio => {
                row == vm.project.subtitle_track_order.len() + vm.project.video_track_order.len()
            }
        };

        if resp.dragged() && !is_first_of_kind {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let target_row = ((pos.y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let target_row = target_row.clamp(1, vm.total_tracks() as isize - 1) as usize;

                let (_, target_type) = vm.get_row_track(target_row);
                if target_type == track_type {
                    let drop_y = origin.y + RULER_H + (target_row as f32 * TRACK_H);
                    painter.line_segment(
                        [
                            Pos2::new(origin.x, drop_y),
                            Pos2::new(origin.x + tracks_rect.width(), drop_y),
                        ],
                        Stroke::new(3.0_f32, ACCENT_CYAN),
                    );
                }
            }
        }
        if resp.drag_stopped() && !is_first_of_kind {
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let target_row = ((pos.y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let target_row = target_row.clamp(1, vm.total_tracks() as isize - 1) as usize;
                if target_row != row {
                    swap_action = Some((row, target_row));
                }
            }
        }

        let name = vm.get_track_name(row);

        resp.context_menu(|ui| {
            ui.label(egui::RichText::new(&name).color(TEXT_BRIGHT).strong());
            ui.separator();

            if ui.button("Select All on Layer").clicked() {
                select_all_track = Some(row);
                ui.close_menu();
            }
            ui.separator();

            let type_str = match track_type {
                TrackType::Subtitle => "Text Track",
                TrackType::Video => "Video Track",
                TrackType::Audio => "Audio Track",
            };

            if ui
                .button(egui::RichText::new(format!("➕ Add {} Above", type_str)))
                .clicked()
            {
                add_track_action = Some((row, true, track_type));
                ui.close_menu();
            }
            if ui
                .button(egui::RichText::new(format!("➕ Add {} Below", type_str)))
                .clicked()
            {
                add_track_action = Some((row, false, track_type));
                ui.close_menu();
            }

            ui.separator();

            if is_first_of_kind {
                ui.label(
                    egui::RichText::new(format!("Primary {} cannot be deleted.", type_str))
                        .color(TEXT_DIM)
                        .size(10.0),
                );
            } else {
                if ui
                    .button(egui::RichText::new("🗑 Delete Track").color(ACCENT_AMBER))
                    .clicked()
                {
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

        painter.line_segment(
            [
                Pos2::new(origin.x, row_y + TRACK_H),
                Pos2::new(origin.x + tracks_rect.width(), row_y + TRACK_H),
            ],
            Stroke::new(1.0_f32, BORDER),
        );
    }

    if let Some((time, row, t_type)) = add_transition_action {
        let (track_id, _) = vm.get_row_track(row);
        vm.add_transition_at(time, track_id, t_type);
    }
    if let Some(row) = select_all_track {
        vm.select_all_on_track(row);
    }
    if let Some((r, above, track_type)) = add_track_action {
        vm.add_track_row(r, above, track_type);
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
