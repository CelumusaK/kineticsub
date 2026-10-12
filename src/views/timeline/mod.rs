pub mod audio_blocks;
pub mod box_select;
pub mod header;
pub mod playhead;
pub mod ruler;
pub mod subtitle_blocks;
pub mod toolbar;
pub mod tracks;

use egui::{Context, Pos2, Rect, Stroke, TopBottomPanel, Vec2};
use crate::models::types::TransitionType;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL_ALT, BORDER, ACCENT_CYAN, ACCENT_AMBER};

pub const HEADER_H: f32 = 26.0;
pub const TOOLBAR_H: f32 = 28.0;
pub const RULER_H: f32 = 22.0;
pub const TRACK_H: f32 = 40.0;
pub const LABEL_W: f32 = 100.0; 

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    TopBottomPanel::bottom("timeline_panel")
        .default_height(240.0)
        .height_range(150.0..=600.0)
        .resizable(true)
        .frame(egui::Frame {
            fill: BG_PANEL_ALT,
            stroke: egui::Stroke::new(1.0_f32, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            let panel_rect = ui.max_rect();
            
            if ui.rect_contains_pointer(panel_rect) {
                vm.hovered_panel = crate::viewmodels::editor_vm::HoverPanel::Timeline;
            }

            header::draw_header(ui, vm, panel_rect);
            toolbar::draw_toolbar(ui, vm, panel_rect);

            let total_tracks = vm.total_tracks();
            let total_height = RULER_H + (total_tracks as f32 * TRACK_H);

            let shift_down = ctx.input(|i| i.modifiers.shift);

            let scroll_out = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .enable_scrolling(false)
                .vertical_scroll_offset(vm.timeline_vertical_scroll)
                .show(ui, |ui| {
                let (bg_resp, painter) = ui.allocate_painter(Vec2::new(panel_rect.width(), total_height), egui::Sense::click_and_drag());
                let origin = bg_resp.rect.min;
                let tracks_rect = bg_resp.rect;

                let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

                {
                    let current_t = vm.current_time();
                    let margin    = view_w_secs * 0.1;

                    let is_scrubbing = ctx.input(|i| i.pointer.primary_down());
                    if vm.is_playing() || is_scrubbing {
                        if current_t > vm.timeline_scroll + view_w_secs - margin {
                            vm.timeline_scroll = current_t - view_w_secs + margin;
                        } else if current_t < vm.timeline_scroll + margin && current_t > margin {
                            vm.timeline_scroll = current_t - margin;
                        } else if current_t <= margin {
                            vm.timeline_scroll = 0.0;
                        }
                    }
                }

                let max_scroll = (vm.project.duration - view_w_secs).max(0.0);
                vm.timeline_scroll = vm.timeline_scroll.clamp(0.0, max_scroll);

                tracks::draw_track_labels(&painter, origin, tracks_rect, vm, ui);
                ruler::draw_ruler(ui, &painter, vm, origin, tracks_rect); 
                tracks::draw_track_lines(&painter, origin, tracks_rect, vm, ui);

                let mut hovered_block = false;
                let mut needs_sort    = false;

                audio_blocks::draw_media_blocks(ui, vm, &painter, origin, tracks_rect, &mut hovered_block, ctx);
                subtitle_blocks::draw_subtitle_blocks(ui, vm, &painter, origin, tracks_rect, &mut hovered_block, &mut needs_sort, ctx);

                draw_transition_blocks(ui, vm, &painter, origin, tracks_rect, &mut hovered_block);

                handle_scrub_and_pan(ui, vm, &bg_resp, origin, tracks_rect, hovered_block, ctx);
                box_select::draw_box_select(ui, vm, &painter, origin, tracks_rect, &bg_resp, ctx);

                if let Some(drag_id) = vm.dragging_media_id.clone() {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        if tracks_rect.contains(pos) && pos.x > origin.x + LABEL_W {
                            let track_idx_raw = ((pos.y - (origin.y + RULER_H)) / TRACK_H).floor() as isize;
                            let target_row = track_idx_raw.clamp(1, vm.total_tracks() as isize - 1) as usize; 
                            let target_time = vm.px_to_time(pos.x - origin.x - LABEL_W).max(0.0);

                            let highlight_y = origin.y + RULER_H + (target_row as f32 * TRACK_H);
                            let highlight_rect = Rect::from_min_size(
                                Pos2::new(pos.x, highlight_y),
                                Vec2::new(100.0, TRACK_H),
                            );
                            painter.rect_filled(highlight_rect, 4.0, ACCENT_CYAN.linear_multiply(0.3));
                            painter.rect_stroke(highlight_rect, 4.0, egui::Stroke::new(1.0_f32, ACCENT_CYAN));

                            if ctx.input(|i| i.pointer.any_released()) {
                                vm.place_media_on_timeline(drag_id, target_time, target_row);
                                vm.dragging_media_id = None;
                            }
                        }
                    }
                }

                if ctx.input(|i| i.pointer.any_released()) {
                    vm.dragging_media_id = None;
                }

                playhead::draw_playhead(&painter, vm, origin, tracks_rect);

                if needs_sort { vm.sort_subtitles(); }
            });

            if !shift_down && scroll_out.state.offset.y != vm.timeline_vertical_scroll {
                vm.timeline_vertical_scroll = scroll_out.state.offset.y;
            }
        });
}

