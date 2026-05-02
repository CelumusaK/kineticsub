// File: src/views/inspector/animate_tab.rs
// ─────────────────────────────────────────────────────────────────────────────
use egui;
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::models::types::{AnimationPreset, Easing, LoopMode};
use crate::views::theme::{BG_HOVER, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER, BG_BASE};
use super::widgets::*;

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

    // ── LOOP BEHAVIOR ─────────────────────────────────────────────────────────
    section_label(ui, "LOOP BEHAVIOR");
    let mut loop_mode = vm.selected_subtitle().map(|s| s.loop_mode.clone()).unwrap_or_default();

    ui.horizontal(|ui| {
        let mut loop_changed = false;
        egui::ComboBox::from_id_salt("loop_mode_cmb")
            .selected_text(format!("{:?}", loop_mode))
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut loop_mode, LoopMode::None, "None").changed() { loop_changed = true; }
                if ui.selectable_value(&mut loop_mode, LoopMode::Loop, "Loop (Repeat)").changed() { loop_changed = true; }
                if ui.selectable_value(&mut loop_mode, LoopMode::PingPong, "Ping-Pong (Reverse)").changed() { loop_changed = true; }
            });
        if loop_changed {
            let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
            for sub in vm.project.subtitles.iter_mut() {
                if ids.contains(&sub.id) { sub.loop_mode = loop_mode.clone(); }
            }
            vm.mark_modified();
        }
    });

    ui.add_space(8.0);
    ui.add(egui::Separator::default());
    ui.add_space(4.0);

    // ── WORD ANIMATION (KARAOKE) ──────────────────────────────────────────────
    section_label(ui, "WORD ANIMATION (KARAOKE)");
    ui.label(egui::RichText::new("Requires transcribed words.").color(TEXT_DIM).size(10.0));

    let mut wa = vm.selected_subtitle().map(|s| s.word_animation.clone()).unwrap_or_default();
    let mut wa_changed = false;

    egui::ComboBox::from_id_salt("wa_combo")
        .selected_text(match wa {
            crate::models::types::WordAnimation::None => "None",
            crate::models::types::WordAnimation::KaraokeHighlight{..} => "Karaoke Highlight",
            crate::models::types::WordAnimation::KaraokePop{..} => "Karaoke Pop",
            crate::models::types::WordAnimation::CascadeFade => "Cascade Fade",
        })
        .show_ui(ui, |ui| {
            if ui.selectable_value(&mut wa, crate::models::types::WordAnimation::None, "None").changed() { wa_changed = true; }
            if ui.selectable_value(&mut wa, crate::models::types::WordAnimation::KaraokeHighlight{ color:[1.0, 0.8, 0.0, 1.0] }, "Karaoke Highlight").changed() { wa_changed = true; }
            if ui.selectable_value(&mut wa, crate::models::types::WordAnimation::KaraokePop{ scale: 1.2 }, "Karaoke Pop").changed() { wa_changed = true; }
            if ui.selectable_value(&mut wa, crate::models::types::WordAnimation::CascadeFade, "Cascade Fade").changed() { wa_changed = true; }
        });

    match &mut wa {
        crate::models::types::WordAnimation::KaraokeHighlight { color } => {
            if ui.color_edit_button_rgba_unmultiplied(color).changed() { wa_changed = true; }
        }
        crate::models::types::WordAnimation::KaraokePop { scale } => {
            if ui.add(egui::Slider::new(scale, 1.0..=2.0).text("Pop Scale")).changed() { wa_changed = true; }
        }
        _ => {}
    }

    if wa_changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) { sub.word_animation = wa.clone(); }
        }
        vm.mark_modified();
    }

    ui.add_space(8.0);
    ui.add(egui::Separator::default());
    ui.add_space(4.0);

    // ── ANIMATION PRESETS ─────────────────────────────────────────────────────
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
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                let kfs = preset.generate_keyframes(sub);
                sub.keyframes = kfs;
            }
        }
        vm.snapshot();
    }

    ui.add_space(8.0);
    ui.add(egui::Separator::default());
    ui.add_space(4.0);

    // ── KEYFRAME LIST ─────────────────────────────────────────────────────────
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

                        {
                            let (mut c1, mut c2) = (false, false);
                            two_col_row(ui, |ui| {
                                ui.label(egui::RichText::new("Scale").color(TEXT_DIM).size(10.0));
                                c1 = ui.add(egui::DragValue::new(&mut ks).speed(0.01).range(0.0..=10.0)).changed();
                            }, |ui| {
                                ui.label(egui::RichText::new("Opac").color(TEXT_DIM).size(10.0));
                                c2 = ui.add(egui::DragValue::new(&mut ko).speed(0.01).range(0.0..=1.0)).changed();
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

                        // ── Bezier curve editor ───────────────────────────────
                        if let Easing::Custom(ref mut handles) = ke {
                            ui.add_space(6.0);
                            let bezier_changed = draw_bezier_editor(ui, handles, &kf_id);
                            if bezier_changed { kf_changed = true; }
                        }

                        if kf_changed {
                            if let Some(sub) = vm.selected_subtitle_mut() {
                                if let Some(kf) = sub.keyframes.iter_mut()
                                    .find(|k| (k.time_offset - kf_t).abs() < 0.02)
                                {
                                    kf.x = kx; kf.y = ky; kf.scale = ks;
                                    kf.opacity = ko; kf.rotation = kr; kf.easing = ke;
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
        vm.snapshot();
    }

    ui.add_space(8.0);
    ui.add(egui::Separator::default());
    ui.add_space(4.0);

    draw_animation_graph(ui, vm, local_time);
}

// ── Bezier Curve Editor with Draggable Error-Bar Style Handles ────────────────

/// Returns true if any handle moved.
fn draw_bezier_editor(ui: &mut egui::Ui, handles: &mut [f32; 4], kf_id: &str) -> bool {
    let mut changed = false;

    ui.label(egui::RichText::new("Bezier Curve Editor").color(TEXT_DIM).size(10.0));

    let graph_size = egui::Vec2::new(ui.available_width() - 8.0, 120.0);
    let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::hover());
    let rect = resp.rect;

    painter.rect_filled(rect, 4.0, BG_BASE);
    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, BORDER));

    // Grid lines
    for i in 1..4 {
        let x = rect.left() + (i as f32 / 4.0) * rect.width();
        let y = rect.top() + (i as f32 / 4.0) * rect.height();
        painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(0.5, egui::Color32::from_white_alpha(12)));
        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5, egui::Color32::from_white_alpha(12)));
    }

    // Diagonal reference line (linear)
    painter.line_segment(
        [rect.left_bottom(), rect.right_top()],
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(20)),
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

    // ── Draw the bezier curve ─────────────────────────────────────────────────
    let steps = 40;
    let mut curve_pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let x = 3.0*u*u*t*handles[0] + 3.0*u*t*t*handles[2] + t*t*t;
        let y = 3.0*u*u*t*handles[1] + 3.0*u*t*t*handles[3] + t*t*t;
        curve_pts.push(to_screen(x, y));
    }
    painter.add(egui::Shape::line(curve_pts, egui::Stroke::new(2.5, ACCENT_CYAN)));

    // ── Handle lines from anchor to control point ─────────────────────────────
    painter.line_segment([p0, p1_screen], egui::Stroke::new(1.0, TEXT_DIM));
    painter.line_segment([p3, p2_screen], egui::Stroke::new(1.0, TEXT_DIM));

    // ── Draw "error bar" style cross handles ──────────────────────────────────
    draw_error_bar_handle(&painter, p1_screen, ACCENT_AMBER, 6.0);
    draw_error_bar_handle(&painter, p2_screen, ACCENT_AMBER, 6.0);

    // ── Anchor points ─────────────────────────────────────────────────────────
    painter.circle_filled(p0, 3.5, egui::Color32::from_rgb(100, 255, 120));
    painter.circle_filled(p3, 3.5, egui::Color32::from_rgb(100, 255, 120));

    // ── Interactive drag for P1 ───────────────────────────────────────────────
    let p1_hit = egui::Rect::from_center_size(p1_screen, egui::Vec2::splat(20.0));
    let p1_resp = ui.interact(p1_hit, ui.id().with(format!("bez_p1_{}", kf_id)), egui::Sense::drag());

    if p1_resp.dragged() {
        let new_pos = p1_screen + p1_resp.drag_delta();
        let (hx, hy) = from_screen(new_pos);
        handles[0] = hx.clamp(0.0, 1.0);
        handles[1] = hy; // Y can go outside 0..1 for overshoot
        changed = true;
    }

    if p1_resp.hovered() {
        draw_error_bar_handle(&painter, p1_screen, ACCENT_CYAN, 7.0);
    }

    // ── Interactive drag for P2 ───────────────────────────────────────────────
    let p2_hit = egui::Rect::from_center_size(p2_screen, egui::Vec2::splat(20.0));
    let p2_resp = ui.interact(p2_hit, ui.id().with(format!("bez_p2_{}", kf_id)), egui::Sense::drag());

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

    // Legend
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.colored_label(ACCENT_CYAN, "━ curve");
        ui.add_space(8.0);
        ui.colored_label(TEXT_DIM, &format!("P1({:.2},{:.2}) P2({:.2},{:.2})", handles[0], handles[1], handles[2], handles[3]));
    });

    changed
}

