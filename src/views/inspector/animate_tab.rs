use egui;
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::models::types::{AnimationPreset, Easing};
use crate::views::theme::{BG_HOVER, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER, BG_BASE};
use super::widgets::*;

// ═══════════════════════════════════════════════════════════════════════════════
// ANIMATE TAB
// ═══════════════════════════════════════════════════════════════════════════════

pub fn draw_animate(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
let current_time = vm.current_time();

section_label(ui, "KEYFRAME MODE");
ui.horizontal(|ui| {
    let in_record = vm.keyframe_mode == KeyframeMode::Record;
    let off_color = if !in_record { TEXT_BRIGHT } else { TEXT_DIM };
    if ui.add(egui::Button::new(egui::RichText::new("● Off").color(off_color).size(11.0))
        .fill(if !in_record { BG_HOVER } else { egui::Color32::TRANSPARENT })
        .stroke(egui::Stroke::new(1.0, BORDER))).clicked()
    {
        vm.keyframe_mode = KeyframeMode::Off;
    }
    let rec_color = egui::Color32::from_rgb(255, 200, 30);
    let rec_fill  = if in_record { egui::Color32::from_rgba_unmultiplied(255,200,30,30) } else { egui::Color32::TRANSPARENT };
    if ui.add(egui::Button::new(egui::RichText::new("⏺ Record").color(if in_record { rec_color } else { TEXT_DIM }).size(11.0))
        .fill(rec_fill)
        .stroke(egui::Stroke::new(1.0, if in_record { rec_color } else { BORDER }))).clicked()
    {
        vm.keyframe_mode = KeyframeMode::Record;
    }
});

ui.add_space(6.0);

let (local_time, is_inside) = {
    let sub = match vm.selected_subtitle() { Some(s) => s, None => return };
    let lt = current_time - sub.timeline_start;
    (lt, lt >= 0.0 && lt <= sub.duration())
};

ui.horizontal(|ui| {
    ui.add_enabled_ui(is_inside, |ui| {
        if ui.button(format!("◆ Add Keyframe  {:.2}s", local_time)).clicked() {
            vm.write_keyframe_now();
        }
    });
});

ui.add_space(4.0);
ui.horizontal(|ui| {
    let (prev, next) = {
        let sub = match vm.selected_subtitle() { Some(s) => s, None => return };
        (sub.prev_keyframe_time(local_time), sub.next_keyframe_time(local_time))
    };
    if ui.add_enabled(prev.is_some(), egui::Button::new("⏮ Prev KF")).clicked() {
        if let Some(t) = prev {
            let sub_start = vm.selected_subtitle().unwrap().timeline_start;
            vm.seek_to(sub_start + t);
        }
    }
    if ui.add_enabled(next.is_some(), egui::Button::new("Next KF ⏭")).clicked() {
        if let Some(t) = next {
            let sub_start = vm.selected_subtitle().unwrap().timeline_start;
            vm.seek_to(sub_start + t);
        }
    }
});

ui.add_space(8.0);
ui.add(egui::Separator::default());
ui.add_space(4.0);

section_label(ui, "ANIMATION PRESETS");
ui.label(egui::RichText::new("Replaces existing keyframes").color(TEXT_DIM).size(10.0));
ui.add_space(4.0);

let preset_clicked: Option<AnimationPreset> = {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for preset in AnimationPreset::all() {
            if ui.add(egui::Button::new(egui::RichText::new(preset.label()).size(10.5).color(TEXT_NORM))
                .fill(BG_HOVER)
                .stroke(egui::Stroke::new(1.0, BORDER))).clicked()
            {
                clicked = Some(preset.clone());
            }
        }
    });
    clicked
};

if let Some(preset) = preset_clicked {
    if let Some(sub) = vm.selected_subtitle_mut() {
        let kfs = preset.generate_keyframes(sub);
        sub.keyframes = kfs;
    }
}

ui.add_space(8.0);
ui.add(egui::Separator::default());
ui.add_space(4.0);

section_label(ui, "KEYFRAMES");

let kf_count = vm.selected_subtitle().map(|s| s.keyframes.len()).unwrap_or(0);
if kf_count == 0 {
    ui.label(egui::RichText::new("No keyframes. Use presets or add manually.").color(TEXT_DIM).size(10.5));
}

let mut remove_id: Option<String> = None;
let kf_data: Vec<(String, f64)> = vm.selected_subtitle()
    .map(|s| s.keyframes.iter().map(|k| (k.id.clone(), k.time_offset)).collect())
    .unwrap_or_default();

