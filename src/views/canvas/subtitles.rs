use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::subtitle::{WordAnimation, TextDeform, MaskType};
use crate::views::theme::{ACCENT_CYAN};

pub fn draw(ctx: &Context, canvas_painter: &egui::Painter, preview_rect: Rect, scale_factor: f32, vm: &mut EditorViewModel) {
    if let Some(sub) = vm.active_subtitle() {
        let max_steps = if sub.motion_blur > 0.0 { 3 } else { 0 };
        
        for step in (0..=max_steps).rev() {
            let offset_t = vm.current_time() - (step as f64 * 0.02 * sub.motion_blur as f64);
            if offset_t < sub.timeline_start && step > 0 { continue; }
            
            let state = sub.get_interpolated_state(offset_t, &vm.project.subtitles, 0);
            
            let opacity_mult = if step > 0 { 0.25 } else { 1.0 };
            let final_opacity = state.opacity * opacity_mult;
            
            if final_opacity <= 0.01 { continue; }

            let clip_rects = get_mask_clip_rects(sub, &state, preview_rect, scale_factor);
            if clip_rects.is_empty() { continue; }

            let font_size = (sub.font_size * state.scale * scale_factor).max(0.1);

            let (px, py, pang) = sub.evaluate_path(state.path_progress);
            let center_x = preview_rect.center().x + (state.x + px) * scale_factor;
            let center_y = preview_rect.center().y + (state.y + py) * scale_factor;
            
            let total_rot = state.rotation + if sub.path_orient { pang.to_degrees() } else { 0.0 };
            let rot_rad = total_rot * std::f32::consts::PI / 180.0;

            let text_color = {
                let c = sub.color;
                egui::Color32::from_rgba_unmultiplied(
                    (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, (c[3] * final_opacity * 255.0) as u8,
                )
            };

            let has_words = !sub.words.is_empty() && sub.words.len() == sub.text.split_whitespace().count();
            let path_is_active = sub.path_type != crate::models::types::subtitle::PathType::None;

            let current_t = vm.current_time();
            let text_deform = sub.text_deform.clone();
            let deform_amt = sub.text_deform_amount;
            let skew_x = state.skew_x;
            let skew_y = state.skew_y;
            let pitch = state.pitch.to_radians();
            let yaw = state.yaw.to_radians();

            // apply_transform: takes a local offset (dx, dy) from the text center
            // and applies deform + 3D + rotation to produce a screen position.
            let apply_transform = |mut dx: f32, mut dy: f32| -> Pos2 {
                match text_deform {
                    TextDeform::Wave => {
                        dy += (dx * 0.02 + current_t as f32 * 5.0).sin() * deform_amt;
                    }
                    TextDeform::Arc => {
                        let arc_radius = 500.0 / deform_amt.abs().max(0.01);
                        if deform_amt != 0.0 {
                            dy -= (dx / arc_radius).cos() * deform_amt * 2.0;
                        }
                    }
                    TextDeform::Bulge => {
                        let dist = dx.abs();
                        dy -= (1.0 - (dist / 200.0).clamp(0.0, 1.0)) * deform_amt;
                    }
                    TextDeform::Flag => {
                        dy += (dx * 0.01).sin() * deform_amt;
                        dx += (dy * 0.01).cos() * deform_amt;
                    }
                    _ => {}
                }
                let sx = dx + dy * skew_x;
                let sy = dy + dx * skew_y;
                let py_val = sy * pitch.cos();
                let pz = sy * pitch.sin();
                let yx = sx * yaw.cos() + pz * yaw.sin();
                let yz = -sx * yaw.sin() + pz * yaw.cos();
                let rx = yx * rot_rad.cos() - py_val * rot_rad.sin();
                let ry = yx * rot_rad.sin() + py_val * rot_rad.cos();
                let dist = 800.0;
                let z_factor = dist / (dist + yz).max(1.0);
                Pos2::new(center_x + rx * z_factor, center_y + ry * z_factor)
            };

            // Determine if we need per-character deformation
            let needs_char_deform = text_deform != TextDeform::None && deform_amt != 0.0;

            let mut final_text_size = Vec2::ZERO;

            for clip_rect in &clip_rects {
                let mut active_painter = canvas_painter.clone();
                active_painter.set_clip_rect(clip_rect.intersect(canvas_painter.clip_rect()));

                if path_is_active {
                    draw_text_on_path(
                        ctx, &active_painter, sub, &state, preview_rect, scale_factor,
                        font_size, text_color, final_opacity, current_t,
                        has_words,
                    );
                } else {
                    // Build a galley to measure text size
                    let create_job = |override_color: Option<egui::Color32>| -> std::sync::Arc<egui::Galley> {
                        build_layout_job(ctx, sub, has_words, font_size, text_color, final_opacity, override_color, current_t)
                    };

                    let galley = create_job(None);
                    let text_size = galley.size();
                    final_text_size = text_size;

                    // Helper to draw text — uses per-character deform if active, otherwise single-galley
                    let draw_text_with_deform = |painter: &egui::Painter,
                                                  g: std::sync::Arc<egui::Galley>,
                                                  offset: Vec2,
                                                  deform: bool| {
                        if deform {
                            draw_deformed_text(
                                ctx, painter, sub, has_words, font_size,
                                g.size(), offset, &apply_transform,
                                text_color, final_opacity, current_t,
                            );
                        } else {
                            let dx = offset.x - g.size().x * 0.5;
                            let dy = offset.y - g.size().y * 0.5;
                            let pos = apply_transform(dx, dy);
                            let shape = egui::epaint::TextShape {
                                pos,
                                galley: g,
                                underline: Stroke::NONE,
                                override_text_color: None,
                                angle: rot_rad,
                                fallback_color: egui::Color32::WHITE,
                                opacity_factor: 1.0,
                            };
                            painter.add(shape);
                        }
                    };

                    // For effects that need a specific override color, we can't easily do per-char,
                    // so we use the single-galley path for those passes (shadow, strokes, bloom).
                    let draw_projected_text = |painter: &egui::Painter, g: std::sync::Arc<egui::Galley>, offset: Vec2| {
                        let dx = offset.x - g.size().x * 0.5;
                        let dy = offset.y - g.size().y * 0.5;
                        let pos = apply_transform(dx, dy);
                        let shape = egui::epaint::TextShape {
                            pos, galley: g, underline: Stroke::NONE, override_text_color: None,
                            angle: rot_rad, fallback_color: egui::Color32::WHITE, opacity_factor: 1.0,
                        };
                        painter.add(shape);
                    };

                    // Background box
                    if sub.bg_box_enabled {
                        let pad = sub.bg_box_padding * scale_factor;
                        let w = text_size.x + pad * 2.0;
                        let h = text_size.y + pad * 2.0;
                        let r = sub.bg_box_radius;
                        let mut raw_pts = Vec::new();
                        let segments = 6;
                        let r_tl = (r[0] * scale_factor).min(w/2.0).min(h/2.0);
                        if r_tl > 0.0 { for i in 0..=segments { let a = std::f32::consts::PI + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2; raw_pts.push(Vec2::new(-w/2.0 + r_tl + r_tl * a.cos(), -h/2.0 + r_tl + r_tl * a.sin())); } } else { raw_pts.push(Vec2::new(-w/2.0, -h/2.0)); }
                        let r_tr = (r[1] * scale_factor).min(w/2.0).min(h/2.0);
                        if r_tr > 0.0 { for i in 0..=segments { let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2; raw_pts.push(Vec2::new(w/2.0 - r_tr + r_tr * a.cos(), -h/2.0 + r_tr + r_tr * a.sin())); } } else { raw_pts.push(Vec2::new(w/2.0, -h/2.0)); }
                        let r_br = (r[2] * scale_factor).min(w/2.0).min(h/2.0);
                        if r_br > 0.0 { for i in 0..=segments { let a = 0.0 + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2; raw_pts.push(Vec2::new(w/2.0 - r_br + r_br * a.cos(), h/2.0 - r_br + r_br * a.sin())); } } else { raw_pts.push(Vec2::new(w/2.0, h/2.0)); }
                        let r_bl = (r[3] * scale_factor).min(w/2.0).min(h/2.0);
                        if r_bl > 0.0 { for i in 0..=segments { let a = std::f32::consts::FRAC_PI_2 + (i as f32 / segments as f32) * std::f32::consts::FRAC_PI_2; raw_pts.push(Vec2::new(-w/2.0 + r_bl + r_bl * a.cos(), h/2.0 - r_bl + r_bl * a.sin())); } } else { raw_pts.push(Vec2::new(-w/2.0, h/2.0)); }
                        let pts = raw_pts.iter().map(|v| apply_transform(v.x, v.y)).collect::<Vec<_>>();
                        let bc = sub.bg_box_color;
                        let box_color = egui::Color32::from_rgba_unmultiplied((bc[0] * 255.0) as u8, (bc[1] * 255.0) as u8, (bc[2] * 255.0) as u8, (bc[3] * final_opacity * 255.0) as u8);
                        active_painter.add(egui::Shape::convex_polygon(pts, box_color, Stroke::new(4.0 * scale_factor, box_color)));
                    }

                    // Bloom passes (use single-galley — bloom is a blur effect)
                    if sub.bloom.enabled {
                        let b_col = egui::Color32::from_rgba_unmultiplied(
                            (sub.bloom.color[0]*255.) as u8, (sub.bloom.color[1]*255.) as u8,
                            (sub.bloom.color[2]*255.) as u8, (sub.bloom.color[3]*255.*final_opacity*sub.bloom.intensity*0.1) as u8,
                        );
                        let r = sub.bloom.radius * scale_factor;
                        let passes = 5;
                        for i in 1..=passes {
                            let spread = (i as f32 / passes as f32) * r;
                            for dx in [-spread, 0.0, spread] {
                                for dy in [-spread, 0.0, spread] {
                                    if dx == 0.0 && dy == 0.0 { continue; }
                                    let g = create_job(Some(b_col));
                                    draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                                }
                            }
                        }
                    }

                    // Additional strokes (use single-galley)
                    if sub.additional_strokes_enabled {
                        let mut sorted_strokes = sub.additional_strokes.clone();
                        sorted_strokes.sort_by(|a, b| b.width.partial_cmp(&a.width).unwrap());
                        for stroke in &sorted_strokes {
                            if stroke.enabled && stroke.width > 0.0 {
                                let sw = stroke.width * scale_factor;
                                let stc = stroke.color;
                                let stroke_color = egui::Color32::from_rgba_unmultiplied(
                                    (stc[0] * 255.0) as u8, (stc[1] * 255.0) as u8, (stc[2] * 255.0) as u8, (stc[3] * final_opacity * 255.0) as u8,
                                );
                                let offsets: &[(f32, f32)] = &[(-sw,-sw),(0.0,-sw),(sw,-sw),(-sw,0.0),(sw,0.0),(-sw,sw),(0.0,sw),(sw,sw)];
                                for &(dx, dy) in offsets {
                                    let g = create_job(Some(stroke_color));
                                    draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                                }
                            }
                        }
                    }

                    // Drop shadow (use single-galley)
                    if sub.shadow_enabled {
                        let sc = sub.shadow_color;
                        let s_col = egui::Color32::from_rgba_unmultiplied((sc[0]*255.) as u8,(sc[1]*255.) as u8,(sc[2]*255.) as u8,(sc[3]*final_opacity*255.) as u8);
                        let raw_off = Vec2::new(sub.shadow_offset[0]*scale_factor, sub.shadow_offset[1]*scale_factor);
                        let roff_x = raw_off.x * rot_rad.cos() - raw_off.y * rot_rad.sin();
                        let roff_y = raw_off.x * rot_rad.sin() + raw_off.y * rot_rad.cos();
                        let offset = Vec2::new(roff_x, roff_y);
                        let blur_steps_shadow = 3u32;
                        let blur_r = sub.shadow_blur * scale_factor / blur_steps_shadow as f32;
                        for b_step in 1..=blur_steps_shadow {
                            let alpha_frac = (blur_steps_shadow + 1 - b_step) as f32 / (blur_steps_shadow + 1) as f32;
                            let step_col = egui::Color32::from_rgba_unmultiplied(s_col.r(), s_col.g(), s_col.b(), (s_col.a() as f32 * alpha_frac * 0.6) as u8);
                            let blur_offset = b_step as f32 * blur_r;
                            for dx in [-blur_offset, 0.0, blur_offset] {
                                for dy in [-blur_offset, 0.0, blur_offset] {
                                    if dx == 0.0 && dy == 0.0 { continue; }
                                    let g = create_job(Some(step_col));
                                    draw_projected_text(&active_painter, g, offset + Vec2::new(dx, dy));
                                }
                            }
                        }
                        let g = create_job(Some(s_col));
                        draw_projected_text(&active_painter, g, offset);
                    }

                    // Primary stroke (use single-galley)
                    if sub.stroke_enabled && sub.stroke_width > 0.0 {
                        let sw = sub.stroke_width * scale_factor;
                        let stc = sub.stroke_color;
                        let stroke_color = egui::Color32::from_rgba_unmultiplied((stc[0]*255.) as u8,(stc[1]*255.) as u8,(stc[2]*255.) as u8,(stc[3]*final_opacity*255.) as u8);
                        let offsets: &[(f32, f32)] = &[(-sw,-sw),(0.0,-sw),(sw,-sw),(-sw,0.0),(sw,0.0),(-sw,sw),(0.0,sw),(sw,sw)];
                        for &(dx, dy) in offsets {
                            let g = create_job(Some(stroke_color));
                            draw_projected_text(&active_painter, g, Vec2::new(dx, dy));
                        }
                    }

                    // ── Main text — use per-char deform when active ────────────
                    if sub.glitch.enabled {
                        if sub.glitch.rgb_split > 0.0 {
                            let split = sub.glitch.rgb_split * scale_factor;
                            let r_col = egui::Color32::from_rgba_unmultiplied(255, 0, 0, (180.0 * final_opacity) as u8);
                            let b_col2 = egui::Color32::from_rgba_unmultiplied(0, 255, 255, (180.0 * final_opacity) as u8);
                            let g_red = create_job(Some(r_col));
                            draw_projected_text(&active_painter, g_red, Vec2::new(-split, 0.0));
                            let g_blue = create_job(Some(b_col2));
                            draw_projected_text(&active_painter, g_blue, Vec2::new(split, 0.0));
                        }
                        if sub.gradient_enabled {
                            let gc = sub.gradient_color;
                            let top_color = egui::Color32::from_rgba_unmultiplied((gc[0]*255.) as u8,(gc[1]*255.) as u8,(gc[2]*255.) as u8,(gc[3]*final_opacity*255.) as u8);
                            let g = create_job(Some(top_color));
                            draw_text_with_deform(&active_painter, g, Vec2::ZERO, needs_char_deform);
                        } else {
                            let galley2 = create_job(None);
                            draw_text_with_deform(&active_painter, galley2, Vec2::ZERO, needs_char_deform);
                        }
                        if sub.glitch.scanlines {
                            let mut sy = preview_rect.min.y;
                            while sy < preview_rect.max.y {
                                active_painter.line_segment(
                                    [Pos2::new(preview_rect.min.x, sy), Pos2::new(preview_rect.max.x, sy)],
                                    Stroke::new(1.0, egui::Color32::from_black_alpha(40))
                                );
                                sy += 4.0;
                            }
                        }
                    } else {
                        if sub.gradient_enabled {
                            let gc = sub.gradient_color;
                            let top_color = egui::Color32::from_rgba_unmultiplied((gc[0]*255.) as u8,(gc[1]*255.) as u8,(gc[2]*255.) as u8,(gc[3]*final_opacity*255.) as u8);
                            let g = create_job(Some(top_color));
                            draw_text_with_deform(&active_painter, g, Vec2::ZERO, needs_char_deform);
                        } else {
                            let galley2 = create_job(None);
                            draw_text_with_deform(&active_painter, galley2, Vec2::ZERO, needs_char_deform);
                        }
                    }
                }
            } // end loop over clip rects

            // Selection highlight
            if !path_is_active && step == 0 && vm.selected_id.as_deref() == Some(&sub.id) {
                let text_size = final_text_size;
                let pad = 4.0;
                let w = text_size.x + pad * 2.0;
                let h = text_size.y + 4.0;
                let pts = [
                    Vec2::new(-w/2.0, -h/2.0),
                    Vec2::new(w/2.0, -h/2.0),
                    Vec2::new(w/2.0, h/2.0),
                    Vec2::new(-w/2.0, h/2.0),
                ].iter().map(|v| apply_transform(v.x, v.y)).collect::<Vec<_>>();
                let shape = egui::Shape::convex_polygon(pts, egui::Color32::TRANSPARENT, Stroke::new(1.5, ACCENT_CYAN));
                canvas_painter.add(shape);
            }
        }
    }
}

// ── Per-character deformed text rendering ────────────────────────────────────
//
// Renders text character-by-character so each glyph can be displaced
// independently by the deform function.  This is what makes Wave, Arc,
// Bulge and Flag actually *warp* the text instead of just translating it.

fn draw_deformed_text(
    ctx: &Context,
    painter: &egui::Painter,
    sub: &crate::models::types::subtitle::Subtitle,
    has_words: bool,
    font_size: f32,
    text_size: Vec2,
    offset: Vec2,           // additional whole-text offset (e.g. Vec2::ZERO for main pass)
    apply_transform: &impl Fn(f32, f32) -> Pos2,
    text_color: egui::Color32,
    final_opacity: f32,
    current_time: f64,
) {
    // Build a list of (char_string, color) pairs
    let chars_with_color: Vec<(String, egui::Color32)> = if has_words {
        let mut out = Vec::new();
        for (wi, word) in sub.words.iter().enumerate() {
            let is_active = current_time >= word.start && current_time <= word.end;
            let mut wc = text_color;
            match &sub.word_animation {
                WordAnimation::KaraokeHighlight { color } => {
                    if is_active {
                        wc = egui::Color32::from_rgba_unmultiplied(
                            (color[0]*255.) as u8, (color[1]*255.) as u8,
                            (color[2]*255.) as u8, (color[3]*final_opacity*255.) as u8,
                        );
                    }
                }
                WordAnimation::KaraokePop { .. } => {
                    if is_active {
                        wc = egui::Color32::from_rgba_unmultiplied(255, 255, 0, (final_opacity*255.) as u8);
                    }
                }
                WordAnimation::CascadeFade => {
                    let alpha = if current_time < word.start { 0.0 }
                        else if is_active { ((current_time - word.start) / (word.end - word.start).max(0.01)) as f32 }
                        else { 1.0 };
                    wc = egui::Color32::from_rgba_unmultiplied(wc.r(), wc.g(), wc.b(), (wc.a() as f32 * alpha) as u8);
                }
                WordAnimation::None => {}
            }
            if let Some(c) = word.custom_color {
                wc = egui::Color32::from_rgba_unmultiplied(
                    (c[0]*255.) as u8, (c[1]*255.) as u8, (c[2]*255.) as u8, (c[3]*final_opacity*255.) as u8,
                );
            }
            for (ci, ch) in word.text.chars().enumerate() {
                out.push((ch.to_string(), wc));
            }
            if wi < sub.words.len() - 1 {
                out.push((" ".to_string(), text_color));
            }
        }
        out
    } else {
        sub.text.chars().map(|c| (c.to_string(), text_color)).collect()
    };

    if chars_with_color.is_empty() { return; }

    // Measure cumulative x positions using the font
    // We walk through and compute each character's x offset from the left edge of the text.
    let mut char_x_positions: Vec<f32> = Vec::with_capacity(chars_with_color.len() + 1);
    {
        let mut current_str = String::new();
        let mut prev_w = 0.0f32;
        char_x_positions.push(0.0);
        for (s, _) in &chars_with_color {
            current_str.push_str(s);
            let w = ctx.fonts(|f| {
                f.layout_no_wrap(
                    current_str.clone(),
                    egui::FontId::proportional(font_size),
                    egui::Color32::WHITE,
                )
                .size()
                .x
            });
            char_x_positions.push(w);
            prev_w = w;
        }
        // total width is the last entry
        let _ = prev_w;
    }

    let total_w = *char_x_positions.last().unwrap_or(&text_size.x);
    let row_height = ctx.fonts(|f| f.row_height(&egui::FontId::proportional(font_size)));

    for (i, (ch_str, color)) in chars_with_color.iter().enumerate() {
        let char_left = char_x_positions[i];
        let char_right = char_x_positions[i + 1];
        let char_center_x = (char_left + char_right) * 0.5 - total_w * 0.5 + offset.x;
        let char_center_y = offset.y;  // vertical center offset

        // Apply the deform function at this character's x position
        let screen_pos = apply_transform(char_center_x, char_center_y);

        // We render each character placed so its center is at screen_pos
        let char_w = char_right - char_left;
        let galley = ctx.fonts(|f| {
            f.layout_no_wrap(
                ch_str.clone(),
                egui::FontId::proportional(font_size),
                *color,
            )
        });

        // Position glyph so its center aligns with screen_pos
        let draw_pos = Pos2::new(
            screen_pos.x - galley.size().x * 0.5,
            screen_pos.y - row_height * 0.5,
        );

        let _ = char_w;

        // For wave/arc/flag the local rotation follows the deform tangent
        // Compute a small tangent for the character angle
        let dx_next = (char_center_x + 2.0).max(char_center_x + 0.1);
        let next_pos = apply_transform(dx_next, char_center_y);
        let char_angle = (next_pos.y - screen_pos.y).atan2(next_pos.x - screen_pos.x);

        let shape = egui::epaint::TextShape {
            pos: draw_pos,
            galley,
            underline: Stroke::NONE,
            override_text_color: None,
            angle: char_angle,
            fallback_color: *color,
            opacity_factor: final_opacity,
        };
        painter.add(shape);
    }
}

// ── Build mask clip painter ───────────────────────────────────────────────────

fn get_mask_clip_rects(
    sub: &crate::models::types::subtitle::Subtitle,
    state: &crate::models::types::animation::InterpolatedState,
    preview_rect: Rect,
    scale_factor: f32,
) -> Vec<Rect> {
    if sub.mask_type == MaskType::None {
        return vec![preview_rect];
    }

    let mc_x = preview_rect.center().x + state.mask_center[0] * scale_factor;
    let mc_y = preview_rect.center().y + state.mask_center[1] * scale_factor;
    let rot = state.mask_rotation.to_radians();

    let rot_pt = |dx: f32, dy: f32| -> Pos2 {
        let rx = dx * rot.cos() - dy * rot.sin();
        let ry = dx * rot.sin() + dy * rot.cos();
        Pos2::new(mc_x + rx, mc_y + ry)
    };

    match sub.mask_type {
        MaskType::Rectangle | MaskType::Circle => {
            let hw = if sub.mask_type == MaskType::Circle { state.mask_size[0] * scale_factor } else { state.mask_size[0] * 0.5 * scale_factor };
            let hh = if sub.mask_type == MaskType::Circle { state.mask_size[0] * scale_factor } else { state.mask_size[1] * 0.5 * scale_factor };
            
            let corners = [rot_pt(-hw,-hh), rot_pt(hw,-hh), rot_pt(hw,hh), rot_pt(-hw,hh)];
            let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
            
            let inner = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
            
            if sub.mask_invert {
                let mut rects = Vec::new();
                if inner.min.y > preview_rect.min.y {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, preview_rect.min.y),
                        Pos2::new(preview_rect.max.x, inner.min.y.min(preview_rect.max.y))
                    );
                    if r.is_positive() { rects.push(r); }
                }
                if inner.max.y < preview_rect.max.y {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, inner.max.y.max(preview_rect.min.y)),
                        Pos2::new(preview_rect.max.x, preview_rect.max.y)
                    );
                    if r.is_positive() { rects.push(r); }
                }
                if inner.min.x > preview_rect.min.x {
                    let r = Rect::from_min_max(
                        Pos2::new(preview_rect.min.x, inner.min.y.max(preview_rect.min.y)),
                        Pos2::new(inner.min.x.min(preview_rect.max.x), inner.max.y.min(preview_rect.max.y))
                    );
                    if r.is_positive() { rects.push(r); }
                }
                if inner.max.x < preview_rect.max.x {
                    let r = Rect::from_min_max(
                        Pos2::new(inner.max.x.max(preview_rect.min.x), inner.min.y.max(preview_rect.min.y)),
                        Pos2::new(preview_rect.max.x, inner.max.y.min(preview_rect.max.y))
                    );
                    if r.is_positive() { rects.push(r); }
                }
                rects
            } else {
                let r = inner.intersect(preview_rect);
                if r.is_positive() { vec![r] } else { vec![] }
            }
        }
        MaskType::Straight => {
            let mut valid_pts = Vec::new();
            let sin_r = rot.sin();
            let cos_r = rot.cos();
            
            let keep = |x: f32, y: f32| -> bool {
                let dx = x - mc_x;
                let dy = y - mc_y;
                let local_y = -dx * sin_r + dy * cos_r;
                if sub.mask_invert { local_y <= 0.0 } else { local_y >= 0.0 }
            };

            let corners = [
                preview_rect.left_top(),
                preview_rect.right_top(),
                preview_rect.right_bottom(),
                preview_rect.left_bottom(),
            ];
            for p in &corners {
                if keep(p.x, p.y) {
                    valid_pts.push(*p);
                }
            }

            if cos_r.abs() > 1e-5 {
                let tan_r = sin_r / cos_r;
                for &x in &[preview_rect.min.x, preview_rect.max.x] {
                    let dy = (x - mc_x) * tan_r;
                    let y = mc_y + dy;
                    if y >= preview_rect.min.y && y <= preview_rect.max.y {
                        valid_pts.push(Pos2::new(x, y));
                    }
                }
            }

            if sin_r.abs() > 1e-5 {
                let cot_r = cos_r / sin_r;
                for &y in &[preview_rect.min.y, preview_rect.max.y] {
                    let dx = (y - mc_y) * cot_r;
                    let x = mc_x + dx;
                    if x >= preview_rect.min.x && x <= preview_rect.max.x {
                        valid_pts.push(Pos2::new(x, y));
                    }
                }
            }

            if valid_pts.is_empty() {
                return vec![];
            }

            let min_x = valid_pts.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = valid_pts.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = valid_pts.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = valid_pts.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

            let r = Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y));
            if r.is_positive() {
                vec![r.intersect(preview_rect)]
            } else {
                vec![]
            }
        }
        MaskType::None => vec![preview_rect]
    }
}

