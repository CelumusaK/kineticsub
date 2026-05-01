use crate::models::types::{Project, Subtitle, InterpolatedState, MaskType, WordAnimation};
use super::helpers::{format_time, color_to_ass, alpha_to_ass};

pub fn generate_ass_baked(project: &Project, fps: f64, res_w: u32, res_h: u32) -> String {
    let mut out = String::new();
    out.push_str("[Script Info]\n");
    out.push_str("ScriptType: v4.00+\n");
    out.push_str(&format!("PlayResX: {}\n", res_w));
    out.push_str(&format!("PlayResY: {}\n", res_h));
    out.push_str("WrapStyle: 0\n");
    out.push_str("ScaledBorderAndShadow: yes\n\n");
    
    out.push_str("[V4+ Styles]\n");
    out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    out.push_str("Style: Default,Arial,36,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,5,0,0,0,1\n\n");
    
    out.push_str("[Events]\n");
    out.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    
    let dt = 1.0 / fps;
    let total_frames = (project.duration * fps).ceil() as usize;

    for frame in 0..=total_frames {
        let t_start = frame as f64 * dt;
        let t_end   = t_start + dt + 0.01;

        for sub in &project.subtitles {
            if t_end > sub.timeline_start && t_start < sub.timeline_end {
                // Supply all_subs to properly bake recursive Parents and Math Simulations per frame
                let mut state = sub.get_interpolated_state(t_start, &project.subtitles, 0);
                
                let (px, py, pang) = sub.evaluate_path(state.path_progress);
                state.x += px;
                state.y += py;
                state.rotation += if sub.path_orient { pang.to_degrees() } else { 0.0 };

                if state.opacity > 0.01 {
                    let mut lines_to_write = Vec::new();
                    
                    let mut stage_pass = |_b_t: f64, mut b_s: InterpolatedState, m_op: f32, f_exp: f32| {
                        let (b_px, b_py, b_pang) = sub.evaluate_path(b_s.path_progress);
                        b_s.x += b_px; b_s.y += b_py;
                        b_s.rotation += if sub.path_orient { b_pang.to_degrees() } else { 0.0 };
                        b_s.opacity *= m_op;

                        if b_s.opacity > 0.01 {
                            let b_tags = build_static_ass_tags(sub, &b_s, res_w, res_h, f_exp);
                            
                            let has_words = !sub.words.is_empty() && sub.words.len() == sub.text.split_whitespace().count();
                            let mut formatted_text = String::new();

                            if has_words {
                                for (i, word) in sub.words.iter().enumerate() {
                                    let is_active = t_start >= word.start && t_start <= word.end;
                                    let mut w_color = sub.color;
                                    let mut w_scale = 1.0;
                                    let mut w_alpha = b_s.opacity;
                                    
                                    match &sub.word_animation {
                                        WordAnimation::KaraokeHighlight { color } => { if is_active { w_color = *color; } }
                                        WordAnimation::KaraokePop { scale } => { if is_active { w_scale = *scale; w_color =[1.0, 1.0, 0.0, 1.0]; } }
                                        WordAnimation::CascadeFade => {
                                            if t_start < word.start { w_alpha = 0.0; }
                                            else if is_active { w_alpha = b_s.opacity * ((t_start - word.start) / (word.end - word.start).max(0.01)) as f32; }
                                        }
                                        WordAnimation::None => {}
                                    }
                                    
                                    if let Some(c) = word.custom_color { w_color = c; }
                                    
                                    let alpha_tag = alpha_to_ass(w_alpha);
                                    let color_tag = color_to_ass(&w_color);
                                    let scale_tag_x = b_s.scale * 100.0 * w_scale;
                                    let scale_tag_y = b_s.scale * 100.0 * w_scale;
                                    
                                    formatted_text.push_str(&format!("{{\\alpha{}\\c{}\\fscx{:.1}\\fscy{:.1}}}{}", alpha_tag, color_tag, scale_tag_x, scale_tag_y, word.text));
                                    if i < sub.words.len() - 1 { formatted_text.push_str(" "); }
                                }
                            } else {
                                formatted_text = sub.text.replace("\n", "\\N");
                            }

                            // Output sequence: Bloom -> Strokes -> Base -> Glitch Split
                            let base_line = format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", format_time(t_start), format_time(t_end), b_tags, formatted_text);
                            
                            // 1. Bloom Passes
                            if sub.bloom.enabled {
                                let bloom_passes = 3;
                                for i in 1..=bloom_passes {
                                    let blur_amt = (sub.bloom.radius / bloom_passes as f32) * i as f32;
                                    let alpha_factor = 1.0 - (i as f32 / bloom_passes as f32);
                                    let b_alpha = b_s.opacity * sub.bloom.intensity * alpha_factor * 0.5;
                                    
                                    let mut bloom_tags = b_tags.clone();
                                    bloom_tags = bloom_tags.replace(&format!("\\alpha{}", alpha_to_ass(b_s.opacity)), &format!("\\alpha{}", alpha_to_ass(b_alpha)));
                                    bloom_tags = bloom_tags.replace(&format!("\\c{}", color_to_ass(&sub.color)), &format!("\\c{}", color_to_ass(&sub.bloom.color)));
                                    bloom_tags.push_str(&format!("\\blur{:.1}\\bord0\\shad0", blur_amt));
                                    
                                    lines_to_write.push(format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", format_time(t_start), format_time(t_end), bloom_tags, formatted_text));
                                }
                            }

                            // 2. Multiple Strokes
                            if sub.additional_strokes_enabled {
                                let mut sorted_strokes = sub.additional_strokes.clone();
                                sorted_strokes.sort_by(|a, b| b.width.partial_cmp(&a.width).unwrap());
                                for stroke in sorted_strokes {
                                    if stroke.enabled {
                                        let mut stroke_tags = b_tags.clone();
                                        stroke_tags = stroke_tags.replace(&format!("\\c{}", color_to_ass(&sub.color)), &format!("\\c{}", color_to_ass(&stroke.color)));
                                        stroke_tags.push_str(&format!("\\bord{:.1}\\3c{}\\shad0", stroke.width * b_s.scale, color_to_ass(&stroke.color)));
                                        lines_to_write.push(format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", format_time(t_start), format_time(t_end), stroke_tags, formatted_text));
                                    }
                                }
                            }

                            // 3. Main Text Line
                            lines_to_write.push(base_line);

                            // 4. Glitch RGB Split
                            if sub.glitch.enabled && sub.glitch.rgb_split > 0.0 {
                                let split_amt = sub.glitch.rgb_split;
                                let r_tags = b_tags.replace(&format!("\\c{}", color_to_ass(&sub.color)), "\\c&H0000FF&").replace(&format!("\\alpha{}", alpha_to_ass(b_s.opacity)), &format!("\\alpha{}", alpha_to_ass(b_s.opacity * 0.7)));
                                let b_tags_split = b_tags.replace(&format!("\\c{}", color_to_ass(&sub.color)), "\\c&HFFFF00&").replace(&format!("\\alpha{}", alpha_to_ass(b_s.opacity)), &format!("\\alpha{}", alpha_to_ass(b_s.opacity * 0.7)));
                                
                                let base_x = (res_w as f32) / 2.0 + b_s.x;
                                let base_y = (res_h as f32) / 2.0 + b_s.y;
                                let r_pos_str = format!("\\pos({:.1},{:.1})", base_x - split_amt, base_y);
                                let b_pos_str = format!("\\pos({:.1},{:.1})", base_x + split_amt, base_y);
                                
                                let r_tags_final = r_tags.replace(&format!("\\pos({:.1},{:.1})", base_x, base_y), &r_pos_str);
                                let b_tags_final = b_tags_split.replace(&format!("\\pos({:.1},{:.1})", base_x, base_y), &b_pos_str);

                                lines_to_write.push(format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", format_time(t_start), format_time(t_end), r_tags_final, formatted_text));
                                lines_to_write.push(format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", format_time(t_start), format_time(t_end), b_tags_final, formatted_text));
                            }
                        }
                    };

                    let blur_steps = if sub.motion_blur > 0.0 { 3 } else { 0 };
                    let feather_steps = if sub.mask_feather > 0.0 && sub.mask_type != MaskType::None { 4 } else { 0 };

                    for b_step in (0..=blur_steps).rev() {
                        let blur_t = t_start - (b_step as f64 * 0.02 * sub.motion_blur as f64);
                        if blur_t < sub.timeline_start && b_step > 0 { continue; }
                        
                        let base_s = sub.get_interpolated_state(blur_t, &project.subtitles, 0);
                        let base_op = if b_step > 0 { 0.25 } else { 1.0 };
                        
                        if feather_steps > 0 {
                            for f_step in (0..=feather_steps).rev() {
                                let f_pct = f_step as f32 / feather_steps as f32;
                                let f_expand = f_pct * sub.mask_feather;
                                let op_mult = base_op * (1.0 - f_pct).powi(2);
                                
                                if f_step == 0 { stage_pass(blur_t, base_s.clone(), base_op, 0.0); }
                                else { stage_pass(blur_t, base_s.clone(), op_mult, f_expand); }
                            }
                        } else {
                            stage_pass(blur_t, base_s, base_op, 0.0);
                        }
                    }

                    for l in lines_to_write {
                        out.push_str(&l);
                    }
                }
            }
        }
    }
    out
}

fn build_static_ass_tags(sub: &Subtitle, state: &InterpolatedState, res_w: u32, res_h: u32, feather_expand: f32) -> String {
    let mut t = String::new();
    t.push_str("\\an5"); 
    
    let base_x = (res_w as f32) / 2.0 + state.x;
    let base_y = (res_h as f32) / 2.0 + state.y;
    t.push_str(&format!("\\pos({:.1},{:.1})", base_x, base_y));
    t.push_str(&format!("\\fs{:.1}", sub.font_size));
    
    t.push_str(&format!("\\fscx{:.1}\\fscy{:.1}", state.scale * 100.0, state.scale * 100.0));
    
    t.push_str(&format!("\\frz{:.1}", -state.rotation));
    if state.pitch != 0.0 { t.push_str(&format!("\\frx{:.1}", state.pitch)); }
    if state.yaw != 0.0 { t.push_str(&format!("\\fry{:.1}", state.yaw)); }
    if state.skew_x != 0.0 { t.push_str(&format!("\\fax{:.2}", state.skew_x)); }
    if state.skew_y != 0.0 { t.push_str(&format!("\\fay{:.2}", state.skew_y)); }
    
    t.push_str(&format!("\\alpha{}", alpha_to_ass(state.opacity)));

    if sub.bold { t.push_str("\\b1"); }
    if sub.italic { t.push_str("\\i1"); }
    t.push_str(&format!("\\c{}", color_to_ass(&sub.color)));
    
    if sub.stroke_enabled {
        t.push_str(&format!("\\bord{:.1}", sub.stroke_width * state.scale));
        t.push_str(&format!("\\3c{}", color_to_ass(&sub.stroke_color)));
    } else {
        t.push_str("\\bord0");
    }
    
    if sub.shadow_enabled {
        t.push_str(&format!("\\shad{:.1}", sub.shadow_offset[0].abs().max(sub.shadow_offset[1].abs()) * state.scale));
        t.push_str(&format!("\\4c{}", color_to_ass(&sub.shadow_color)));
    } else {
        t.push_str("\\shad0");
    }

    if sub.mask_type != MaskType::None {
        let tag_name = if sub.mask_invert { "\\iclip" } else { "\\clip" };
        let mx = base_x + state.mask_center[0];
        let my = base_y + state.mask_center[1];
        let rot = state.mask_rotation.to_radians();

        let rot_pt = |dx: f32, dy: f32| -> (f32, f32) {
            let rx = dx * rot.cos() - dy * rot.sin();
            let ry = dx * rot.sin() + dy * rot.cos();
            (mx + rx, my + ry)
        };

        let draw_polygon = |pts: Vec<(f32, f32)>| -> String {
            let mut s = String::new();
            s.push_str("m ");
            for (i, p) in pts.iter().enumerate() {
                if i == 1 { s.push_str("l "); }
                s.push_str(&format!("{:.1} {:.1} ", p.0, p.1));
            }
            s
        };

        match sub.mask_type {
            MaskType::Rectangle => {
                let hw = state.mask_size[0] / 2.0 + feather_expand;
                let hh = state.mask_size[1] / 2.0 + feather_expand;
                let pts = vec![rot_pt(-hw, -hh), rot_pt(hw, -hh), rot_pt(hw, hh), rot_pt(-hw, hh)];
                t.push_str(&format!("{}(1,{})", tag_name, draw_polygon(pts)));
            }
            MaskType::Straight => {
                let dist = 10000.0;
                let ex = feather_expand;
                let pts = vec![rot_pt(-dist, -ex), rot_pt(dist, -ex), rot_pt(dist, dist), rot_pt(-dist, dist)];
                t.push_str(&format!("{}(1,{})", tag_name, draw_polygon(pts)));
            }
            MaskType::Circle => {
                let r = state.mask_size[0].max(0.1) + feather_expand;
                let mut pts = vec![];
                let segments = 16;
                for i in 0..segments {
                    let a = (i as f32 / segments as f32) * std::f32::consts::TAU;
                    pts.push((mx + a.cos() * r, my + a.sin() * r));
                }
                t.push_str(&format!("{}(1,{})", tag_name, draw_polygon(pts)));
            }
            _ => {}
        }
    }
    t
}