fn draw_transition_blocks(
    ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: &mut bool,
) {
    let track_clip = Rect::from_min_max(
        Pos2::new(origin.x + LABEL_W, tracks_rect.min.y),
        tracks_rect.max
    );
    let p = painter.with_clip_rect(track_clip);

    let mut to_delete = None;
    let mut to_resize = None;
    let mut to_move = None;
    let mut change_type = None;

    for (idx, trans) in vm.project.transitions.iter().enumerate() {
        let is_valid = vm.is_transition_valid(trans);
        let row_idx = vm.get_track_row(trans.track_index, false);
        let row_y = origin.y + RULER_H + (row_idx as f32 * TRACK_H);

        let block_x = origin.x + LABEL_W + vm.time_to_px(trans.timeline_start);
        let block_w = (trans.duration * vm.timeline_zoom as f64) as f32;

        let rect = Rect::from_min_size(
            Pos2::new(block_x, row_y + 3.0),
            Vec2::new(block_w.max(12.0), TRACK_H - 6.0),
        );

        let interact_rect = rect.intersect(track_clip);
        if !interact_rect.is_positive() { continue; }

        let resp = ui.interact(interact_rect, ui.id().with(format!("trans_{}", trans.id)), egui::Sense::click_and_drag());
        if resp.hovered() { *hovered_block = true; }

        let drag_id = ui.id().with(("trans_drag_mode", idx));
        let mut drag_mode = ui.data(|d| d.get_temp::<u8>(drag_id).unwrap_or(0));

        if resp.drag_started() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if pos.x < rect.min.x + 10.0 { drag_mode = 1; }
                else if pos.x > rect.max.x - 10.0 { drag_mode = 2; }
                else { drag_mode = 0; }
                ui.data_mut(|d| d.insert_temp(drag_id, drag_mode));
            }
        }

        if resp.dragged() {
            let dt = resp.drag_delta().x as f64 / vm.timeline_zoom as f64;
            if drag_mode == 0 {
                to_move = Some((idx, dt));
            } else {
                to_resize = Some((idx, drag_mode, dt));
            }
        }
        if resp.drag_stopped() {
            ui.data_mut(|d| d.remove_temp::<u8>(drag_id));
        }

        resp.context_menu(|ui| {
            ui.label(egui::RichText::new(&trans.name).strong());
            ui.separator();
            ui.menu_button("Transition Type", |ui| {
                for t in TransitionType::all() {
                    if ui.button(t.label()).clicked() {
                        change_type = Some((idx, t.clone()));
                        ui.close_menu();
                    }
                }
            });
            ui.separator();
            if ui.button(egui::RichText::new("🗑 Delete Transition").color(ACCENT_AMBER)).clicked() {
                to_delete = Some(idx);
                ui.close_menu();
            }
        });

        let (fill_col, stroke_col) = if is_valid {
            (
                egui::Color32::from_rgba_unmultiplied(168, 85, 247, 130),
                egui::Color32::from_rgb(216, 180, 254),
            )
        } else {
            (
                egui::Color32::from_rgba_unmultiplied(239, 68, 68, 140),
                egui::Color32::from_rgb(252, 165, 165),
            )
        };

        p.rect_filled(rect, 4.0, fill_col);
        p.rect_stroke(rect, 4.0, Stroke::new(1.5_f32, stroke_col));

        let mid_x = rect.center().x;
        p.line_segment(
            [Pos2::new(mid_x, rect.min.y + 2.0), Pos2::new(mid_x, rect.max.y - 2.0)],
            Stroke::new(1.0_f32, egui::Color32::from_white_alpha(150)),
        );

        let label = if is_valid {
            format!("⚡ {}", trans.name)
        } else {
            "⚠ No Cut".to_string()
        };

        p.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(9.0),
            egui::Color32::WHITE,
        );
    }

    if let Some((idx, dt)) = to_move {
        if let Some(t) = vm.project.transitions.get_mut(idx) {
            t.timeline_start = (t.timeline_start + dt).max(0.0);
        }
        vm.mark_modified();
        vm.update_duration();
    }
    if let Some((idx, mode, dt)) = to_resize {
        if let Some(t) = vm.project.transitions.get_mut(idx) {
            if mode == 1 {
                let actual = dt.clamp(-t.timeline_start, t.duration - 0.1);
                t.timeline_start += actual;
                t.duration = (t.duration - actual).max(0.1);
            } else {
                t.duration = (t.duration + dt).max(0.1);
            }
        }
        vm.mark_modified();
        vm.update_duration();
    }
    if let Some((idx, new_type)) = change_type {
        if let Some(t) = vm.project.transitions.get_mut(idx) {
            t.name = new_type.label().to_string();
            t.transition_type = new_type;
        }
        vm.mark_modified();
    }
    if let Some(idx) = to_delete {
        vm.project.transitions.remove(idx);
        vm.mark_modified();
        vm.update_duration();
    }
}

