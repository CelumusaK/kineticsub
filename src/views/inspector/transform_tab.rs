use egui::Context;
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::models::types::subtitle::{PathType, PathNode};
use crate::models::types::MediaType;
use crate::views::theme::TEXT_DIM;
use super::widgets::*;

pub fn draw_transform(ui: &mut egui::Ui, vm: &mut EditorViewModel, _ctx: &Context) {
    let mut is_media = false;
    let mut is_audio_track = false;
    
    let (mut px, mut py, mut pscale, mut prot, mut popac, mut pstart, mut pend,
         mut pskewx, mut pskewy, mut pyaw, mut ppitch,
         mut path_type, mut ps_x, mut ps_y, mut porient, mut pprog, mut palign_words,
         mut parent_id, mut exprs, mut physics, mut pvolume, mut pplayback_speed, _ptrim_start,
         mask_center, mask_size, mask_rotation, mask_feather) = {
        if let Some(s) = vm.selected_subtitle() {
            let base = crate::models::types::animation::InterpolatedState {
                x: s.x, y: s.y, scale: s.scale, rotation: s.rotation, opacity: s.opacity,
                skew_x: s.skew_x, skew_y: s.skew_y, yaw: s.yaw, pitch: s.pitch, path_progress: s.path_progress,
                mask_center: s.mask_center, mask_size: s.mask_size, mask_rotation: s.mask_rotation, mask_feather: s.mask_feather,
            };
            let st = crate::models::types::animation::interpolate_keyframes(
                vm.current_time(), s.timeline_start, &s.keyframes, &s.loop_mode, base
            );
            (
                st.x, st.y, st.scale, st.rotation, st.opacity, s.timeline_start, s.timeline_end,
                st.skew_x, st.skew_y, st.yaw, st.pitch,
                s.path_type.clone(), s.path_scale_x, s.path_scale_y, s.path_orient, st.path_progress, s.path_align_words,
                s.parent_id.clone(), s.expressions.clone(), s.physics.clone(), 1.0, 1.0, 0.0,
                st.mask_center, st.mask_size, st.mask_rotation, st.mask_feather
            )
        } else if let Some(m) = vm.selected_media() {
            is_media = true;
            if m.media_type == MediaType::Audio {
                is_audio_track = true;
            }
            let base = crate::models::types::animation::InterpolatedState {
                x: m.x, y: m.y, scale: m.scale, rotation: m.rotation, opacity: m.opacity,
                skew_x: 0.0, skew_y: 0.0, yaw: 0.0, pitch: 0.0, path_progress: 0.0,
                mask_center: [0.0, 0.0], mask_size: [0.0, 0.0], mask_rotation: 0.0, mask_feather: 0.0,
            };
            let st = crate::models::types::animation::interpolate_keyframes(
                vm.current_time(), m.timeline_offset, &m.keyframes, &m.loop_mode, base
            );
            (
                st.x, st.y, st.scale, st.rotation, st.opacity, m.timeline_offset, m.timeline_offset + m.duration,
                0.0, 0.0, 0.0, 0.0,
                PathType::None, 0.0, 0.0, false, 0.0, false,
                None, Default::default(), Default::default(), m.volume, m.playback_speed, m.trim_start,
                [0.0, 0.0], [0.0, 0.0], 0.0, 0.0
            )
        } else {
            return;
        }
    };

    let mut state_changed = false;
    let mut start_changed = false;
    let mut end_changed   = false;
    let mut path_changed  = false;
    let mut parent_changed = false;
    let mut expr_changed  = false;
    let mut phys_changed  = false;
    let mut vol_changed   = false;
    let mut speed_changed = false;
    
    let mut kf_add = false;
    let mut kf_clear = false;

    if !is_audio_track {
        section_label(ui, "ALIGN");

        let mut align_x: Option<f32> = None;
        let mut align_y: Option<f32> = None;

        ui.vertical_centered(|ui| {
            let size = egui::Vec2::new(30.0, 24.0);
            ui.spacing_mut().item_spacing = egui::Vec2::new(4.0, 4.0);
            
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (size.x * 3.0 + 8.0)) / 2.0);
                if ui.add_sized(size, egui::Button::new("↖")).clicked() { align_x = Some(-800.0); align_y = Some(-400.0); }
                if ui.add_sized(size, egui::Button::new("↑")).clicked() { align_x = Some(0.0); align_y = Some(-400.0); }
                if ui.add_sized(size, egui::Button::new("↗")).clicked() { align_x = Some(800.0); align_y = Some(-400.0); }
            });
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (size.x * 3.0 + 8.0)) / 2.0);
                if ui.add_sized(size, egui::Button::new("←")).clicked() { align_x = Some(-800.0); align_y = Some(0.0); }
                if ui.add_sized(size, egui::Button::new("•")).clicked() { align_x = Some(0.0); align_y = Some(0.0); }
                if ui.add_sized(size, egui::Button::new("→")).clicked() { align_x = Some(800.0); align_y = Some(0.0); }
            });
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - (size.x * 3.0 + 8.0)) / 2.0);
                if ui.add_sized(size, egui::Button::new("↙")).clicked() { align_x = Some(-800.0); align_y = Some(400.0); }
                if ui.add_sized(size, egui::Button::new("↓")).clicked() { align_x = Some(0.0); align_y = Some(400.0); }
                if ui.add_sized(size, egui::Button::new("↘")).clicked() { align_x = Some(800.0); align_y = Some(400.0); }
            });
        });

        if align_x.is_some() || align_y.is_some() {
            px = align_x.unwrap_or(px);
            py = align_y.unwrap_or(py);
            state_changed = true;
        }

        ui.add_space(8.0);

        if !is_media {
            section_label(ui, "PARENT & LINK");
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Parent").color(TEXT_DIM).size(10.5));
                
                let all_subs = vm.project.subtitles.clone();
                let my_id = vm.selected_id.clone().unwrap_or_default();
                
                egui::ComboBox::from_id_salt("parent_cmb")
                    .selected_text(parent_id.as_deref().unwrap_or("None"))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut parent_id, None, "None").changed() { parent_changed = true; }
                        for s in all_subs {
                            if s.id != my_id {
                                let short_text = if s.text.len() > 10 { format!("{}...", &s.text[..10]) } else { s.text.clone() };
                                let label = format!("{} ({})", s.id, short_text);
                                if ui.selectable_value(&mut parent_id, Some(s.id), label).changed() { parent_changed = true; }
                            }
                        }
                    });
            });
            ui.add_space(8.0);
        }

        section_label(ui, "POSITION");
        {
            let mut c1 = false; let mut c2 = false;
            let mut a1 = false; let mut cl1 = false;
            let mut a2 = false; let mut cl2 = false;
            two_col_row(ui, |ui| {
                ui.label(egui::RichText::new("X").color(TEXT_DIM).size(10.5));
                let r = ui.add(egui::DragValue::new(&mut px).speed(0.5).suffix(" px").range(-1920.0..=1920.0));
                if r.changed() { c1 = true; }
                r.context_menu(|ui| {
                    if ui.button("Add Keyframe Here").clicked() { a1 = true; ui.close_menu(); }
                    if ui.button("Clear All Keyframes").clicked() { cl1 = true; ui.close_menu(); }
                });
            }, |ui| {
                ui.label(egui::RichText::new("Y").color(TEXT_DIM).size(10.5));
                let r = ui.add(egui::DragValue::new(&mut py).speed(0.5).suffix(" px").range(-1080.0..=1080.0));
                if r.changed() { c2 = true; }
                r.context_menu(|ui| {
                    if ui.button("Add Keyframe Here").clicked() { a2 = true; ui.close_menu(); }
                    if ui.button("Clear All Keyframes").clicked() { cl2 = true; ui.close_menu(); }
                });
            });
            state_changed |= c1 | c2;
            kf_add |= a1 | a2; kf_clear |= cl1 | cl2;
        }

        ui.add_space(4.0);

        section_label(ui, "TRANSFORM");
        {
            let mut c1 = false; let mut c2 = false;
            let mut a1 = false; let mut cl1 = false;
            let mut a2 = false; let mut cl2 = false;
            two_col_row(ui, |ui| {
                ui.label(egui::RichText::new("Scale").color(TEXT_DIM).size(10.5));
                ui.horizontal(|ui| {
                    let r = ui.add(egui::DragValue::new(&mut pscale).speed(0.005).range(0.0..=10.0));
                    if r.changed() { c1 = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { a1 = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { cl1 = true; ui.close_menu(); }
                    });

                    if is_media {
                        if ui.button("1:1").on_hover_text("Reset scale to original native resolution size").clicked() {
                            pscale = 1.0;
                            c1 = true;
                        }
                    }
                });
            }, |ui| {
                ui.label(egui::RichText::new("Rot°").color(TEXT_DIM).size(10.5));
                let r = ui.add(egui::DragValue::new(&mut prot).speed(0.5).suffix("°").range(-360.0..=360.0));
                if r.changed() { c2 = true; }
                r.context_menu(|ui| {
                    if ui.button("Add Keyframe Here").clicked() { a2 = true; ui.close_menu(); }
                    if ui.button("Clear All Keyframes").clicked() { cl2 = true; ui.close_menu(); }
                });
            });
            state_changed |= c1 | c2;
            kf_add |= a1 | a2; kf_clear |= cl1 | cl2;
        }

        ui.add_space(2.0);
        prop_row(ui, "Opacity", |ui| {
            let r = ui.add(egui::Slider::new(&mut popac, 0.0..=1.0).show_value(true));
            if r.changed() { state_changed = true; }
            r.context_menu(|ui| {
                if ui.button("Add Keyframe Here").clicked() { kf_add = true; ui.close_menu(); }
                if ui.button("Clear All Keyframes").clicked() { kf_clear = true; ui.close_menu(); }
            });
        });

        ui.add_space(8.0);
        
        if !is_media {
            collapsible_section(ui, "EXPRESSIONS", &mut true, &mut expr_changed, |ui, changed| {
                ui.label(egui::RichText::new("Use wiggle(freq, amp) or time * val").color(TEXT_DIM).size(9.5));
                ui.add_space(4.0);
                let mut e_c1 = false; let mut e_c2 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("X Expr").color(TEXT_DIM).size(10.0));
                    if ui.text_edit_singleline(&mut exprs.x).changed() { e_c1 = true; }
                }, |ui| {
                    ui.label(egui::RichText::new("Y Expr").color(TEXT_DIM).size(10.0));
                    if ui.text_edit_singleline(&mut exprs.y).changed() { e_c2 = true; }
                });
                *changed |= e_c1 | e_c2;

                let mut e_c3 = false; let mut e_c4 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Scale Expr").color(TEXT_DIM).size(10.0));
                    if ui.text_edit_singleline(&mut exprs.scale).changed() { e_c3 = true; }
                }, |ui| {
                    ui.label(egui::RichText::new("Rot Expr").color(TEXT_DIM).size(10.0));
                    if ui.text_edit_singleline(&mut exprs.rotation).changed() { e_c4 = true; }
                });
                *changed |= e_c3 | e_c4;
            });
            
            ui.add_space(4.0);

            collapsible_section(ui, "PHYSICS & DYNAMICS", &mut physics.enabled, &mut phys_changed, |ui, changed| {
                ui.label(egui::RichText::new("Simulates deterministic falling and bouncing.").color(TEXT_DIM).size(9.5));
                ui.add_space(4.0);
                
                let mut p1 = false; let mut p2 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Gravity").color(TEXT_DIM).size(10.0));
                    if ui.add(egui::DragValue::new(&mut physics.gravity).speed(10.0).range(-5000.0..=5000.0)).changed() { p1 = true; }
                }, |ui| {
                    ui.label(egui::RichText::new("Bounce").color(TEXT_DIM).size(10.0));
                    if ui.add(egui::DragValue::new(&mut physics.bounce).speed(0.05).range(0.0..=2.0)).changed() { p2 = true; }
                });
                *changed |= p1 | p2;

                let mut p3 = false; let mut p4 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Floor Y").color(TEXT_DIM).size(10.0));
                    if ui.add(egui::DragValue::new(&mut physics.floor_y).speed(10.0).range(-2000.0..=2000.0)).changed() { p3 = true; }
                }, |ui| {
                    ui.label(egui::RichText::new("Init Vel Y").color(TEXT_DIM).size(10.0));
                    if ui.add(egui::DragValue::new(&mut physics.initial_velocity_y).speed(10.0)).changed() { p4 = true; }
                });
                *changed |= p3 | p4;
            });

            ui.add_space(8.0);

            section_label(ui, "3D & SKEW (ASS Only)");
            {
                let mut c1 = false; let mut c2 = false;
                let mut a1 = false; let mut cl1 = false;
                let mut a2 = false; let mut cl2 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Yaw° (Y)").color(TEXT_DIM).size(10.5));
                    let r = ui.add(egui::DragValue::new(&mut pyaw).speed(0.5).suffix("°"));
                    if r.changed() { c1 = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { a1 = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { cl1 = true; ui.close_menu(); }
                    });
                }, |ui| {
                    ui.label(egui::RichText::new("Pitch° (X)").color(TEXT_DIM).size(10.5));
                    let r = ui.add(egui::DragValue::new(&mut ppitch).speed(0.5).suffix("°"));
                    if r.changed() { c2 = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { a2 = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { cl2 = true; ui.close_menu(); }
                    });
                });
                state_changed |= c1 | c2;
                kf_add |= a1 | a2; kf_clear |= cl1 | cl2;
            }
            {
                let mut c1 = false; let mut c2 = false;
                let mut a1 = false; let mut cl1 = false;
                let mut a2 = false; let mut cl2 = false;
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Skew X").color(TEXT_DIM).size(10.5));
                    let r = ui.add(egui::DragValue::new(&mut pskewx).speed(0.01));
                    if r.changed() { c1 = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { a1 = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { cl1 = true; ui.close_menu(); }
                    });
                }, |ui| {
                    ui.label(egui::RichText::new("Skew Y").color(TEXT_DIM).size(10.5));
                    let r = ui.add(egui::DragValue::new(&mut pskewy).speed(0.01));
                    if r.changed() { c2 = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { a2 = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { cl2 = true; ui.close_menu(); }
                    });
                });
                state_changed |= c1 | c2;
                kf_add |= a1 | a2; kf_clear |= cl1 | cl2;
            }

            ui.add_space(8.0);

            section_label(ui, "MOTION PATH");
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Type").color(TEXT_DIM).size(10.5));
                egui::ComboBox::from_id_salt("path_type_cmb")
                    .selected_text(format!("{:?}", path_type))
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut path_type, PathType::None, "None").changed() { path_changed = true; }
                        if ui.selectable_value(&mut path_type, PathType::Circle, "Circle").changed() { path_changed = true; }
                        if ui.selectable_value(&mut path_type, PathType::Star, "Star").changed() { path_changed = true; }
                        if ui.selectable_value(&mut path_type, PathType::Custom, "Custom").changed() { path_changed = true; }
                    });
            });

            if path_type != PathType::None {
                let (mut c1, mut c2) = (false, false);
                two_col_row(ui, |ui| {
                    ui.label(egui::RichText::new("Scale X").color(TEXT_DIM).size(10.5));
                    if ui.add(egui::DragValue::new(&mut ps_x).speed(1.0)).changed() { c1 = true; }
                }, |ui| {
                    ui.label(egui::RichText::new("Scale Y").color(TEXT_DIM).size(10.5));
                    if ui.add(egui::DragValue::new(&mut ps_y).speed(1.0)).changed() { c2 = true; }
                });
                path_changed |= c1 | c2;

                if ui.checkbox(&mut porient, "Orient to Path (Fix Rotation)").changed() { path_changed = true; }
                if ui.checkbox(&mut palign_words, "Word-by-Word Alignment").changed() { path_changed = true; }

                prop_row(ui, "Progress", |ui| {
                    let r = ui.add(egui::Slider::new(&mut pprog, 0.0..=1.0));
                    if r.changed() { state_changed = true; }
                    r.context_menu(|ui| {
                        if ui.button("Add Keyframe Here").clicked() { kf_add = true; ui.close_menu(); }
                        if ui.button("Clear All Keyframes").clicked() { kf_clear = true; ui.close_menu(); }
                    });
                });
            }

            ui.add_space(8.0);
        }
    }
    
    if is_media {
        section_label(ui, "MEDIA PROPERTIES");
        prop_row(ui, "Volume", |ui| {
            if ui.add(egui::Slider::new(&mut pvolume, 0.0..=3.0).text("x").show_value(true)).changed() {
                vol_changed = true;
            }
        });
        prop_row(ui, "Speed", |ui| {
            if ui.add(egui::DragValue::new(&mut pplayback_speed).speed(0.05).range(0.1..=10.0)).changed() {
                speed_changed = true;
            }
        });
        ui.add_space(8.0);
    }

    section_label(ui, "TIMING");
    {
        let pend_limit   = pend;
        let pstart_limit = pstart;
        let max_pend_bound = if let Some(m) = vm.selected_media() {
            if m.media_type == MediaType::Image {
                m.timeline_offset + 5.0
            } else {
                let total_dur = m.source_duration.or_else(|| {
                    vm.project.media_files.iter()
                        .find(|other| other.media_type == MediaType::Video && m.path.contains(&other.id))
                        .and_then(|v| v.source_duration)
                });
                if let Some(total) = total_dur.filter(|&d| d > 0.0 && m.color.is_none()) {
                    let speed = m.playback_speed.max(0.001);
                    m.timeline_offset + ((total - m.trim_start) / speed).max(0.1)
                } else {
                    m.timeline_offset + m.duration
                }
            }
        } else {
            3600.0
        };
        let (mut sc1, mut sc2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Start").color(TEXT_DIM).size(10.5));
            if ui.add(egui::DragValue::new(&mut pstart).speed(0.05).suffix("s").range(0.0..=(pend_limit - 0.05))).changed() { sc1 = true; }
        }, |ui| {
            ui.label(egui::RichText::new("End").color(TEXT_DIM).size(10.5));
            if ui.add(egui::DragValue::new(&mut pend).speed(0.05).suffix("s").range((pstart_limit + 0.05)..=max_pend_bound)).changed() { sc2 = true; }
        });
        start_changed |= sc1;
        end_changed   |= sc2;
    }

    let dur = pend - pstart;
    ui.label(egui::RichText::new(format!("Duration: {:.3}s", dur)).color(TEXT_DIM).size(10.0));

    if state_changed || start_changed || end_changed || path_changed || parent_changed || expr_changed || phys_changed || vol_changed || speed_changed {
        let is_recording = vm.keyframe_mode == KeyframeMode::Record;
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        let local_t = vm.current_time() - pstart;

        let st = crate::models::types::animation::InterpolatedState {
            x: px, y: py, scale: pscale, rotation: prot, opacity: popac,
            skew_x: pskewx, skew_y: pskewy, yaw: pyaw, pitch: ppitch,
            path_progress: pprog, mask_center, mask_size, mask_rotation, mask_feather,
        };

        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if state_changed { sub.apply_state_auto_key(local_t, is_recording, &st); }
                if path_changed  { sub.path_type = path_type.clone(); sub.path_scale_x = ps_x; sub.path_scale_y = ps_y; sub.path_orient = porient; sub.path_align_words = palign_words; }
                if start_changed { sub.timeline_start = pstart; }
                if end_changed   { sub.timeline_end = pend; }
                if parent_changed { sub.parent_id = parent_id.clone(); }
                if expr_changed  { sub.expressions = exprs.clone(); }
                if phys_changed  { sub.physics = physics.clone(); }
            }
        }
        
        if speed_changed && pplayback_speed > 0.0 {
            for m in &vm.project.media_files {
                if ids.contains(&m.id) && m.playback_speed > 0.0 {
                    let ratio = m.playback_speed / pplayback_speed;
                    for sub in vm.project.subtitles.iter_mut() {
                        if sub.media_id.as_deref() == Some(&m.id) {
                            let rel_start = (sub.timeline_start - m.timeline_offset) * ratio;
                            let rel_end = (sub.timeline_end - m.timeline_offset) * ratio;
                            sub.timeline_start = m.timeline_offset + rel_start;
                            sub.timeline_end = (m.timeline_offset + rel_end).max(sub.timeline_start + 0.05);
                            for kf in &mut sub.keyframes {
                                kf.time_offset *= ratio;
                            }
                            for w in &mut sub.words {
                                let w_rel_start = (w.start - m.timeline_offset) * ratio;
                                let w_rel_end = (w.end - m.timeline_offset) * ratio;
                                w.start = m.timeline_offset + w_rel_start;
                                w.end = (m.timeline_offset + w_rel_end).max(w.start + 0.01);
                            }
                        }
                    }
                }
            }
        }

        let parent_video_durs: std::collections::HashMap<String, f64> = vm.project.media_files.iter()
            .filter(|m| m.media_type == MediaType::Video)
            .filter_map(|m| m.source_duration.map(|d| (m.id.clone(), d)))
            .collect();

        for m in vm.project.media_files.iter_mut() {
            if ids.contains(&m.id) {
                if state_changed { m.apply_state_auto_key(local_t, is_recording, &st); }
                if vol_changed   { m.volume = pvolume; }
                if speed_changed && pplayback_speed > 0.0 && m.playback_speed > 0.0 { 
                    let ratio = m.playback_speed / pplayback_speed;
                    m.duration *= ratio;
                    for kf in &mut m.keyframes {
                        kf.time_offset *= ratio;
                    }
                    m.playback_speed = pplayback_speed; 
                }

                let total_dur = m.source_duration.or_else(|| {
                    parent_video_durs.iter()
                        .find(|(vid_id, _)| m.path.contains(*vid_id))
                        .map(|(_, &d)| d)
                });

                if start_changed { 
                    m.timeline_offset = pstart; 
                    let mut new_dur = (pend - pstart).max(0.1);
                    if m.media_type == MediaType::Image {
                        new_dur = new_dur.min(5.0);
                    } else if let Some(total) = total_dur.filter(|&d| d > 0.0 && m.color.is_none()) {
                        let max_dur = ((total - m.trim_start) / m.playback_speed.max(0.001)).max(0.1);
                        new_dur = new_dur.min(max_dur);
                    }
                    m.duration = new_dur; 
                }
                if end_changed { 
                    let mut new_dur = (pend - m.timeline_offset).max(0.1);
                    if m.media_type == MediaType::Image {
                        new_dur = new_dur.min(5.0);
                    } else if let Some(total) = total_dur.filter(|&d| d > 0.0 && m.color.is_none()) {
                        let max_dur = ((total - m.trim_start) / m.playback_speed.max(0.001)).max(0.1);
                        new_dur = new_dur.min(max_dur);
                    }
                    m.duration = new_dur; 
                }
            }
        }

        vm.mark_modified();
        if start_changed || end_changed || speed_changed { vm.update_duration(); }
    }

    if kf_add { vm.write_keyframe_now(); }
    if kf_clear { vm.clear_keyframes(); }

    if !is_media && path_type == PathType::Custom {
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        
        if ui.button("+ Add Point").clicked() {
            if let Some(sub) = vm.selected_subtitle_mut() {
                sub.custom_path.push(PathNode { x: 0.0, y: 0.0, smooth: true });
            }
            vm.snapshot();
        }
        
        let mut to_remove = None;
        if let Some(sub) = vm.selected_subtitle_mut() {
            let mut custom_modified = false;
            for (idx, node) in sub.custom_path.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("P{}", idx)).color(TEXT_DIM).size(9.0));
                    if ui.add(egui::DragValue::new(&mut node.x).speed(1.0)).changed() { custom_modified = true; }
                    if ui.add(egui::DragValue::new(&mut node.y).speed(1.0)).changed() { custom_modified = true; }
                    if ui.checkbox(&mut node.smooth, "Smooth").changed() { custom_modified = true; }
                    if ui.small_button("✖").clicked() { to_remove = Some(idx); }
                });
            }
            if let Some(idx) = to_remove {
                sub.custom_path.remove(idx);
                custom_modified = true;
            }
            if custom_modified {
                if vm.keyframe_mode == KeyframeMode::Record {
                    vm.write_keyframe_now();
                } else {
                    vm.mark_modified();
                }
            }
        }
    }
}