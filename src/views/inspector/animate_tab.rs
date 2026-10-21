use super::widgets::*;
use crate::models::types::animation::{Easing, InPreset, OutPreset};
use crate::viewmodels::editor_vm::{EditorViewModel, GraphChannel, KeyframeMode};
use crate::views::theme::{
    ACCENT_AMBER, ACCENT_CYAN, BG_BASE, BG_HOVER, BG_SELECTED, BORDER, TEXT_BRIGHT, TEXT_DIM,
    TEXT_NORM,
};
use egui::{self, Pos2, Rect, Vec2};

pub fn draw_animate(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let current_time = vm.current_time();

    section_label(ui, "KEYFRAME MODE");
    ui.horizontal(|ui| {
        let in_record = vm.keyframe_mode == KeyframeMode::Record;
        let off_color = if !in_record { TEXT_BRIGHT } else { TEXT_DIM };
        if ui
            .add(
                egui::Button::new(egui::RichText::new("● Off").color(off_color).size(11.0))
                    .fill(if !in_record {
                        BG_HOVER
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .stroke(egui::Stroke::new(1.0_f32, BORDER)),
            )
            .clicked()
        {
            vm.keyframe_mode = KeyframeMode::Off;
        }
        let rec_color = egui::Color32::from_rgb(255, 200, 30);
        let rec_fill = if in_record {
            egui::Color32::from_rgba_unmultiplied(255, 200, 30, 30)
        } else {
            egui::Color32::TRANSPARENT
        };
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("⏺ Record")
                        .color(if in_record { rec_color } else { TEXT_DIM })
                        .size(11.0),
                )
                .fill(rec_fill)
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    if in_record { rec_color } else { BORDER },
                )),
            )
            .clicked()
        {
            vm.keyframe_mode = KeyframeMode::Record;
        }
    });

    ui.add_space(6.0);

    let (local_time, is_inside, kf_data) = if let Some(sub) = vm.selected_subtitle() {
        let lt = current_time - sub.timeline_start;
        (
            lt,
            lt >= 0.0 && lt <= sub.duration(),
            sub.keyframes
                .iter()
                .map(|k| (k.id.clone(), k.time_offset))
                .collect::<Vec<_>>(),
        )
    } else if let Some(m) = vm.selected_media() {
        let lt = current_time - m.timeline_offset;
        (
            lt,
            lt >= 0.0 && lt <= m.duration,
            m.keyframes
                .iter()
                .map(|k| (k.id.clone(), k.time_offset))
                .collect::<Vec<_>>(),
        )
    } else {
        return;
    };

    ui.horizontal(|ui| {
        ui.add_enabled_ui(is_inside, |ui| {
            if ui
                .button(format!("◆ Add Keyframe  {:.2}s", local_time))
                .clicked()
            {
                vm.write_keyframe_now();
            }
        });
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let mut sorted_times: Vec<f64> = kf_data.iter().map(|x| x.1).collect();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let prev = sorted_times
            .iter()
            .filter(|&&t| t <= local_time - 0.02)
            .last()
            .copied();
        let next = sorted_times
            .iter()
            .find(|&&t| t > local_time + 0.02)
            .copied();

        if ui
            .add_enabled(prev.is_some(), egui::Button::new("⏮ Prev KF"))
            .clicked()
        {
            if let Some(t) = prev {
                let start_t = vm
                    .selected_subtitle()
                    .map(|s| s.timeline_start)
                    .unwrap_or_else(|| {
                        vm.selected_media()
                            .map(|m| m.timeline_offset)
                            .unwrap_or(0.0)
                    });
                vm.seek_to(start_t + t);
            }
        }
        if ui
            .add_enabled(next.is_some(), egui::Button::new("Next KF ⏭"))
            .clicked()
        {
            if let Some(t) = next {
                let start_t = vm
                    .selected_subtitle()
                    .map(|s| s.timeline_start)
                    .unwrap_or_else(|| {
                        vm.selected_media()
                            .map(|m| m.timeline_offset)
                            .unwrap_or(0.0)
                    });
                vm.seek_to(start_t + t);
            }
        }
    });

    ui.add_space(8.0);
    ui.add(egui::Separator::default());
    ui.add_space(4.0);

    if vm.selected_subtitle().is_some() {
        // ── IN PRESETS ────────────────────────────────────────────────────────
        section_label(ui, "IN ANIMATION (ENTRANCE)");
        ui.label(
            egui::RichText::new("In presets override each other without affecting Out animation.")
                .color(TEXT_DIM)
                .size(10.0),
        );
        ui.add_space(4.0);

        let cur_in = vm.selected_subtitle().and_then(|s| s.in_preset);
        let mut target_in_action: Option<Option<InPreset>> = None;

        ui.horizontal_wrapped(|ui| {
            let is_none = cur_in.is_none();
            let none_btn =
                egui::Button::new(egui::RichText::new("None").size(10.5).color(if is_none {
                    TEXT_BRIGHT
                } else {
                    TEXT_DIM
                }))
                .fill(if is_none { BG_SELECTED } else { BG_HOVER })
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    if is_none { ACCENT_CYAN } else { BORDER },
                ));

            if ui.add(none_btn).clicked() {
                target_in_action = Some(None);
            }

            for preset in InPreset::all() {
                let is_sel = cur_in == Some(*preset);
                let btn = egui::Button::new(
                    egui::RichText::new(preset.label())
                        .size(10.5)
                        .color(if is_sel { TEXT_BRIGHT } else { TEXT_NORM }),
                )
                .fill(if is_sel { BG_SELECTED } else { BG_HOVER })
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    if is_sel { ACCENT_CYAN } else { BORDER },
                ));

                if ui.add(btn).clicked() {
                    target_in_action = Some(if is_sel { None } else { Some(*preset) });
                }
            }
        });

        if let Some(target_in) = target_in_action {
            let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
            for sub in vm.project.subtitles.iter_mut() {
                if ids.contains(&sub.id) {
                    sub.apply_in_preset(target_in);
                }
            }
            vm.snapshot();
        }

        ui.add_space(8.0);
        ui.add(egui::Separator::default());
        ui.add_space(4.0);

        // ── OUT PRESETS ───────────────────────────────────────────────────────
        section_label(ui, "OUT ANIMATION (EXIT)");
        ui.label(
            egui::RichText::new("Out presets override each other without affecting In animation.")
                .color(TEXT_DIM)
                .size(10.0),
        );
        ui.add_space(4.0);

        let cur_out = vm.selected_subtitle().and_then(|s| s.out_preset);
        let mut target_out_action: Option<Option<OutPreset>> = None;

        ui.horizontal_wrapped(|ui| {
            let is_none = cur_out.is_none();
            let none_btn =
                egui::Button::new(egui::RichText::new("None").size(10.5).color(if is_none {
                    TEXT_BRIGHT
                } else {
                    TEXT_DIM
                }))
                .fill(if is_none { BG_SELECTED } else { BG_HOVER })
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    if is_none { ACCENT_CYAN } else { BORDER },
                ));

            if ui.add(none_btn).clicked() {
                target_out_action = Some(None);
            }

            for preset in OutPreset::all() {
                let is_sel = cur_out == Some(*preset);
                let btn = egui::Button::new(
                    egui::RichText::new(preset.label())
                        .size(10.5)
                        .color(if is_sel { TEXT_BRIGHT } else { TEXT_NORM }),
                )
                .fill(if is_sel { BG_SELECTED } else { BG_HOVER })
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    if is_sel { ACCENT_CYAN } else { BORDER },
                ));

                if ui.add(btn).clicked() {
                    target_out_action = Some(if is_sel { None } else { Some(*preset) });
                }
            }
        });

        if let Some(target_out) = target_out_action {
            let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
            for sub in vm.project.subtitles.iter_mut() {
                if ids.contains(&sub.id) {
                    sub.apply_out_preset(target_out);
                }
            }
            vm.snapshot();
        }

        ui.add_space(8.0);
        ui.add(egui::Separator::default());
        ui.add_space(4.0);
    }

    // ── KEYFRAME LIST ─────────────────────────────────────────────────────────
    section_label(ui, "KEYFRAMES");

    let kf_count = kf_data.len();
    if kf_count == 0 {
        ui.label(
            egui::RichText::new("No keyframes. Use presets or add manually.")
                .color(TEXT_DIM)
                .size(10.5),
        );
    }

    let mut remove_id: Option<String> = None;
    let opened_kf_id_slot = ui.id().with("opened_kf_card");
    let mut opened_kf_id: Option<String> = ui.data(|d| d.get_temp(opened_kf_id_slot));

    for (idx, (kf_id, kf_t)) in kf_data.iter().enumerate() {
        let kf_t = *kf_t;
        let kf_id_str = kf_id.clone();
        let is_at_playhead = (kf_t - local_time).abs() < 0.04;
        let is_selected_kf = vm.selected_keyframe_ids.contains(&kf_id_str);
        let is_highlighted = is_at_playhead || is_selected_kf;
        let is_open = opened_kf_id.as_deref() == Some(&kf_id_str);

        let kf_fill = if is_highlighted {
            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 20)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4)
        };

        egui::Frame::none()
            .fill(kf_fill)
            .stroke(egui::Stroke::new(
                1.0_f32,
                if is_highlighted { ACCENT_CYAN } else { BORDER },
            ))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(8.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let dot = if is_highlighted { "◆" } else { "◇" };
                    ui.label(
                        egui::RichText::new(format!("{} KF {} — {:.2}s", dot, idx + 1, kf_t))
                            .color(if is_highlighted {
                                ACCENT_CYAN
                            } else {
                                TEXT_NORM
                            })
                            .size(10.5),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✖").clicked() {
                            remove_id = Some(kf_id_str.clone());
                        }

                        let toggle_label = if is_open { "▲ Close" } else { "▼ Edit" };
                        if ui.small_button(toggle_label).clicked() {
                            if is_open {
                                opened_kf_id = None;
                            } else {
                                opened_kf_id = Some(kf_id_str.clone());
                            }
                            ui.data_mut(|d| d.insert_temp(opened_kf_id_slot, opened_kf_id.clone()));
                        }
                    });
                });

                if is_open {
                    let kf_vals = if let Some(sub) = vm.selected_subtitle() {
                        sub.keyframes
                            .iter()
                            .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                            .map(|k| (k.x, k.y, k.scale, k.opacity, k.rotation, k.easing.clone()))
                    } else if let Some(m) = vm.selected_media() {
                        m.keyframes
                            .iter()
                            .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                            .map(|k| (k.x, k.y, k.scale, k.opacity, k.rotation, k.easing.clone()))
                    } else {
                        None
                    };

                    if let Some((mut kx, mut ky, mut ks, mut ko, mut kr, mut ke)) = kf_vals {
                        let mut kf_changed = false;

                        ui.add_space(4.0);

                        {
                            let (mut c1, mut c2) = (false, false);
                            two_col_row(
                                ui,
                                |ui| {
                                    ui.label(egui::RichText::new("X").color(TEXT_DIM).size(10.0));
                                    c1 = ui.add(egui::DragValue::new(&mut kx).speed(0.5)).changed();
                                },
                                |ui| {
                                    ui.label(egui::RichText::new("Y").color(TEXT_DIM).size(10.0));
                                    c2 = ui.add(egui::DragValue::new(&mut ky).speed(0.5)).changed();
                                },
                            );
                            kf_changed |= c1 | c2;
                        }

                        {
                            let (mut c1, mut c2) = (false, false);
                            two_col_row(
                                ui,
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("Scale").color(TEXT_DIM).size(10.0),
                                    );
                                    c1 = ui
                                        .add(
                                            egui::DragValue::new(&mut ks)
                                                .speed(0.01)
                                                .range(0.0..=10.0),
                                        )
                                        .changed();
                                },
                                |ui| {
                                    ui.label(
                                        egui::RichText::new("Opac").color(TEXT_DIM).size(10.0),
                                    );
                                    c2 = ui
                                        .add(
                                            egui::DragValue::new(&mut ko)
                                                .speed(0.01)
                                                .range(0.0..=1.0),
                                        )
                                        .changed();
                                },
                            );
                            kf_changed |= c1 | c2;
                        }

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Rot°").color(TEXT_DIM).size(10.0));
                            if ui
                                .add(egui::DragValue::new(&mut kr).speed(0.5).suffix("°"))
                                .changed()
                            {
                                kf_changed = true;
                            }
                            ui.label(egui::RichText::new("Ease").color(TEXT_DIM).size(10.0));
                            egui::ComboBox::from_id_salt(format!("ease_{}", kf_id_str))
                                .selected_text(ke.label())
                                .width(80.0)
                                .show_ui(ui, |ui| {
                                    for e in Easing::all() {
                                        if ui
                                            .selectable_value(&mut ke, e.clone(), e.label())
                                            .changed()
                                        {
                                            kf_changed = true;
                                        }
                                    }
                                });
                        });

                        if let Easing::Custom(ref mut handles) = ke {
                            ui.add_space(6.0);
                            let bezier_changed = draw_bezier_editor(ui, handles, &kf_id_str);
                            if bezier_changed {
                                kf_changed = true;
                            }
                        }

                        if kf_changed {
                            if let Some(sub) = vm.selected_subtitle_mut() {
                                if let Some(kf) = sub
                                    .keyframes
                                    .iter_mut()
                                    .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                                {
                                    kf.x = kx;
                                    kf.y = ky;
                                    kf.scale = ks;
                                    kf.opacity = ko;
                                    kf.rotation = kr;
                                    kf.easing = ke.clone();
                                }
                            }
                            if let Some(m) = vm.selected_media_mut() {
                                if let Some(kf) = m
                                    .keyframes
                                    .iter_mut()
                                    .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                                {
                                    kf.x = kx;
                                    kf.y = ky;
                                    kf.scale = ks;
                                    kf.opacity = ko;
                                    kf.rotation = kr;
                                    kf.easing = ke.clone();
                                }
                            }
                            vm.mark_modified();
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
        if let Some(m) = vm.selected_media_mut() {
            m.keyframes.retain(|k| k.id != id);
        }
        vm.snapshot();
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);
    draw_animation_graph(ui, vm);
}

