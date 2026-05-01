use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{ACCENT_CYAN, ACCENT_CYAN_DIM, TEXT_NORM};
use super::{LABEL_W, RULER_H, TRACK_H};

// ── Subtitle blocks ────────────────────────────────────────────────────────────

pub fn draw_subtitle_blocks(
ui: &mut egui::Ui,
vm: &mut EditorViewModel,
painter: &egui::Painter,
origin: Pos2,
tracks_rect: Rect,
hovered_block: &mut bool,
needs_sort: &mut bool,
ctx: &Context,
) {
let sub_row_y = origin.y + RULER_H;

// Snapshot only what we need for rendering — one allocation, no per-block clones.
struct SubSnap {
    idx:            usize,
    id:             String,
    start:          f64,
    end:            f64,
    text:           String,
    kfs:            Vec<f64>,
    has_media_link: bool,
}

// Cull to visible range before building the snapshot.
let view_start = vm.timeline_scroll;
let view_end   = view_start
    + (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

let snaps: Vec<SubSnap> = vm.project.subtitles
    .iter()
    .enumerate()
    .filter(|(_, s)| s.timeline_end > view_start && s.timeline_start < view_end)
    .map(|(i, s)| SubSnap {
        idx:            i,
        id:             s.id.clone(),
        start:          s.timeline_start,
        end:            s.timeline_end,
        text:           s.text.clone(),
        kfs:            s.keyframes.iter().map(|k| k.time_offset).collect(),
        has_media_link: s.media_id.is_some(),
    })
    .collect();

let mut drag_delta:  Option<(usize, bool, f64)> = None; // (idx, is_multi, dt)
let mut click_info:  Option<(String, f64, bool)> = None; // (id, start, shift)
let mut drag_ended  = false;

for snap in &snaps {
    let is_primary  = vm.selected_id.as_deref() == Some(&snap.id);
    let is_selected = vm.selected_ids.contains(&snap.id);

    let block_x = origin.x + LABEL_W + vm.time_to_px(snap.start);
    let block_w = (vm.time_to_px(snap.end) - vm.time_to_px(snap.start)).max(4.0);

    let block_rect = Rect::from_min_size(
        Pos2::new(block_x.max(origin.x + LABEL_W), sub_row_y + 4.0),
        Vec2::new(block_w, TRACK_H - 8.0),
    );

    let resp = ui.interact(
        block_rect,
        ui.id().with(("sub", snap.idx)),
        egui::Sense::click_and_drag(),
    );

    if resp.hovered() { *hovered_block = true; }

    if resp.clicked() {
        let shift = ctx.input(|i| i.modifiers.shift);
        click_info = Some((snap.id.clone(), snap.start, shift));
    }

    if resp.dragged() && !snap.has_media_link {
        let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
        drag_delta = Some((snap.idx, is_selected && vm.selected_ids.len() > 1, dt));
    }
    if resp.drag_stopped() { drag_ended = true; }

    // ── Paint ─────────────────────────────────────────────────────────────
    let fill = if is_primary {
        egui::Color32::from_rgba_premultiplied(34, 211, 238, 65)
    } else if is_selected {
        egui::Color32::from_rgba_premultiplied(34, 211, 238, 40)
    } else {
        egui::Color32::from_rgba_premultiplied(34, 211, 238, 22)
    };
    let stroke_col = if is_primary { ACCENT_CYAN } else { ACCENT_CYAN_DIM };

    painter.rect_filled(block_rect, 3.0, fill);
    painter.rect_stroke(
        block_rect,
        3.0,
        Stroke::new(if is_primary { 1.5 } else { 1.0 }, stroke_col),
    );

    if snap.has_media_link {
        painter.text(
            Pos2::new(block_rect.min.x + 2.0, block_rect.min.y + 1.0),
            egui::Align2::LEFT_TOP,
            "⛓",
            egui::FontId::proportional(7.0),
            egui::Color32::from_rgba_unmultiplied(245, 158, 11, 160),
        );
    }

    let max_chars = ((block_w / 7.0) as usize).max(1);
    let label: String = snap.text.chars().take(max_chars).collect();
    painter.text(
        block_rect.center_top() + Vec2::new(0.0, 5.0),
        egui::Align2::CENTER_TOP,
        label,
        egui::FontId::proportional(9.0),
        if is_primary { ACCENT_CYAN } else { TEXT_NORM },
    );

    let sub_dur = snap.end - snap.start;
    for kf_t in &snap.kfs {
        if *kf_t < 0.0 || *kf_t > sub_dur { continue; }
        let kf_frac = *kf_t / sub_dur;
        let kf_x    = block_rect.min.x + kf_frac as f32 * block_rect.width();
        let kf_pos  = Pos2::new(kf_x, block_rect.max.y - 4.0);
        let d       = 3.5f32;
        let diamond = vec![
            Pos2::new(kf_pos.x,     kf_pos.y - d),
            Pos2::new(kf_pos.x + d, kf_pos.y    ),
            Pos2::new(kf_pos.x,     kf_pos.y + d),
            Pos2::new(kf_pos.x - d, kf_pos.y    ),
        ];
        painter.add(egui::Shape::convex_polygon(
            diamond,
            egui::Color32::from_rgb(34, 211, 238),
            Stroke::NONE,
        ));
    }
}

// ── Apply mutations after the immutable snapshot loop ─────────────────────
if let Some((id, start, shift)) = click_info {
    if shift {
        vm.toggle_select(&id);
    } else {
        vm.select_subtitle(Some(id));
        vm.seek_to(start);
    }
}

if let Some((idx, is_multi, dt)) = drag_delta {
    if is_multi {
        let ids: Vec<String> = vm.selected_ids.iter().cloned().collect();
        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) && sub.media_id.is_none() {
                sub.timeline_start = (sub.timeline_start + dt).max(0.0);
                sub.timeline_end   = (sub.timeline_end   + dt).max(sub.timeline_start + 0.05);
            }
        }
        vm.update_duration();
    } else {
        vm.move_subtitle_idx(idx, dt);
    }
}

if drag_ended { *needs_sort = true; }

}