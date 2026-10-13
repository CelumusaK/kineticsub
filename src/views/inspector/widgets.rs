use egui::{self, Color32, Margin, RichText, Rounding, Stroke};
use crate::views::theme::{BG_HOVER, BORDER, TEXT_BRIGHT, TEXT_MUTED, TEXT_NORM};

pub fn section_label(ui: &mut egui::Ui, label: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(label).color(TEXT_MUTED).size(10.0).strong());
    ui.add_space(3.0);
}

pub fn collapsible_section(
    ui: &mut egui::Ui,
    label: &str,
    enabled: &mut bool,
    outer_changed: &mut bool,
    body: impl FnOnce(&mut egui::Ui, &mut bool),
) {
    ui.horizontal(|ui| {
        if ui.checkbox(enabled, RichText::new(label)
            .color(if *enabled { TEXT_BRIGHT } else { TEXT_MUTED })
            .size(10.5).strong()).changed()
        {
            *outer_changed = true;
        }
    });
    if *enabled {
        egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 4))
            .stroke(Stroke::new(1.0, BORDER))
            .inner_margin(Margin::symmetric(10.0, 8.0))
            .rounding(Rounding::same(6.0))
            .show(ui, |ui| {
                body(ui, outer_changed);
            });
    }
    ui.add_space(3.0);
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
        ui.label(RichText::new(label).color(TEXT_NORM).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
    });
}

pub fn circle_dot_interact(ui: &mut egui::Ui, color: Color32) -> (bool, bool) {
    let (r, resp) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::click());
    ui.painter().circle_filled(r.center(), 3.5, color);
    let mut add = false;
    let mut clear = false;
    
    resp.context_menu(|ui| {
        if ui.button("Add Keyframe").clicked() { add = true; ui.close_menu(); }
        if ui.button("Clear All Keyframes").clicked() { clear = true; ui.close_menu(); }
    });

    if resp.clicked() { add = true; }
    
    (add, clear)
}

pub fn align_btn(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
    let resp = ui.add(
        egui::Button::new(RichText::new(icon).size(11.5).color(TEXT_NORM))
            .fill(BG_HOVER)
            .rounding(Rounding::same(4.0))
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(egui::Vec2::new(28.0, 24.0)),
    );
    resp.on_hover_text(tooltip)
}