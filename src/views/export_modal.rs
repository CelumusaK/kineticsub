use egui::{Context, RichText, Window};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::RenderMode;
use crate::views::theme::{ACCENT_AMBER, ACCENT_CYAN, BG_BASE, BG_PANEL, BORDER, TEXT_BRIGHT, TEXT_DIM};
use crate::views::inspector::widgets::{prop_row, section_label, two_col_row};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    if !vm.show_export_modal {
        return;
    }

    let mut open = vm.show_export_modal;
    let mut close_requested = false;

    Window::new("Export Project")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .frame(egui::Frame {
            fill: BG_PANEL,
            stroke: egui::Stroke::new(1.0_f32, BORDER),
            inner_margin: egui::Margin::same(16.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.add_enabled_ui(!vm.is_rendering, |ui| {
                section_label(ui, "PROJECT SETTINGS");
                
                ui.add_space(4.0);
                two_col_row(ui, |ui| {
                    ui.label(RichText::new("Width").color(TEXT_DIM).size(11.0));
                    ui.add(egui::DragValue::new(&mut vm.project.resolution.0).speed(10.0).range(100..=7680));
                }, |ui| {
                    ui.label(RichText::new("Height").color(TEXT_DIM).size(11.0));
                    ui.add(egui::DragValue::new(&mut vm.project.resolution.1).speed(10.0).range(100..=7680));
                });
                
                ui.add_space(4.0);
                prop_row(ui, "FPS", |ui| {
                    ui.add(egui::DragValue::new(&mut vm.project.fps).speed(1.0).range(12..=120));
                });
                
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                section_label(ui, "OUTPUT FORMAT");

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.radio_value(&mut vm.render_mode, RenderMode::Video, "Full Video (Solid MP4)");
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut vm.render_mode, RenderMode::ImageSequence, "Image Sequence (PNG Zip)");
                });

                ui.add_space(12.0);

                ui.add_enabled_ui(vm.render_mode == RenderMode::ImageSequence, |ui| {
                    ui.checkbox(&mut vm.render_transparent_bg, "Transparent Background");
                });

                ui.add_space(4.0);

                ui.add_enabled_ui(vm.render_mode == RenderMode::Video, |ui| {
                    ui.checkbox(&mut vm.render_include_audio, "Include Audio Tracks (Mix)");
                });
            });

            ui.add_space(24.0);
            ui.separator();
            ui.add_space(16.0);

            let btn_text = if vm.is_rendering { "RENDERING..." } else { "▶ EXPORT PROJECT" };

            ui.vertical_centered(|ui| {
                if ui.add_sized([ui.available_width(), 36.0],
                    egui::Button::new(RichText::new(btn_text).strong().color(BG_BASE).size(13.0))
                        .fill(ACCENT_CYAN)
                ).clicked() {
                    if !vm.is_rendering {
                        vm.start_render();
                    }
                }
                
                if vm.is_rendering || vm.render_status.contains("Complete") || vm.render_status.contains("Error") {
                    ui.add_space(16.0);
                    
                    let color = if vm.render_status.contains("Error") {
                        ACCENT_AMBER
                    } else if vm.render_status.contains("Complete") {
                        egui::Color32::from_rgb(34, 197, 94)
                    } else {
                        TEXT_BRIGHT
                    };

                    ui.label(RichText::new(&vm.render_status).color(color).size(12.0));
                    
                    ui.add_space(8.0);
                    
                    let progress_bar = egui::ProgressBar::new(vm.render_progress)
                        .fill(ACCENT_CYAN)
                        .animate(vm.is_rendering);
                    ui.add(progress_bar);

                    if !vm.is_rendering {
                        ui.add_space(12.0);
                        if ui.button("Close").clicked() {
                            vm.render_status.clear();
                            close_requested = true;
                        }
                    }
                }
            });
        });

    if close_requested {
        open = false;
    }
    
    vm.show_export_modal = open;
}