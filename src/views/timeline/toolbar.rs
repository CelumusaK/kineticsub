use egui::{Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, ACCENT_AMBER, BG_PANEL, TEXT_DIM};
use super::{HEADER_H, TOOLBAR_H};

// ── Toolbar ────────────────────────────────────────────────────────────────────

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
                ui.add(egui::TextEdit::singleline(&mut vm.new_sub_text)
                    .desired_width(180.0)
                    .hint_text("Subtitle text…")
                    .font(egui::TextStyle::Body));

                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("+ Add at playhead").color(ACCENT_CYAN).size(11.0),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, ACCENT_CYAN)),
                ).clicked() {
                    vm.insert_subtitle_at_playhead();
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("Zoom").color(TEXT_DIM).size(10.0));
                ui.add(egui::Slider::new(&mut vm.timeline_zoom, 10.0..=1000.0).show_value(false));

                let has_sel = !vm.selected_ids.is_empty();
                if has_sel {
                    ui.add_space(8.0);
                    let label = if vm.selected_ids.len() > 1 {
                        format!("✖ Delete ({})", vm.selected_ids.len())
                    } else {
                        "✖ Delete".into()
                    };
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new(label).color(ACCENT_AMBER).size(10.5),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    ).clicked() {
                        if vm.selected_ids.len() > 1 {
                            vm.delete_selected_subtitles();
                        } else if let Some(id) = vm.selected_id.clone() {
                            vm.delete_subtitle(&id);
                        }
                    }
                }

                // Whisper status (shown in toolbar when running)
                if vm.whisper_is_running() || !vm.whisper_status.is_empty() {
                    ui.add_space(12.0);
                    let icon = if vm.whisper_is_running() { "⟳ " } else { "✓ " };
                    ui.label(
                        egui::RichText::new(format!("{}{}", icon, vm.whisper_status))
                            .color(if vm.whisper_is_running() { ACCENT_CYAN } else { TEXT_DIM })
                            .size(10.0)
                            .monospace(),
                    );
                }
            });
        });
});

}