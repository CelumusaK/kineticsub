// File: src/views/timeline/mod.rs
pub mod audio_blocks;
pub mod box_select;
pub mod header;
pub mod playhead;
pub mod ruler;
pub mod subtitle_blocks;
pub mod toolbar;
pub mod tracks;

use egui::{Context, Pos2, Rect, TopBottomPanel, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL_ALT, BORDER};

pub const HEADER_H: f32 = 26.0;
pub const TOOLBAR_H: f32 = 28.0;
pub const RULER_H: f32 = 22.0;
pub const TRACK_H: f32 = 40.0;
pub const LABEL_W: f32 = 100.0;
pub const TIMELINE_H: f32 = 240.0;
pub const TRACKS: &[&str] = &["SUBTITLES", "VIDEO", "AUDIO"];

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    TopBottomPanel::bottom("timeline_panel")
        .exact_height(TIMELINE_H)
        .frame(egui::Frame {
            fill: BG_PANEL_ALT,
            stroke: egui::Stroke::new(1.0, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            let panel_rect = ui.max_rect();

            header::draw_header(ui, vm, panel_rect);
            toolbar::draw_toolbar(ui, vm, panel_rect);

            let tracks_y    = panel_rect.min.y + HEADER_H + TOOLBAR_H;
            let tracks_rect = Rect::from_min_size(
                Pos2::new(panel_rect.min.x, tracks_y),
                Vec2::new(panel_rect.width(), panel_rect.max.y - tracks_y),
            );

            // ── Scroll clamping & Auto-pan ───────────────────────────────────────────
            let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;

            {
                let current_t = vm.current_time();
                let margin    = view_w_secs * 0.1;

                // ONLY auto-pan if playing or actively dragging the playhead
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

            // Huge empty space at the end of the timeline
            let max_scroll = (vm.project.duration + view_w_secs * 0.8).max(0.0);
            vm.timeline_scroll = vm.timeline_scroll.clamp(0.0, max_scroll);

            let (bg_resp, painter) = ui.allocate_painter(tracks_rect.size(), egui::Sense::click_and_drag());
            let origin = tracks_rect.min;

            tracks::draw_track_labels(&painter, origin, tracks_rect);
            ruler::draw_ruler(&painter, vm, origin, tracks_rect);
            tracks::draw_track_lines(&painter, origin, tracks_rect);

            // ── Fancy Out-Of-Bounds Area ────────────────────────────────────────────────
            let project_end_x = origin.x + LABEL_W + vm.time_to_px(vm.project.duration);
            
            // Only draw if the end of the project is visible on screen
            if project_end_x < tracks_rect.max.x {
                let start_x = project_end_x.max(origin.x + LABEL_W);
                let out_bounds_rect = Rect::from_min_max(
                    Pos2::new(start_x, tracks_rect.min.y),
                    tracks_rect.max
                );
                
                // Darken the empty space
                painter.rect_filled(
                    out_bounds_rect,
                    0.0,
                    egui::Color32::from_black_alpha(100),
                );
                
                // Draw diagonal stripes moving seamlessly with the timeline
                let p_clip = painter.with_clip_rect(out_bounds_rect);
                let stripe_spacing = 20.0;
                let mut current_px = project_end_x - tracks_rect.height();
                
                while current_px < out_bounds_rect.max.x {
                    p_clip.line_segment([
                            Pos2::new(current_px, tracks_rect.max.y),
                            Pos2::new(current_px + tracks_rect.height(), tracks_rect.min.y),
                        ],
                        egui::Stroke::new(2.0, egui::Color32::from_white_alpha(8)),
                    );
                    current_px += stripe_spacing;
                }

                // Draw a distinct red end-of-project line indicator
                if project_end_x >= origin.x + LABEL_W {
                    painter.line_segment([
                            Pos2::new(project_end_x, tracks_rect.min.y),
                            Pos2::new(project_end_x, tracks_rect.max.y),
                        ],
                        egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 50, 50)),
                    );
                }
            }

            let mut hovered_block = false;
            let mut needs_sort    = false;

            audio_blocks::draw_audio_blocks(ui, vm, &painter, origin, tracks_rect, &mut hovered_block, ctx);
            subtitle_blocks::draw_subtitle_blocks(ui, vm, &painter, origin, tracks_rect, &mut hovered_block, &mut needs_sort, ctx);

            handle_scrub_and_pan(ui, vm, &bg_resp, origin, tracks_rect, hovered_block, ctx);
            box_select::draw_box_select(ui, vm, &painter, origin, tracks_rect, &bg_resp, ctx);
            playhead::draw_playhead(&painter, vm, origin, tracks_rect);

            if needs_sort { vm.sort_subtitles(); }
        });
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

    if !hovered_block && !shift && (bg_resp.dragged() || bg_resp.clicked()) {
        if let Some(pos) = bg_resp.interact_pointer_pos() {
            if pos.x > origin.x + LABEL_W {
                let t = vm.px_to_time(pos.x - origin.x - LABEL_W);
                vm.seek_to(t.max(0.0));
            }
        }
    }

    if bg_resp.hovered() {
        ctx.input(|i| {
            let scroll = i.raw_scroll_delta;
            if i.modifiers.alt {
                vm.timeline_zoom = (vm.timeline_zoom + scroll.y * 0.5).clamp(10.0, 1000.0);
                let center_t = vm.px_to_time((tracks_rect.width() - LABEL_W) * 0.5);
                let new_view = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;
                vm.timeline_scroll = (center_t - new_view * 0.5).max(0.0);
            } else if scroll.x != 0.0 || scroll.y != 0.0 {
                let pan = scroll.x + scroll.y;
                vm.timeline_scroll = (vm.timeline_scroll - pan as f64 / vm.timeline_zoom as f64).max(0.0);

                // Auto-Seek: Keep playhead on screen if user scrolls past it
                let view_w_secs = (tracks_rect.width() - LABEL_W) as f64 / vm.timeline_zoom as f64;
                let margin = view_w_secs * 0.1;
                let current_t = vm.current_time();
                
                let left_bound = if vm.timeline_scroll > 0.001 { vm.timeline_scroll + margin } else { 0.0 };
                let right_bound = (vm.timeline_scroll + view_w_secs - margin).max(0.0);

                if current_t > right_bound {
                    vm.seek_to(right_bound);
                } else if current_t < left_bound {
                    vm.seek_to(left_bound);
                }
            }
        });
    }
}