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

            let content_rect = ui.available_rect_before_wrap();
            let (bg_resp, painter) = ui.allocate_painter(content_rect.size(), egui::Sense::click_and_drag());
            
            let tracks_rect = bg_resp.rect; 
            let origin = tracks_rect.min; 

            let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;
            let padding_secs = 50.0 / vm.timeline_zoom as f64; 

            {
                let current_t = vm.current_time();
                let margin    = view_w_secs * 0.1;

                if vm.is_playing() || vm.is_scrubbing {
                    if current_t > vm.timeline_scroll + view_w_secs - margin {
                        vm.timeline_scroll = current_t - view_w_secs + margin;
                    } else if current_t < vm.timeline_scroll + margin {
                        vm.timeline_scroll = current_t - margin;
                    }
                }
            }

            let max_scroll = (vm.project.duration + padding_secs - view_w_secs).max(-padding_secs);
            vm.timeline_scroll = vm.timeline_scroll.clamp(-padding_secs, max_scroll);

            tracks::draw_track_labels(&painter, origin, tracks_rect, vm, ui);
            ruler::draw_ruler(ui, &painter, vm, origin, tracks_rect);

            let total_tracks = vm.total_tracks();
            let total_height = RULER_H + (total_tracks as f32 * TRACK_H);
            let max_v_scroll = (total_height - tracks_rect.height()).max(0.0);
            vm.timeline_vertical_scroll = vm.timeline_vertical_scroll.clamp(0.0, max_v_scroll);

            let mut track_origin = origin;
            track_origin.y -= vm.timeline_vertical_scroll;
            
            let track_area_rect = Rect::from_min_max(
                Pos2::new(tracks_rect.min.x, tracks_rect.min.y + RULER_H),
                tracks_rect.max
            );
            let track_painter = painter.with_clip_rect(track_area_rect);

            tracks::draw_track_lines(&track_painter, track_origin, tracks_rect, vm, ui);

            let mut hovered_block = false;
            let mut needs_sort    = false;

            audio_blocks::draw_media_blocks(ui, vm, &track_painter, track_origin, tracks_rect, &mut hovered_block, ctx);
            subtitle_blocks::draw_subtitle_blocks(ui, vm, &track_painter, track_origin, tracks_rect, &mut hovered_block, &mut needs_sort, ctx);

            draw_transition_blocks(ui, vm, &track_painter, track_origin, tracks_rect, &mut hovered_block, ctx);

            handle_scrub_and_pan(ui, vm, &bg_resp, origin, tracks_rect, hovered_block, ctx);
            box_select::draw_box_select(ui, vm, &track_painter, track_origin, tracks_rect, &bg_resp, ctx);

            if let Some(drag_id) = vm.dragging_media_id.clone() {
                if let Some(pos) = ctx.pointer_hover_pos() {
                    if tracks_rect.contains(pos) && pos.x > origin.x + LABEL_W {
                        let track_idx_raw = ((pos.y - (track_origin.y + RULER_H)) / TRACK_H).floor() as isize;
                        let target_row = track_idx_raw.clamp(1, vm.total_tracks() as isize - 1) as usize; 
                        let target_time = vm.px_to_time(pos.x - origin.x - LABEL_W).max(0.0);

                        let highlight_y = track_origin.y + RULER_H + (target_row as f32 * TRACK_H);
                        let highlight_rect = Rect::from_min_size(
                            Pos2::new(pos.x, highlight_y),
                            Vec2::new(100.0, TRACK_H),
                        );
                        track_painter.rect_filled(highlight_rect, 4.0, ACCENT_CYAN.linear_multiply(0.3));
                        track_painter.rect_stroke(highlight_rect, 4.0, egui::Stroke::new(1.0_f32, ACCENT_CYAN));

                        if ctx.input(|i| i.pointer.any_released()) {
                            vm.place_media_on_timeline(drag_id, target_time, target_row);
                            vm.dragging_media_id = None;
                        }
                    }
                }
            }

            if ctx.input(|i| i.pointer.any_released()) {
                vm.dragging_media_id = None;
                vm.is_scrubbing = false;
            }

            playhead::draw_playhead(&painter, vm, origin, tracks_rect);

            if needs_sort { vm.sort_subtitles(); }
        });
}

