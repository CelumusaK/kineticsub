use egui::{Pos2, Rect, Vec2, Color32, Stroke};
use crate::models::types::{MediaFile, MediaType, TransitionType};
use crate::viewmodels::editor_vm::EditorViewModel;

pub fn draw(painter: &egui::Painter, preview_rect: Rect, vm: &mut EditorViewModel, ctx: &egui::Context) {
    let current_t = vm.current_time();

    // Check if current playhead is inside any active transition
    let active_trans = vm.project.transitions.iter()
        .find(|t| current_t >= t.timeline_start && current_t < t.timeline_start + t.duration && vm.is_transition_valid(t))
        .cloned();

    let scale_factor = preview_rect.width() / vm.project.resolution.0 as f32;

    if let Some(trans) = active_trans {
        let p = ((current_t - trans.timeline_start) / trans.duration.max(0.01)).clamp(0.0, 1.0) as f32;
        let mid_point = trans.timeline_start + trans.duration * 0.5;

        // Collect outgoing clips (ending near midpoint) and incoming clips (starting near midpoint)
        let outgoing_clips: Vec<_> = vm.project.media_files.iter()
            .filter(|m| m.on_timeline && m.media_type != MediaType::Audio && (m.timeline_offset + m.duration - mid_point).abs() <= trans.duration * 0.6)
            .cloned().collect();

        let incoming_clips: Vec<_> = vm.project.media_files.iter()
            .filter(|m| m.on_timeline && m.media_type != MediaType::Audio && (m.timeline_offset - mid_point).abs() <= trans.duration * 0.6)
            .cloned().collect();

        match trans.transition_type {
            TransitionType::Crossfade => {
                // Render outgoing clips with fading opacity (1.0 -> 0.0)
                for out_clip in &outgoing_clips {
                    render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, 1.0 - p);
                }
                // Render incoming clips with rising opacity (0.0 -> 1.0)
                for in_clip in &incoming_clips {
                    render_single_media(painter, preview_rect, vm, ctx, in_clip, current_t, scale_factor, p);
                }
            }
            TransitionType::DipToBlack => {
                if p < 0.5 {
                    let fade = (1.0 - p * 2.0).clamp(0.0, 1.0);
                    for out_clip in &outgoing_clips {
                        render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, fade);
                    }
                    let black_alpha = ((p * 2.0) * 255.0) as u8;
                    painter.rect_filled(preview_rect, 0.0, Color32::from_black_alpha(black_alpha));
                } else {
                    let fade = ((p - 0.5) * 2.0).clamp(0.0, 1.0);
                    for in_clip in &incoming_clips {
                        render_single_media(painter, preview_rect, vm, ctx, in_clip, current_t, scale_factor, fade);
                    }
                    let black_alpha = ((1.0 - (p - 0.5) * 2.0) * 255.0) as u8;
                    painter.rect_filled(preview_rect, 0.0, Color32::from_black_alpha(black_alpha));
                }
            }
            TransitionType::DipToWhite => {
                if p < 0.5 {
                    let fade = (1.0 - p * 2.0).clamp(0.0, 1.0);
                    for out_clip in &outgoing_clips {
                        render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, fade);
                    }
                    let white_alpha = ((p * 2.0) * 255.0) as u8;
                    painter.rect_filled(preview_rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 255, white_alpha));
                } else {
                    let fade = ((p - 0.5) * 2.0).clamp(0.0, 1.0);
                    for in_clip in &incoming_clips {
                        render_single_media(painter, preview_rect, vm, ctx, in_clip, current_t, scale_factor, fade);
                    }
                    let white_alpha = ((1.0 - (p - 0.5) * 2.0) * 255.0) as u8;
                    painter.rect_filled(preview_rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 255, white_alpha));
                }
            }
            TransitionType::WipeLeft => {
                for out_clip in &outgoing_clips {
                    render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, 1.0);
                }
                let split_x = preview_rect.max.x - preview_rect.width() * p;
                let clip_in_rect = Rect::from_min_max(Pos2::new(split_x, preview_rect.min.y), preview_rect.max);
                let p_in = painter.with_clip_rect(clip_in_rect.intersect(preview_rect));
                for in_clip in &incoming_clips {
                    render_single_media(&p_in, preview_rect, vm, ctx, in_clip, current_t, scale_factor, 1.0);
                }
            }
            TransitionType::WipeRight => {
                for out_clip in &outgoing_clips {
                    render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, 1.0);
                }
                let split_x = preview_rect.min.x + preview_rect.width() * p;
                let clip_in_rect = Rect::from_min_max(preview_rect.min, Pos2::new(split_x, preview_rect.max.y));
                let p_in = painter.with_clip_rect(clip_in_rect.intersect(preview_rect));
                for in_clip in &incoming_clips {
                    render_single_media(&p_in, preview_rect, vm, ctx, in_clip, current_t, scale_factor, 1.0);
                }
            }
            TransitionType::SlideLeft => {
                for out_clip in &outgoing_clips {
                    render_single_media(painter, preview_rect, vm, ctx, out_clip, current_t, scale_factor, 1.0);
                }
                let offset_x = preview_rect.width() * (1.0 - p);
                let slide_rect = preview_rect.translate(Vec2::new(offset_x, 0.0));
                let p_in = painter.with_clip_rect(preview_rect);
                for in_clip in &incoming_clips {
                    render_single_media(&p_in, slide_rect, vm, ctx, in_clip, current_t, scale_factor, 1.0);
                }
            }
        }
        return;
    }

    // Standard Timeline Rendering
    let mut visible_visuals: Vec<_> = vm.project.media_files.iter().filter(|m| {
        m.on_timeline 
        && m.media_type != MediaType::Audio 
        && current_t >= m.timeline_offset 
        && current_t < m.timeline_offset + m.duration
    }).cloned().collect();

    visible_visuals.sort_by(|a, b| vm.get_track_row(b.track_index, false).cmp(&vm.get_track_row(a.track_index, false)));

    if visible_visuals.is_empty() {
        let cell_size = 16.0;
        let cols = (preview_rect.width() / cell_size).ceil() as usize;
        let rows = (preview_rect.height() / cell_size).ceil() as usize;
        
        for r in 0..rows {
            for c in 0..cols {
                let color = if (r + c) % 2 == 0 {
                    Color32::from_rgb(60, 60, 60)
                } else {
                    Color32::from_rgb(40, 40, 40)
                };
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
        for bg in &visible_visuals {
            render_single_media(painter, preview_rect, vm, ctx, bg, current_t, scale_factor, 1.0);
        }
    }
}

fn render_single_media(
    painter: &egui::Painter,
    preview_rect: Rect,
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

    let cx = preview_rect.center().x + state.x * scale_factor;
    let cy = preview_rect.center().y + state.y * scale_factor;
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
        painter.with_clip_rect(preview_rect).add(mesh);
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
        painter.with_clip_rect(preview_rect).add(egui::Shape::convex_polygon(rot_pts, egui_col, Stroke::NONE));
        
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