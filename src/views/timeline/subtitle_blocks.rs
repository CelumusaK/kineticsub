use super::{LABEL_W, RULER_H, TRACK_H};
use crate::models::types::TrackType;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_AMBER, ACCENT_CYAN, TEXT_BRIGHT};
use egui::{Context, Pos2, Rect, Stroke, Vec2};

pub fn draw_subtitle_blocks(
    ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: &mut bool,
    needs_sort: &mut bool,
    ctx: &Context,
) {
    let view_start = vm.timeline_scroll;
    let view_end = view_start + (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

    let mut drag_delta: Option<(usize, bool, f64, usize, isize)> = None;
    let mut click_info: Option<(String, f64, bool, usize)> = None;
    let mut delete_block: Option<String> = None;
    let mut sub_resize: Option<(String, u8, f64)> = None;
    let mut drag_ended = false;
    let mut do_group = false;
    let mut do_ungroup = false;
    let mut clear_collisions = false;
    let mut kf_select_action: Option<(String, String)> = None;

    let selected_ids = vm.selected_ids.clone();
    let primary_id = vm.selected_id.clone();

    let track_clip = Rect::from_min_max(
        Pos2::new(origin.x + LABEL_W, tracks_rect.min.y),
        tracks_rect.max,
    );
    let p = painter.with_clip_rect(track_clip);

    for (i, sub) in vm.project.subtitles.iter().enumerate() {
        if sub.timeline_end <= view_start {
            continue;
        }
        if sub.timeline_start >= view_end {
            continue;
        }

        let row_idx = vm.get_track_row(sub.track_index, TrackType::Subtitle);
        let sub_row_y = origin.y + RULER_H + (row_idx as f32 * TRACK_H);

        let is_primary = primary_id.as_deref() == Some(sub.id.as_str());
        let is_selected = selected_ids.contains(&sub.id);
        let has_media_link = sub.media_id.is_some();
        let is_grouped = sub.group_id.is_some();

        let block_x = origin.x + LABEL_W + vm.time_to_px(sub.timeline_start);
        let block_w = ((sub.timeline_end - sub.timeline_start) * vm.timeline_zoom as f64) as f32;

        let block_rect = Rect::from_min_size(
            Pos2::new(block_x, sub_row_y + 4.0),
            Vec2::new(block_w.max(6.0), TRACK_H - 8.0),
        );

        let interact_rect = block_rect.intersect(track_clip);
        if !interact_rect.is_positive() {
            continue;
        }

        let drag_state_id = ui.id().with(("sub_drag_mode", i));
        let mut drag_mode = ui.data(|d| d.get_temp::<u8>(drag_state_id).unwrap_or(0));
        let handle_w = 14.0_f32.min(block_rect.width() * 0.4);

        let resp = ui.interact(
            interact_rect,
            ui.id().with(("sub", i)),
            egui::Sense::click_and_drag(),
        );

        if resp.drag_started() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let dist_left = (pos.x - block_rect.min.x).abs();
                let dist_right = (pos.x - block_rect.max.x).abs();

                if dist_left <= handle_w {
                    drag_mode = 1;
                } else if dist_right <= handle_w {
                    drag_mode = 2;
                } else {
                    drag_mode = 0;
                }
                ui.data_mut(|d| d.insert_temp(drag_state_id, drag_mode));
            }
        }

        if resp.hovered() && !resp.dragged() {
            *hovered_block = true;
            if let Some(pos) = ctx.pointer_hover_pos() {
                let dist_left = (pos.x - block_rect.min.x).abs();
                let dist_right = (pos.x - block_rect.max.x).abs();
                if dist_left <= handle_w || dist_right <= handle_w {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                } else {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                }
            }
        }

        if resp.clicked() {
            let shift = ctx.input(|inp| inp.modifiers.shift);
            let click_pos = ctx.input(|inp| inp.pointer.interact_pos().or(inp.pointer.hover_pos()));
            let seek_time = click_pos
                .map(|pos| {
                    vm.px_to_time(pos.x - (origin.x + LABEL_W))
                        .clamp(0.0, vm.project.duration)
                })
                .unwrap_or(sub.timeline_start);
            click_info = Some((sub.id.clone(), seek_time, shift, row_idx));
        }

        if resp.dragged() {
            let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
            if drag_mode == 0 {
                let pointer_y = ctx.pointer_hover_pos().map(|p| p.y).unwrap_or(sub_row_y);
                let target_row = ((pointer_y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let max_sub_row = vm.project.subtitle_track_order.len().saturating_sub(1) as isize;
                let clamped_row = target_row.clamp(0, max_sub_row) as usize;
                let row_delta = clamped_row as isize - row_idx as isize;

                drag_delta = Some((
                    i,
                    is_selected && selected_ids.len() > 1,
                    dt,
                    clamped_row,
                    row_delta,
                ));
            } else {
                sub_resize = Some((sub.id.clone(), drag_mode, dt));
            }
        }

        if resp.drag_stopped() {
            ui.data_mut(|d| d.remove_temp::<u8>(drag_state_id));
            drag_ended = true;
            clear_collisions = true;
        }

        resp.context_menu(|ui| {
            if selected_ids.len() > 1 {
                if ui.button("🔗 Group Selected").clicked() {
                    do_group = true;
                    ui.close_menu();
                }
            }
            if is_grouped {
                if ui.button("🔓 Ungroup").clicked() {
                    do_ungroup = true;
                    ui.close_menu();
                }
            }
            ui.separator();
            if ui
                .button(egui::RichText::new("🗑 Delete").color(ACCENT_AMBER))
                .clicked()
            {
                delete_block = Some(sub.id.clone());
                ui.close_menu();
            }
        });

        let is_active_playback =
            vm.current_time() >= sub.timeline_start && vm.current_time() < sub.timeline_end;

        // Distinct, high-visibility highlight colors for text clips
        let (fill, stroke_col, stroke_width) = if sub.has_collision {
            (
                egui::Color32::from_rgba_unmultiplied(239, 68, 68, 140),
                egui::Color32::from_rgb(252, 165, 165),
                2.0_f32,
            )
        } else if is_primary {
            (
                // Vibrant gold/amber for primary selected clip
                egui::Color32::from_rgba_unmultiplied(245, 158, 11, 110),
                egui::Color32::from_rgb(253, 224, 71),
                2.0_f32,
            )
        } else if is_selected {
            (
                // Multi-select highlight
                egui::Color32::from_rgba_unmultiplied(245, 158, 11, 75),
                egui::Color32::from_rgb(251, 191, 36),
                1.5_f32,
            )
        } else if is_active_playback {
            (
                egui::Color32::from_rgba_unmultiplied(34, 211, 238, 80),
                egui::Color32::from_rgb(103, 232, 249),
                1.5_f32,
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(14, 116, 144, 85),
                egui::Color32::from_rgb(56, 189, 248),
                1.0_f32,
            )
        };

        p.rect_filled(block_rect, 4.0, fill);
        p.rect_stroke(block_rect, 4.0, Stroke::new(stroke_width, stroke_col));

        if resp.hovered() && block_w > 20.0 {
            let left_grip = Rect::from_min_max(
                block_rect.min,
                Pos2::new(block_rect.min.x + 3.0, block_rect.max.y),
            );
            let right_grip = Rect::from_min_max(
                Pos2::new(block_rect.max.x - 3.0, block_rect.min.y),
                block_rect.max,
            );
            p.rect_filled(left_grip, 1.0, stroke_col);
            p.rect_filled(right_grip, 1.0, stroke_col);
        }

        if has_media_link {
            p.text(
                Pos2::new(block_rect.min.x + 3.0, block_rect.min.y + 2.0),
                egui::Align2::LEFT_TOP,
                "⛓",
                egui::FontId::proportional(7.5),
                egui::Color32::from_rgba_unmultiplied(245, 158, 11, 180),
            );
        } else if is_grouped {
            p.text(
                Pos2::new(block_rect.min.x + 3.0, block_rect.min.y + 2.0),
                egui::Align2::LEFT_TOP,
                "🔗",
                egui::FontId::proportional(7.5),
                ACCENT_CYAN,
            );
        }

        if block_w > 15.0 {
            let text_color = if is_primary || is_selected {
                egui::Color32::WHITE
            } else if is_active_playback {
                ACCENT_CYAN
            } else {
                TEXT_BRIGHT
            };

            let mut clip_rect = block_rect.intersect(track_clip);
            clip_rect.max.x -= 2.0;
            clip_rect.min.x += 2.0;

            p.with_clip_rect(clip_rect).text(
                block_rect.center_top() + Vec2::new(0.0, 5.0),
                egui::Align2::CENTER_TOP,
                &sub.text,
                egui::FontId::proportional(9.5),
                text_color,
            );
        }

        let sub_dur = sub.timeline_end - sub.timeline_start;
        if sub_dur > 0.0 && block_w > 10.0 {
            for kf in &sub.keyframes {
                let kf_t = kf.time_offset;
                if kf_t < 0.0 || kf_t > sub_dur {
                    continue;
                }
                let kf_frac = kf_t / sub_dur;
                let kf_x = block_rect.min.x + kf_frac as f32 * block_rect.width();

                if kf_x < track_clip.min.x || kf_x > track_clip.max.x {
                    continue;
                }

                let kf_pos = Pos2::new(kf_x, block_rect.max.y - 4.0);
                let d = 3.5f32;

                let kf_rect = Rect::from_center_size(kf_pos, Vec2::splat(d * 2.5));
                let kf_resp = ui.interact(
                    kf_rect,
                    ui.id().with(("kf_sub", &sub.id, kf_t.to_bits())),
                    egui::Sense::click(),
                );
                if kf_resp.clicked() {
                    kf_select_action = Some((sub.id.clone(), kf.id.clone()));
                }

                let kf_color = if kf_resp.hovered() {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(34, 211, 238)
                };

                p.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(kf_pos.x, kf_pos.y - d),
                        Pos2::new(kf_pos.x + d, kf_pos.y),
                        Pos2::new(kf_pos.x, kf_pos.y + d),
                        Pos2::new(kf_pos.x - d, kf_pos.y),
                    ],
                    kf_color,
                    Stroke::NONE,
                ));
            }
        }
    }

    if clear_collisions {
        for m in &mut vm.project.media_files {
            m.has_collision = false;
        }
        for s in &mut vm.project.subtitles {
            s.has_collision = false;
        }
    }

    if do_group {
        vm.group_selected();
    }
    if do_ungroup {
        vm.ungroup_selected();
    }

    if let Some((sub_id, kf_id)) = kf_select_action {
        vm.select_with_group(&sub_id);
        vm.selected_keyframe_ids.clear();
        vm.selected_keyframe_ids.insert(kf_id);
    }

    if let Some((id, mode, dt)) = sub_resize {
        vm.resize_subtitle_clip(&id, mode, dt);
    }

    if let Some((id, start, shift, row_idx)) = click_info {
        if shift {
            vm.toggle_select(&id);
        } else {
            vm.select_with_group(&id);
            vm.selected_track = row_idx;
            vm.seek_to(start);
        }
    }

    if let Some(id) = delete_block {
        vm.delete_subtitle(&id);
    }

    if let Some((idx, is_multi, dt, target_row, row_delta)) = drag_delta {
        if is_multi {
            vm.move_selected_items(dt);
            if row_delta != 0 {
                if let Some(s) = vm.project.subtitles.get(idx) {
                    let dragged_id = s.id.clone();
                    vm.shift_selected_tracks(&dragged_id, row_delta);
                }
            }
        } else {
            vm.move_subtitle(idx, dt, target_row);
        }
    }

    if drag_ended {
        *needs_sort = true;
    }
}
