use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, ACCENT_AMBER, TEXT_BRIGHT, TEXT_DIM};
use super::{LABEL_W, RULER_H, TRACK_H};

pub fn draw_audio_blocks(
    ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: &mut bool,
    _ctx: &Context,
) {
    let view_start = vm.timeline_scroll;
    let view_end   = view_start + (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

    let mut transcribe_id: Option<String> = None;
    let mut delete_linked: Option<String> = None;
    let mut drag_info: Option<(usize, f64)> = None;

    let track_clip = Rect::from_min_max(
        Pos2::new(origin.x + LABEL_W, tracks_rect.min.y),
        tracks_rect.max
    );
    let p = painter.with_clip_rect(track_clip);

    for (i, media) in vm.project.media_files.iter().enumerate() {
        if !media.on_timeline { continue; }

        if media.timeline_offset + media.duration <= view_start || media.timeline_offset >= view_end {
            continue;
        }
        
        // Push Video items to Track 1, Audio items to Track 2
        let row_y = if media.is_video_track {
            origin.y + RULER_H + 1.0 * TRACK_H
        } else {
            origin.y + RULER_H + 2.0 * TRACK_H
        };

        let block_x = origin.x + LABEL_W + vm.time_to_px(media.timeline_offset);
        let block_w = (media.duration * vm.timeline_zoom as f64) as f32;

        let rect = Rect::from_min_size(
            Pos2::new(block_x, row_y + 4.0),
            Vec2::new(block_w.max(4.0), TRACK_H - 8.0),
        );

        let interact_rect = rect.intersect(track_clip);
        if !interact_rect.is_positive() { continue; }

        let id = &media.id;
        let is_transcribing = vm.transcribing_media_id.as_deref() == Some(id.as_str());
        let has_transcription = vm.project.subtitles.iter()
            .any(|s| s.media_id.as_deref() == Some(id.as_str()));

        let resp = ui.interact(
            interact_rect,
            ui.id().with(format!("media_{}", id)),
            egui::Sense::click_and_drag(),
        );

        if resp.hovered() { *hovered_block = true; }

        if resp.dragged() {
            let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
            drag_info = Some((i, dt));
        }

        resp.context_menu(|ui| {
            ui.set_min_width(180.0);

            let mut display_name: String = media.name.chars().take(24).collect();
            if media.name.len() > display_name.len() { display_name.push('…'); }

            ui.label(egui::RichText::new(&display_name).color(TEXT_BRIGHT).size(11.0).strong());
            ui.separator();

            if !media.is_video_track {
                let whisper_busy  = vm.whisper_is_running();
                let transcribe_label = if is_transcribing { "⟳  Transcribing…" } else if has_transcription { "🎙  Re-transcribe" } else { "🎙  Transcribe" };

                if ui.add_enabled(!whisper_busy, egui::Button::new(egui::RichText::new(transcribe_label).color(ACCENT_CYAN).size(11.0))).clicked() {
                    transcribe_id = Some(id.clone());
                    ui.close_menu();
                }

                if has_transcription {
                    ui.separator();
                    let count = vm.project.subtitles.iter().filter(|s| s.media_id.as_deref() == Some(id.as_str())).count();
                    ui.label(egui::RichText::new(format!("{} subtitle(s) linked", count)).color(TEXT_DIM).size(10.0));

                    if ui.add(egui::Button::new(egui::RichText::new("✖  Remove Transcription").color(ACCENT_AMBER).size(11.0)).fill(egui::Color32::TRANSPARENT)).clicked() {
                        delete_linked = Some(id.clone());
                        ui.close_menu();
                    }
                }
            } else {
                ui.label(egui::RichText::new("Edit background color in the Media Bin!").color(TEXT_DIM).size(10.0));
            }
        });

        // Visually distinguish video tracks from audio tracks
        let block_col = if media.is_video_track {
            egui::Color32::from_rgba_unmultiplied(200, 50, 100, 35)
        } else if is_transcribing {
            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 25) 
        } else {
            egui::Color32::from_rgba_unmultiplied(245, 158, 11, 35)
        };
        
        p.rect_filled(rect, 3.0, block_col);

        let border_col = if is_transcribing { ACCENT_CYAN } else if media.is_video_track { egui::Color32::from_rgb(220, 80, 130) } else { ACCENT_AMBER };
        p.rect_stroke(rect, 3.0, Stroke::new(if is_transcribing { 1.5 } else { 1.0 }, border_col));

        // Draw audio waveform stripes (only for audio files)
        if !media.is_video_track {
            let stripe_spacing = 6.0;
            let mut sx = rect.min.x + 4.0;
            while sx < rect.max.x - 2.0 {
                let h = (((sx * 0.3).sin() * 0.5 + 0.5) * (rect.height() * 0.6)) as f32;
                let cy = rect.center().y;
                p.line_segment([Pos2::new(sx, cy - h * 0.5), Pos2::new(sx, cy + h * 0.5)],
                    Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(245, 158, 11, 80)),
                );
                sx += stripe_spacing;
            }
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

        let font_col = if media.is_video_track { egui::Color32::from_rgba_unmultiplied(220, 80, 130, 200) } else { egui::Color32::from_rgba_unmultiplied(245, 158, 11, 200) };

        p.with_clip_rect(clip_rect).text(
            Pos2::new(rect.min.x + 4.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label_text,
            egui::FontId::proportional(8.5),
            font_col,
        );

        if !media.is_video_track && !has_transcription && !vm.whisper_is_running() && block_w > 80.0 {
            p.with_clip_rect(clip_rect).text(
                Pos2::new(rect.center().x, rect.max.y - 4.0),
                egui::Align2::CENTER_BOTTOM,
                "Right-click → Transcribe",
                egui::FontId::proportional(7.5),
                egui::Color32::from_rgba_unmultiplied(245, 158, 11, 100),
            );
        }
    }

    if let Some(id) = transcribe_id { vm.start_auto_transcription(id); }
    if let Some(id) = delete_linked { vm.project.subtitles.retain(|s| s.media_id.as_deref() != Some(id.as_str())); vm.update_duration(); }
    if let Some((i, dt)) = drag_info { vm.move_media(i, dt); }
}