// ── Letter/Word-by-word path text rendering ──────────────────────────────────

fn draw_text_on_path(
    ctx: &Context,
    painter: &egui::Painter,
    sub: &crate::models::types::subtitle::Subtitle,
    state: &crate::models::types::animation::InterpolatedState,
    preview_rect: Rect,
    scale_factor: f32,
    font_size: f32,
    text_color: egui::Color32,
    final_opacity: f32,
    _current_time: f64,
    _has_words: bool,
) {
    let (chunks, chunk_widths, total_width) = if sub.path_align_words {
        let words: Vec<String> = sub.text.split_whitespace().map(|s| s.to_string()).collect();
        let mut widths = Vec::new();
        let mut total = 0.0;
        let space_w = ctx.fonts(|f| f.layout_no_wrap(" ".to_string(), egui::FontId::proportional(font_size), egui::Color32::WHITE)).size().x;
        
        for w in &words {
            let width = ctx.fonts(|f| f.layout_no_wrap(w.clone(), egui::FontId::proportional(font_size), egui::Color32::WHITE)).size().x;
            widths.push(width);
            total += width + space_w;
        }
        total -= space_w;
        if total < 0.0 { total = 0.0; }
        (words, widths, total)
    } else {
        let chars: Vec<char> = sub.text.chars().collect();
        let mut widths = Vec::new();
        let mut accum_widths = Vec::new();
        let mut current_str = String::new();
        for c in &chars {
            current_str.push(*c);
            let width = ctx.fonts(|f| f.layout_no_wrap(current_str.clone(), egui::FontId::proportional(font_size), egui::Color32::WHITE)).size().x;
            accum_widths.push(width);
        }
        let mut prev = 0.0;
        for &w in &accum_widths {
            widths.push(w - prev);
            prev = w;
        }
        let total = accum_widths.last().copied().unwrap_or(0.0);
        let strings = chars.iter().map(|c| c.to_string()).collect();
        (strings, widths, total)
    };

    if chunks.is_empty() || total_width <= 0.0 { return; }

    let space_w = if sub.path_align_words {
        ctx.fonts(|f| f.layout_no_wrap(" ".to_string(), egui::FontId::proportional(font_size), egui::Color32::WHITE)).size().x
    } else {
        0.0
    };

    let row_height = ctx.fonts(|f| f.row_height(&egui::FontId::proportional(font_size)));

    let path_samples = 200;
    let path_points: Vec<(f32, f32)> = (0..=path_samples).map(|i| {
        let p = i as f32 / path_samples as f32;
        let (px, py, _) = sub.evaluate_path(p);
        (
            preview_rect.center().x + (state.x + px) * scale_factor,
            preview_rect.center().y + (state.y + py) * scale_factor,
        )
    }).collect();

    let mut arc_lengths = vec![0.0f32; path_samples + 1];
    for i in 1..=path_samples {
        let dx = path_points[i].0 - path_points[i-1].0;
        let dy = path_points[i].1 - path_points[i-1].1;
        arc_lengths[i] = arc_lengths[i-1] + (dx*dx + dy*dy).sqrt();
    }
    let total_arc = arc_lengths[path_samples];

    if total_arc < 1.0 { return; }

    let arc_to_t = |arc: f32| -> f32 {
        let clamped = arc.clamp(0.0, total_arc);
        let idx = arc_lengths.partition_point(|&a| a <= clamped).saturating_sub(1).min(path_samples - 1);
        let seg_len = arc_lengths[idx + 1] - arc_lengths[idx];
        let frac = if seg_len > 0.0 { (clamped - arc_lengths[idx]) / seg_len } else { 0.0 };
        ((idx as f32 + frac) / path_samples as f32).clamp(0.0, 1.0)
    };

    let path_start_arc = state.path_progress * total_arc;
    let start_arc = path_start_arc - total_width * 0.5;

    let mut cursor_arc = start_arc;
    for (i, chunk) in chunks.iter().enumerate() {
        let cw = chunk_widths[i];
        let center_arc = cursor_arc + cw * 0.5;

        let t = arc_to_t(center_arc.rem_euclid(total_arc));
        let (cx, cy, _angle) = sub.evaluate_path(t);
        let screen_x = preview_rect.center().x + (state.x + cx) * scale_factor;
        let screen_y = preview_rect.center().y + (state.y + cy) * scale_factor;

        let t2 = arc_to_t((center_arc + 5.0).rem_euclid(total_arc));
        let (cx2, cy2, _) = sub.evaluate_path(t2);
        let sx2 = preview_rect.center().x + (state.x + cx2) * scale_factor;
        let sy2 = preview_rect.center().y + (state.y + cy2) * scale_factor;

        let tangent_angle = if sub.path_orient {
            (sy2 - screen_y).atan2(sx2 - screen_x)
        } else {
            state.rotation.to_radians()
        };

        let galley = ctx.fonts(|f| f.layout_no_wrap(
            chunk.clone(),
            egui::FontId::proportional(font_size),
            text_color,
        ));
        let g_size = galley.size();

        let char_pos = Pos2::new(
            screen_x - g_size.x * 0.5,
            screen_y - row_height * 0.5,
        );

        let shape = egui::epaint::TextShape {
            pos: char_pos,
            galley,
            underline: Stroke::NONE,
            override_text_color: None,
            angle: tangent_angle,
            fallback_color: text_color,
            opacity_factor: final_opacity,
        };
        painter.add(shape);

        cursor_arc += cw + space_w;
    }
}

