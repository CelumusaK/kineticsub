use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::models::types::MediaType;
use crate::viewmodels::editor_vm::{EditorViewModel, TranscribeMode};
use crate::views::theme::{ACCENT_CYAN, ACCENT_AMBER, TEXT_BRIGHT, TEXT_DIM};
use super::{LABEL_W, RULER_H, TRACK_H};

pub fn draw_media_blocks(
    ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: &mut bool,
    ctx: &Context,
) {
    let view_start = vm.timeline_scroll;
    let view_end   = view_start + (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

    let mut transcribe_id: Option<String> = None;
    let mut set_transcribe_mode: Option<TranscribeMode> = None;
    let mut delete_linked: Option<String> = None;
    let mut drag_info: Option<(usize, f64, bool, usize, isize)> = None;
    let mut block_delete: Option<String> = None;
    let mut selection_action: Option<(String, bool, usize, f64)> = None;
    let mut media_resize: Option<(String, u8, f64)> = None;
    let mut do_group = false;
    let mut do_ungroup = false;
    let mut clear_collisions = false;

    let track_clip = Rect::from_min_max(
        Pos2::new(origin.x + LABEL_W, tracks_rect.min.y),
        tracks_rect.max
    );
    let p = painter.with_clip_rect(track_clip);

    let whisper_busy = vm.whisper_is_running();
    let selected_ids = vm.selected_ids.clone();
    let transcribing_id = vm.transcribing_media_id.clone();
    
    let mut linked_sub_counts = std::collections::HashMap::new();
    for sub in &vm.project.subtitles {
        if let Some(mid) = &sub.media_id {
            *linked_sub_counts.entry(mid.clone()).or_insert(0) += 1;
        }
    }

    for (i, media) in vm.project.media_files.iter().enumerate() {
        if !media.on_timeline { continue; }

        if media.timeline_offset + media.duration <= view_start || media.timeline_offset >= view_end {
            continue;
        }
        
        let row_idx = vm.get_track_row(media.track_index, media.media_type == MediaType::Audio);
        let row_y = origin.y + RULER_H + (row_idx as f32 * TRACK_H);

        let block_x = origin.x + LABEL_W + vm.time_to_px(media.timeline_offset);
        let block_w = (media.duration * vm.timeline_zoom as f64) as f32;

        let rect = Rect::from_min_size(
            Pos2::new(block_x, row_y + 4.0),
            Vec2::new(block_w.max(4.0), TRACK_H - 8.0),
        );

        let interact_rect = rect.intersect(track_clip);
        if !interact_rect.is_positive() { continue; }

        let id = &media.id;
        let is_selected = selected_ids.contains(id.as_str());
        let is_transcribing = transcribing_id.as_deref() == Some(id.as_str());
        let linked_count = *linked_sub_counts.get(id).unwrap_or(&0);
        let has_transcription = linked_count > 0;
        let is_grouped = media.group_id.is_some();

        let drag_state_id = ui.id().with(("media_drag_mode", i));
        let mut drag_mode = ui.data(|d| d.get_temp::<u8>(drag_state_id).unwrap_or(0));

        let handle_width = 14.0f32.min(rect.width() * 0.4);

        let resp = ui.interact(
            interact_rect,
            ui.id().with(format!("media_{}", id)),
            egui::Sense::click_and_drag(),
        );
        
        if resp.drag_started() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let dist_left = (pos.x - rect.min.x).abs();
                let dist_right = (pos.x - rect.max.x).abs();
                
                if dist_left <= handle_width { drag_mode = 1; }
                else if dist_right <= handle_width { drag_mode = 2; }
                else { drag_mode = 0; }
                ui.data_mut(|d| d.insert_temp(drag_state_id, drag_mode));
            }
            if !is_selected || is_grouped {
                selection_action = Some((id.clone(), false, row_idx, media.timeline_offset));
            }
        }
        
        if resp.hovered() && !resp.dragged() {
            *hovered_block = true;
            if let Some(pos) = ctx.pointer_hover_pos() {
                let dist_left = (pos.x - rect.min.x).abs();
                let dist_right = (pos.x - rect.max.x).abs();
                if dist_left <= handle_width || dist_right <= handle_width {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                } else {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                }
            }
        }
        
        if resp.clicked() {
            let shift = ctx.input(|inp| inp.modifiers.shift);
            let click_pos = ctx.input(|inp| inp.pointer.interact_pos().or(inp.pointer.hover_pos()));
            let seek_time = click_pos.map(|pos| {
                vm.px_to_time(pos.x - (origin.x + LABEL_W)).clamp(0.0, vm.project.duration)
            }).unwrap_or(media.timeline_offset);
            selection_action = Some((id.clone(), shift, row_idx, seek_time));
        }

        if resp.dragged() {
            let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
            if drag_mode == 0 {
                let pointer_y = ctx.pointer_hover_pos().unwrap().y;
                let target_row = ((pointer_y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                let clamped_row = target_row.clamp(1, vm.total_tracks() as isize - 1) as usize;
                let row_delta = clamped_row as isize - row_idx as isize;
                let (target_track, _) = vm.get_row_track(clamped_row);

                let is_multi = is_selected && selected_ids.len() > 1;
                drag_info = Some((i, dt, is_multi, target_track, row_delta));
            } else {
                media_resize = Some((id.clone(), drag_mode, dt));
            }
        }
        
        if resp.drag_stopped() {
            ui.data_mut(|d| d.remove_temp::<u8>(drag_state_id));
            clear_collisions = true;
        }

        resp.context_menu(|ui| {
            ui.set_min_width(180.0);

            let mut display_name: String = media.name.chars().take(24).collect();
            if media.name.len() > display_name.len() { display_name.push('…'); }

            ui.label(egui::RichText::new(&display_name).color(TEXT_BRIGHT).size(11.0).strong());
            ui.separator();

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

            if media.media_type == MediaType::Audio {
                let transcribe_label = if is_transcribing { "⟳  Transcribing…" } else if has_transcription { "🎙  Re-transcribe" } else { "🎙  Transcribe" };

                ui.add_enabled_ui(!whisper_busy, |ui| {
                    ui.menu_button(egui::RichText::new(transcribe_label).color(ACCENT_CYAN).size(11.0), |ui| {
                        if ui.button("As Words").clicked() {
                            set_transcribe_mode = Some(TranscribeMode::Word);
                            transcribe_id = Some(id.clone());
                            ui.close_menu();
                        }
                        if ui.button("As Phrases").clicked() {
                            set_transcribe_mode = Some(TranscribeMode::Phrase);
                            transcribe_id = Some(id.clone());
                            ui.close_menu();
                        }
                    });
                });

                if has_transcription {
                    ui.separator();
                    ui.label(egui::RichText::new(format!("{} subtitle(s) linked", linked_count)).color(TEXT_DIM).size(10.0));

                    if ui.add(egui::Button::new(egui::RichText::new("✖  Remove Transcription").color(ACCENT_AMBER)).fill(egui::Color32::TRANSPARENT)).clicked() {
                        delete_linked = Some(id.clone());
                        ui.close_menu();
                    }
                }
            }

            ui.separator();
            if ui.button(egui::RichText::new("🗑 Delete from Timeline").color(ACCENT_AMBER)).clicked() {
                block_delete = Some(id.clone());
                ui.close_menu();
            }
        });

        let block_col = if is_transcribing {
            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 35) 
        } else if media.has_collision {
            egui::Color32::from_rgba_unmultiplied(239, 68, 68, 120) // Red collision indicator
        } else if media.media_type == MediaType::Video {
            egui::Color32::from_rgba_unmultiplied(200, 50, 100, 45)
        } else if media.media_type == MediaType::Image {
            egui::Color32::from_rgba_unmultiplied(50, 150, 200, 45)
        } else {
            egui::Color32::from_rgba_unmultiplied(245, 158, 11, 45)
        };
        
        p.rect_filled(rect, 3.0, block_col);

        let border_col = if is_transcribing { ACCENT_CYAN } 
            else if media.has_collision { egui::Color32::from_rgb(252, 165, 165) }
            else if is_selected { ACCENT_CYAN }
            else if media.media_type == MediaType::Video { egui::Color32::from_rgb(220, 80, 130) } 
            else if media.media_type == MediaType::Image { egui::Color32::from_rgb(80, 180, 220) } 
            else { ACCENT_AMBER };
            
        p.rect_stroke(rect, 3.0, Stroke::new(if is_transcribing || is_selected || media.has_collision { 2.0_f32 } else { 1.0_f32 }, border_col));

        if media.media_type == MediaType::Audio || media.media_type == MediaType::Video {
            let stripe_spacing = 6.0;
            let mut sx = rect.min.x + 4.0;
            while sx < rect.max.x - 2.0 {
                let h = (((sx * 0.3).sin() * 0.5 + 0.5) * (rect.height() * 0.6)) as f32;
                let cy = rect.center().y;
                p.line_segment([Pos2::new(sx, cy - h * 0.5), Pos2::new(sx, cy + h * 0.5)],
                    Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
                );
                sx += stripe_spacing;
            }
        }

        if is_grouped {
            let link_pos = Pos2::new(rect.min.x + 3.0, rect.min.y + 1.0);
            p.text(link_pos, egui::Align2::LEFT_TOP, "🔗", egui::FontId::proportional(8.0), ACCENT_CYAN);
        }

        if has_transcription {
            let dot_pos = Pos2::new(rect.max.x - 5.0, rect.min.y + 5.0);
            p.circle_filled(dot_pos, 3.5, ACCENT_CYAN);
        }

        let mut short_name: String = media.name.chars().take(14).collect();
        if media.name.len() > short_name.len() { short_name.push('…'); }
        let label_text = format!("{} {:.1}s", short_name, media.duration);

        let mut clip_rect = rect.intersect(track_clip);
        clip_rect.max.x -= 2.0;

        let text_offset_x = if is_grouped { 14.0 } else { 4.0 };
        p.with_clip_rect(clip_rect).text(
            Pos2::new(rect.min.x + text_offset_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label_text,
            egui::FontId::proportional(9.0),
            egui::Color32::WHITE,
        );

        if media.duration > 0.0 && block_w > 10.0 {
            for kf in &media.keyframes {
                let kf_t = kf.time_offset;
                if kf_t < 0.0 || kf_t > media.duration { continue; }
                let kf_frac = kf_t / media.duration;
                let kf_x    = rect.min.x + kf_frac as f32 * rect.width();
                
                if kf_x < track_clip.min.x || kf_x > track_clip.max.x { continue; }

                let kf_pos  = Pos2::new(kf_x, rect.max.y - 4.0);
                let d       = 3.5f32;
                
                let kf_rect = Rect::from_center_size(kf_pos, Vec2::splat(d * 2.5));
                let kf_resp = ui.interact(kf_rect, ui.id().with(("kf_med", &media.id, kf_t.to_bits())), egui::Sense::click());
                if kf_resp.clicked() {
                    let click_pos = ctx.input(|inp| inp.pointer.interact_pos().or(inp.pointer.hover_pos()));
                    let seek_time = click_pos.map(|pos| {
                        vm.px_to_time(pos.x - (origin.x + LABEL_W)).clamp(0.0, vm.project.duration)
                    }).unwrap_or(media.timeline_offset + kf_t);
                    selection_action = Some((media.id.clone(), false, row_idx, seek_time));
                }

                let kf_color = if kf_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_rgb(34, 211, 238) };

                p.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(kf_pos.x,     kf_pos.y - d),
                        Pos2::new(kf_pos.x + d, kf_pos.y    ),
                        Pos2::new(kf_pos.x,     kf_pos.y + d),
                        Pos2::new(kf_pos.x - d, kf_pos.y    ),
                    ],
                    kf_color,
                    Stroke::NONE,
                ));
            }
        }
    }
    
    if clear_collisions {
        for m in &mut vm.project.media_files { m.has_collision = false; }
        for s in &mut vm.project.subtitles { s.has_collision = false; }
    }

    if do_group { vm.group_selected(); }
    if do_ungroup { vm.ungroup_selected(); }

    if let Some((id, mode, dt)) = media_resize {
        vm.resize_media_clip(&id, mode, dt);
    }

    if let Some((id, shift, row_idx, target_time)) = selection_action {
        if shift {
            vm.toggle_select(&id);
        } else {
            vm.select_with_group(&id);
            vm.selected_track = row_idx;
            vm.seek_to(target_time);
        }
    }

    if let Some(mode) = set_transcribe_mode { vm.transcribe_mode = mode; }
    if let Some(id) = transcribe_id { vm.start_auto_transcription(id); }
    if let Some(id) = delete_linked { vm.project.subtitles.retain(|s| s.media_id.as_deref() != Some(id.as_str())); vm.update_duration(); }
    if let Some(id) = block_delete { 
        if let Some(m) = vm.project.media_files.iter_mut().find(|m| m.id == id) {
            m.on_timeline = false;
        }
        vm.update_duration(); 
    }
    
    if let Some((i, dt, multi, track, row_delta)) = drag_info { 
        if multi {
            vm.move_selected_items(dt);
            if row_delta != 0 {
                if let Some(m) = vm.project.media_files.get(i) {
                    let dragged_id = m.id.clone();
                    vm.shift_selected_tracks(&dragged_id, row_delta);
                }
            }
        } else {
            vm.move_media(i, dt, track); 
        }
    }
}