use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::subtitle::{WordAnimation, TextDeform};
use crate::views::theme::{ACCENT_CYAN};

pub fn draw(ctx: &Context, canvas_painter: &egui::Painter, preview_rect: Rect, scale_factor: f32, vm: &mut EditorViewModel) {
    if let Some(sub) = vm.active_subtitle() {
        let max_steps = if sub.motion_blur > 0.0 { 3 } else { 0 };
        
        for step in (0..=max_steps).rev() {
            let offset_t = vm.current_time() - (step as f64 * 0.02 * sub.motion_blur as f64);
            if offset_t < sub.timeline_start && step > 0 { continue; }
            
            // Pass the entire project subtitle list to resolve parent logic
            let state = sub.get_interpolated_state(offset_t, &vm.project.subtitles, 0);
            
            let opacity_mult = if step > 0 { 0.25 } else { 1.0 };
            let final_opacity = state.opacity * opacity_mult;
            
            if final_opacity <= 0.01 { continue; }

            let font_size = (sub.font_size * state.scale * scale_factor).max(0.1);
            let font_id  = egui::FontId::proportional(font_size);

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

            // Generate a dynamic LayoutJob allowing multiple formats per word!
            let create_job = |override_color: Option<egui::Color32>| -> std::sync::Arc<egui::Galley> {
                let mut job = egui::text::LayoutJob::default();
                
                if has_words {
                    let t_now = vm.current_time();
                    for (i, word) in sub.words.iter().enumerate() {
                        let is_active = t_now >= word.start && t_now <= word.end;
                        
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
                                if t_now < word.start { word_opacity = 0.0; }
                                else if is_active { word_opacity = ((t_now - word.start) / (word.end - word.start).max(0.01)) as f32; }
                                
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
                        font_id: font_id.clone(),
                        color: override_color.unwrap_or(text_color),
                        ..Default::default()
                    });
                }
                
                ctx.fonts(|f| f.layout_job(job))
            };

            let galley = create_job(None);
            let text_size = galley.size();

            // ── 3D Projection Math & Subtitle Deformers ──────────────────
            let (skew_x, skew_y) = (state.skew_x, state.skew_y);
            let (yaw, pitch) = (state.yaw.to_radians(), state.pitch.to_radians());
            
            let text_deform = sub.text_deform.clone();
            let deform_amt = sub.text_deform_amount;
            let current_t = vm.current_time();

            let apply_transform = |mut dx: f32, mut dy: f32| -> Pos2 {
                match text_deform {
                    TextDeform::Wave => { dy += (dx * 0.02 + current_t as f32 * 5.0).sin() * deform_amt; }
                    TextDeform::Arc => {
                        let arc_radius = 500.0 / deform_amt.abs().max(0.01);
                        if deform_amt != 0.0 { dy -= (dx / arc_radius).cos() * deform_amt * 2.0; }
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
                let py = sy * pitch.cos();
                let pz = sy * pitch.sin();
                let yx = sx * yaw.cos() + pz * yaw.sin();
                let yz = -sx * yaw.sin() + pz * yaw.cos();
                let rx = yx * rot_rad.cos() - py * rot_rad.sin();
                let ry = yx * rot_rad.sin() + py * rot_rad.cos();
                let dist = 800.0; 
                let z_factor = dist / (dist + yz).max(1.0); 

                Pos2::new(center_x + rx * z_factor, center_y + ry * z_factor)
            };

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

            // 1. Draw Background Box
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
                canvas_painter.add(egui::Shape::convex_polygon(pts, box_color, Stroke::new(4.0 * scale_factor, box_color)));
            }

            // 2. Draw Bloom Engine Passes
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
                            draw_projected_text(canvas_painter, g, Vec2::new(dx, dy));
                        }
                    }
                }
            }

            // 3. Draw Additional Multiple Strokes (Largest First)
            if sub.additional_strokes_enabled {
                let mut sorted_strokes = sub.additional_strokes.clone();
                sorted_strokes.sort_by(|a, b| b.width.partial_cmp(&a.width).unwrap());
                for stroke in &sorted_strokes {
                    if stroke.enabled && stroke.width > 0.0 {
                        let sw  = stroke.width * scale_factor;
                        let stc = stroke.color;
                        let stroke_color = egui::Color32::from_rgba_unmultiplied(
                            (stc[0] * 255.0) as u8, (stc[1] * 255.0) as u8, (stc[2] * 255.0) as u8, (stc[3] * final_opacity * 255.0) as u8,
                        );
                        let offsets: &[(f32, f32)] = &[ (-sw, -sw), (0.0, -sw), (sw, -sw), (-sw,  0.0), (sw,  0.0), (-sw,  sw), (0.0,  sw), (sw,  sw) ];
                        for &(dx, dy) in offsets {
                            let g = create_job(Some(stroke_color));
                            draw_projected_text(canvas_painter, g, Vec2::new(dx, dy));
                        }
                    }
                }
            }

            // 4. Draw Drop Shadow
            if sub.shadow_enabled {
                let sc = sub.shadow_color;
                let s_col = egui::Color32::from_rgba_unmultiplied((sc[0] * 255.0) as u8, (sc[1] * 255.0) as u8, (sc[2] * 255.0) as u8, (sc[3] * final_opacity * 255.0) as u8);
                let raw_off = Vec2::new(sub.shadow_offset[0] * scale_factor, sub.shadow_offset[1] * scale_factor);
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
                        for dy in[-blur_offset, 0.0, blur_offset] {
                            if dx == 0.0 && dy == 0.0 { continue; }
                            let g = create_job(Some(step_col));
                            draw_projected_text(canvas_painter, g, offset + Vec2::new(dx, dy));
                        }
                    }
                }
                let g = create_job(Some(s_col));
                draw_projected_text(canvas_painter, g, offset);
            }

            // 5. Draw Primary Stroke
            if sub.stroke_enabled && sub.stroke_width > 0.0 {
                let sw = sub.stroke_width * scale_factor;
                let stc = sub.stroke_color;
                let stroke_color = egui::Color32::from_rgba_unmultiplied((stc[0] * 255.0) as u8, (stc[1] * 255.0) as u8, (stc[2] * 255.0) as u8, (stc[3] * final_opacity * 255.0) as u8);
                let offsets: &[(f32, f32)] = &[(-sw, -sw), (0.0, -sw), (sw, -sw), (-sw,  0.0), (sw,  0.0), (-sw,  sw), (0.0,  sw), (sw,  sw)];
                for &(dx, dy) in offsets {
                    let g = create_job(Some(stroke_color));
                    draw_projected_text(canvas_painter, g, Vec2::new(dx, dy));
                }
            }

            // 6. Draw Glitch & Main Text Layer
            if sub.glitch.enabled {
                if sub.glitch.rgb_split > 0.0 {
                    let split = sub.glitch.rgb_split * scale_factor;
                    let r_col = egui::Color32::from_rgba_unmultiplied(255, 0, 0, (180.0 * final_opacity) as u8);
                    let b_col = egui::Color32::from_rgba_unmultiplied(0, 255, 255, (180.0 * final_opacity) as u8);
                    
                    let g_red = create_job(Some(r_col));
                    draw_projected_text(canvas_painter, g_red, Vec2::new(-split, 0.0));
                    
                    let g_blue = create_job(Some(b_col));
                    draw_projected_text(canvas_painter, g_blue, Vec2::new(split, 0.0));
                }
                
                if sub.gradient_enabled {
                    let gc  = sub.gradient_color;
                    let top_color = egui::Color32::from_rgba_unmultiplied((gc[0] * 255.0) as u8, (gc[1] * 255.0) as u8, (gc[2] * 255.0) as u8, (gc[3] * final_opacity * 255.0) as u8);
                    let g = create_job(Some(top_color));
                    draw_projected_text(canvas_painter, g, Vec2::ZERO);
                } else {
                    draw_projected_text(canvas_painter, galley.clone(), Vec2::ZERO);
                }

                if sub.glitch.scanlines {
                    let mut sy = preview_rect.min.y;
                    while sy < preview_rect.max.y {
                        canvas_painter.line_segment(
                            [Pos2::new(preview_rect.min.x, sy), Pos2::new(preview_rect.max.x, sy)],
                            Stroke::new(1.0, egui::Color32::from_black_alpha(40))
                        );
                        sy += 4.0;
                    }
                }
            } else {
                // Standard unglitched main text draw
                if sub.gradient_enabled {
                    let gc  = sub.gradient_color;
                    let top_color = egui::Color32::from_rgba_unmultiplied((gc[0] * 255.0) as u8, (gc[1] * 255.0) as u8, (gc[2] * 255.0) as u8, (gc[3] * final_opacity * 255.0) as u8);
                    let g = create_job(Some(top_color));
                    draw_projected_text(canvas_painter, g, Vec2::ZERO);
                } else {
                    draw_projected_text(canvas_painter, galley.clone(), Vec2::ZERO);
                }
            }

            // Highlight Active Block
            if step == 0 && vm.selected_id.as_deref() == Some(&sub.id) {
                let pad = 4.0;
                let w = text_size.x + pad * 2.0;
                let h = text_size.y + 4.0;
                let pts =[
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