// File: src/views/inspector/transform_tab.rs
use egui::{Context};
use crate::viewmodels::editor_vm::{EditorViewModel, KeyframeMode};
use crate::views::theme::{TEXT_DIM};
use super::widgets::*;

pub fn draw_transform(ui: &mut egui::Ui, vm: &mut EditorViewModel, ctx: &Context) {
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

    ui.horizontal(|ui| {
        ui.add_space(2.0);
        if align_btn(ui, "⬛◻◻", "Align Left Edge").clicked()  { align_x = Some(-800.0); }
        if align_btn(ui, "◻⬛◻", "Center H").clicked()          { align_x = Some(0.0); }
        if align_btn(ui, "◻◻⬛", "Align Right Edge").clicked()  { align_x = Some(800.0); }
        ui.add_space(6.0);
        if align_btn(ui, "▀▁▁", "Align Top Edge").clicked()    { align_y = Some(-400.0); }
        if align_btn(ui, "▁▀▁", "Center V").clicked()          { align_y = Some(0.0); }
        if align_btn(ui, "▁▁▀", "Align Bottom Edge").clicked() { align_y = Some(400.0); }
    });
    ui.horizontal(|ui| {
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
    }

    ui.add_space(8.0);

    // ── Hoist all values ──────────────────────────────────────────────────────
    let (mut px, mut py, mut pscale, mut prot, mut popac, mut pstart, mut pend) = {
        match vm.selected_subtitle() {
            Some(s) => (s.x, s.y, s.scale, s.rotation, s.opacity, s.timeline_start, s.timeline_end),
            None    => return,
        }
    };

    let mut pos_changed   = false;
    let mut trs_changed   = false;
    let mut opac_changed  = false;
    let mut start_changed = false;
    let mut end_changed   = false;

    // ── POSITION ──────────────────────────────────────────────────────────────
    section_label(ui, "POSITION");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("X").color(TEXT_DIM).size(10.5));
            circle_dot(ui, dot_color);
            c1 = ui.add(egui::DragValue::new(&mut px).speed(0.5).suffix(" px").range(-1920.0..=1920.0)).changed();
        }, |ui| {
            ui.label(egui::RichText::new("Y").color(TEXT_DIM).size(10.5));
            circle_dot(ui, dot_color);
            c2 = ui.add(egui::DragValue::new(&mut py).speed(0.5).suffix(" px").range(-1080.0..=1080.0)).changed();
        });
        pos_changed |= c1 | c2;
    }

    ui.add_space(4.0);

    // ── TRANSFORM ─────────────────────────────────────────────────────────────
    section_label(ui, "TRANSFORM");
    {
        let (mut c1, mut c2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Scale").color(TEXT_DIM).size(10.5));
            circle_dot(ui, dot_color);
            c1 = ui.add(egui::DragValue::new(&mut pscale).speed(0.005).range(0.0..=10.0)).changed();
        }, |ui| {
            ui.label(egui::RichText::new("Rot°").color(TEXT_DIM).size(10.5));
            circle_dot(ui, dot_color);
            c2 = ui.add(egui::DragValue::new(&mut prot).speed(0.5).suffix("°").range(-360.0..=360.0)).changed();
        });
        trs_changed |= c1 | c2;
    }

    ui.add_space(2.0);
    prop_row(ui, "Opacity", |ui| {
        circle_dot(ui, dot_color);
        ui.add(egui::Slider::new(&mut popac, 0.0..=1.0).show_value(true));
        if ctx.dragged_id().is_some() { opac_changed = true; }
    });

    ui.add_space(8.0);

    // ── TIMING ────────────────────────────────────────────────────────────────
    section_label(ui, "TIMING");
    {
        let pend_limit   = pend;
        let pstart_limit = pstart;
        let (mut sc1, mut sc2) = (false, false);
        two_col_row(ui, |ui| {
            ui.label(egui::RichText::new("Start").color(TEXT_DIM).size(10.5));
            sc1 = ui.add(egui::DragValue::new(&mut pstart).speed(0.05).suffix("s").range(0.0..=(pend_limit - 0.05))).changed();
        }, |ui| {
            ui.label(egui::RichText::new("End").color(TEXT_DIM).size(10.5));
            sc2 = ui.add(egui::DragValue::new(&mut pend).speed(0.05).suffix("s").range((pstart_limit + 0.05)..=3600.0)).changed();
        });
        start_changed |= sc1;
        end_changed   |= sc2;
    }

    let dur = pend - pstart;
    ui.label(egui::RichText::new(format!("Duration: {:.3}s", dur)).color(TEXT_DIM).size(10.0));

    // ── Write back safely to MULTIPLE SELECT ──────────────────────────────────
    if pos_changed || trs_changed || opac_changed || start_changed || end_changed {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                if pos_changed   { sub.x = px; sub.y = py; }
                if trs_changed   { sub.scale = pscale; sub.rotation = prot; }
                if opac_changed  { sub.opacity = popac; }
                if start_changed { sub.timeline_start = pstart; }
                if end_changed   { sub.timeline_end = pend; }
            }
        }
        if start_changed || end_changed { vm.update_duration(); }
    }

    if (pos_changed || trs_changed || opac_changed) && vm.keyframe_mode == KeyframeMode::Record {
        vm.write_keyframe_now();
    }
}