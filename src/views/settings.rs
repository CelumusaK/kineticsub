use egui::{Context, RichText, Window};
use crate::services::whisper_service::{
    delete_model, get_models_dir, is_model_downloaded, WhisperModel,
};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{
    ACCENT_AMBER, ACCENT_CYAN, BG_HOVER, BG_PANEL, BORDER, TEXT_BRIGHT, TEXT_DIM, TEXT_NORM,
};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    if !vm.show_settings_modal {
        return;
    }

    let mut open = vm.show_settings_modal;
    let mut close_requested = false;

    Window::new("Settings")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(520.0)
        .frame(egui::Frame {
            fill: BG_PANEL,
            stroke: egui::Stroke::new(1.0_f32, BORDER),
            inner_margin: egui::Margin::same(14.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                ui.label(RichText::new("WHISPER AI SPEECH RECOGNITION").color(TEXT_DIM).size(11.0).strong());
                ui.label(
                    RichText::new("Choose and download Whisper models locally. Models are only downloaded when requested.")
                        .color(TEXT_NORM)
                        .size(11.0),
                );

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Active Model:").color(TEXT_BRIGHT).size(12.0).strong());
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
                        ui.label(RichText::new("● Ready").color(egui::Color32::from_rgb(34, 197, 94)).size(11.0));
                    } else {
                        ui.label(RichText::new("⚠ Not Downloaded").color(ACCENT_AMBER).size(11.0));
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(RichText::new("AVAILABLE MODELS").color(TEXT_DIM).size(10.5).strong());
                ui.add_space(4.0);

                let is_downloading = vm.is_downloading_model();
                let mut model_to_download = None;
                let mut model_to_delete = None;

                for model in WhisperModel::all() {
                    let is_active = vm.selected_whisper_model == *model;
                    let downloaded = is_model_downloaded(*model);
                    let is_this_downloading = vm.downloading_model == Some(*model);

                    egui::Frame::none()
                        .fill(if is_active {
                            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 14)
                        } else {
                            BG_HOVER
                        })
                        .stroke(egui::Stroke::new(
                            1.0_f32,
                            if is_active { ACCENT_CYAN } else { BORDER },
                        ))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.radio(is_active, "").clicked() {
                                    vm.selected_whisper_model = *model;
                                }

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(model.label()).color(TEXT_BRIGHT).size(11.5).strong());
                                        if is_active {
                                            ui.label(RichText::new("[Selected]").color(ACCENT_CYAN).size(10.0));
                                        }
                                    });

                                    if is_this_downloading {
                                        ui.add_space(3.0);
                                        ui.add(egui::ProgressBar::new(vm.model_download_progress).animate(true));
                                        ui.label(RichText::new(&vm.model_download_status).color(ACCENT_CYAN).size(10.0));
                                    } else if downloaded {
                                        ui.label(RichText::new("Installed locally").color(egui::Color32::from_rgb(34, 197, 94)).size(10.0));
                                    } else {
                                        ui.label(RichText::new("Not downloaded").color(TEXT_DIM).size(10.0));
                                    }
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !downloaded && !is_this_downloading {
                                        let btn = egui::Button::new(RichText::new("⬇ Download").color(ACCENT_CYAN).size(11.0))
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::new(1.0_f32, ACCENT_CYAN));
                                        if ui.add_enabled(!is_downloading, btn).clicked() {
                                            model_to_download = Some(*model);
                                        }
                                    } else if downloaded && !is_this_downloading {
                                        if ui.button(RichText::new("🗑 Delete").color(TEXT_DIM).size(10.5)).clicked() {
                                            model_to_delete = Some(*model);
                                        }
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }

                if let Some(m) = model_to_download {
                    vm.start_model_download(m);
                }
                if let Some(m) = model_to_delete {
                    let _ = delete_model(m);
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("Storage directory: {}", get_models_dir().display()))
                        .color(TEXT_DIM)
                        .size(9.5),
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(RichText::new("KEYBOARD SHORTCUTS").color(TEXT_DIM).size(10.5).strong());
                ui.add_space(4.0);
                
                let shortcuts = [
                    ("Space", "Play / Pause playback"),
                    ("J / L", "Skip -5s / +5s backward and forward"),
                    ("Left / Right", "Skip ±1 frame precision"),
                    ("Ctrl + S", "Save project"),
                    ("Ctrl + Z", "Undo action"),
                    ("Ctrl + Shift + Z", "Redo action"),
                    ("Escape", "Deselect active item"),
                    ("Alt + Scroll", "Zoom Timeline smoothly"),
                ];

                for (keys, action) in shortcuts {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(keys).color(ACCENT_CYAN).size(11.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(action).color(TEXT_NORM).size(11.0));
                        });
                    });
                    ui.add_space(2.0);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(RichText::new("GENERAL SETTINGS").color(TEXT_DIM).size(10.5).strong());
                ui.checkbox(&mut vm.show_fps, "Show Real-Time Uncapped FPS");

            }); // End ScrollArea

            ui.add_space(14.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Close").color(TEXT_BRIGHT)).clicked() {
                    close_requested = true;
                }
            });
        });

    if close_requested {
        open = false;
    }
    
    vm.show_settings_modal = open;
}