pub mod background;
pub mod masks;
pub mod paths;
pub mod subtitles;

use egui::{CentralPanel, Context, Frame, Rect, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL, BG_PANEL_ALT, BORDER};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    CentralPanel::default()
        .frame(Frame { fill: BG_PANEL, ..Default::default() })
        .show(ctx, |ui| {
            let canvas_rect = ui.available_rect_before_wrap();

            let (canvas_response, canvas_painter) =
                ui.allocate_painter(canvas_rect.size(), egui::Sense::click());
            canvas_painter.rect_filled(canvas_rect, 0.0, BG_PANEL_ALT);

            let res_w = vm.project.resolution.0 as f32;
            let res_h = vm.project.resolution.1 as f32;
            let aspect = res_w / res_h;
            
            let preview_w = (canvas_rect.width()  - 40.0).min((canvas_rect.height() - 40.0) * aspect);
            let preview_h = preview_w / aspect;
            let preview_rect = Rect::from_center_size(
                canvas_rect.center(), Vec2::new(preview_w, preview_h),
            );
            
            let scale_factor = preview_w / res_w;

            // ── 1. Background Rendering ──────────────────────────────────────────
            background::draw(&canvas_painter, preview_rect, vm);

            // ── 2. Draw Motion Path Preview (if selected) ────────────────────────
            paths::draw_preview(&canvas_painter, preview_rect, scale_factor, vm);

            // ── 3. Draw Subtitles & 3D Math Projection ───────────────────────────
            subtitles::draw(ctx, &canvas_painter, preview_rect, scale_factor, vm);

            // ── 4. Interactive Custom Path (On Canvas Edit) ──────────────────────
            paths::handle_interaction(ctx, ui, &canvas_response, &canvas_painter, preview_rect, scale_factor, vm);

            // ── 5. Draw Mask Guides over Canvas ──────────────────────────────────
            masks::draw_guides(&canvas_painter, preview_rect, scale_factor, vm);

            // Draw bounding box border over the canvas area
            canvas_painter.rect_stroke(preview_rect, 2.0, egui::Stroke::new(1.0, BORDER));

            // ── Standard Keyboard Controls ───────────────────────────────────────
            if canvas_response.hovered() {
                ctx.input(|i| {
                    if i.key_pressed(egui::Key::Space)      { vm.toggle_play(); }
                    if i.key_pressed(egui::Key::J)          { vm.skip(-5.0); }
                    if i.key_pressed(egui::Key::L)          { vm.skip(5.0); }
                    if i.key_pressed(egui::Key::ArrowLeft)  { vm.skip(-1.0 / 30.0); }
                    if i.key_pressed(egui::Key::ArrowRight) { vm.skip(1.0 / 30.0); }
                });
            }
        });
}