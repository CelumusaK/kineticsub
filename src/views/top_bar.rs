use egui::{Align, Context, Layout, TopBottomPanel};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{ACCENT_CYAN, ACCENT_AMBER, TEXT_DIM, TEXT_NORM, BG_BASE, BORDER};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    TopBottomPanel::top("top_bar")
        .exact_height(34.0)
        .frame(egui::Frame {
            fill: BG_BASE,
            stroke: egui::Stroke::new(1.0_f32, BORDER),
            inner_margin: egui::Margin::symmetric(12.0, 0.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.set_min_height(34.0);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label(egui::RichText::new("KINETICSUB").color(ACCENT_CYAN).size(13.0).strong());
                ui.add_space(20.0);

                ui.menu_button(egui::RichText::new("File").color(TEXT_NORM).size(12.0), |ui| {
                    if ui.button("Open Project...").clicked() {
                        vm.load_project();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save (Ctrl+S)").clicked() {
                        vm.save_project();
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        vm.save_project_as();
                        ui.close_menu();
                    }
                });

                ui.with_layout(Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                    let file_name = vm.filepath.as_ref()
                        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                        .unwrap_or("Untitled.ksub".to_string());
                    ui.label(egui::RichText::new(file_name).color(TEXT_DIM).size(12.0));
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("▶ EXPORT").color(BG_BASE).strong().size(11.0)).fill(ACCENT_CYAN)).clicked() {
                        vm.show_export_modal = true;
                    }
                    
                    ui.add_space(8.0);

                    if ui.button(egui::RichText::new("⚙ Settings").color(TEXT_NORM).size(11.0)).clicked() {
                        vm.show_settings_modal = true;
                    }
                    
                    ui.add_space(4.0);

                    ui.menu_button(egui::RichText::new("✨ Presets").color(TEXT_NORM).size(11.0), |ui| {
                        ui.menu_button("Text", |ui| {
                            if ui.button("Default").clicked() { vm.apply_preset("Text", "Default"); ui.close_menu(); }
                            if ui.button("Lower Third").clicked() { vm.apply_preset("Text", "Lower Third"); ui.close_menu(); }
                            if ui.button("Title Card").clicked() { vm.apply_preset("Text", "Title Card"); ui.close_menu(); }
                            if ui.button("Caption").clicked() { vm.apply_preset("Text", "Caption"); ui.close_menu(); }
                        });
                        ui.menu_button("Video & Photo", |ui| {
                            if ui.button("Fit to Screen").clicked() { vm.apply_preset("Video", "Fit to Screen"); ui.close_menu(); }
                            if ui.button("Picture in Picture").clicked() { vm.apply_preset("Video", "Picture in Picture"); ui.close_menu(); }
                        });
                    });

                    if vm.show_fps {
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(format!("{:.0} FPS", vm.current_fps))
                                .color(ACCENT_AMBER)
                                .size(11.0)
                                .strong()
                        );
                    }
                });
            });
        });
}