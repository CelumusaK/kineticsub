use egui;
use crate::views::theme::{BG_HOVER, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT};

pub fn section_label(ui: &mut egui::Ui, label: &str) {
ui.add_space(2.0);
ui.label(egui::RichText::new(label).color(TEXT_DIM).size(10.5).strong());
ui.add_space(2.0);
}

pub fn collapsible_section(
ui: &mut egui::Ui,
label: &str,
enabled: &mut bool,
outer_changed: &mut bool,
body: impl FnOnce(&mut egui::Ui, &mut bool),
) {
ui.horizontal(|ui| {
if ui.checkbox(enabled, egui::RichText::new(label)
.color(if *enabled { TEXT_BRIGHT } else { TEXT_DIM })
.size(10.5).strong()).changed()
{
*outer_changed = true;
}
});
if *enabled {
egui::Frame::none()
.fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 3))
.inner_margin(egui::Margin::symmetric(8.0, 4.0))
.show(ui, |ui| {
body(ui, outer_changed);
});
}
ui.add_space(2.0);
}

pub fn two_col_row(
ui: &mut egui::Ui,
left: impl FnOnce(&mut egui::Ui),
right: impl FnOnce(&mut egui::Ui),
) {
ui.columns(2, |cols| {
left(&mut cols[0]);
right(&mut cols[1]);
});
}

pub fn prop_row(ui: &mut egui::Ui, label: &str, content: impl FnOnce(&mut egui::Ui)) {
ui.horizontal(|ui| {
ui.label(egui::RichText::new(label).color(TEXT_NORM).size(11.0));
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
});
}

pub fn circle_dot(ui: &mut egui::Ui, color: egui::Color32) {
let (r, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
ui.painter().circle_filled(r.center(), 4.0, color);
}

pub fn align_btn(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
let resp = ui.add(
egui::Button::new(egui::RichText::new(icon).size(12.0).color(TEXT_NORM))
.fill(BG_HOVER)
.stroke(egui::Stroke::new(1.0, BORDER))
.min_size(egui::Vec2::new(30.0, 24.0)),
);
resp.on_hover_text(tooltip)
}