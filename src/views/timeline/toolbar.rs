use egui::{Pos2, Rect, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL, TEXT_DIM};
use super::{HEADER_H, TOOLBAR_H};

pub fn draw_toolbar(ui: &mut egui::Ui, vm: &mut EditorViewModel, panel_rect: Rect) {
    let tb_y = panel_rect.min.y + HEADER_H;
    let tb_rect = Rect::from_min_size(
        Pos2::new(panel_rect.min.x, tb_y),
        Vec2::new(panel_rect.width(), TOOLBAR_H),
    );

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(tb_rect), |ui| {
        ui.set_clip_rect(tb_rect);
        egui::Frame::none()
            .fill(BG_PANEL)
            .inner_margin(egui::Margin::symmetric(12.0, 0.0))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(panel_rect.width(), TOOLBAR_H));
                ui.horizontal_centered(|ui| {
                    if ui.button("✂").on_hover_text("Split selected clips at playhead (K)").clicked() {
                        vm.split_at_playhead();
                    }
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("Zoom Timeline").color(TEXT_DIM).size(10.0));
                    ui.add(egui::Slider::new(&mut vm.timeline_zoom, 10.0..=1000.0).show_value(false));
                });
            });
    });
}