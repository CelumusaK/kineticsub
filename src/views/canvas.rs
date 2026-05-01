use egui::{CentralPanel, Context, Frame, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{BG_PANEL, BG_PANEL_ALT, BG_BASE, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    CentralPanel::default()
        .frame(Frame { fill: BG_PANEL, ..Default::default() })
        .show(ctx, |ui| {
            let avail       = ui.available_rect_before_wrap();
            let toolbar_h   = 36.0;
            let toolbar_rect = Rect::from_min_size(avail.min, Vec2::new(avail.width(), toolbar_h));
            let canvas_rect  = Rect::from_min_size(
                Pos2::new(avail.min.x, avail.min.y + toolbar_h),
                Vec2::new(avail.width(), avail.height() - toolbar_h),
            );

            // ── Toolbar ───────────────────────────────────────────────────
            let (_tb_resp, tb_painter) = ui.allocate_painter(
                Vec2::new(avail.width(), toolbar_h), egui::Sense::hover(),
            );
            tb_painter.rect_filled(toolbar_rect, 0.0, BG_PANEL);
            tb_painter.rect_stroke(toolbar_rect, 0.0, Stroke::new(1.0, BORDER));

            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(toolbar_rect), |ui| {
                ui.set_clip_rect(toolbar_rect);
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    let play_label = if vm.is_playing() { "⏸  PAUSE" } else { "▶  PLAY" };
                    if ui.add(
                        egui::Button::new(egui::RichText::new(play_label).color(ACCENT_CYAN).size(12.0).strong())
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, ACCENT_CYAN)),
                    ).clicked() { vm.toggle_play(); }

                    ui.add_space(10.0);
                    if ui.small_button(egui::RichText::new("⏮  −5s").color(TEXT_NORM).size(11.0)).clicked() { vm.skip(-5.0); }
                    if ui.small_button(egui::RichText::new("+5s  ⏭").color(TEXT_NORM).size(11.0)).clicked() { vm.skip(5.0); }

                    ui.add_space(12.0);
                    let t = vm.current_time();
                    ui.label(
                        egui::RichText::new(format!("{:02}:{:06.3}", (t / 60.0) as i32, t % 60.0))
                            .color(TEXT_BRIGHT).size(13.0).monospace(),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("Space · J/L ±5s · ←→ frame · Ctrl+S save")
                            .color(TEXT_DIM).size(11.0),
                    );
                });
            });

            // ── Canvas area ───────────────────────────────────────────────
            let (canvas_response, canvas_painter) =
                ui.allocate_painter(canvas_rect.size(), egui::Sense::click());
            canvas_painter.rect_filled(canvas_rect, 0.0, BG_PANEL_ALT);

            // 16:9 preview rect centred in the canvas area
            let aspect   = 16.0 / 9.0;
            let preview_w = (canvas_rect.width()  - 40.0).min((canvas_rect.height() - 40.0) * aspect);
            let preview_h = preview_w / aspect;
            let preview_rect = Rect::from_center_size(
                canvas_rect.center(), Vec2::new(preview_w, preview_h),
            );

            canvas_painter.rect_filled(preview_rect, 2.0, BG_BASE);
            canvas_painter.rect_stroke(preview_rect, 2.0, Stroke::new(1.0, BORDER));

            // Scale factor: subtitle coords are in 1920×1080 space
            let scale_factor = preview_w / 1920.0;

            if let Some(sub) = vm.active_subtitle() {
                let state    = sub.get_interpolated_state(vm.current_time());
                let font_size = sub.font_size * state.scale * scale_factor;
                let font_id  = egui::FontId::proportional(font_size);

                // World-space anchor point
                let center_x = preview_rect.center().x + state.x * scale_factor;
                let center_y = preview_rect.center().y + state.y * scale_factor;

                // ── Text colour (opacity applied) ─────────────────────────
                let text_color = {
                    let c = sub.color;
                    egui::Color32::from_rgba_unmultiplied(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                        (c[3] * state.opacity * 255.0) as u8,
                    )
                };

                // Measure text extents once
                let galley = ctx.fonts(|f| {
                    f.layout_no_wrap(sub.text.clone(), font_id.clone(), text_color)
                });
                let text_size = galley.size();

                // Text top-left position (centred)
                let text_pos = Pos2::new(
                    center_x - text_size.x * 0.5,
                    center_y - text_size.y * 0.5,
                );

                // ── Background box ────────────────────────────────────────
                if sub.bg_box_enabled {
                    let pad = sub.bg_box_padding * scale_factor;
                    let box_rect = Rect::from_min_size(
                        Pos2::new(text_pos.x - pad, text_pos.y - pad),
                        Vec2::new(text_size.x + pad * 2.0, text_size.y + pad * 2.0),
                    );
                    let bc = sub.bg_box_color;
                    let box_color = egui::Color32::from_rgba_unmultiplied(
                        (bc[0] * 255.0) as u8, (bc[1] * 255.0) as u8,
                        (bc[2] * 255.0) as u8, (bc[3] * state.opacity * 255.0) as u8,
                    );
                    canvas_painter.rect_filled(box_rect, 4.0 * scale_factor, box_color);
                }

                // ── Drop shadow ───────────────────────────────────────────
                if sub.shadow_enabled {
                    let sc     = sub.shadow_color;
                    let s_col  = egui::Color32::from_rgba_unmultiplied(
                        (sc[0] * 255.0) as u8, (sc[1] * 255.0) as u8,
                        (sc[2] * 255.0) as u8, (sc[3] * state.opacity * 255.0) as u8,
                    );
                    let offset = Vec2::new(
                        sub.shadow_offset[0] * scale_factor,
                        sub.shadow_offset[1] * scale_factor,
                    );

                    // Simulate soft shadow with a few offset passes at decreasing opacity
                    let blur_steps = 3u32;
                    let blur_r     = sub.shadow_blur * scale_factor / blur_steps as f32;
                    for step in 1..=blur_steps {
                        let alpha_frac = (blur_steps + 1 - step) as f32 / (blur_steps + 1) as f32;
                        let step_col   = egui::Color32::from_rgba_unmultiplied(
                            s_col.r(), s_col.g(), s_col.b(),
                            (s_col.a() as f32 * alpha_frac * 0.6) as u8,
                        );
                        let blur_offset = step as f32 * blur_r;
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

                    // Hard shadow pass
                    canvas_painter.galley(
                        text_pos + offset,
                        ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), s_col)),
                        s_col,
                    );
                }

                // ── Stroke / outline ─────────────────────────────────────
                // egui doesn't have native stroke-text, so we simulate it by
                // rendering shifted copies in the stroke colour.
                if sub.stroke_enabled && sub.stroke_width > 0.0 {
                    let sw  = sub.stroke_width * scale_factor;
                    let stc = sub.stroke_color;
                    let stroke_color = egui::Color32::from_rgba_unmultiplied(
                        (stc[0] * 255.0) as u8, (stc[1] * 255.0) as u8,
                        (stc[2] * 255.0) as u8, (stc[3] * state.opacity * 255.0) as u8,
                    );

                    // 8-directional stroke at one step
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

                // ── Main text (gradient or solid) ─────────────────────────
                if sub.gradient_enabled {
                    // Draw with gradient_color on top half, text_color on bottom half
                    // We achieve this by drawing two galleys clipped to top/bottom halves.
                    let gc  = sub.gradient_color;
                    let top_color = egui::Color32::from_rgba_unmultiplied(
                        (gc[0] * 255.0) as u8, (gc[1] * 255.0) as u8,
                        (gc[2] * 255.0) as u8, (gc[3] * state.opacity * 255.0) as u8,
                    );

                    // Top half — gradient colour
                    let top_clip = Rect::from_min_size(
                        text_pos,
                        Vec2::new(text_size.x, text_size.y * 0.5),
                    );
                    canvas_painter.with_clip_rect(top_clip).galley(
                        text_pos,
                        ctx.fonts(|f| f.layout_no_wrap(sub.text.clone(), font_id.clone(), top_color)),
                        top_color,
                    );

                    // Bottom half — fill colour
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

                // ── Selection highlight ───────────────────────────────────
                if vm.selected_id.as_deref() == Some(&sub.id) {
                    let pad = 4.0;
                    let highlight = Rect::from_min_size(
                        Pos2::new(text_pos.x - pad, text_pos.y - 2.0),
                        Vec2::new(text_size.x + pad * 2.0, text_size.y + 4.0),
                    );
                    canvas_painter.rect_stroke(highlight, 3.0, Stroke::new(1.5, ACCENT_CYAN));
                }
            }

            // ── Keyboard shortcuts when canvas is hovered ─────────────────
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