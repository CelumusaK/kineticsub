use super::widgets::*;
use crate::models::types::subtitle::MaskType;
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::views::theme::TEXT_DIM;
use egui;

pub fn draw_effects(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let mut motion_blur = match vm.selected_subtitle() {
        Some(s) => s.motion_blur,
        None => {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Effects are currently only available for subtitles.")
                        .color(TEXT_DIM)
                        .size(11.0),
                );
            });
            return;
        }
    };

    let (
        mut m_type,
        mut m_invert,
        mut mc_x,
        mut mc_y,
        mut ms_w,
        mut ms_h,
        mut m_rot,
        mut m_feather,
    ) = match vm.selected_subtitle() {
        Some(s) => (
            s.mask_type.clone(),
            s.mask_invert,
            s.mask_center[0],
            s.mask_center[1],
            s.mask_size[0],
            s.mask_size[1],
            s.mask_rotation,
            s.mask_feather,
        ),
        None => return,
    };

    let mut changed = false;
    let mut mask_kf_changed = false;

    // ── MOTION BLUR ───────────────────────────────────────────────────────────
    section_label(ui, "EFFECTS");

    ui.add_space(4.0);
    prop_row(ui, "Motion Blur", |ui| {
        if ui
            .add(egui::Slider::new(&mut motion_blur, 0.0..=1.0))
            .changed()
        {
            changed = true;
        }
    });
    ui.label(
        egui::RichText::new("Generates a trailing visual blur on moving objects.")
            .color(TEXT_DIM)
            .size(10.0),
    );

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(4.0);

    // ── MASKING ───────────────────────────────────────────────────────────────
    section_label(ui, "MASK");

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Shape").color(TEXT_DIM).size(10.5));
        egui::ComboBox::from_id_salt("mask_type_cmb")
            .selected_text(format!("{:?}", m_type))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(&mut m_type, MaskType::None, "None")
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(&mut m_type, MaskType::Straight, "Straight (Split)")
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(&mut m_type, MaskType::Rectangle, "Rectangle")
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(&mut m_type, MaskType::Circle, "Circle")
                    .changed()
                {
                    changed = true;
                }
            });
    });

    if m_type != MaskType::None {
        ui.add_space(4.0);
        if ui
            .checkbox(&mut m_invert, "Invert Mask (Hide Inside)")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);
        {
            let (mut c1, mut c2) = (false, false);
            two_col_row(
                ui,
                |ui| {
                    ui.label(egui::RichText::new("Center X").color(TEXT_DIM).size(10.5));
                    c1 = ui.add(egui::DragValue::new(&mut mc_x).speed(1.0)).changed();
                },
                |ui| {
                    ui.label(egui::RichText::new("Center Y").color(TEXT_DIM).size(10.5));
                    c2 = ui.add(egui::DragValue::new(&mut mc_y).speed(1.0)).changed();
                },
            );
            mask_kf_changed |= c1 | c2;
        }

        if m_type == MaskType::Rectangle || m_type == MaskType::Circle {
            let (mut c1, mut c2) = (false, false);
            two_col_row(
                ui,
                |ui| {
                    ui.label(
                        egui::RichText::new(if m_type == MaskType::Circle {
                            "Radius"
                        } else {
                            "Width"
                        })
                        .color(TEXT_DIM)
                        .size(10.5),
                    );
                    c1 = ui
                        .add(
                            egui::DragValue::new(&mut ms_w)
                                .speed(1.0)
                                .range(0.0..=5000.0),
                        )
                        .changed();
                },
                |ui| {
                    ui.add_enabled_ui(m_type != MaskType::Circle, |ui| {
                        ui.label(egui::RichText::new("Height").color(TEXT_DIM).size(10.5));
                        c2 = ui
                            .add(
                                egui::DragValue::new(&mut ms_h)
                                    .speed(1.0)
                                    .range(0.0..=5000.0),
                            )
                            .changed();
                    });
                },
            );
            mask_kf_changed |= c1 | c2;
        }

        if m_type == MaskType::Straight || m_type == MaskType::Rectangle {
            prop_row(ui, "Rotation", |ui| {
                if ui
                    .add(
                        egui::DragValue::new(&mut m_rot)
                            .speed(0.5)
                            .suffix("°")
                            .range(-360.0..=360.0),
                    )
                    .changed()
                {
                    mask_kf_changed = true;
                }
            });
        }

        prop_row(ui, "Feathering", |ui| {
            if ui
                .add(egui::Slider::new(&mut m_feather, 0.0..=100.0))
                .changed()
            {
                mask_kf_changed = true;
            }
        });
        ui.label(
            egui::RichText::new("Softens the edges of the mask.")
                .color(TEXT_DIM)
                .size(9.5),
        );
    }

    if changed || mask_kf_changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                sub.motion_blur = motion_blur;
                sub.mask_type = m_type.clone();
                sub.mask_invert = m_invert;
                sub.mask_center = [mc_x, mc_y];
                sub.mask_size = [ms_w, ms_h];
                sub.mask_rotation = m_rot;
                sub.mask_feather = m_feather;
            }
        }

        vm.mark_modified();

        if mask_kf_changed && vm.keyframe_mode == KeyframeMode::Record {
            vm.write_keyframe_now();
        }
    }
}