fn easing_to_handles(e: &Easing) -> [f32; 4] {
    match e {
        Easing::Linear => [0.25, 0.25, 0.75, 0.75],
        Easing::EaseIn => [0.42, 0.0, 1.0, 1.0],
        Easing::EaseOut => [0.0, 0.0, 0.58, 1.0],
        Easing::EaseInOut => [0.42, 0.0, 0.58, 1.0],
        Easing::Bounce => [0.3, 0.1, 0.7, 1.0],
        Easing::Elastic => [0.2, 0.0, 0.8, 1.2],
        Easing::Back => [0.35, -0.2, 0.65, 1.2],
        Easing::Custom(h) => *h,
    }
}

pub fn draw_animation_graph(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    if vm.selected_id.is_none() {
        let active_id = vm
            .active_subtitle()
            .filter(|s| !s.keyframes.is_empty())
            .map(|s| s.id.clone());
        if let Some(id) = active_id {
            vm.selected_id = Some(id.clone());
            vm.selected_ids.insert(id);
        }
    }

    let (has_kfs, dur, start, local_time, keyframes) = if let Some(sub) = vm.selected_subtitle() {
        (
            sub.keyframes.len() >= 2,
            sub.duration(),
            sub.timeline_start,
            vm.current_time() - sub.timeline_start,
            sub.keyframes.clone(),
        )
    } else if let Some(m) = vm.selected_media() {
        (
            m.keyframes.len() >= 2,
            m.duration,
            m.timeline_offset,
            vm.current_time() - m.timeline_offset,
            m.keyframes.clone(),
        )
    } else {
        (false, 0.0, 0.0, 0.0, vec![])
    };

    if !has_kfs || dur <= 0.0 {
        ui.label(
            egui::RichText::new("ANIMATION GRAPH")
                .color(TEXT_DIM)
                .size(10.5)
                .strong(),
        );
        ui.indent("graph_empty", |ui| {
            ui.label(
                egui::RichText::new("Requires at least 2 keyframes.")
                    .color(TEXT_DIM)
                    .size(11.0),
            );
        });
        return;
    }

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("ANIMATION GRAPH")
                .color(TEXT_DIM)
                .size(10.5)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("⌖").on_hover_text("Center on Playhead").clicked() {
                let view_w_secs = (ui.available_width() - 8.0) as f64 / vm.graph_zoom_x as f64;
                vm.graph_scroll = (local_time - view_w_secs * 0.5).max(0.0);
            }
        });
    });

    ui.add_space(2.0);

    egui::ComboBox::from_id_salt("graph_channel")
        .selected_text(match vm.graph_channel {
            GraphChannel::X => "Position X",
            GraphChannel::Y => "Position Y",
            GraphChannel::Scale => "Scale",
            GraphChannel::Rotation => "Rotation",
            GraphChannel::Opacity => "Opacity",
        })
        .width(120.0)
        .show_ui(ui, |ui| {
            let mut switched = false;
            if ui
                .selectable_value(&mut vm.graph_channel, GraphChannel::X, "Position X")
                .clicked()
            {
                switched = true;
            }
            if ui
                .selectable_value(&mut vm.graph_channel, GraphChannel::Y, "Position Y")
                .clicked()
            {
                switched = true;
            }
            if ui
                .selectable_value(&mut vm.graph_channel, GraphChannel::Scale, "Scale")
                .clicked()
            {
                switched = true;
            }
            if ui
                .selectable_value(&mut vm.graph_channel, GraphChannel::Rotation, "Rotation")
                .clicked()
            {
                switched = true;
            }
            if ui
                .selectable_value(&mut vm.graph_channel, GraphChannel::Opacity, "Opacity")
                .clicked()
            {
                switched = true;
            }

            if switched {
                match vm.graph_channel {
                    GraphChannel::X | GraphChannel::Y => {
                        vm.graph_zoom_y = 0.1;
                        vm.graph_offset_y = 0.0;
                    }
                    GraphChannel::Scale => {
                        vm.graph_zoom_y = 50.0;
                        vm.graph_offset_y = 1.0;
                    }
                    GraphChannel::Rotation => {
                        vm.graph_zoom_y = 0.5;
                        vm.graph_offset_y = 0.0;
                    }
                    GraphChannel::Opacity => {
                        vm.graph_zoom_y = 100.0;
                        vm.graph_offset_y = 0.5;
                    }
                }
            }
        });

    ui.add_space(4.0);

    let graph_size = egui::Vec2::new(ui.available_width() - 8.0, 160.0);
    let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::click_and_drag());
    let rect = resp.rect;
    let p = painter.with_clip_rect(rect);

    let view_w_secs = (rect.width() / vm.graph_zoom_x) as f64;
    let max_scroll = (dur - view_w_secs * 0.4).max(0.0);
    vm.graph_scroll = vm.graph_scroll.clamp(0.0, max_scroll);

    p.rect_filled(rect, 3.0, BG_BASE);
    p.rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, BORDER));

    // Top scrubber ruler strip
    let ruler_h = 16.0f32;
    let top_ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), ruler_h));
    p.rect_filled(top_ruler_rect, 0.0, egui::Color32::from_black_alpha(45));
    p.line_segment(
        [
            egui::pos2(rect.left(), rect.top() + ruler_h),
            egui::pos2(rect.right(), rect.top() + ruler_h),
        ],
        egui::Stroke::new(1.0_f32, BORDER),
    );

    let tick_interval = if vm.graph_zoom_x >= 200.0 {
        0.2
    } else if vm.graph_zoom_x >= 80.0 {
        0.5
    } else {
        1.0
    };
    let start_tick = (vm.graph_scroll / tick_interval).floor() as i64;
    let end_tick = ((vm.graph_scroll + view_w_secs) / tick_interval).ceil() as i64;
    for tick_i in start_tick..=end_tick {
        let t = tick_i as f64 * tick_interval;
        if t < 0.0 || t > dur {
            continue;
        }
        let tx = rect.min.x + ((t - vm.graph_scroll) as f32 * vm.graph_zoom_x);
        if tx >= rect.min.x && tx <= rect.max.x {
            p.line_segment(
                [
                    egui::pos2(tx, rect.top() + ruler_h - 4.0),
                    egui::pos2(tx, rect.top() + ruler_h),
                ],
                egui::Stroke::new(1.0_f32, TEXT_DIM.linear_multiply(0.4)),
            );
        }
    }

    for i in 1..4 {
        let y = (rect.top() + ruler_h) + (i as f32 / 4.0) * (rect.height() - ruler_h);
        p.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(10)),
        );
    }

    let get_val = |state: &crate::models::types::animation::InterpolatedState| -> f32 {
        match vm.graph_channel {
            GraphChannel::X => state.x,
            GraphChannel::Y => state.y,
            GraphChannel::Scale => state.scale,
            GraphChannel::Rotation => state.rotation,
            GraphChannel::Opacity => state.opacity,
        }
    };

    let get_kf_val = |kf: &crate::models::types::animation::Keyframe| -> f32 {
        match vm.graph_channel {
            GraphChannel::X => kf.x,
            GraphChannel::Y => kf.y,
            GraphChannel::Scale => kf.scale,
            GraphChannel::Rotation => kf.rotation,
            GraphChannel::Opacity => kf.opacity,
        }
    };

    let steps = 120usize;
    let mut prev_pt: Option<egui::Pos2> = None;
    for step in 0..=steps {
        let t_norm = step as f64 / steps as f64;
        let t = vm.graph_scroll + t_norm * view_w_secs;
        let val = if let Some(sub) = vm.selected_subtitle() {
            get_val(&sub.get_interpolated_state(
                start + t.clamp(0.0, dur),
                &vm.project.subtitles,
                0,
            ))
        } else if let Some(m) = vm.selected_media() {
            get_val(&m.get_interpolated_state(start + t.clamp(0.0, dur)))
        } else {
            Default::default()
        };

        let x = rect.min.x + (step as f32 / steps as f32) * rect.width();
        let normalized_y = (val - vm.graph_offset_y) * vm.graph_zoom_y;
        let draw_y = (rect.center().y + ruler_h * 0.5) - normalized_y;
        let clamped_y = draw_y.clamp(rect.min.y + ruler_h, rect.max.y);

        let pt = egui::Pos2::new(x, clamped_y);
        if let Some(prev) = prev_pt {
            p.line_segment([prev, pt], egui::Stroke::new(1.5_f32, ACCENT_CYAN));
        }
        prev_pt = Some(pt);
    }

    let mut sorted = keyframes;
    sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());

    let mut seek_target = None;
    let mut modified_kf_delta: Option<(f64, f32)> = None;
    let mut changed_easing_all: Option<Easing> = None;
    let mut modified_handles: Option<(String, [f32; 4])> = None;
    let mut kf_point_drag_started = false;
    let mut kf_point_clicked = false;

    // ── Bezier Handles for Selected Keyframes ─────────────────────────────────
    for sel_id in vm.selected_keyframe_ids.clone() {
        if let Some(sel_idx) = sorted.iter().position(|k| k.id == sel_id) {
            let curr_kf = &sorted[sel_idx];
            let curr_val = get_kf_val(curr_kf);
            let curr_x =
                rect.min.x + ((curr_kf.time_offset - vm.graph_scroll) as f32 * vm.graph_zoom_x);
            let curr_y = ((rect.center().y + ruler_h * 0.5)
                - (curr_val - vm.graph_offset_y) * vm.graph_zoom_y)
                .clamp(rect.min.y + ruler_h + 6.0, rect.max.y - 6.0);
            let curr_pos = egui::pos2(curr_x, curr_y);

            if sel_idx > 0 {
                let prev_kf = &sorted[sel_idx - 1];
                let prev_val = get_kf_val(prev_kf);
                let seg_dur = (curr_kf.time_offset - prev_kf.time_offset).max(0.001);
                let val_diff = curr_val - prev_val;
                let mut handles = easing_to_handles(&curr_kf.easing);

                let h2_time = prev_kf.time_offset + handles[2] as f64 * seg_dur;
                let h2_val = if val_diff.abs() > 0.001 {
                    prev_val + handles[3] * val_diff
                } else {
                    curr_val + (handles[3] - 0.75) * (40.0 / vm.graph_zoom_y.max(0.01))
                };
                let h2_x = rect.min.x + ((h2_time - vm.graph_scroll) as f32 * vm.graph_zoom_x);
                let h2_y = ((rect.center().y + ruler_h * 0.5)
                    - (h2_val - vm.graph_offset_y) * vm.graph_zoom_y)
                    .clamp(rect.min.y + ruler_h + 6.0, rect.max.y - 6.0);
                let h2_pos = egui::pos2(h2_x, h2_y);

                p.line_segment(
                    [curr_pos, h2_pos],
                    egui::Stroke::new(1.2_f32, ACCENT_AMBER.linear_multiply(0.7)),
                );

                let h2_rect = Rect::from_center_size(h2_pos, Vec2::splat(14.0));
                let h2_resp = ui.interact(
                    h2_rect,
                    ui.id().with(("h_in", &curr_kf.id)),
                    egui::Sense::drag(),
                );
                if h2_resp.drag_started() {
                    kf_point_drag_started = true;
                }

                let h_col = if h2_resp.hovered() || h2_resp.dragged() {
                    egui::Color32::WHITE
                } else {
                    ACCENT_AMBER
                };
                p.circle_filled(h2_pos, 4.0, h_col);
                p.circle_stroke(h2_pos, 5.0, egui::Stroke::new(1.0_f32, BG_BASE));

                if !vm.is_scrubbing && h2_resp.dragged() {
                    let d = h2_resp.drag_delta();
                    let dt = (d.x / vm.graph_zoom_x) as f64;
                    handles[2] = (handles[2] + (dt / seg_dur) as f32).clamp(0.0, 1.0);
                    let d_val = -d.y / vm.graph_zoom_y.max(0.01);
                    let dh = if val_diff.abs() > 0.001 {
                        d_val / val_diff
                    } else {
                        d_val * vm.graph_zoom_y.max(0.01) / 40.0
                    };
                    handles[3] = (handles[3] + dh).clamp(-0.5, 1.5);
                    modified_handles = Some((curr_kf.id.clone(), handles));
                }
            }

            if sel_idx + 1 < sorted.len() {
                let next_kf = &sorted[sel_idx + 1];
                let next_val = get_kf_val(next_kf);
                let seg_dur = (next_kf.time_offset - curr_kf.time_offset).max(0.001);
                let val_diff = next_val - curr_val;
                let mut handles = easing_to_handles(&next_kf.easing);

                let h1_time = curr_kf.time_offset + handles[0] as f64 * seg_dur;
                let h1_val = if val_diff.abs() > 0.001 {
                    curr_val + handles[1] * val_diff
                } else {
                    curr_val + (handles[1] - 0.25) * (40.0 / vm.graph_zoom_y.max(0.01))
                };
                let h1_x = rect.min.x + ((h1_time - vm.graph_scroll) as f32 * vm.graph_zoom_x);
                let h1_y = ((rect.center().y + ruler_h * 0.5)
                    - (h1_val - vm.graph_offset_y) * vm.graph_zoom_y)
                    .clamp(rect.min.y + ruler_h + 6.0, rect.max.y - 6.0);
                let h1_pos = egui::pos2(h1_x, h1_y);

                p.line_segment(
                    [curr_pos, h1_pos],
                    egui::Stroke::new(1.2_f32, ACCENT_AMBER.linear_multiply(0.7)),
                );

                let h1_rect = Rect::from_center_size(h1_pos, Vec2::splat(14.0));
                let h1_resp = ui.interact(
                    h1_rect,
                    ui.id().with(("h_out", &next_kf.id)),
                    egui::Sense::drag(),
                );
                if h1_resp.drag_started() {
                    kf_point_drag_started = true;
                }

                let h_col = if h1_resp.hovered() || h1_resp.dragged() {
                    egui::Color32::WHITE
                } else {
                    ACCENT_AMBER
                };
                p.circle_filled(h1_pos, 4.0, h_col);
                p.circle_stroke(h1_pos, 5.0, egui::Stroke::new(1.0_f32, BG_BASE));

                if !vm.is_scrubbing && h1_resp.dragged() {
                    let d = h1_resp.drag_delta();
                    let dt = (d.x / vm.graph_zoom_x) as f64;
                    handles[0] = (handles[0] + (dt / seg_dur) as f32).clamp(0.0, 1.0);
                    let d_val = -d.y / vm.graph_zoom_y.max(0.01);
                    let dh = if val_diff.abs() > 0.001 {
                        d_val / val_diff
                    } else {
                        d_val * vm.graph_zoom_y.max(0.01) / 40.0
                    };
                    handles[1] = (handles[1] + dh).clamp(-0.5, 1.5);
                    modified_handles = Some((next_kf.id.clone(), handles));
                }
            }
        }
    }

    // ── Keyframe Points ───────────────────────────────────────────────────────
    for kf in &sorted {
        let kx = rect.min.x + ((kf.time_offset - vm.graph_scroll) as f32 * vm.graph_zoom_x);
        let is_selected_kf = vm.selected_keyframe_ids.contains(&kf.id);

        let val = get_kf_val(kf);
        let normalized_y = (val - vm.graph_offset_y) * vm.graph_zoom_y;
        let ky = ((rect.center().y + ruler_h * 0.5) - normalized_y)
            .clamp(rect.min.y + ruler_h + 6.0, rect.max.y - 6.0);
        let pos = egui::Pos2::new(kx, ky);

        if kx >= rect.min.x - 14.0 && kx <= rect.max.x + 14.0 {
            let kf_rect = Rect::from_center_size(pos, Vec2::splat(14.0));
            let kf_resp = ui.interact(
                kf_rect,
                ui.id().with(("kf_graph", &kf.id)),
                egui::Sense::click_and_drag(),
            );

            if kf_resp.drag_started() {
                kf_point_drag_started = true;
            }

            if kf_resp.clicked() {
                kf_point_clicked = true;
            }

            if !vm.is_scrubbing && kf_resp.clicked() {
                if ui.input(|i| i.modifiers.shift) {
                    if is_selected_kf {
                        vm.selected_keyframe_ids.remove(&kf.id);
                    } else {
                        vm.selected_keyframe_ids.insert(kf.id.clone());
                    }
                } else {
                    vm.selected_keyframe_ids.clear();
                    vm.selected_keyframe_ids.insert(kf.id.clone());
                }
            }

            let color = if is_selected_kf {
                ACCENT_AMBER
            } else if kf_resp.hovered() && !vm.is_scrubbing {
                egui::Color32::WHITE
            } else {
                ACCENT_CYAN
            };

            p.circle_filled(pos, if is_selected_kf { 5.5 } else { 4.5 }, color);
            p.circle_stroke(
                pos,
                if is_selected_kf { 7.0 } else { 5.5 },
                egui::Stroke::new(1.0_f32, BG_BASE),
            );

            if !vm.is_scrubbing && kf_resp.dragged() {
                if !is_selected_kf {
                    vm.selected_keyframe_ids.clear();
                    vm.selected_keyframe_ids.insert(kf.id.clone());
                }
                let d = kf_resp.drag_delta();
                let dt = (d.x / vm.graph_zoom_x) as f64;
                let dy = -d.y / vm.graph_zoom_y;
                modified_kf_delta = Some((dt, dy));
            }

            if !vm.is_scrubbing && kf_resp.secondary_clicked() && !is_selected_kf {
                vm.selected_keyframe_ids.clear();
                vm.selected_keyframe_ids.insert(kf.id.clone());
            }

            kf_resp.context_menu(|ui| {
                ui.label(egui::RichText::new("Keyframe Easing").strong());
                ui.separator();
                if ui.button("Linear").clicked() {
                    changed_easing_all = Some(Easing::Linear);
                    ui.close_menu();
                }
                if ui.button("Cubic Out (Ease Out)").clicked() {
                    changed_easing_all = Some(Easing::EaseOut);
                    ui.close_menu();
                }
                if ui.button("Cubic In (Ease In)").clicked() {
                    changed_easing_all = Some(Easing::EaseIn);
                    ui.close_menu();
                }
                if ui.button("Cubic In/Out (Smooth)").clicked() {
                    changed_easing_all = Some(Easing::EaseInOut);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Bounce").clicked() {
                    changed_easing_all = Some(Easing::Bounce);
                    ui.close_menu();
                }
                if ui.button("Elastic").clicked() {
                    changed_easing_all = Some(Easing::Elastic);
                    ui.close_menu();
                }
                if ui.button("Back").clicked() {
                    changed_easing_all = Some(Easing::Back);
                    ui.close_menu();
                }
            });
        }
    }

    // ── Playhead Representation ───────────────────────────────────────────────
    let ph_x = rect.min.x + ((local_time - vm.graph_scroll) as f32 * vm.graph_zoom_x);
    if ph_x >= rect.min.x && ph_x <= rect.max.x {
        p.line_segment(
            [
                egui::Pos2::new(ph_x, rect.min.y),
                egui::Pos2::new(ph_x, rect.max.y),
            ],
            egui::Stroke::new(1.5_f32, ACCENT_AMBER),
        );
        p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(ph_x - 6.0, rect.min.y),
                egui::pos2(ph_x + 6.0, rect.min.y),
                egui::pos2(ph_x, rect.min.y + 10.0),
            ],
            ACCENT_AMBER,
            egui::Stroke::NONE,
        ));
    }

    // ── Scrubbing & Interaction Handling ──────────────────────────────────────
    let ruler_resp = ui.interact(
        top_ruler_rect,
        ui.id().with("graph_top_ruler"),
        egui::Sense::click_and_drag(),
    );
    let ph_head_rect =
        Rect::from_center_size(Pos2::new(ph_x, rect.min.y + 7.0), Vec2::new(18.0, 18.0));
    let ph_head_resp = ui.interact(
        ph_head_rect,
        ui.id().with("graph_ph_head"),
        egui::Sense::click_and_drag(),
    );

    if ruler_resp.drag_started() || ph_head_resp.drag_started() {
        vm.is_scrubbing = true;
    }

    if (resp.drag_started() || resp.clicked()) && !kf_point_drag_started && !kf_point_clicked {
        vm.is_scrubbing = true;
    }

    if ui.input(|i| i.pointer.any_released()) {
        vm.is_scrubbing = false;
    }

    if vm.is_scrubbing {
        if let Some(pos) = ui
            .ctx()
            .pointer_hover_pos()
            .or_else(|| ui.ctx().input(|i| i.pointer.interact_pos()))
        {
            let t = vm.graph_scroll + ((pos.x - rect.min.x) / vm.graph_zoom_x) as f64;
            seek_target = Some(start + t.clamp(0.0, dur));
        }
    } else if (resp.clicked() || ruler_resp.clicked()) && !kf_point_clicked {
        if let Some(pos) = ui
            .ctx()
            .pointer_hover_pos()
            .or_else(|| ui.ctx().input(|i| i.pointer.interact_pos()))
        {
            let t = vm.graph_scroll + ((pos.x - rect.min.x) / vm.graph_zoom_x) as f64;
            seek_target = Some(start + t.clamp(0.0, dur));
        }
    }

    // ── Zoom and Pan ──────────────────────────────────────────────────────────
    if resp.hovered() && !vm.is_scrubbing {
        let mut zoom_delta = 0.0;
        let mut pan_delta = 0.0;

        ui.input_mut(|i| {
            let scroll_y = if i.raw_scroll_delta.y != 0.0 {
                i.raw_scroll_delta.y
            } else {
                i.smooth_scroll_delta.y
            };
            let scroll_x = if i.raw_scroll_delta.x != 0.0 {
                i.raw_scroll_delta.x
            } else {
                i.smooth_scroll_delta.x
            };

            if i.modifiers.alt && scroll_y != 0.0 {
                zoom_delta = scroll_y;
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
                i.events
                    .retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
            } else if i.modifiers.shift && scroll_y != 0.0 {
                pan_delta = scroll_y;
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
                i.events
                    .retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
            } else if scroll_x != 0.0 {
                pan_delta = scroll_x;
                i.raw_scroll_delta.x = 0.0;
                i.smooth_scroll_delta.x = 0.0;
            }
        });

        if zoom_delta != 0.0 {
            let old_zoom = vm.graph_zoom_x;
            vm.graph_zoom_x = (vm.graph_zoom_x + zoom_delta * 0.35).clamp(10.0, 1000.0);
            if vm.graph_zoom_x != old_zoom {
                let focus_time = if vm.timeline_zoom_to_mouse {
                    if let Some(pos) = ui.ctx().pointer_hover_pos() {
                        vm.graph_scroll + ((pos.x - rect.min.x) / old_zoom) as f64
                    } else {
                        local_time
                    }
                } else {
                    local_time
                };

                let px_old = (focus_time - vm.graph_scroll) * old_zoom as f64;
                vm.graph_scroll = focus_time - (px_old / vm.graph_zoom_x as f64);
            }
        }

        if pan_delta != 0.0 {
            let pan = pan_delta as f64 / vm.graph_zoom_x as f64;
            vm.graph_scroll = (vm.graph_scroll - pan).max(0.0);
        }

        if resp.dragged_by(egui::PointerButton::Middle) {
            vm.graph_offset_y += resp.drag_delta().y / vm.graph_zoom_y;
        }
    }

    if let Some((id, handles)) = modified_handles {
        if let Some(sub) = vm.selected_subtitle_mut() {
            if let Some(kf) = sub.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = Easing::Custom(handles);
            }
        }
        if let Some(m) = vm.selected_media_mut() {
            if let Some(kf) = m.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = Easing::Custom(handles);
            }
        }
        vm.mark_modified();
    }

    if let Some((dt, dy)) = modified_kf_delta {
        let ch = vm.graph_channel;
        let selected_kfs = vm.selected_keyframe_ids.clone();

        if let Some(sub) = vm.selected_subtitle_mut() {
            for kf in sub.keyframes.iter_mut() {
                if selected_kfs.contains(&kf.id) {
                    kf.time_offset = (kf.time_offset + dt).clamp(0.0, dur);
                    match ch {
                        GraphChannel::X => kf.x += dy,
                        GraphChannel::Y => kf.y += dy,
                        GraphChannel::Scale => kf.scale += dy,
                        GraphChannel::Rotation => kf.rotation += dy,
                        GraphChannel::Opacity => kf.opacity += dy,
                    }
                }
            }
            sub.keyframes
                .sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }

        if let Some(m) = vm.selected_media_mut() {
            for kf in m.keyframes.iter_mut() {
                if selected_kfs.contains(&kf.id) {
                    kf.time_offset = (kf.time_offset + dt).clamp(0.0, dur);
                    match ch {
                        GraphChannel::X => kf.x += dy,
                        GraphChannel::Y => kf.y += dy,
                        GraphChannel::Scale => kf.scale += dy,
                        GraphChannel::Rotation => kf.rotation += dy,
                        GraphChannel::Opacity => kf.opacity += dy,
                    }
                }
            }
            m.keyframes
                .sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }
        vm.mark_modified();
    }

    if let Some(new_easing) = changed_easing_all {
        let selected_kfs = vm.selected_keyframe_ids.clone();

        if let Some(sub) = vm.selected_subtitle_mut() {
            for kf in sub.keyframes.iter_mut() {
                if selected_kfs.contains(&kf.id) {
                    kf.easing = new_easing.clone();
                }
            }
        }
        if let Some(m) = vm.selected_media_mut() {
            for kf in m.keyframes.iter_mut() {
                if selected_kfs.contains(&kf.id) {
                    kf.easing = new_easing.clone();
                }
            }
        }
        vm.mark_modified();
    }

    if let Some(t) = seek_target {
        vm.seek_to(t);
    }
}

