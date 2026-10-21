use egui::{Align, Context, Direction, Layout, RichText, Stroke, TopBottomPanel, vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{
    ACCENT_AMBER, ACCENT_CYAN, BG_BASE, BG_HOVER, BORDER, TEXT_BRIGHT,
    TEXT_DIM, TEXT_NORM,
};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    TopBottomPanel::top("top_bar")
        .exact_height(36.0)
        .frame(egui::Frame {
            fill: BG_BASE,
            stroke: Stroke::new(1.0_f32, BORDER),
            inner_margin: egui::Margin::symmetric(14.0, 0.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.set_min_height(36.0);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                // Refined, subtle branding
                ui.label(RichText::new("kineticsub").size(13.0).strong().color(TEXT_BRIGHT));

                ui.add_space(16.0);
                ui.painter().line_segment(
                    [egui::pos2(ui.cursor().min.x, ui.cursor().min.y + 10.0), egui::pos2(ui.cursor().min.x, ui.cursor().min.y + 24.0)],
                    Stroke::new(1.0_f32, BORDER),
                );
                ui.add_space(16.0);

                // Menu items
                ui.menu_button(RichText::new("File").color(TEXT_NORM).size(11.5), |ui| {
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

                ui.menu_button(RichText::new("Presets").color(TEXT_NORM).size(11.5), |ui| {
                    ui.menu_button("Text Styles", |ui| {
                        if ui.button("Default").clicked() { vm.apply_preset("Text", "Default"); ui.close_menu(); }
                        if ui.button("Lower Third").clicked() { vm.apply_preset("Text", "Lower Third"); ui.close_menu(); }
                        if ui.button("Title Card").clicked() { vm.apply_preset("Text", "Title Card"); ui.close_menu(); }
                        if ui.button("Caption").clicked() { vm.apply_preset("Text", "Caption"); ui.close_menu(); }
                    });
                    ui.menu_button("Visual Plates", |ui| {
                        if ui.button("Fit to Screen").clicked() { vm.apply_preset("Video", "Fit to Screen"); ui.close_menu(); }
                        if ui.button("Picture in Picture").clicked() { vm.apply_preset("Video", "Picture in Picture"); ui.close_menu(); }
                    });
                });

                // Centered project file label
                ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui| {
                    let file_name = vm.filepath.as_ref()
                        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled Project".to_string());
                    ui.label(RichText::new(file_name).color(TEXT_DIM).size(11.5));
                });

                // Right-aligned actions
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Export Button
                    let export_btn = egui::Button::new(
                        RichText::new("Export").color(BG_BASE).strong().size(11.5),
                    )
                    .fill(ACCENT_CYAN)
                    .rounding(4.0)
                    .min_size(vec2(68.0, 24.0));

                    if ui.add(export_btn).clicked() {
                        vm.show_export_modal = true;
                    }

                    ui.add_space(8.0);

                    // Settings Button
                    let settings_btn = egui::Button::new(
                        RichText::new("Settings").color(TEXT_NORM).size(11.5),
                    )
                    .fill(BG_HOVER)
                    .stroke(Stroke::new(1.0_f32, BORDER))
                    .rounding(4.0);

                    if ui.add(settings_btn).clicked() {
                        vm.show_settings_modal = true;
                    }

                    if vm.show_fps {
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(format!("{:.0} fps", vm.current_fps))
                                .color(ACCENT_AMBER)
                                .size(11.0)
                                .monospace(),
                        );
                    }
                });
            });
        });
}
