use egui::{Context, SidePanel};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{BG_PANEL, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    SidePanel::left("left_panel")
        .exact_width(240.0)
        .resizable(false)
        .frame(egui::Frame {
            fill: BG_PANEL,
            stroke: egui::Stroke::new(1.0, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.add_space(2.0);
            draw_media_bin(ui, vm);
            separator(ui);
            draw_subtitle_list(ui, vm);
            separator(ui);
            draw_presets(ui, vm);
        });
}

// ── Section: Media Bin ────────────────────────────────────────────────────────

fn draw_media_bin(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_header(ui, "MEDIA BIN");
    ui.indent("media_indent", |ui| {
        if ui.button(egui::RichText::new("⬆  Import Audio").color(ACCENT_CYAN)).clicked() {
            vm.import_audio();
        }
        ui.add_space(6.0);

        for media in vm.project.media_files.clone() {
            draw_media_card(ui, vm, &media);
            ui.add_space(4.0);
        }

        if vm.project.media_files.is_empty() {
            ui.label(egui::RichText::new("No media imported.").color(TEXT_DIM).size(10.5));
        }
    });

    if vm.whisper_is_running() {
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(format!("⟳  {}", vm.whisper_status))
                .color(ACCENT_CYAN)
                .size(10.0)
                .monospace(),
        );
    }
}

fn draw_media_card(ui: &mut egui::Ui, vm: &mut EditorViewModel, media: &crate::models::types::MediaFile) {
    let has_transcription = vm.project.subtitles.iter()
        .any(|s| s.media_id.as_deref() == Some(&media.id));

    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 5))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            let name = if media.name.len() > 22 {
                format!("{}…", &media.name[..22])
            } else {
                media.name.clone()
            };
            ui.label(egui::RichText::new(&name).color(TEXT_BRIGHT).size(11.0));

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{:.1}s", media.duration)).color(TEXT_DIM).size(10.0));
                if has_transcription {
                    ui.add_space(6.0);
                    let count = vm.project.subtitles.iter()
                        .filter(|s| s.media_id.as_deref() == Some(&media.id))
                        .count();
                    ui.label(
                        egui::RichText::new(format!("⛓ {} subs", count))
                            .color(ACCENT_AMBER)
                            .size(9.5),
                    );
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let on_tl = media.on_timeline;
                if on_tl {
                    if ui.add(small_btn("➖ Remove", ACCENT_AMBER)).clicked() {
                        vm.toggle_media_timeline(&media.id);
                    }
                    ui.label(
                        egui::RichText::new("Right-click block to transcribe")
                            .color(TEXT_DIM)
                            .size(9.0),
                    );
                } else {
                    if ui.add(small_btn("➕ Add to Timeline", ACCENT_CYAN)).clicked() {
                        vm.toggle_media_timeline(&media.id);
                    }
                }
            });
        });
}

// ── Section: Subtitle List ────────────────────────────────────────────────────

fn draw_subtitle_list(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_header(ui, "SUBTITLES");

    let subtitle_ids: Vec<(String, String, f64)> = vm.project.subtitles.iter()
        .map(|s| (s.id.clone(), s.text.clone(), s.timeline_start))
        .collect();

    if subtitle_ids.is_empty() {
        ui.indent("subs_empty", |ui| {
            ui.label(egui::RichText::new("No subtitles yet.").color(TEXT_DIM).size(11.0));
        });
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("sub_list_scroll")
        .max_height(180.0)
        .show(ui, |ui| {
            for (id, text, start) in &subtitle_ids {
                draw_subtitle_row(ui, vm, id, text, *start);
            }
        });

    if vm.selected_ids.len() > 1 {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("{} selected", vm.selected_ids.len()))
                .color(ACCENT_CYAN)
                .size(10.5),
        );
        if ui.add(small_btn("✖ Delete Selected", ACCENT_AMBER)).clicked() {
            vm.delete_selected_subtitles();
        }
    }
}

fn draw_subtitle_row(ui: &mut egui::Ui, vm: &mut EditorViewModel, id: &str, text: &str, start: f64) {
    let text_preview = if text.len() > 22 { format!("{}…", &text[..22]) } else { text.to_string() };
    let time_label   = format!("{:02}:{:04.1}", (start / 60.0) as i32, start % 60.0);

    let is_selected = vm.selected_ids.contains(id);
    let is_primary  = vm.selected_id.as_deref() == Some(id);

    let has_link = vm.project.subtitles.iter()
        .find(|s| s.id == id)
        .and_then(|s| s.media_id.as_ref())
        .is_some();

    let row_fill = if is_primary {
        egui::Color32::from_rgba_unmultiplied(34, 211, 238, 28)
    } else if is_selected {
        egui::Color32::from_rgba_unmultiplied(34, 211, 238, 12)
    } else {
        egui::Color32::TRANSPARENT
    };

    let resp = egui::Frame::none()
        .fill(row_fill)
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .rounding(egui::Rounding::same(3.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&time_label).color(TEXT_DIM).size(10.5).monospace(),
                );
                ui.add_space(2.0);
                if has_link {
                    ui.label(egui::RichText::new("⛓").size(9.0).color(ACCENT_AMBER));
                }
                ui.add_space(2.0);
                let label_color = if is_primary { ACCENT_CYAN } else { TEXT_NORM };
                ui.label(egui::RichText::new(&text_preview).color(label_color).size(11.5));
            });
        })
        .response;

    // FIX: clicking a subtitle in the list only selects it — does NOT seek.
    // The user can then use the timeline playhead to position manually.
    if resp.interact(egui::Sense::click()).clicked() {
        vm.selected_id = Some(id.to_string());
        vm.selected_ids.clear();
        vm.selected_ids.insert(id.to_string());
        // Do NOT call vm.seek_to(start) — let the user control playhead position.
    }
}

// ── Section: Presets ──────────────────────────────────────────────────────────

fn draw_presets(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_header(ui, "STYLE PRESETS");
    ui.indent("presets_indent", |ui| {
        for preset_name in &["Default", "Lower Third", "Title Card", "Caption"] {
            if ui.selectable_label(
                false,
                egui::RichText::new(*preset_name).color(TEXT_NORM).size(11.5),
            ).clicked() {
                apply_style_preset(vm, preset_name);
            }
        }
    });
}

fn apply_style_preset(vm: &mut EditorViewModel, preset: &str) {
    if let Some(sub) = vm.selected_subtitle_mut() {
        match preset {
            "Default"     => { sub.y = 300.0;  sub.x = 0.0;    sub.font_size = 36.0; }
            "Lower Third" => { sub.y = 380.0;  sub.x = -600.0; sub.font_size = 30.0; }
            "Title Card"  => { sub.y = 0.0;    sub.x = 0.0;    sub.font_size = 72.0; }
            "Caption"     => { sub.y = 420.0;  sub.x = 0.0;    sub.font_size = 28.0; }
            _ => {}
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(title).color(TEXT_DIM).size(11.0).strong());
    ui.add_space(2.0);
}

fn separator(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(0.0));
}

fn small_btn(label: &str, color: egui::Color32) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(label).color(color).size(10.5))
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.5)))
}