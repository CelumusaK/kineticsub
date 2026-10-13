use egui::{Pos2, Rect, Vec2, Color32, Stroke};
use crate::models::types::{MediaFile, MediaType, TransitionType};
use crate::viewmodels::editor_vm::EditorViewModel;

pub fn draw(painter: &egui::Painter, preview_rect: Rect, vm: &mut EditorViewModel, ctx: &egui::Context) {
    let current_t = vm.current_time();
    let scale_factor = preview_rect.width() / vm.project.resolution.0 as f32;

    let mut active_visuals = Vec::new();
    let mut dip_to_black_amount = 0.0;
    let mut dip_to_white_amount = 0.0;

    for m in &vm.project.media_files {
        if m.on_timeline && m.media_type != MediaType::Audio {
            let is_visible = current_t >= m.timeline_offset && current_t < m.timeline_offset + m.duration;
            
            let mut applied_trans = None;

            for t in &vm.project.transitions {
                if current_t >= t.timeline_start && current_t < t.timeline_start + t.duration {
                    if t.track_index == 0 || t.track_index == m.track_index {
                        // Is m an outgoing clip?
                        let is_out = m.timeline_offset + m.duration >= t.timeline_start - 0.1 && m.timeline_offset + m.duration <= t.timeline_start + t.duration + 0.1;
                        // Is m an incoming clip?
                        let is_in = m.timeline_offset >= t.timeline_start - 0.1 && m.timeline_offset <= t.timeline_start + t.duration + 0.1;

                        if is_out || is_in {
                            let p = ((current_t - t.timeline_start) / t.duration.max(0.01)).clamp(0.0, 1.0) as f32;
                            applied_trans = Some((t.transition_type.clone(), is_out, is_in, p));
                            
                            if t.transition_type == TransitionType::DipToBlack {
                                let amt = if p < 0.5 { p * 2.0 } else { 1.0 - (p - 0.5) * 2.0 };
                                if amt > dip_to_black_amount { dip_to_black_amount = amt; }
                            }
                            if t.transition_type == TransitionType::DipToWhite {
                                let amt = if p < 0.5 { p * 2.0 } else { 1.0 - (p - 0.5) * 2.0 };
                                if amt > dip_to_white_amount { dip_to_white_amount = amt; }
                            }
                            break;
                        }
                    }
                }
            }

            if is_visible || applied_trans.is_some() {
                active_visuals.push((m.clone(), applied_trans));
            }
        }
    }

    active_visuals.sort_by(|a, b| vm.get_track_row(b.0.track_index, false).cmp(&vm.get_track_row(a.0.track_index, false)));

    if active_visuals.is_empty() {
        let cell_size = 16.0;
        let cols = (preview_rect.width() / cell_size).ceil() as usize;
        let rows = (preview_rect.height() / cell_size).ceil() as usize;
        
        for r in 0..rows {
            for c in 0..cols {
                let color = if (r + c) % 2 == 0 { Color32::from_rgb(60, 60, 60) } else { Color32::from_rgb(40, 40, 40) };
                let x = preview_rect.min.x + c as f32 * cell_size;
                let y = preview_rect.min.y + r as f32 * cell_size;
                let rect = Rect::from_min_size(
                    Pos2::new(x, y),
                    Vec2::new(cell_size, cell_size)
                ).intersect(preview_rect);
                painter.rect_filled(rect, 0.0, color);
            }
        }
    } else {
        for (bg, trans_mod) in &active_visuals {
            let mut alpha_mult = 1.0;
            let mut custom_clip = preview_rect;
            let mut offset_px = Vec2::ZERO;

            if let Some((t_type, is_out, is_in, p)) = trans_mod {
                match t_type {
                    TransitionType::Crossfade | TransitionType::DipToBlack | TransitionType::DipToWhite => {
                        if *is_out { alpha_mult = 1.0 - p; }
                        else if *is_in { alpha_mult = *p; }
                    }
                    TransitionType::WipeLeft => {
                        if *is_in {
                            let split_x = preview_rect.max.x - preview_rect.width() * p;
                            custom_clip = Rect::from_min_max(Pos2::new(split_x, preview_rect.min.y), preview_rect.max);
                        }
                    }
                    TransitionType::WipeRight => {
                        if *is_in {
                            let split_x = preview_rect.min.x + preview_rect.width() * p;
                            custom_clip = Rect::from_min_max(preview_rect.min, Pos2::new(split_x, preview_rect.max.y));
                        }
                    }
                    TransitionType::SlideLeft => {
                        if *is_in { offset_px.x = preview_rect.width() * (1.0 - p); }
                        if *is_out { offset_px.x = -preview_rect.width() * p; }
                    }
                }
            }

            let p_clipped = painter.with_clip_rect(custom_clip.intersect(preview_rect));
            render_single_media(&p_clipped, preview_rect, offset_px, vm, ctx, bg, current_t, scale_factor, alpha_mult);
        }
    }

    if dip_to_black_amount > 0.0 {
        painter.rect_filled(preview_rect, 0.0, Color32::from_black_alpha((dip_to_black_amount * 255.0) as u8));
    }
    if dip_to_white_amount > 0.0 {
        painter.rect_filled(preview_rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 255, (dip_to_white_amount * 255.0) as u8));
    }
}

