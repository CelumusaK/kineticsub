// File: src/views/inspector/text_tab.rs
use egui;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{TEXT_DIM, TEXT_BRIGHT};
use super::widgets::*;

pub fn draw_text_props(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let vals = {
        match vm.selected_subtitle() {
            Some(s) => Some((
                s.text.clone(), s.font_size, s.bold, s.italic, s.color, s.letter_spacing,
                s.stroke_enabled, s.stroke_width, s.stroke_color,
                s.gradient_enabled, s.gradient_color,
                s.shadow_enabled, s.shadow_offset, s.shadow_blur, s.shadow_color,
                s.bg_box_enabled, s.bg_box_color, s.bg_box_padding,
            )),
            None => return,
        }
    };

    let (
        mut text, mut font_size, mut bold, mut italic, mut color, mut letter_spacing,
        mut stroke_enabled, mut stroke_width, mut stroke_color,
        mut gradient_enabled, mut gradient_color,
        mut shadow_enabled, mut shadow_offset, mut shadow_blur, mut shadow_color,
        mut bg_box_enabled, mut bg_box_color, mut bg_box_padding,
    ) = vals.unwrap();

    let mut text_changed   = false;
    let mut font_changed   = false;
    let mut style_changed  = false;
    let mut color_changed  = false;
    let mut grad_changed   = false;
    let mut stroke_changed = false;
    let mut shadow_changed = false;
    let mut bg_box_changed = false;

    // ── CONTENT ───────────────────────────────────────────────────────────────
    section_label(ui, "CONTENT");
    if ui.add(egui::TextEdit::multiline(&mut text).desired_width(f32::INFINITY).desired_rows(3).text_color(TEXT_BRIGHT)).changed() { 
        text_changed = true; 
    }

    ui.add_space(6.0);

    // ── FONT ──────────────────────────────────────────────────────────────────
    section_label(ui, "FONT");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Size").color(TEXT_DIM).size(10.5));
            c1 = ui.add(egui::DragValue::new(&mut font_size).speed(0.5).suffix("px").range(8.0..=200.0)).changed();
        }, |ui| {
            ui.label(egui::RichText::new("Spacing").color(TEXT_DIM).size(10.5));
            c2 = ui.add(egui::DragValue::new(&mut letter_spacing).speed(0.1).suffix("px").range(-10.0..=50.0)).changed();
        });
        font_changed |= c1 | c2;
    }

    ui.horizontal(|ui| {
        if ui.toggle_value(&mut bold,   egui::RichText::new(" B ").strong().size(11.0)).changed()  { style_changed = true; }
        if ui.toggle_value(&mut italic, egui::RichText::new(" I ").italics().size(11.0)).changed() { style_changed = true; }
    });

    ui.add_space(6.0);

    // ── FILL COLOR ────────────────────────────────────────────────────────────
    section_label(ui, "FILL COLOR");
    ui.horizontal(|ui| {
        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() { color_changed = true; }
        ui.label(egui::RichText::new(format!("R:{:.0} G:{:.0} B:{:.0} A:{:.0}",
            color[0]*255.0, color[1]*255.0, color[2]*255.0, color[3]*255.0))
            .color(TEXT_DIM).size(9.5));
    });

    ui.add_space(6.0);

    // ── GRADIENT ──────────────────────────────────────────────────────────────
    collapsible_section(ui, "GRADIENT", &mut gradient_enabled, &mut grad_changed, |ui, changed| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Color A").color(TEXT_DIM).size(10.5));
            let swatch = egui::Color32::from_rgba_unmultiplied(
                (color[0]*255.0) as u8, (color[1]*255.0) as u8,
                (color[2]*255.0) as u8, (color[3]*255.0) as u8,
            );
            let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(18.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, swatch);
            ui.label(egui::RichText::new("(fill)").color(TEXT_DIM).size(9.5));
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Color B").color(TEXT_DIM).size(10.5));
            if ui.color_edit_button_rgba_unmultiplied(&mut gradient_color).changed() { *changed = true; }
        });
        ui.label(egui::RichText::new("Gradient flows fill → Color B").color(TEXT_DIM).size(9.5));
    });

    ui.add_space(2.0);

    // ── STROKE ────────────────────────────────────────────────────────────────
    collapsible_section(ui, "STROKE / OUTLINE", &mut stroke_enabled, &mut stroke_changed, |ui, changed| {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Width").color(TEXT_DIM).size(10.5));
            c1 = ui.add(egui::DragValue::new(&mut stroke_width).speed(0.1).suffix("px").range(0.0..=20.0)).changed();
        }, |ui| {
            ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
            c2 = ui.color_edit_button_rgba_unmultiplied(&mut stroke_color).changed();
        });
        *changed |= c1 | c2;
    });

    ui.add_space(2.0);

    // ── SHADOW ────────────────────────────────────────────────────────────────
    collapsible_section(ui, "DROP SHADOW", &mut shadow_enabled, &mut shadow_changed, |ui, changed| {
        let mut off0 = shadow_offset[0];
        let mut off1 = shadow_offset[1];
        {
            let (mut c1, mut c2) = (false, false);
            two_col_row(ui, |ui| {
                ui.label(egui::RichText::new("Offset X").color(TEXT_DIM).size(10.5));
                c1 = ui.add(egui::DragValue::new(&mut off0).speed(0.5).suffix("px").range(-50.0..=50.0)).changed();
            }, |ui| {
                ui.label(egui::RichText::new("Offset Y").color(TEXT_DIM).size(10.5));
                c2 = ui.add(egui::DragValue::new(&mut off1).speed(0.5).suffix("px").range(-50.0..=50.0)).changed();
            });
            if c1 | c2 { shadow_offset[0] = off0; shadow_offset[1] = off1; *changed = true; }
        }
        prop_row(ui, "Blur", |ui| {
            if ui.add(egui::DragValue::new(&mut shadow_blur).speed(0.2).suffix("px").range(0.0..=40.0)).changed() { *changed = true; }
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
            if ui.color_edit_button_rgba_unmultiplied(&mut shadow_color).changed() { *changed = true; }
        });
    });

    ui.add_space(2.0);

    // ── BACKGROUND BOX ────────────────────────────────────────────────────────
    collapsible_section(ui, "BACKGROUND BOX", &mut bg_box_enabled, &mut bg_box_changed, |ui, changed| {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Padding").color(TEXT_DIM).size(10.5));
            c1 = ui.add(egui::DragValue::new(&mut bg_box_padding).speed(0.5).suffix("px").range(0.0..=60.0)).changed();
        }, |ui| {
            ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
            c2 = ui.color_edit_button_rgba_unmultiplied(&mut bg_box_color).changed();
        });
        *changed |= c1 | c2;
    });

    // ── WRITE TO ALL SELECTED ─────────────────────────────────────────────────
    if text_changed || font_changed || style_changed || color_changed || grad_changed || stroke_changed || shadow_changed || bg_box_changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if text_changed   { sub.text = text.clone(); }
                if font_changed   { sub.font_size = font_size; sub.letter_spacing = letter_spacing; }
                if style_changed  { sub.bold = bold; sub.italic = italic; }
                if color_changed  { sub.color = color; }
                if grad_changed   { sub.gradient_enabled = gradient_enabled; sub.gradient_color = gradient_color; }
                if stroke_changed { sub.stroke_enabled = stroke_enabled; sub.stroke_width = stroke_width; sub.stroke_color = stroke_color; }
                if shadow_changed { sub.shadow_enabled = shadow_enabled; sub.shadow_offset = shadow_offset; sub.shadow_blur = shadow_blur; sub.shadow_color = shadow_color; }
                if bg_box_changed { sub.bg_box_enabled = bg_box_enabled; sub.bg_box_color = bg_box_color; sub.bg_box_padding = bg_box_padding; }
            }
        }
    }
}