use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, ACCENT_AMBER, TEXT_BRIGHT, TEXT_DIM};
use super::{LABEL_W, RULER_H, TRACK_H};

// ── Audio blocks ───────────────────────────────────────────────────────────────

pub fn draw_audio_blocks(
ui: &mut egui::Ui,
vm: &mut EditorViewModel,
painter: &egui::Painter,
origin: Pos2,
tracks_rect: Rect,
hovered_block: &mut bool,
ctx: &Context,
) {
let audio_row_y = origin.y + RULER_H + 2.0 * TRACK_H;

let media_snapshot: Vec<(usize, bool, f64, f64, String, String, bool)> =
    vm.project.media_files.iter().enumerate()
        .map(|(i, m)| (i, m.on_timeline, m.timeline_offset, m.duration, m.id.clone(), m.name.clone(),
                       vm.transcribing_media_id.as_deref() == Some(&m.id)))
        .collect();

let mut transcribe_id: Option<String> = None;

for (i, on_tl, offset, duration, id, name, is_transcribing) in media_snapshot {
    if !on_tl { continue; }

    let block_x = origin.x + LABEL_W + vm.time_to_px(offset);
    let block_w = (vm.time_to_px(offset + duration) - vm.time_to_px(offset)).max(4.0);

    if block_x + block_w < origin.x + LABEL_W
        || block_x > origin.x + tracks_rect.width()
    { continue; }

    let rect = Rect::from_min_size(
        Pos2::new(block_x.max(origin.x + LABEL_W), audio_row_y + 4.0),
        Vec2::new(block_w, TRACK_H - 8.0),
    );

    let has_transcription = vm.project.subtitles.iter()
        .any(|s| s.media_id.as_deref() == Some(&id));

    let resp = ui.interact(
        rect,
        ui.id().with(format!("audio_{}", id)),
        egui::Sense::click_and_drag(),
    );

    if resp.hovered() { *hovered_block = true; }

    if resp.dragged() {
        let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
        vm.move_media(i, dt);
    }

    resp.context_menu(|ui| {
        ui.set_min_width(180.0);

        let display_name = if name.len() > 24 { format!("{}…", &name[..24]) } else { name.clone() };
        ui.label(egui::RichText::new(&display_name).color(TEXT_BRIGHT).size(11.0).strong());
        ui.separator();

        let whisper_busy  = vm.whisper_is_running();
        let already_runs  = is_transcribing;

        let transcribe_label = if already_runs {
            "⟳  Transcribing…"
        } else if has_transcription {
            "🎙  Re-transcribe"
        } else {
            "🎙  Transcribe"
        };

        if ui.add_enabled(
            !whisper_busy,
            egui::Button::new(
                egui::RichText::new(transcribe_label).color(ACCENT_CYAN).size(11.0),
            ),
        ).clicked() {
            transcribe_id = Some(id.clone());
            ui.close_menu();
        }

        if has_transcription {
            ui.separator();
            let count = vm.project.subtitles.iter()
                .filter(|s| s.media_id.as_deref() == Some(&id))
                .count();
            ui.label(
                egui::RichText::new(format!("{} subtitle(s) linked", count))
                    .color(TEXT_DIM)
                    .size(10.0),
            );

            if ui.add(
                egui::Button::new(
                    egui::RichText::new("✖  Remove Transcription").color(ACCENT_AMBER).size(11.0),
                )
                .fill(egui::Color32::TRANSPARENT),
            ).clicked() {
                vm.project.subtitles.retain(|s| s.media_id.as_deref() != Some(&id));
                vm.update_duration();
                ui.close_menu();
            }
        }
    });

    let block_col = if is_transcribing {
        egui::Color32::from_rgba_unmultiplied(34, 211, 238, 25) 
    } else {
        egui::Color32::from_rgba_unmultiplied(245, 158, 11, 35)
    };
    painter.rect_filled(rect, 3.0, block_col);

    let border_col = if is_transcribing { ACCENT_CYAN } else { ACCENT_AMBER };
    painter.rect_stroke(rect, 3.0, Stroke::new(if is_transcribing { 1.5 } else { 1.0 }, border_col));

    let stripe_spacing = 6.0;
    let mut sx = rect.min.x + 4.0;
    while sx < rect.max.x - 2.0 {
        let h = (((sx * 0.3).sin() * 0.5 + 0.5) * (rect.height() * 0.6)) as f32;
        let cy = rect.center().y;
        painter.line_segment([Pos2::new(sx, cy - h * 0.5), Pos2::new(sx, cy + h * 0.5)],
            Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(245, 158, 11, 80)),
        );
        sx += stripe_spacing;
    }

    if has_transcription {
        let dot_pos = Pos2::new(rect.max.x - 5.0, rect.min.y + 5.0);
        painter.circle_filled(dot_pos, 3.5, ACCENT_CYAN);
    }

    let short_name = if name.len() > 14 { format!("{}…", &name[..14]) } else { name.clone() };
    let label_text = format!("{} {:.1}s", short_name, duration);
    painter.text(
        Pos2::new(rect.min.x + 4.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label_text,
        egui::FontId::proportional(8.5),
        egui::Color32::from_rgba_unmultiplied(245, 158, 11, 200),
    );

    if !has_transcription && !vm.whisper_is_running() && block_w > 80.0 {
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 4.0),
            egui::Align2::CENTER_BOTTOM,
            "Right-click → Transcribe",
            egui::FontId::proportional(7.5),
            egui::Color32::from_rgba_unmultiplied(245, 158, 11, 100),
        );
    }
}

if let Some(id) = transcribe_id {
    vm.start_auto_transcription(id);
}

}