fn render_single_media(
    painter: &egui::Painter,
    preview_rect: Rect,
    offset_px: Vec2,
    vm: &mut EditorViewModel,
    ctx: &egui::Context,
    bg: &MediaFile,
    current_t: f64,
    scale_factor: f32,
    alpha_multiplier: f32,
) {
    let state = bg.get_interpolated_state(current_t);
    let final_opacity = (state.opacity * alpha_multiplier).clamp(0.0, 1.0);
    if final_opacity <= 0.005 { return; }

    let cx = preview_rect.center().x + state.x * scale_factor + offset_px.x;
    let cy = preview_rect.center().y + state.y * scale_factor + offset_px.y;
    let rot = egui::emath::Rot2::from_angle(state.rotation.to_radians());
    let color = Color32::from_white_alpha((final_opacity * 255.0) as u8);

    let draw_mesh = |tex_id: egui::TextureId, out_w: f32, out_h: f32, color_override: Option<Color32>| {
        let hw = out_w / 2.0;
        let hh = out_h / 2.0;
        let center = Pos2::new(cx, cy);
        let pts = [ Pos2::new(-hw, -hh), Pos2::new(hw, -hh), Pos2::new(hw, hh), Pos2::new(-hw, hh) ];
        let uvs = [ Pos2::new(0.0, 0.0), Pos2::new(1.0, 0.0), Pos2::new(1.0, 1.0), Pos2::new(0.0, 1.0) ];
        
        let mut mesh = egui::Mesh::with_texture(tex_id);
        let col = color_override.unwrap_or(color);
        for i in 0..4 {
            let p = rot * pts[i].to_vec2();
            mesh.vertices.push(egui::epaint::Vertex {
                pos: center + p,
                uv: uvs[i],
                color: col,
            });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        painter.add(mesh);
    };

    if let Some(col) = bg.color {
        let egui_col = Color32::from_rgba_unmultiplied(
            (col[0]*255.0) as u8, (col[1]*255.0) as u8, (col[2]*255.0) as u8, (col[3]*final_opacity*255.0) as u8
        );
        let out_w = preview_rect.width() * state.scale;
        let out_h = preview_rect.height() * state.scale;
        
        let hw = out_w / 2.0;
        let hh = out_h / 2.0;
        let center = Pos2::new(cx, cy);
        let pts = [ Pos2::new(-hw, -hh), Pos2::new(hw, -hh), Pos2::new(hw, hh), Pos2::new(-hw, hh) ];
        let rot_pts: Vec<Pos2> = pts.iter().map(|p| center + rot * p.to_vec2()).collect();
        painter.add(egui::Shape::convex_polygon(rot_pts, egui_col, Stroke::NONE));
        
    } else if bg.media_type == MediaType::Image {
        if let Some(tex) = vm.get_or_load_texture(ctx, &bg.path) {
            let img_w = tex.size()[0] as f32;
            let img_h = tex.size()[1] as f32;
            let aspect = img_w / img_h;
            
            let out_w = preview_rect.width().min(preview_rect.height() * aspect) * state.scale;
            let out_h = out_w / aspect;
            
            draw_mesh(tex.id(), out_w, out_h, None);
        }
    } else if bg.media_type == MediaType::Video {
        let local_t = (current_t - bg.timeline_offset).clamp(0.0, bg.duration);
        let mut media_t = bg.trim_start + local_t * bg.playback_speed;
        if let Some(total) = bg.source_duration {
            media_t = media_t.min((total - 0.001).max(0.0));
        }
        let frame_number = (media_t * vm.project.fps as f64).floor() as usize + 1;
        let frame_name = format!("frame_{:04}.jpg", frame_number);

        let cache_dir = bg.preview_dir.as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("kineticsub_cache").join(&bg.id));

        let frame_path = cache_dir.join(&frame_name);
        let first_frame = cache_dir.join("frame_0001.jpg");

        if bg.preview_ready || first_frame.exists() {
            if let Some(tex) = vm.get_or_load_texture(ctx, frame_path.to_str().unwrap_or("")) {
                let img_w = tex.size()[0] as f32;
                let img_h = tex.size()[1] as f32;
                let aspect = img_w / img_h;
                
                let out_w = preview_rect.width().min(preview_rect.height() * aspect) * state.scale;
                let out_h = out_w / aspect;
                
                draw_mesh(tex.id(), out_w, out_h, None);
            }
        }
    }
}