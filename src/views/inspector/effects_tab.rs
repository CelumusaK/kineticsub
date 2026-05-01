use egui;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::TEXT_DIM;
use super::widgets::*;

pub fn draw_effects(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let mut motion_blur = match vm.selected_subtitle() {
        Some(s) => s.motion_blur,
        None => return,
    };
    
    let mut changed = false;

    section_label(ui, "EFFECTS");
    
    ui.add_space(4.0);
    prop_row(ui, "Motion Blur", |ui| {
        if ui.add(egui::Slider::new(&mut motion_blur, 0.0..=1.0)).changed() {
            changed = true;
        }
    });
    ui.label(egui::RichText::new("Generates a trailing visual blur on moving objects.").color(TEXT_DIM).size(10.0));

    if changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                sub.motion_blur = motion_blur;
            }
        }
    }
}