// ── Layout job builder ────────────────────────────────────────────────────────

fn build_layout_job(
    ctx: &Context,
    sub: &crate::models::types::subtitle::Subtitle,
    has_words: bool,
    font_size: f32,
    text_color: egui::Color32,
    final_opacity: f32,
    override_color: Option<egui::Color32>,
    current_time: f64,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();

    if has_words {
        for (i, word) in sub.words.iter().enumerate() {
            let is_active = current_time >= word.start && current_time <= word.end;
            let mut word_color = text_color;
            let mut word_scale = 1.0;
            let mut word_opacity = 1.0;

            match &sub.word_animation {
                WordAnimation::KaraokeHighlight { color } => {
                    if is_active {
                        word_color = egui::Color32::from_rgba_unmultiplied(
                            (color[0]*255.) as u8, (color[1]*255.) as u8, (color[2]*255.) as u8, (color[3]*final_opacity*255.) as u8
                        );
                    }
                }
                WordAnimation::KaraokePop { scale } => {
                    if is_active {
                        word_scale = *scale;
                        word_color = egui::Color32::from_rgba_unmultiplied(255, 255, 0, (final_opacity*255.) as u8);
                    }
                }
                WordAnimation::CascadeFade => {
                    if current_time < word.start { word_opacity = 0.0; }
                    else if is_active { word_opacity = ((current_time - word.start) / (word.end - word.start).max(0.01)) as f32; }
                    word_color = egui::Color32::from_rgba_unmultiplied(
                        word_color.r(), word_color.g(), word_color.b(), (word_color.a() as f32 * word_opacity) as u8
                    );
                }
                WordAnimation::None => {}
            }

            if let Some(c) = word.custom_color {
                word_color = egui::Color32::from_rgba_unmultiplied(
                    (c[0]*255.) as u8, (c[1]*255.) as u8, (c[2]*255.) as u8, (c[3]*final_opacity*255.) as u8
                );
            }

            let final_col = if let Some(oc) = override_color {
                egui::Color32::from_rgba_unmultiplied(oc.r(), oc.g(), oc.b(), (oc.a() as f32 * word_opacity) as u8)
            } else {
                word_color
            };

            job.append(&word.text, 0.0, egui::text::TextFormat {
                font_id: egui::FontId::proportional(font_size * word_scale),
                color: final_col,
                ..Default::default()
            });

            if i < sub.words.len() - 1 {
                let spc_col = override_color.unwrap_or(text_color);
                job.append(" ", 0.0, egui::text::TextFormat {
                    font_id: egui::FontId::proportional(font_size),
                    color: spc_col,
                    ..Default::default()
                });
            }
        }
    } else {
        job.append(&sub.text, 0.0, egui::text::TextFormat {
            font_id: egui::FontId::proportional(font_size),
            color: override_color.unwrap_or(text_color),
            ..Default::default()
        });
    }

    ctx.fonts(|f| f.layout_job(job))
}