pub fn handle_scrub_and_pan(
    _ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    bg_resp: &egui::Response,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: bool,
    ctx: &Context,
) {
    let shift = ctx.input(|i| i.modifiers.shift);
    let alt   = ctx.input(|i| i.modifiers.alt);

    if !hovered_block && !shift && (bg_resp.dragged() || bg_resp.clicked()) {
        let pointer_pos = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()));
        if let Some(pos) = pointer_pos {
            if pos.x >= origin.x + LABEL_W {
                let t = vm.px_to_time(pos.x - (origin.x + LABEL_W));
                vm.seek_to(t.clamp(0.0, vm.project.duration));
            }
        }
    }

    if bg_resp.hovered() {
        ctx.input(|i| {
            let scroll = i.raw_scroll_delta;
            if alt {
                let delta = if scroll.y.abs() > 0.0 { scroll.y } else { scroll.x };
                if delta != 0.0 {
                    vm.timeline_zoom = (vm.timeline_zoom + delta * 0.5).clamp(10.0, 1000.0);
                }
            } else if shift {
                let delta = if scroll.y.abs() > 0.0 { scroll.y } else { scroll.x };
                if delta != 0.0 {
                    let total_tracks = vm.total_tracks();
                    let total_height = RULER_H + (total_tracks as f32 * TRACK_H);
                    let max_v_scroll = (total_height - tracks_rect.height()).max(0.0);
                    vm.timeline_vertical_scroll = (vm.timeline_vertical_scroll - delta).clamp(0.0, max_v_scroll);
                }
            } else {
                let delta = if scroll.x.abs() > 0.0 { scroll.x } else { scroll.y };
                if delta != 0.0 {
                    let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;
                    let max_scroll = (vm.project.duration - view_w_secs).max(0.0);
                    vm.timeline_scroll = (vm.timeline_scroll - delta as f64 / vm.timeline_zoom as f64).clamp(0.0, max_scroll);
                }
            }
        });
    }
}