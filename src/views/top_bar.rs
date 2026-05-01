use egui::{Align, Context, Layout, TopBottomPanel};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{ACCENT_CYAN, ACCENT_AMBER, TEXT_DIM, TEXT_NORM, BG_BASE, BORDER};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
TopBottomPanel::top("top_bar")
.exact_height(30.0)
.frame(egui::Frame {
fill: BG_BASE,
stroke: egui::Stroke::new(1.0, BORDER),
inner_margin: egui::Margin::symmetric(12.0, 0.0),
..Default::default()
})
.show(ctx, |ui| {
ui.set_min_height(30.0);
ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
ui.label(egui::RichText::new("KINETICSUB").color(ACCENT_CYAN).size(12.5).strong());
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

            ui.add_space(8.0);
            
            ui.menu_button(egui::RichText::new("Edit").color(TEXT_NORM).size(12.0), |ui| {
                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut vm.show_fps, "Show Uncapped FPS");
                });
            });
            
            ui.add_space(8.0);
            
            ui.menu_button(egui::RichText::new("Help").color(TEXT_NORM).size(12.0), |ui| {
                ui.label(egui::RichText::new("Shortcuts:").strong());
                ui.separator();
                ui.label("Space        - Play / Pause");
                ui.label("J / L        - Skip -5s / +5s");
                ui.label("Left / Right - Skip ±1 frame");
                ui.label("Ctrl+S       - Save Project");
                ui.label("Escape       - Deselect Subtitle");
                ui.label("Scroll       - Pan Timeline");
                ui.label("Alt+Scroll   - Zoom Timeline");
            });

            ui.with_layout(Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                let file_name = vm.filepath.as_ref()
                    .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                    .unwrap_or("Untitled.ksub".to_string());
                ui.label(egui::RichText::new(file_name).color(TEXT_DIM).size(12.0));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(egui::RichText::new("v0.3.0-RS").color(TEXT_DIM).size(11.0));
                
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