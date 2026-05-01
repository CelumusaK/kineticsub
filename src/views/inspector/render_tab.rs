use egui;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::RenderMode;
use crate::views::theme::{TEXT_DIM, TEXT_BRIGHT, ACCENT_CYAN};
use super::widgets::*;

pub fn draw_render(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_label(ui, "PROJECT SETTINGS");
    
    ui.add_space(4.0);
    two_col_row(ui, |ui| {
        ui.label(egui::RichText::new("Width").color(TEXT_DIM).size(10.5));
        ui.add(egui::DragValue::new(&mut vm.project.resolution.0).speed(10.0).range(100..=7680));
    }, |ui| {
        ui.label(egui::RichText::new("Height").color(TEXT_DIM).size(10.5));
        ui.add(egui::DragValue::new(&mut vm.project.resolution.1).speed(10.0).range(100..=7680));
    });
    
    ui.add_space(4.0);
    prop_row(ui, "FPS", |ui| {
        ui.add(egui::DragValue::new(&mut vm.project.fps).speed(1.0).range(12..=120));
    });
    
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    section_label(ui, "EXPORT / RENDER");

    ui.add_space(8.0);
    ui.label(egui::RichText::new("Output Format").color(TEXT_BRIGHT).size(11.0));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.radio_value(&mut vm.render_mode, RenderMode::ImageSequence, "Image Sequence\n(PNGs)");
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.radio_value(&mut vm.render_mode, RenderMode::Video, "Full Video\n(Solid MP4)");
    });

    ui.add_space(12.0);

    ui.add_enabled_ui(vm.render_mode == RenderMode::ImageSequence, |ui| {
        ui.checkbox(&mut vm.render_transparent_bg, "Transparent Background");
    });

    ui.add_space(4.0);

    ui.add_enabled_ui(vm.render_mode == RenderMode::Video, |ui| {
        ui.checkbox(&mut vm.render_include_audio, "Include Audio Track");
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);

    let btn_text = if vm.is_rendering { "RENDERING..." } else { "▶ EXPORT PROJECT" };

    ui.vertical_centered(|ui| {
        if ui.add_sized([ui.available_width() - 16.0, 32.0],
            egui::Button::new(egui::RichText::new(btn_text).strong().color(crate::views::theme::BG_BASE).size(12.0))
                .fill(ACCENT_CYAN)
        ).clicked() {
            if !vm.is_rendering {
                vm.start_render();
            }
        }
        
        if vm.is_rendering || vm.render_status.contains("Complete") || vm.render_status.contains("Error") {
            ui.add_space(12.0);
            
            let color = if vm.render_status.contains("Error") {
                crate::views::theme::ACCENT_AMBER
            } else {
                TEXT_DIM
            };

            ui.label(egui::RichText::new(&vm.render_status).color(color).size(11.0));
            
            ui.add_space(4.0);
            
            if vm.is_rendering {
                let progress_bar = egui::ProgressBar::new(vm.render_progress)
                    .fill(ACCENT_CYAN)
                    .animate(true);
                ui.add(progress_bar);
            }

            ui.add_space(8.0);
            if ui.button("Dismiss").clicked() {
                vm.is_rendering = false;
                vm.render_status.clear();
            }
        }
    });
}