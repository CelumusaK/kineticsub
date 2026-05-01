// File: src/views/timeline/subtitle_blocks.rs
use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, ACCENT_CYAN_DIM, TEXT_NORM};
use super::{LABEL_W, RULER_H, TRACK_H};

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
    let sub_row_y = origin.y + RULER_H;

    let view_start = vm.timeline_scroll;
    let view_end   = view_start + (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

    let mut drag_delta:  Option<(usize, bool, f64)> = None;
    let mut click_info:  Option<(String, f64, bool)> = None;
    let mut drag_ended = false;

    let selected_ids = &vm.selected_ids;
    let primary_id = vm.selected_id.as_deref();

    // Create a clipped painter so items don't render over the track labels
    let track_clip = Rect::from_min_max(
        Pos2::new(origin.x + LABEL_W, tracks_rect.min.y),
        tracks_rect.max
    );
    let p = painter.with_clip_rect(track_clip);

    for (i, sub) in vm.project.subtitles.iter().enumerate() {
        if sub.timeline_end <= view_start {
            continue;
        }
        if sub.timeline_start >= view_end {
            break; 
        }

        let is_primary  = primary_id == Some(sub.id.as_str());
        let is_selected = selected_ids.contains(&sub.id);
        let has_media_link = sub.media_id.is_some();

        let block_x = origin.x + LABEL_W + vm.time_to_px(sub.timeline_start);
        let block_w = ((sub.timeline_end - sub.timeline_start) * vm.timeline_zoom as f64) as f32;

        let block_rect = Rect::from_min_size(
            Pos2::new(block_x, sub_row_y + 4.0),
            Vec2::new(block_w.max(4.0), TRACK_H - 8.0),
        );

        // Intersect with track bounds to prevent dragging elements hidden behind the label panel
        let interact_rect = block_rect.intersect(track_clip);
        if !interact_rect.is_positive() { continue; }

        let resp = ui.interact(
            interact_rect,
            ui.id().with(("sub", i)),
            egui::Sense::click_and_drag(),
        );

        if resp.hovered() { *hovered_block = true; }

        if resp.clicked() {
            let shift = ctx.input(|inp| inp.modifiers.shift);
            
            if let Some(pos) = resp.interact_pointer_pos() {
                // Drop playhead exactly where the mouse clicked...
                let mut target_time = vm.px_to_time(pos.x - origin.x - LABEL_W);
                
                // ...UNLESS we clicked directly on a keyframe diamond
                let sub_dur = sub.timeline_end - sub.timeline_start;
                if sub_dur > 0.0 && block_w > 10.0 {
                    for kf in &sub.keyframes {
                        let kf_t = kf.time_offset;
                        if kf_t < 0.0 || kf_t > sub_dur { continue; }
                        let kf_x = block_rect.min.x + (kf_t / sub_dur) as f32 * block_rect.width();
                        let kf_pos = Pos2::new(kf_x, block_rect.max.y - 4.0);
                        
                        // Hitbox check (the diamond visual is roughly 3.5px wide)
                        if pos.distance(kf_pos) <= 6.0 {
                            target_time = sub.timeline_start + kf_t;
                            break;
                        }
                    }
                }
                
                click_info = Some((sub.id.clone(), target_time, shift));
            } else {
                click_info = Some((sub.id.clone(), sub.timeline_start, shift));
            }
        }

        if resp.dragged() && !has_media_link {
            let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
            drag_delta = Some((i, is_selected && selected_ids.len() > 1, dt));
        }
        if resp.drag_stopped() { drag_ended = true; }

        // ── Paint ─────────────────────────────────────────────────────────────
        let fill = if is_primary {
            egui::Color32::from_rgba_premultiplied(34, 211, 238, 65)
        } else if is_selected {
            egui::Color32::from_rgba_premultiplied(34, 211, 238, 40)
        } else {
            egui::Color32::from_rgba_premultiplied(34, 211, 238, 22)
        };
        let stroke_col = if is_primary { ACCENT_CYAN } else { ACCENT_CYAN_DIM };

        p.rect_filled(block_rect, 3.0, fill);
        p.rect_stroke(
            block_rect,
            3.0,
            Stroke::new(if is_primary { 1.5 } else { 1.0 }, stroke_col),
        );

        if has_media_link {
            p.text(
                Pos2::new(block_rect.min.x + 2.0, block_rect.min.y + 1.0),
                egui::Align2::LEFT_TOP,
                "⛓",
                egui::FontId::proportional(7.0),
                egui::Color32::from_rgba_unmultiplied(245, 158, 11, 160),
            );
        }

        if block_w > 15.0 {
            let text_color = if is_primary { ACCENT_CYAN } else { TEXT_NORM };
            let mut clip_rect = block_rect.intersect(track_clip);
            clip_rect.max.x -= 2.0; 
            clip_rect.min.x += 2.0;

            p.with_clip_rect(clip_rect).text(
                block_rect.center_top() + Vec2::new(0.0, 5.0),
                egui::Align2::CENTER_TOP,
                &sub.text,
                egui::FontId::proportional(9.0),
                text_color,
            );
        }

        let sub_dur = sub.timeline_end - sub.timeline_start;
        if sub_dur > 0.0 && block_w > 10.0 {
            for kf in &sub.keyframes {
                let kf_t = kf.time_offset;
                if kf_t < 0.0 || kf_t > sub_dur { continue; }
                let kf_frac = kf_t / sub_dur;
                let kf_x    = block_rect.min.x + kf_frac as f32 * block_rect.width();
                
                if kf_x < track_clip.min.x || kf_x > track_clip.max.x { continue; }

                let kf_pos  = Pos2::new(kf_x, block_rect.max.y - 4.0);
                let d       = 3.5f32;
                p.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(kf_pos.x,     kf_pos.y - d),
                        Pos2::new(kf_pos.x + d, kf_pos.y    ),
                        Pos2::new(kf_pos.x,     kf_pos.y + d),
                        Pos2::new(kf_pos.x - d, kf_pos.y    ),
                    ],
                    egui::Color32::from_rgb(34, 211, 238),
                    Stroke::NONE,
                ));
            }
        }
    }

    // ── Apply mutations after the immutable loop ─────────────────────
    if let Some((id, start, shift)) = click_info {
        if shift {
            vm.toggle_select(&id);
        } else {
            vm.select_subtitle(Some(id));
            vm.seek_to(start); // Jumps EXACTLY to mouse or keyframe hit-box
        }
    }

    if let Some((idx, is_multi, dt)) = drag_delta {
        if is_multi {
            let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
            for sub in vm.project.subtitles.iter_mut() {
                if ids.contains(&sub.id) && sub.media_id.is_none() {
                    sub.timeline_start = (sub.timeline_start + dt).max(0.0);
                    sub.timeline_end   = (sub.timeline_end   + dt).max(sub.timeline_start + 0.05);
                }
            }
            vm.update_duration();
        } else {
            vm.move_subtitle_idx(idx, dt);
        }
    }

    if drag_ended { *needs_sort = true; }
}