fn draw_transition_blocks(
    ui: &mut egui::Ui,
    vm: &mut EditorViewModel,
    painter: &egui::Painter,
    origin: Pos2,
    tracks_rect: Rect,
    hovered_block: &mut bool,
    ctx: &Context,
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
    let mut selection_action = None;

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
        let is_selected = vm.selected_ids.contains(&trans.id);

        if resp.clicked() {
            let shift = ctx.input(|inp| inp.modifiers.shift);
            selection_action = Some((trans.id.clone(), trans.timeline_start, shift));
        }

        let drag_id = ui.id().with(("trans_drag_mode", idx));
        let mut drag_mode = ui.data(|d| d.get_temp::<u8>(drag_id).unwrap_or(0));

        let handle_w = 8.0_f32.min(rect.width() * 0.35);

        if resp.hovered() && !resp.dragged() {
            *hovered_block = true;
            if let Some(pos) = ctx.pointer_hover_pos() {
                let dist_left = (pos.x - rect.min.x).abs();
                let dist_right = (pos.x - rect.max.x).abs();
                if dist_left <= handle_w || dist_right <= handle_w {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
                } else {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
                }
            }
        }

        if resp.drag_started() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if pos.x < rect.min.x + handle_w { drag_mode = 1; }
                else if pos.x > rect.max.x - handle_w { drag_mode = 2; }
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

        let (fill_col, mut stroke_col) = if is_valid {
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

        if is_selected {
            stroke_col = ACCENT_CYAN;
        }

        p.rect_filled(rect, 4.0, fill_col);
        p.rect_stroke(rect, 4.0, Stroke::new(if is_selected { 2.0_f32 } else { 1.5_f32 }, stroke_col));

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

        if block_w > 30.0 {
            p.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(9.0),
                egui::Color32::WHITE,
            );
        }
    }

    if let Some((id, start, shift)) = selection_action {
        if shift {
            vm.toggle_select(&id);
        } else {
            vm.selected_ids.clear();
            vm.selected_ids.insert(id.clone());
            vm.selected_id = Some(id);
            vm.seek_to(start);
        }
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
    ui: &mut egui::Ui,
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
        let mut zoom_delta = 0.0;
        let mut v_pan_delta = 0.0;
        let mut h_pan_delta = 0.0;

        ui.input_mut(|i| {
            let scroll = i.raw_scroll_delta;
            let smooth = i.smooth_scroll_delta;
            let scroll_y = if scroll.y != 0.0 { scroll.y } else { smooth.y };
            let scroll_x = if scroll.x != 0.0 { scroll.x } else { smooth.x };

            if i.modifiers.alt && scroll_y != 0.0 {
                zoom_delta = scroll_y;
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
                i.events.retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
            } else if i.modifiers.shift && scroll_y != 0.0 {
                v_pan_delta = scroll_y;
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
                i.events.retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
            } else if scroll_x != 0.0 || scroll_y != 0.0 {
                h_pan_delta = if scroll_x.abs() > 0.0 { scroll_x } else { scroll_y };
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
                i.events.retain(|e| !matches!(e, egui::Event::MouseWheel { .. }));
            }
        });

        if zoom_delta != 0.0 {
            let old_zoom = vm.timeline_zoom;
            vm.timeline_zoom = (vm.timeline_zoom + zoom_delta * 0.5).clamp(10.0, 1000.0);
            
            if vm.timeline_zoom != old_zoom {
                let focus_time = if vm.timeline_zoom_to_mouse {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        vm.px_to_time(pos.x - (origin.x + LABEL_W))
                    } else {
                        vm.current_time()
                    }
                } else {
                    vm.current_time()
                };
                
                let px_old = (focus_time - vm.timeline_scroll) * old_zoom as f64;
                vm.timeline_scroll = focus_time - (px_old / vm.timeline_zoom as f64);
            }
        }
        
        if v_pan_delta != 0.0 {
            let total_tracks = vm.total_tracks();
            let total_height = RULER_H + (total_tracks as f32 * TRACK_H);
            let max_v_scroll = (total_height - tracks_rect.height()).max(0.0);
            vm.timeline_vertical_scroll = (vm.timeline_vertical_scroll - v_pan_delta).clamp(0.0, max_v_scroll);
        }

        if h_pan_delta != 0.0 {
            let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;
            let padding_secs = 50.0 / vm.timeline_zoom as f64;
            let max_scroll = (vm.project.duration + padding_secs - view_w_secs).max(-padding_secs);
            vm.timeline_scroll = (vm.timeline_scroll - h_pan_delta as f64 / vm.timeline_zoom as f64).clamp(-padding_secs, max_scroll);
        }
    }
}