use super::{HEADER_H, TOOLBAR_H};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_HOVER, BG_PANEL, BORDER, TEXT_DIM, TEXT_NORM};
use egui::{Pos2, Rect, RichText, Rounding, Stroke, Vec2};

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
            .inner_margin(egui::Margin::symmetric(10.0, 0.0))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(panel_rect.width(), TOOLBAR_H));
                ui.horizontal_centered(|ui| {
                    let cut_btn =
                        egui::Button::new(RichText::new("✂ Split (K)").color(TEXT_NORM).size(11.0))
                            .fill(BG_HOVER)
                            .stroke(Stroke::new(1.0_f32, BORDER))
                            .rounding(Rounding::same(4.0));

                    if ui
                        .add(cut_btn)
                        .on_hover_text("Split active clips at the playhead position")
                        .clicked()
                    {
                        vm.split_at_playhead();
                    }

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Non-Linear Multi-Track")
                            .color(TEXT_DIM)
                            .size(10.5),
                    );
                });
            });
    });
}