fn draw_bezier_editor(ui: &mut egui::Ui, handles: &mut [f32; 4], kf_id: &str) -> bool {
    let mut changed = false;

    ui.label(
        egui::RichText::new("Bezier Curve Editor")
            .color(TEXT_DIM)
            .size(10.0),
    );

    let graph_size = egui::Vec2::new(ui.available_width() - 8.0, 120.0);
    let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::hover());
    let rect = resp.rect;

    painter.rect_filled(rect, 4.0, BG_BASE);
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, BORDER));

    for i in 1..4 {
        let x = rect.left() + (i as f32 / 4.0) * rect.width();
        let y = rect.top() + (i as f32 / 4.0) * rect.height();
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(12)),
        );
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(12)),
        );
    }

    painter.line_segment(
        [rect.left_bottom(), rect.right_top()],
        egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(20)),
    );

    let p0 = rect.left_bottom();
    let p3 = rect.right_top();

    let to_screen = |x: f32, y: f32| -> egui::Pos2 {
        egui::pos2(
            rect.left() + x * rect.width(),
            rect.bottom() - y * rect.height(),
        )
    };
    let from_screen = |p: egui::Pos2| -> (f32, f32) {
        (
            ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
            ((rect.bottom() - p.y) / rect.height()),
        )
    };

    let p1_screen = to_screen(handles[0], handles[1]);
    let p2_screen = to_screen(handles[2], handles[3]);

    let steps = 40;
    let mut curve_pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let x = 3.0 * u * u * t * handles[0] + 3.0 * u * t * t * handles[2] + t * t * t;
        let y = 3.0 * u * u * t * handles[1] + 3.0 * u * t * t * handles[3] + t * t * t;
        curve_pts.push(to_screen(x, y));
    }
    painter.add(egui::Shape::line(
        curve_pts,
        egui::Stroke::new(2.5_f32, ACCENT_CYAN),
    ));

    painter.line_segment([p0, p1_screen], egui::Stroke::new(1.0_f32, TEXT_DIM));
    painter.line_segment([p3, p2_screen], egui::Stroke::new(1.0_f32, TEXT_DIM));

    draw_error_bar_handle(&painter, p1_screen, ACCENT_AMBER, 6.0);
    draw_error_bar_handle(&painter, p2_screen, ACCENT_AMBER, 6.0);

    painter.circle_filled(p0, 3.5, egui::Color32::from_rgb(100, 255, 120));
    painter.circle_filled(p3, 3.5, egui::Color32::from_rgb(100, 255, 120));

    let p1_hit = egui::Rect::from_center_size(p1_screen, egui::Vec2::splat(20.0));
    let p1_resp = ui.interact(
        p1_hit,
        ui.id().with(format!("bez_p1_{}", kf_id)),
        egui::Sense::drag(),
    );

    if p1_resp.dragged() {
        let new_pos = p1_screen + p1_resp.drag_delta();
        let (hx, hy) = from_screen(new_pos);
        handles[0] = hx.clamp(0.0, 1.0);
        handles[1] = hy;
        changed = true;
    }

    if p1_resp.hovered() {
        draw_error_bar_handle(&painter, p1_screen, ACCENT_CYAN, 7.0);
    }

    let p2_hit = egui::Rect::from_center_size(p2_screen, egui::Vec2::splat(20.0));
    let p2_resp = ui.interact(
        p2_hit,
        ui.id().with(format!("bez_p2_{}", kf_id)),
        egui::Sense::drag(),
    );

    if p2_resp.dragged() {
        let new_pos = p2_screen + p2_resp.drag_delta();
        let (hx, hy) = from_screen(new_pos);
        handles[2] = hx.clamp(0.0, 1.0);
        handles[3] = hy;
        changed = true;
    }

    if p2_resp.hovered() {
        draw_error_bar_handle(&painter, p2_screen, ACCENT_CYAN, 7.0);
    }

    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.colored_label(ACCENT_CYAN, "━ curve");
        ui.add_space(8.0);
        ui.colored_label(
            TEXT_DIM,
            &format!(
                "P1({:.2},{:.2}) P2({:.2},{:.2})",
                handles[0], handles[1], handles[2], handles[3]
            ),
        );
    });

    changed
}

fn draw_error_bar_handle(
    painter: &egui::Painter,
    pos: egui::Pos2,
    color: egui::Color32,
    size: f32,
) {
    painter.line_segment(
        [
            egui::pos2(pos.x - size, pos.y),
            egui::pos2(pos.x + size, pos.y),
        ],
        egui::Stroke::new(1.5_f32, color),
    );
    for dx in [-size, size] {
        painter.line_segment(
            [
                egui::pos2(pos.x + dx, pos.y - size * 0.4),
                egui::pos2(pos.x + dx, pos.y + size * 0.4),
            ],
            egui::Stroke::new(1.5_f32, color),
        );
    }
    painter.line_segment(
        [
            egui::pos2(pos.x, pos.y - size),
            egui::pos2(pos.x, pos.y + size),
        ],
        egui::Stroke::new(1.5_f32, color),
    );
    for dy in [-size, size] {
        painter.line_segment(
            [
                egui::pos2(pos.x - size * 0.4, pos.y + dy),
                egui::pos2(pos.x + size * 0.4, pos.y + dy),
            ],
            egui::Stroke::new(1.5_f32, color),
        );
    }
    painter.circle_filled(pos, 3.0, color);
}