/// Draw an error-bar style handle: a cross with a circle center.
fn draw_error_bar_handle(painter: &egui::Painter, pos: egui::Pos2, color: egui::Color32, size: f32) {
    // Horizontal bar
    painter.line_segment(
        [egui::pos2(pos.x - size, pos.y), egui::pos2(pos.x + size, pos.y)],
        egui::Stroke::new(1.5, color),
    );
    // End caps horizontal
    for dx in [-size, size] {
        painter.line_segment(
            [egui::pos2(pos.x + dx, pos.y - size * 0.4), egui::pos2(pos.x + dx, pos.y + size * 0.4)],
            egui::Stroke::new(1.5, color),
        );
    }
    // Vertical bar
    painter.line_segment(
        [egui::pos2(pos.x, pos.y - size), egui::pos2(pos.x, pos.y + size)],
        egui::Stroke::new(1.5, color),
    );
    // End caps vertical
    for dy in [-size, size] {
        painter.line_segment(
            [egui::pos2(pos.x - size * 0.4, pos.y + dy), egui::pos2(pos.x + size * 0.4, pos.y + dy)],
            egui::Stroke::new(1.5, color),
        );
    }
    // Center dot
    painter.circle_filled(pos, 3.0, color);
}

// ── Interactive Animation Graph ───────────────────────────────────────────────

