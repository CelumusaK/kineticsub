use super::widgets::*;
use crate::models::types::subtitle::{StrokeProps, TextAlign};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{TEXT_BRIGHT, TEXT_DIM};
use egui;

pub fn draw_text_props(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let vals = {
        match vm.selected_subtitle() {
            Some(s) => Some((
                s.text.clone(),
                s.font_family.clone(),
                s.font_size,
                s.bold,
                s.italic,
                s.color,
                s.letter_spacing,
                s.stroke_enabled,
                s.stroke_width,
                s.stroke_color,
                s.gradient_enabled,
                s.gradient_color,
                s.shadow_enabled,
                s.shadow_offset,
                s.shadow_blur,
                s.shadow_color,
                s.bg_box_enabled,
                s.bg_box_color,
                s.bg_box_padding,
                s.bg_box_radius,
                s.additional_strokes_enabled,
                s.additional_strokes.clone(),
                s.bloom.clone(),
                s.text_align.clone(),
            )),
            None => {
                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Text properties are only available for subtitles.")
                            .color(TEXT_DIM)
                            .size(11.0),
                    );
                });
                return;
            }
        }
    };

    let (
        mut text,
        mut font_family,
        mut font_size,
        mut bold,
        mut italic,
        mut color,
        mut letter_spacing,
        mut stroke_enabled,
        mut stroke_width,
        mut stroke_color,
        mut gradient_enabled,
        mut gradient_color,
        mut shadow_enabled,
        mut shadow_offset,
        mut shadow_blur,
        mut shadow_color,
        mut bg_box_enabled,
        mut bg_box_color,
        mut bg_box_padding,
        mut bg_box_radius,
        mut additional_strokes_enabled,
        mut additional_strokes,
        mut bloom,
        mut text_align,
    ) = vals.unwrap();

    let mut text_changed = false;
    let mut font_changed = false;
    let mut style_changed = false;
    let mut color_changed = false;
    let mut grad_changed = false;
    let mut stroke_changed = false;
    let mut shadow_changed = false;
    let mut bg_box_changed = false;
    let mut extra_stroke_changed = false;
    let mut bloom_changed = false;
    let mut align_changed = false;

    // ── CONTENT ───────────────────────────────────────────────────────────────
    section_label(ui, "CONTENT");
    if ui
        .add(
            egui::TextEdit::multiline(&mut text)
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .text_color(TEXT_BRIGHT),
        )
        .changed()
    {
        text_changed = true;
    }

    // ── FONT & STYLE ──────────────────────────────────────────────────────────
    section_label(ui, "FONT & STYLE");

    let available_fonts = crate::views::canvas::subtitles::get_available_fonts();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Font").color(TEXT_DIM).size(10.5));

        egui::ComboBox::from_id_salt("font_family_combo")
            .selected_text(&font_family)
            .width(130.0)
            .show_ui(ui, |ui| {
                for f in &available_fonts {
                    if ui
                        .selectable_value(&mut font_family, f.to_string(), f)
                        .changed()
                    {
                        crate::views::canvas::subtitles::try_load_system_font(
                            ui.ctx(),
                            &font_family,
                            bold,
                        );
                        font_changed = true;
                    }
                }
            });

        if ui
            .button("+ Add Font")
            .on_hover_text("Load custom font file (.ttf / .otf)")
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Font Files", &["ttf", "otf", "woff", "woff2"])
                .pick_file()
            {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let stem_str = stem.to_string();
                    if let Ok(bytes) = std::fs::read(&path) {
                        crate::views::canvas::subtitles::register_font(ui.ctx(), &stem_str, bytes);
                    }
                    font_family = stem_str;
                    font_changed = true;
                }
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Name").color(TEXT_DIM).size(10.5));
        if ui
            .add(
                egui::TextEdit::singleline(&mut font_family)
                    .hint_text("Custom font name...")
                    .desired_width(ui.available_width() - 8.0),
            )
            .changed()
        {
            crate::views::canvas::subtitles::try_load_system_font(ui.ctx(), &font_family, bold);
            font_changed = true;
        }
    });

    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(
            ui,
            |ui| {
                ui.label(egui::RichText::new("Size").color(TEXT_DIM).size(10.5));
                if ui
                    .add(
                        egui::DragValue::new(&mut font_size)
                            .speed(0.5)
                            .suffix("px")
                            .range(8.0..=200.0),
                    )
                    .changed()
                {
                    c1 = true;
                }
            },
            |ui| {
                ui.label(egui::RichText::new("Spacing").color(TEXT_DIM).size(10.5));
                if ui
                    .add(
                        egui::DragValue::new(&mut letter_spacing)
                            .speed(0.1)
                            .suffix("px")
                            .range(-10.0..=50.0),
                    )
                    .changed()
                {
                    c2 = true;
                }
            },
        );
        font_changed |= c1 | c2;
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Style").color(TEXT_DIM).size(10.5));
        if ui
            .toggle_value(&mut bold, egui::RichText::new(" B ").strong().size(11.0))
            .changed()
        {
            if bold {
                crate::views::canvas::subtitles::try_load_system_font(ui.ctx(), &font_family, true);
            }
            style_changed = true;
        }
        if ui
            .toggle_value(&mut italic, egui::RichText::new(" I ").italics().size(11.0))
            .changed()
        {
            style_changed = true;
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Align").color(TEXT_DIM).size(10.5));
        if ui
            .selectable_value(&mut text_align, TextAlign::Left, "Left")
            .changed()
        {
            align_changed = true;
        }
        if ui
            .selectable_value(&mut text_align, TextAlign::Center, "Center")
            .changed()
        {
            align_changed = true;
        }
        if ui
            .selectable_value(&mut text_align, TextAlign::Right, "Right")
            .changed()
        {
            align_changed = true;
        }
    });

    // ── COLOR ─────────────────────────────────────────────────────────────────
    section_label(ui, "COLOR");
    ui.horizontal(|ui| {
        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
            color_changed = true;
        }
        ui.label(
            egui::RichText::new(format!(
                "R:{:.0} G:{:.0} B:{:.0} A:{:.0}",
                color[0] * 255.0,
                color[1] * 255.0,
                color[2] * 255.0,
                color[3] * 255.0
            ))
            .color(TEXT_DIM)
            .size(9.5),
        );
    });

    // ── GRADIENT ──────────────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "GRADIENT",
        &mut gradient_enabled,
        &mut grad_changed,
        |ui, changed| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Color A").color(TEXT_DIM).size(10.5));
                let swatch = egui::Color32::from_rgba_unmultiplied(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (color[3] * 255.0) as u8,
                );
                let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(18.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, swatch);
                ui.label(egui::RichText::new("(fill)").color(TEXT_DIM).size(9.5));
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Color B").color(TEXT_DIM).size(10.5));
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut gradient_color)
                    .changed()
                {
                    *changed = true;
                }
            });
            ui.label(
                egui::RichText::new("Gradient flows fill → Color B")
                    .color(TEXT_DIM)
                    .size(9.5),
            );
        },
    );

    // ── STROKE ────────────────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "PRIMARY STROKE",
        &mut stroke_enabled,
        &mut stroke_changed,
        |ui, changed| {
            let (mut c1, mut c2) = (false, false);
            two_col_row(
                ui,
                |ui| {
                    ui.label(egui::RichText::new("Width").color(TEXT_DIM).size(10.5));
                    if ui
                        .add(
                            egui::DragValue::new(&mut stroke_width)
                                .speed(0.1)
                                .suffix("px")
                                .range(0.0..=50.0),
                        )
                        .changed()
                    {
                        c1 = true;
                    }
                },
                |ui| {
                    ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
                    if ui
                        .color_edit_button_rgba_unmultiplied(&mut stroke_color)
                        .changed()
                    {
                        c2 = true;
                    }
                },
            );
            *changed |= c1 | c2;
        },
    );

    // ── MULTIPLE STROKES ──────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "MULTIPLE STROKES",
        &mut additional_strokes_enabled,
        &mut extra_stroke_changed,
        |ui, changed| {
            if ui.button("+ Add Stroke Layer").clicked() {
                additional_strokes.push(StrokeProps::default());
                *changed = true;
            }
            ui.add_space(4.0);
            let mut to_remove = None;
            for (i, stroke) in additional_strokes.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("S{}", i + 2))
                            .color(TEXT_DIM)
                            .size(10.0),
                    );
                    if ui
                        .add(
                            egui::DragValue::new(&mut stroke.width)
                                .speed(0.5)
                                .suffix("px"),
                        )
                        .changed()
                    {
                        *changed = true;
                    }
                    if ui
                        .color_edit_button_rgba_unmultiplied(&mut stroke.color)
                        .changed()
                    {
                        *changed = true;
                    }
                    if ui.button("✖").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(idx) = to_remove {
                additional_strokes.remove(idx);
                *changed = true;
            }
        },
    );

    // ── SHADOW ────────────────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "DROP SHADOW",
        &mut shadow_enabled,
        &mut shadow_changed,
        |ui, changed| {
            let mut off0 = shadow_offset[0];
            let mut off1 = shadow_offset[1];
            {
                let (mut c1, mut c2) = (false, false);
                two_col_row(
                    ui,
                    |ui| {
                        ui.label(egui::RichText::new("Offset X").color(TEXT_DIM).size(10.5));
                        if ui
                            .add(
                                egui::DragValue::new(&mut off0)
                                    .speed(0.5)
                                    .suffix("px")
                                    .range(-50.0..=50.0),
                            )
                            .changed()
                        {
                            c1 = true;
                        }
                    },
                    |ui| {
                        ui.label(egui::RichText::new("Offset Y").color(TEXT_DIM).size(10.5));
                        if ui
                            .add(
                                egui::DragValue::new(&mut off1)
                                    .speed(0.5)
                                    .suffix("px")
                                    .range(-50.0..=50.0),
                            )
                            .changed()
                        {
                            c2 = true;
                        }
                    },
                );
                if c1 | c2 {
                    shadow_offset[0] = off0;
                    shadow_offset[1] = off1;
                    *changed = true;
                }
            }
            prop_row(ui, "Blur", |ui| {
                if ui
                    .add(
                        egui::DragValue::new(&mut shadow_blur)
                            .speed(0.2)
                            .suffix("px")
                            .range(0.0..=40.0),
                    )
                    .changed()
                {
                    *changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut shadow_color)
                    .changed()
                {
                    *changed = true;
                }
            });
        },
    );

    // ── GLOW & BLOOM ──────────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "GLOW & BLOOM ENGINE",
        &mut bloom.enabled,
        &mut bloom_changed,
        |ui, changed| {
            prop_row(ui, "Intensity", |ui| {
                if ui
                    .add(egui::Slider::new(&mut bloom.intensity, 0.0..=5.0))
                    .changed()
                {
                    *changed = true;
                }
            });
            prop_row(ui, "Radius", |ui| {
                if ui
                    .add(egui::Slider::new(&mut bloom.radius, 0.0..=200.0))
                    .changed()
                {
                    *changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
                if ui
                    .color_edit_button_rgba_unmultiplied(&mut bloom.color)
                    .changed()
                {
                    *changed = true;
                }
            });
        },
    );

    // ── BACKGROUND BOX ────────────────────────────────────────────────────────
    collapsible_section(
        ui,
        "BACKGROUND BOX",
        &mut bg_box_enabled,
        &mut bg_box_changed,
        |ui, changed| {
            let (mut c1, mut c2) = (false, false);
            two_col_row(
                ui,
                |ui| {
                    ui.label(egui::RichText::new("Padding").color(TEXT_DIM).size(10.5));
                    if ui
                        .add(
                            egui::DragValue::new(&mut bg_box_padding)
                                .speed(0.5)
                                .suffix("px")
                                .range(0.0..=60.0),
                        )
                        .changed()
                    {
                        c1 = true;
                    }
                },
                |ui| {
                    ui.label(egui::RichText::new("Color").color(TEXT_DIM).size(10.5));
                    if ui
                        .color_edit_button_rgba_unmultiplied(&mut bg_box_color)
                        .changed()
                    {
                        c2 = true;
                    }
                },
            );
            *changed |= c1 | c2;

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Corner Radiuses (TL, TR, BR, BL)")
                    .color(TEXT_DIM)
                    .size(10.0),
            );
            ui.horizontal(|ui| {
                let r1 = ui
                    .add(
                        egui::DragValue::new(&mut bg_box_radius[0])
                            .speed(1.0)
                            .range(0.0..=100.0),
                    )
                    .changed();
                let r2 = ui
                    .add(
                        egui::DragValue::new(&mut bg_box_radius[1])
                            .speed(1.0)
                            .range(0.0..=100.0),
                    )
                    .changed();
                let r3 = ui
                    .add(
                        egui::DragValue::new(&mut bg_box_radius[2])
                            .speed(1.0)
                            .range(0.0..=100.0),
                    )
                    .changed();
                let r4 = ui
                    .add(
                        egui::DragValue::new(&mut bg_box_radius[3])
                            .speed(1.0)
                            .range(0.0..=100.0),
                    )
                    .changed();
                *changed |= r1 | r2 | r3 | r4;
            });
        },
    );

    // ── WRITE TO ALL SELECTED ─────────────────────────────────────────────────
    if text_changed
        || font_changed
        || style_changed
        || color_changed
        || grad_changed
        || stroke_changed
        || shadow_changed
        || bg_box_changed
        || extra_stroke_changed
        || bloom_changed
        || align_changed
    {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if text_changed {
                    sub.text = text.clone();
                    sub.sync_words_text();
                }
                if font_changed {
                    sub.font_family = font_family.clone();
                    sub.font_size = font_size;
                    sub.letter_spacing = letter_spacing;
                }
                if style_changed {
                    sub.bold = bold;
                    sub.italic = italic;
                }
                if color_changed {
                    sub.color = color;
                }
                if grad_changed {
                    sub.gradient_enabled = gradient_enabled;
                    sub.gradient_color = gradient_color;
                }
                if stroke_changed {
                    sub.stroke_enabled = stroke_enabled;
                    sub.stroke_width = stroke_width;
                    sub.stroke_color = stroke_color;
                }
                if shadow_changed {
                    sub.shadow_enabled = shadow_enabled;
                    sub.shadow_offset = shadow_offset;
                    sub.shadow_blur = shadow_blur;
                    sub.shadow_color = shadow_color;
                }
                if bg_box_changed {
                    sub.bg_box_enabled = bg_box_enabled;
                    sub.bg_box_color = bg_box_color;
                    sub.bg_box_padding = bg_box_padding;
                    sub.bg_box_radius = bg_box_radius;
                }
                if extra_stroke_changed {
                    sub.additional_strokes_enabled = additional_strokes_enabled;
                    sub.additional_strokes = additional_strokes.clone();
                }
                if bloom_changed {
                    sub.bloom = bloom.clone();
                }
                if align_changed {
                    sub.text_align = text_align.clone();
                }
            }
        }
        vm.mark_modified();
    }
}
