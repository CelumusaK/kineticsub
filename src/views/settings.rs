use egui::{Align, Align2, Color32, Context, Frame, Layout, Margin, ProgressBar, RichText, Rounding, ScrollArea, Stroke, Window};
use crate::services::whisper_service::{
    delete_model, get_models_dir, is_model_downloaded, WhisperModel,
};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{
    ACCENT_AMBER, ACCENT_CYAN, ACCENT_EMERALD, BG_BASE, BG_HOVER, BG_PANEL, BG_PANEL_ALT, BORDER,
    TEXT_BRIGHT, TEXT_DIM, TEXT_MUTED, TEXT_NORM,
};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    if !vm.show_settings_modal {
        return;
    }

    let mut open = vm.show_settings_modal;
    let mut close_requested = false;

    let center_pos = ctx.screen_rect().center();

    Window::new("Settings")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(center_pos)
        .fixed_size([560.0, 540.0])
        .frame(Frame {
            fill: BG_PANEL,
            stroke: Stroke::new(1.0, BORDER),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::same(16.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.set_min_size(egui::vec2(528.0, 508.0));

            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("Preferences").size(15.0).strong().color(TEXT_BRIGHT));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("✕").size(12.0).color(TEXT_DIM)).frame(false)).clicked() {
                        close_requested = true;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Stable Scroll Area
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(430.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("SPEECH RECOGNITION (WHISPER AI)").color(TEXT_MUTED).size(10.5).strong());
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Models run entirely on-device with zero cloud reliance. Download models once to cache locally.")
                            .color(TEXT_DIM)
                            .size(11.0),
                    );

                    ui.add_space(10.0);

                    // Active model selector
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Active Engine:").color(TEXT_NORM).size(11.5).strong());
                        let cur = vm.selected_whisper_model;
                        let downloaded = is_model_downloaded(cur);

                        egui::ComboBox::from_id_salt("active_whisper_model_combo")
                            .selected_text(cur.label())
                            .show_ui(ui, |ui| {
                                for m in WhisperModel::all() {
                                    let tag = if is_model_downloaded(*m) { " (Installed)" } else { "" };
                                    ui.selectable_value(
                                        &mut vm.selected_whisper_model,
                                        *m,
                                        format!("{}{}", m.label(), tag),
                                    );
                                }
                            });

                        if downloaded {
                            ui.label(RichText::new("● Ready").color(ACCENT_EMERALD).size(11.0));
                        } else {
                            ui.label(RichText::new("○ Not Installed").color(ACCENT_AMBER).size(11.0));
                        }
                    });

                    ui.add_space(12.0);
                    ui.label(RichText::new("AVAILABLE MODELS").color(TEXT_MUTED).size(10.5).strong());
                    ui.add_space(4.0);

                    let is_downloading = vm.is_downloading_model();
                    let mut model_to_download = None;
                    let mut model_to_delete = None;

                    for model in WhisperModel::all() {
                        let is_active = vm.selected_whisper_model == *model;
                        let downloaded = is_model_downloaded(*model);
                        let is_this_downloading = vm.downloading_model == Some(*model);

                        let card_fill = if is_active {
                            Color32::from_rgba_unmultiplied(45, 185, 240, 15)
                        } else {
                            BG_PANEL_ALT
                        };
                        let card_stroke = if is_active {
                            Stroke::new(1.0, ACCENT_CYAN)
                        } else {
                            Stroke::new(1.0, BORDER)
                        };

                        Frame::none()
                            .fill(card_fill)
                            .stroke(card_stroke)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_min_height(34.0);

                                ui.horizontal(|ui| {
                                    if ui.radio(is_active, "").clicked() {
                                        vm.selected_whisper_model = *model;
                                    }

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(model.label()).color(TEXT_BRIGHT).size(11.5).strong());
                                            if is_active {
                                                ui.label(RichText::new("SELECTED").color(ACCENT_CYAN).size(9.5).strong());
                                            }
                                        });

                                        if is_this_downloading {
                                            ui.add_space(2.0);
                                            ui.add(ProgressBar::new(vm.model_download_progress).animate(true));
                                            ui.label(RichText::new(&vm.model_download_status).color(ACCENT_CYAN).size(10.0));
                                        } else if downloaded {
                                            ui.label(RichText::new("Installed locally").color(ACCENT_EMERALD).size(10.0));
                                        } else {
                                            ui.label(RichText::new("Ready to download").color(TEXT_MUTED).size(10.0));
                                        }
                                    });

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if !downloaded && !is_this_downloading {
                                            let btn = egui::Button::new(RichText::new("Download").size(11.0).color(TEXT_BRIGHT))
                                                .fill(BG_HOVER)
                                                .stroke(Stroke::new(1.0, BORDER));
                                            if ui.add_enabled(!is_downloading, btn).clicked() {
                                                model_to_download = Some(*model);
                                            }
                                        } else if downloaded && !is_this_downloading {
                                            if ui.button(RichText::new("Remove").color(TEXT_MUTED).size(10.5)).clicked() {
                                                model_to_delete = Some(*model);
                                            }
                                        }
                                    });
                                });
                            });
                        ui.add_space(3.0);
                    }

                    if let Some(m) = model_to_download {
                        vm.start_model_download(m);
                    }
                    if let Some(m) = model_to_delete {
                        let _ = delete_model(m);
                    }

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("Storage Path: {}", get_models_dir().display()))
                            .color(TEXT_MUTED)
                            .size(9.5),
                    );

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Shortcuts section
                    ui.label(RichText::new("KEYBOARD SHORTCUTS").color(TEXT_MUTED).size(10.5).strong());
                    ui.add_space(6.0);

                    let shortcuts = [
                        ("Space", "Play / Pause timeline"),
                        ("K", "Split clip at playhead"),
                        ("J / L", "Jump -5s / +5s"),
                        ("Left / Right", "Nudge ±1 frame"),
                        ("Ctrl + S", "Save project"),
                        ("Ctrl + Z", "Undo"),
                        ("Ctrl + Shift + Z", "Redo"),
                        ("Delete / Backspace", "Remove selected clip"),
                        ("Escape", "Clear selection"),
                    ];

                    for (keys, action) in shortcuts {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(keys).color(ACCENT_CYAN).size(11.0).strong());
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(action).color(TEXT_DIM).size(11.0));
                            });
                        });
                        ui.add_space(1.0);
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // General
                    ui.label(RichText::new("WORKSPACE").color(TEXT_MUTED).size(10.5).strong());
                    ui.add_space(4.0);
                    ui.checkbox(&mut vm.show_fps, "Show framerate counter");
                    ui.checkbox(&mut vm.timeline_zoom_to_mouse, "Zoom to mouse cursor on timeline");
                    ui.add_space(10.0);
                });

            ui.add_space(8.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("Done").color(TEXT_BRIGHT).size(11.5)).fill(BG_HOVER).stroke(Stroke::new(1.0, BORDER))).clicked() {
                    close_requested = true;
                }
            });
        });

    if close_requested {
        open = false;
    }
    vm.show_settings_modal = open;
}