fn draw_animation_graph(ui: &mut egui::Ui, vm: &mut EditorViewModel, local_time: f64) {
    let has_kfs = vm.selected_subtitle().map(|s| s.keyframes.len() >= 2).unwrap_or(false);
    if !has_kfs { return; }

    section_label(ui, "ANIMATION GRAPH");
    ui.label(egui::RichText::new("Drag keyframe dots to adjust values • Click to seek").color(TEXT_DIM).size(9.5));
    ui.add_space(4.0);

    let graph_size = egui::Vec2::new(ui.available_width() - 8.0, 100.0);
    let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::click_and_drag());
    let rect = resp.rect;

    let mut seek_target = None;
    let _kf_drag: Option<(String, f64, f32, &'static str)> = None; // (id, time, new_val, prop)

    if let Some(sub) = vm.selected_subtitle() {
        painter.rect_filled(rect, 3.0, BG_BASE);
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, BORDER));

        // Grid
        for i in 1..4 {
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_white_alpha(10)),
            );
        }

        let dur = sub.duration();
        if dur <= 0.0 { return; }

        // ── Draw opacity and scale curves ─────────────────────────────────────
        let steps = 120usize;
        let mut prev_o: Option<egui::Pos2> = None;
        let mut prev_s: Option<egui::Pos2> = None;

        for step in 0..=steps {
            let t = dur * step as f64 / steps as f64;
            let state = sub.get_interpolated_state(sub.timeline_start + t, &vm.project.subtitles, 0);
            let x = rect.min.x + (t / dur) as f32 * rect.width();
            let o_y = rect.max.y - state.opacity.clamp(0.0, 1.0) * rect.height();
            let s_y = rect.max.y - (state.scale / 3.0).clamp(0.0, 1.0) * rect.height();
            let po = egui::Pos2::new(x, o_y);
            let ps = egui::Pos2::new(x, s_y);
            if let Some(prev) = prev_o { painter.line_segment([prev, po], egui::Stroke::new(1.5, ACCENT_CYAN.linear_multiply(0.8))); }
            if let Some(prev) = prev_s { painter.line_segment([prev, ps], egui::Stroke::new(1.5, ACCENT_AMBER.linear_multiply(0.8))); }
            prev_o = Some(po);
            prev_s = Some(ps);
        }

        // ── Draw keyframe dots with error-bar handles ─────────────────────────
        let mut sorted = sub.keyframes.clone();
        sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());

        for kf in &sorted {
            let kx = rect.min.x + (kf.time_offset / dur) as f32 * rect.width();
            let is_here = (kf.time_offset - local_time).abs() < 0.05;

            // Opacity handle
            let o_ky = rect.max.y - kf.opacity.clamp(0.0, 1.0) * rect.height();
            let o_pos = egui::Pos2::new(kx, o_ky);

            // Scale handle
            let s_ky = rect.max.y - (kf.scale / 3.0).clamp(0.0, 1.0) * rect.height();
            let s_pos = egui::Pos2::new(kx, s_ky);

            let _base_col = if is_here { ACCENT_CYAN } else { egui::Color32::from_rgb(180, 180, 200) };

            // Draw error-bar handles at keyframe positions
            draw_error_bar_handle(&painter, o_pos, ACCENT_CYAN.linear_multiply(if is_here { 1.0 } else { 0.5 }), if is_here { 5.0 } else { 3.5 });
            draw_error_bar_handle(&painter, s_pos, ACCENT_AMBER.linear_multiply(if is_here { 1.0 } else { 0.5 }), if is_here { 5.0 } else { 3.5 });

            // Vertical line connecting the two handles
            painter.line_segment([o_pos, s_pos], egui::Stroke::new(0.75, egui::Color32::from_white_alpha(30)));
        }

        // ── Playhead ──────────────────────────────────────────────────────────
        let ph_x = rect.min.x + (local_time / dur).clamp(0.0, 1.0) as f32 * rect.width();
        painter.line_segment(
            [egui::Pos2::new(ph_x, rect.min.y), egui::Pos2::new(ph_x, rect.max.y)],
            egui::Stroke::new(1.5, ACCENT_CYAN),
        );
        // Playhead triangle
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(ph_x - 4.0, rect.min.y),
                egui::pos2(ph_x + 4.0, rect.min.y),
                egui::pos2(ph_x, rect.min.y + 7.0),
            ],
            ACCENT_CYAN,
            egui::Stroke::NONE,
        ));

        // ── Scrub / seek on click/drag ────────────────────────────────────────
        if resp.dragged() || resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let frac = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0) as f64;
                seek_target = Some(sub.timeline_start + (dur * frac));
            }
        }
    }

    if let Some(t) = seek_target {
        vm.seek_to(t);
    }

    // Legend
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.colored_label(ACCENT_CYAN,  "━ Opacity");
        ui.add_space(8.0);
        ui.colored_label(ACCENT_AMBER, "━ Scale");
        ui.add_space(8.0);
        ui.colored_label(TEXT_DIM, "⊕ Keyframe handles");
    });
}