for (idx, (kf_id, kf_t)) in kf_data.iter().enumerate() {
    let kf_t  = *kf_t;
    let kf_id = kf_id.clone();
    let is_active = (kf_t - local_time).abs() < 0.05;

    let kf_fill = if is_active {
        egui::Color32::from_rgba_unmultiplied(34, 211, 238, 18)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4)
    };

    egui::Frame::none()
        .fill(kf_fill)
        .stroke(egui::Stroke::new(1.0, if is_active { ACCENT_CYAN } else { BORDER }))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot = if is_active { "◆" } else { "◇" };
                ui.label(egui::RichText::new(format!("{} KF {} — {:.2}s", dot, idx+1, kf_t))
                    .color(if is_active { ACCENT_CYAN } else { TEXT_NORM })
                    .size(10.5));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✖").clicked() { remove_id = Some(kf_id.clone()); }
                    if ui.small_button("→").clicked() {
                        let sub_start = vm.selected_subtitle().unwrap().timeline_start;
                        vm.seek_to(sub_start + kf_t);
                    }
                });
            });

            if is_active {
                let kf_vals = vm.selected_subtitle()
                    .and_then(|s| s.keyframes.iter().find(|k| (k.time_offset - kf_t).abs() < 0.02))
                    .map(|k| (k.x, k.y, k.scale, k.opacity, k.rotation, k.easing.clone()));

                if let Some((mut kx, mut ky, mut ks, mut ko, mut kr, mut ke)) = kf_vals {
                    let mut kf_changed = false;

                    ui.add_space(4.0);

                    // XY row
                    {
                        let (mut c1, mut c2) = (false, false);
                        two_col_row(ui, |ui| {
                            ui.label(egui::RichText::new("X").color(TEXT_DIM).size(10.0));
                            c1 = ui.add(egui::DragValue::new(&mut kx).speed(0.5)).changed();
                        }, |ui| {
                            ui.label(egui::RichText::new("Y").color(TEXT_DIM).size(10.0));
                            c2 = ui.add(egui::DragValue::new(&mut ky).speed(0.5)).changed();
                        });
                        kf_changed |= c1 | c2;
                    }

                    // Scale / Opacity row
                    {
                        let (mut c1, mut c2) = (false, false);
                        two_col_row(ui, |ui| {
                            ui.label(egui::RichText::new("Scale").color(TEXT_DIM).size(10.0));
                            c1 = ui.add(egui::DragValue::new(&mut ks)
                                .speed(0.01).range(0.0..=10.0)).changed();
                        }, |ui| {
                            ui.label(egui::RichText::new("Opac").color(TEXT_DIM).size(10.0));
                            c2 = ui.add(egui::DragValue::new(&mut ko)
                                .speed(0.01).range(0.0..=1.0)).changed();
                        });
                        kf_changed |= c1 | c2;
                    }

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Rot°").color(TEXT_DIM).size(10.0));
                        if ui.add(egui::DragValue::new(&mut kr).speed(0.5).suffix("°")).changed() { kf_changed = true; }
                        ui.label(egui::RichText::new("Ease").color(TEXT_DIM).size(10.0));
                        egui::ComboBox::from_id_salt(format!("ease_{}", kf_id))
                            .selected_text(ke.label())
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                for e in Easing::all() {
                                    if ui.selectable_value(&mut ke, e.clone(), e.label()).changed() { kf_changed = true; }
                                }
                            });
                    });

                    if kf_changed {
                        if let Some(sub) = vm.selected_subtitle_mut() {
                            if let Some(kf) = sub.keyframes.iter_mut()
                                .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                            {
                                kf.x = kx; kf.y = ky; kf.scale = ks;
                                kf.opacity = ko; kf.rotation = kr; kf.easing = ke;
                            }
                        }
                    }
                }
            }
        });

    ui.add_space(3.0);
}

if let Some(id) = remove_id {
    if let Some(sub) = vm.selected_subtitle_mut() {
        sub.keyframes.retain(|k| k.id != id);
    }
}

ui.add_space(8.0);
ui.add(egui::Separator::default());
ui.add_space(4.0);
draw_animation_graph(ui, vm, local_time);

}

// ── Mini animation graph ──────────────────────────────────────────────────────

fn draw_animation_graph(ui: &mut egui::Ui, vm: &EditorViewModel, local_time: f64) {
let sub = match vm.selected_subtitle() { Some(s) => s, None => return };
if sub.keyframes.len() < 2 { return; }

section_label(ui, "ANIMATION GRAPH");
let graph_size = egui::Vec2::new(ui.available_width() - 8.0, 80.0);
let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::hover());
let rect = resp.rect;

painter.rect_filled(rect, 3.0, BG_BASE);
painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, BORDER));

let dur = sub.duration();
if dur <= 0.0 { return; }

let steps = 120usize;
let mut prev_o: Option<egui::Pos2> = None;
let mut prev_s: Option<egui::Pos2> = None;

for step in 0..=steps {
    let t = dur * step as f64 / steps as f64;
    let state = sub.get_interpolated_state(sub.timeline_start + t);
    let x   = rect.min.x + (t / dur) as f32 * rect.width();
    let o_y = rect.max.y - state.opacity * rect.height();
    let s_y = rect.max.y - (state.scale / 3.0).clamp(0.0, 1.0) * rect.height();
    let po  = egui::Pos2::new(x, o_y);
    let ps  = egui::Pos2::new(x, s_y);
    if let Some(prev) = prev_o { painter.line_segment([prev, po], egui::Stroke::new(1.5, ACCENT_CYAN.linear_multiply(0.8))); }
    if let Some(prev) = prev_s { painter.line_segment([prev, ps], egui::Stroke::new(1.5, ACCENT_AMBER.linear_multiply(0.8))); }
    prev_o = Some(po);
    prev_s = Some(ps);
}

let mut sorted = sub.keyframes.clone();
sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
for kf in &sorted {
    let kx = rect.min.x + (kf.time_offset / dur) as f32 * rect.width();
    let is_here = (kf.time_offset - local_time).abs() < 0.05;
    let col = if is_here { ACCENT_CYAN } else { egui::Color32::from_rgb(200, 200, 200) };
    painter.circle_filled(egui::Pos2::new(kx, rect.center().y), if is_here { 4.0 } else { 3.0 }, col);
}

let ph_x = rect.min.x + (local_time / dur).clamp(0.0, 1.0) as f32 * rect.width();
painter.line_segment(
    [egui::Pos2::new(ph_x, rect.min.y), egui::Pos2::new(ph_x, rect.max.y)],
    egui::Stroke::new(1.0, ACCENT_CYAN),
);

ui.horizontal(|ui| {
    ui.add_space(4.0);
    ui.colored_label(ACCENT_CYAN,  "━ Opacity");
    ui.add_space(8.0);
    ui.colored_label(ACCENT_AMBER, "━ Scale");
});

}