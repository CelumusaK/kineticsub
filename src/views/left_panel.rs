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
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("⬆  Audio").color(ACCENT_CYAN)).clicked() {
                vm.import_audio();
            }
            if ui.button(egui::RichText::new("➕  Color BG").color(TEXT_BRIGHT)).clicked() {
                vm.add_solid_bg();
            }
        });
        ui.add_space(6.0);

        let media_indices: Vec<usize> = (0..vm.project.media_files.len()).collect();

        for i in media_indices {
            draw_media_card(ui, vm, i);
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

fn draw_media_card(ui: &mut egui::Ui, vm: &mut EditorViewModel, index: usize) {
    // Extract info without holding a borrow on `vm`
    let (id, name, duration, is_video_track, on_timeline) = {
        let m = &vm.project.media_files[index];
        (m.id.clone(), m.name.clone(), m.duration, m.is_video_track, m.on_timeline)
    };
    
    let mut current_color = vm.project.media_files[index].color;
    let mut color_changed = false;

    let transcription_count = vm.project.subtitles.iter()
        .filter(|s| s.media_id.as_deref() == Some(id.as_str()))
        .count();
    let has_transcription = transcription_count > 0;

    let mut remove_clicked = false;
    let mut add_clicked = false;

    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 5))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let display_name = if name.len() > 18 { format!("{}…", &name[..18]) } else { name.clone() };
                let icon = if is_video_track { "🎞" } else { "🎵" };
                ui.label(egui::RichText::new(format!("{} {}", icon, display_name)).color(TEXT_BRIGHT).size(11.0));
                
                if is_video_track {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(col) = &mut current_color {
                            if ui.color_edit_button_rgba_unmultiplied(col).changed() {
                                color_changed = true;
                            }
                        }
                    });
                }
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{:.1}s", duration)).color(TEXT_DIM).size(10.0));
                if has_transcription {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(format!("⛓ {} subs", transcription_count)).color(ACCENT_AMBER).size(9.5));
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if on_timeline {
                    if ui.add(small_btn("➖ Remove", ACCENT_AMBER)).clicked() {
                        remove_clicked = true;
                    }
                    if !is_video_track {
                        ui.label(egui::RichText::new("Right-click block to transcribe").color(TEXT_DIM).size(9.0));
                    }
                } else {
                    if ui.add(small_btn("➕ Add to Timeline", ACCENT_CYAN)).clicked() {
                        add_clicked = true;
                    }
                }
            });
        });

    // Write back changes to `vm` AFTER the UI closure
    if color_changed {
        vm.project.media_files[index].color = current_color;
    }
    if remove_clicked || add_clicked {
        vm.toggle_media_timeline(&id);
    }
}

// ── Section: Subtitle List ────────────────────────────────────────────────────

fn draw_subtitle_list(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_header(ui, "SUBTITLES");

    if vm.project.subtitles.is_empty() {
        ui.indent("subs_empty", |ui| { ui.label(egui::RichText::new("No subtitles yet.").color(TEXT_DIM).size(11.0)); });
        return;
    }

    let mut clicked_id = None;
    let num_rows = vm.project.subtitles.len();
    let row_height = 24.0; 

    egui::ScrollArea::vertical()
        .id_salt("sub_list_scroll")
        .max_height(180.0)
        .show_rows(ui, row_height, num_rows, |ui, row_range| {
            for i in row_range {
                let sub = &vm.project.subtitles[i];

                let mut text_preview: String = sub.text.chars().take(22).collect();
                if sub.text.len() > text_preview.len() { text_preview.push('…'); }
                let time_label = format!("{:02}:{:04.1}", (sub.timeline_start / 60.0) as i32, sub.timeline_start % 60.0);

                let is_selected = vm.selected_ids.contains(&sub.id);
                let is_primary  = vm.selected_id.as_deref() == Some(sub.id.as_str());
                let has_link = sub.media_id.is_some();

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
                            ui.label(egui::RichText::new(&time_label).color(TEXT_DIM).size(10.5).monospace());
                            ui.add_space(2.0);
                            if has_link { ui.label(egui::RichText::new("⛓").size(9.0).color(ACCENT_AMBER)); }
                            ui.add_space(2.0);
                            let label_color = if is_primary { ACCENT_CYAN } else { TEXT_NORM };
                            ui.label(egui::RichText::new(&text_preview).color(label_color).size(11.5));
                        });
                    })
                    .response;

                if resp.interact(egui::Sense::click()).clicked() {
                    clicked_id = Some(sub.id.clone());
                }
            }
        });

    if let Some(id) = clicked_id {
        vm.selected_id = Some(id.clone());
        vm.selected_ids.clear();
        vm.selected_ids.insert(id);
    }

    if vm.selected_ids.len() > 1 {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("{} selected", vm.selected_ids.len())).color(ACCENT_CYAN).size(10.5));
        if ui.add(small_btn("✖ Delete Selected", ACCENT_AMBER)).clicked() {
            vm.delete_selected_subtitles();
        }
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
    let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
    for sub in vm.project.subtitles.iter_mut() {
        if ids.contains(&sub.id) {
            match preset {
                "Default"     => { sub.y = 0.0;    sub.x = 0.0;    sub.font_size = 36.0; } // <-- Center
                "Lower Third" => { sub.y = 380.0;  sub.x = -600.0; sub.font_size = 30.0; }
                "Title Card"  => { sub.y = 0.0;    sub.x = 0.0;    sub.font_size = 72.0; }
                "Caption"     => { sub.y = 420.0;  sub.x = 0.0;    sub.font_size = 28.0; }
                _ => {}
            }
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