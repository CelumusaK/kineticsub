use egui::{Context};
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::models::types::subtitle::{PathType, PathNode};
use crate::views::theme::{TEXT_DIM};
use super::widgets::*;

pub fn draw_transform(ui: &mut egui::Ui, vm: &mut EditorViewModel, _ctx: &Context) {
    let current_time = vm.current_time();
    let kf_mode = vm.keyframe_mode.clone();

    let (has_kf_here, dot_color) = {
        match vm.selected_subtitle() {
            Some(sub) => {
                let lt = current_time - sub.timeline_start;
                let has = sub.has_keyframe_nearby(lt);
                let col = if kf_mode == KeyframeMode::Record {
                    egui::Color32::from_rgb(255, 210, 40)
                } else if has {
                    egui::Color32::from_rgb(80, 220, 80)
                } else {
                    egui::Color32::from_rgb(90, 90, 100)
                };
                (has, col)
            }
            None => return,
        }
    };
    let _ = has_kf_here;

    // ── ALIGN ─────────────────────────────────────────────────────────────────
    section_label(ui, "ALIGN");

    let mut align_x: Option<f32> = None;
    let mut align_y: Option<f32> = None;

    ui.horizontal_wrapped(|ui| {
        ui.add_space(2.0);
        if align_btn(ui, "⬛◻◻", "Align Left Edge").clicked()  { align_x = Some(-800.0); }
        if align_btn(ui, "◻⬛◻", "Center H").clicked()          { align_x = Some(0.0); }
        if align_btn(ui, "◻◻⬛", "Align Right Edge").clicked()  { align_x = Some(800.0); }
        if align_btn(ui, "▀▁▁", "Align Top Edge").clicked()    { align_y = Some(-400.0); }
        if align_btn(ui, "▁▀▁", "Center V").clicked()          { align_y = Some(0.0); }
        if align_btn(ui, "▁▁▀", "Align Bottom Edge").clicked() { align_y = Some(400.0); }
    });
    ui.horizontal_wrapped(|ui| {
        ui.add_space(2.0);
        if align_btn(ui, "↙", "Lower Left").clicked()  { align_x = Some(-700.0); align_y = Some(360.0); }
        if align_btn(ui, "↓", "Bot Center").clicked()  { align_x = Some(0.0);    align_y = Some(360.0); }
        if align_btn(ui, "↘", "Lower Right").clicked() { align_x = Some(700.0);  align_y = Some(360.0); }
        if align_btn(ui, "↑", "Top Center").clicked()  { align_x = Some(0.0);    align_y = Some(-360.0); }
    });

    if align_x.is_some() || align_y.is_some() {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if let Some(v) = align_x { sub.x = v; }
                if let Some(v) = align_y { sub.y = v; }
            }
        }
        if vm.keyframe_mode == KeyframeMode::Record { vm.write_keyframe_now(); }
        vm.snapshot();
    }

    ui.add_space(8.0);

    // ── Hoist all values ──────────────────────────────────────────────────────
    let (mut px, mut py, mut pscale, mut prot, mut popac, mut pstart, mut pend,
         mut pskewx, mut pskewy, mut pyaw, mut ppitch,
         mut path_type, mut ps_x, mut ps_y, mut porient, mut pprog, mut palign_words,
         mut parent_id, mut exprs, mut physics) = {
        match vm.selected_subtitle() {
            Some(s) => (
                s.x, s.y, s.scale, s.rotation, s.opacity, s.timeline_start, s.timeline_end,
                s.skew_x, s.skew_y, s.yaw, s.pitch,
                s.path_type.clone(), s.path_scale_x, s.path_scale_y, s.path_orient, s.path_progress, s.path_align_words,
                s.parent_id.clone(), s.expressions.clone(), s.physics.clone()
            ),
            None    => return,
        }
    };

    let mut pos_changed   = false;
    let mut trs_changed   = false;
    let mut opac_changed  = false;
    let mut start_changed = false;
    let mut end_changed   = false;
    let mut path_changed  = false;
    let mut parent_changed = false;
    let mut expr_changed  = false;
    let mut phys_changed  = false;

    // ── PARENT & LINK (NULL TRACKING) ─────────────────────────────────────────
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

    // ── POSITION ──────────────────────────────────────────────────────────────
    section_label(ui, "POSITION");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("X").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut px).speed(0.5).suffix(" px").range(-1920.0..=1920.0)).changed() { c1 = true; }
        }, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Y").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut py).speed(0.5).suffix(" px").range(-1080.0..=1080.0)).changed() { c2 = true; }
        });
        pos_changed |= c1 | c2;
    }

    ui.add_space(4.0);

    // ── TRANSFORM ─────────────────────────────────────────────────────────────
    section_label(ui, "TRANSFORM");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Scale").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut pscale).speed(0.005).range(0.0..=10.0)).changed() { c1 = true; }
        }, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Rot°").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut prot).speed(0.5).suffix("°").range(-360.0..=360.0)).changed() { c2 = true; }
        });
        trs_changed |= c1 | c2;
    }

    ui.add_space(2.0);
    prop_row(ui, "Opacity", |ui| {
        circle_dot(ui, dot_color);
        if ui.add(egui::Slider::new(&mut popac, 0.0..=1.0).show_value(true)).changed() { opac_changed = true; }
    });

    ui.add_space(8.0);
    
    // ── EXPRESSIONS ───────────────────────────────────────────────────────────
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

    // ── PHYSICS ENGINE ────────────────────────────────────────────────────────
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

    // ── 3D & SKEW ─────────────────────────────────────────────────────────────
    section_label(ui, "3D & SKEW (ASS Only)");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Yaw° (Y)").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut pyaw).speed(0.5).suffix("°")).changed() { c1 = true; }
        }, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Pitch° (X)").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut ppitch).speed(0.5).suffix("°")).changed() { c2 = true; }
        });
        trs_changed |= c1 | c2;
    }
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Skew X").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut pskewx).speed(0.01)).changed() { c1 = true; }
        }, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Skew Y").color(TEXT_DIM).size(10.5));
                circle_dot(ui, dot_color);
            });
            if ui.add(egui::DragValue::new(&mut pskewy).speed(0.01)).changed() { c2 = true; }
        });
        trs_changed |= c1 | c2;
    }

    ui.add_space(8.0);

    // ── MOTION PATH ───────────────────────────────────────────────────────────
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
            circle_dot(ui, dot_color);
            if ui.add(egui::Slider::new(&mut pprog, 0.0..=1.0)).changed() { path_changed = true; }
        });
    }

    ui.add_space(8.0);

    // ── TIMING ────────────────────────────────────────────────────────────────
    section_label(ui, "TIMING");
    {
        let pend_limit   = pend;
        let pstart_limit = pstart;
        let (mut sc1, mut sc2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Start").color(TEXT_DIM).size(10.5));
            if ui.add(egui::DragValue::new(&mut pstart).speed(0.05).suffix("s").range(0.0..=(pend_limit - 0.05))).changed() { sc1 = true; }
        }, |ui| {
            ui.label(egui::RichText::new("End").color(TEXT_DIM).size(10.5));
            if ui.add(egui::DragValue::new(&mut pend).speed(0.05).suffix("s").range((pstart_limit + 0.05)..=3600.0)).changed() { sc2 = true; }
        });
        start_changed |= sc1;
        end_changed   |= sc2;
    }

    let dur = pend - pstart;
    ui.label(egui::RichText::new(format!("Duration: {:.3}s", dur)).color(TEXT_DIM).size(10.0));

    // ── Write back safely to MULTIPLE SELECT ──────────────────────────────────
    if pos_changed || trs_changed || opac_changed || start_changed || end_changed || path_changed || parent_changed || expr_changed || phys_changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if pos_changed   { sub.x = px; sub.y = py; }
                if trs_changed   { sub.scale = pscale; sub.rotation = prot; sub.skew_x = pskewx; sub.skew_y = pskewy; sub.yaw = pyaw; sub.pitch = ppitch; }
                if opac_changed  { sub.opacity = popac; }
                if path_changed  { sub.path_type = path_type.clone(); sub.path_scale_x = ps_x; sub.path_scale_y = ps_y; sub.path_orient = porient; sub.path_progress = pprog; sub.path_align_words = palign_words; }
                if start_changed { sub.timeline_start = pstart; }
                if end_changed   { sub.timeline_end = pend; }
                if parent_changed { sub.parent_id = parent_id.clone(); }
                if expr_changed  { sub.expressions = exprs.clone(); }
                if phys_changed  { sub.physics = physics.clone(); }
            }
        }
        vm.mark_modified();
        if start_changed || end_changed { vm.update_duration(); }
    }

    if (pos_changed || trs_changed || opac_changed || path_changed) && vm.keyframe_mode == KeyframeMode::Record {
        vm.write_keyframe_now();
    }

    // ── Custom Path Point Editing ─────────────────────────────────────────────
    if path_type == PathType::Custom {
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