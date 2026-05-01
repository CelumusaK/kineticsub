use egui::{CentralPanel, Context, Frame, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{BG_PANEL, BG_PANEL_ALT, BORDER, ACCENT_CYAN};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    CentralPanel::default()
        .frame(Frame { fill: BG_PANEL, ..Default::default() })
        .show(ctx, |ui| {
            let canvas_rect = ui.available_rect_before_wrap();

            let (canvas_response, canvas_painter) =
                ui.allocate_painter(canvas_rect.size(), egui::Sense::click());
            canvas_painter.rect_filled(canvas_rect, 0.0, BG_PANEL_ALT);

            // Dynamically evaluate aspect ratio from Project settings!
            let res_w = vm.project.resolution.0 as f32;
            let res_h = vm.project.resolution.1 as f32;
            let aspect = res_w / res_h;
            
            let preview_w = (canvas_rect.width()  - 40.0).min((canvas_rect.height() - 40.0) * aspect);
            let preview_h = preview_w / aspect;
            let preview_rect = Rect::from_center_size(
                canvas_rect.center(), Vec2::new(preview_w, preview_h),
            );

            // ── Background Rendering ──────────────────────────────────────────
            let active_bg = vm.project.media_files.iter().find(|m| {
                m.on_timeline && m.is_video_track && vm.current_time() >= m.timeline_offset && vm.current_time() < m.timeline_offset + m.duration
            });

            if let Some(bg) = active_bg {
                if let Some(col) = bg.color {
                    let egui_col = egui::Color32::from_rgba_unmultiplied(
                        (col[0]*255.0) as u8, (col[1]*255.0) as u8, (col[2]*255.0) as u8, (col[3]*255.0) as u8
                    );
                    canvas_painter.rect_filled(preview_rect, 0.0, egui_col);
                }
            } else {
                // Draw a Transparency Checkerboard if no background track is found
                let cell_size = 16.0;
                let cols = (preview_rect.width() / cell_size).ceil() as usize;
                let rows = (preview_rect.height() / cell_size).ceil() as usize;
                
                for r in 0..rows {
                    for c in 0..cols {
                        let color = if (r + c) % 2 == 0 {
                            egui::Color32::from_rgb(60, 60, 60)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        };
                        let x = preview_rect.min.x + c as f32 * cell_size;
                        let y = preview_rect.min.y + r as f32 * cell_size;
                        let rect = Rect::from_min_size(
                            Pos2::new(x, y),
                            Vec2::new(cell_size, cell_size)
                        ).intersect(preview_rect);
                        canvas_painter.rect_filled(rect, 0.0, color);
                    }
                }
            }
            
            canvas_painter.rect_stroke(preview_rect, 2.0, Stroke::new(1.0, BORDER));

            let scale_factor = preview_w / res_w;

            if let Some(sub) = vm.active_subtitle() {
                let max_steps = if sub.motion_blur > 0.0 { 3 } else { 0 };
                
                for step in (0..=max_steps).rev() {
                    let offset_t = vm.current_time() - (step as f64 * 0.02 * sub.motion_blur as f64);
                    if offset_t < sub.timeline_start && step > 0 { continue; }
                    
                    let state = sub.get_interpolated_state(offset_t);
                    let opacity_mult = if step > 0 { 0.25 } else { 1.0 };
                    let final_opacity = state.opacity * opacity_mult;
                    
                    if final_opacity <= 0.01 { continue; }

                    let font_size = sub.font_size * state.scale * scale_factor;
                    let font_id  = egui::FontId::proportional(font_size);

                    let center_x = preview_rect.center().x + state.x * scale_factor;
                    let center_y = preview_rect.center().y + state.y * scale_factor;

                    let text_color = {
                        let c = sub.color;
                        egui::Color32::from_rgba_unmultiplied(
                            (c[0] * 255.0) as u8,
                            (c[1] * 255.0) as u8,
                            (c[2] * 255.0) as u8,
                            (c[3] * final_opacity * 255.0) as u8,
                        )
                    };

                    let galley = ctx.fonts(|f| {
                        f.layout_no_wrap(sub.text.clone(), font_id.clone(), text_color)
                    });
                    let text_size = galley.size();

                    let text_pos = Pos2::new(
                        center_x - text_size.x * 0.5,
                        center_y - text_size.y * 0.5,
                    );

                    if sub.bg_box_enabled {
                        let pad = sub.bg_box_padding * scale_factor;
                        let box_rect = Rect::from_min_size(
                            Pos2::new(text_pos.x - pad, text_pos.y - pad),
                            Vec2::new(text_size.x + pad * 2.0, text_size.y + pad * 2.0),
                        );
                        let bc = sub.bg_box_color;
                        let box_color = egui::Color32::from_rgba_unmultiplied(
                            (bc[0] * 255.0) as u8, (bc[1] * 255.0) as u8,
                            (bc[2] * 255.0) as u8, (bc[3] * final_opacity * 255.0) as u8,
                        );
                        canvas_painter.rect_filled(box_rect, 4.0 * scale_factor, box_color);
                    }

                    if sub.shadow_enabled {
                        let sc     = sub.shadow_color;
                        let s_col  = egui::Color32::from_rgba_unmultiplied(
                            (sc[0] * 255.0) as u8, (sc[1] * 255.0) as u8,
                            (sc[2] * 255.0) as u8, (sc[3] * final_opacity * 255.0) as u8,
                        );
                        let offset = Vec2::new(
                            sub.shadow_offset[0] * scale_factor,
                            sub.shadow_offset[1] * scale_factor,
                        );

                        let blur_steps_shadow = 3u32;
                        let blur_r = sub.shadow_blur * scale_factor / blur_steps_shadow as f32;
                        for b_step in 1..=blur_steps_shadow {
                            let alpha_frac = (blur_steps_shadow + 1 - b_step) as f32 / (blur_steps_shadow + 1) as f32;
                            let step_col   = egui::Color32::from_rgba_unmultiplied(
                                s_col.r(), s_col.g(), s_col.b(),
                                (s_col.a() as f32 * alpha_frac * 0.6) as u8,
                            );
                            let blur_offset = b_step as f32 * blur_r;
                            for dx in [-blur_offset, 0.0, blur_offset] {
                                for dy in [-blur_offset, 0.0, blur_offset] {
                                    if dx == 0.0 && dy == 0.0 { continue; }
                                    canvas_painter.galley(
                                        text_pos + offset + Vec2::new(dx, dy),
                                        ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), step_col)),
                                        step_col,
                                    );
                                }
                            }
                        }

                        canvas_painter.galley(
                            text_pos + offset,
                            ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), s_col)),
                            s_col,
                        );
                    }

                    if sub.stroke_enabled && sub.stroke_width > 0.0 {
                        let sw  = sub.stroke_width * scale_factor;
                        let stc = sub.stroke_color;
                        let stroke_color = egui::Color32::from_rgba_unmultiplied(
                            (stc[0] * 255.0) as u8, (stc[1] * 255.0) as u8,
                            (stc[2] * 255.0) as u8, (stc[3] * final_opacity * 255.0) as u8,
                        );

                        let offsets: &[(f32, f32)] = &[
                            (-sw, -sw), (0.0, -sw), (sw, -sw),
                            (-sw,  0.0),             (sw,  0.0),
                            (-sw,  sw), (0.0,  sw), (sw,  sw),
                        ];
                        for &(dx, dy) in offsets {
                            canvas_painter.galley(
                                text_pos + Vec2::new(dx, dy),
                                ctx.fonts(|f| {
                                    f.layout_no_wrap(sub.text.clone(), font_id.clone(), stroke_color)
                                }),
                                stroke_color,
                            );
                        }
                    }

                    if sub.gradient_enabled {
                        let gc  = sub.gradient_color;
                        let top_color = egui::Color32::from_rgba_unmultiplied(
                            (gc[0] * 255.0) as u8, (gc[1] * 255.0) as u8,
                            (gc[2] * 255.0) as u8, (gc[3] * final_opacity * 255.0) as u8,
                        );

                        let top_clip = Rect::from_min_size(
                            text_pos,
                            Vec2::new(text_size.x, text_size.y * 0.5),
                        );
                        canvas_painter.with_clip_rect(top_clip).galley(
                            text_pos,
                            ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), top_color)),
                            top_color,
                        );

                        let bot_clip = Rect::from_min_size(
                            Pos2::new(text_pos.x, text_pos.y + text_size.y * 0.5),
                            Vec2::new(text_size.x, text_size.y * 0.5 + 1.0),
                        );
                        canvas_painter.with_clip_rect(bot_clip).galley(
                            text_pos,
                            ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), text_color)),
                            text_color,
                        );
                    } else {
                        canvas_painter.galley(text_pos, galley, text_color);
                    }

                    if step == 0 && vm.selected_id.as_deref() == Some(&sub.id) {
                        let pad = 4.0;
                        let highlight = Rect::from_min_size(
                            Pos2::new(text_pos.x - pad, text_pos.y - 2.0),
                            Vec2::new(text_size.x + pad * 2.0, text_size.y + 4.0),
                        );
                        canvas_painter.rect_stroke(highlight, 3.0, Stroke::new(1.5, ACCENT_CYAN));
                    }
                }
            }

            if canvas_response.hovered() {
                ctx.input(|i| {
                    if i.key_pressed(egui::Key::Space)      { vm.toggle_play(); }
                    if i.key_pressed(egui::Key::J)          { vm.skip(-5.0); }
                    if i.key_pressed(egui::Key::L)          { vm.skip(5.0); }
                    if i.key_pressed(egui::Key::ArrowLeft)  { vm.skip(-1.0 / 30.0); }
                    if i.key_pressed(egui::Key::ArrowRight) { vm.skip(1.0 / 30.0); }
                });
            }
        });
}