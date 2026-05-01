use egui::{Pos2, Rect, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;

pub fn draw(painter: &egui::Painter, preview_rect: Rect, vm: &EditorViewModel) {
    let active_bg = vm.project.media_files.iter().find(|m| {
        m.on_timeline && m.is_video_track && vm.current_time() >= m.timeline_offset && vm.current_time() < m.timeline_offset + m.duration
    });

    if let Some(bg) = active_bg {
        if let Some(col) = bg.color {
            let egui_col = egui::Color32::from_rgba_unmultiplied(
                (col[0]*255.0) as u8, (col[1]*255.0) as u8, (col[2]*255.0) as u8, (col[3]*255.0) as u8
            );
            painter.rect_filled(preview_rect, 0.0, egui_col);
        }
    } else {
        let cell_size = 16.0;
        let cols = (preview_rect.width() / cell_size).ceil() as usize;
        let rows = (preview_rect.height() / cell_size).ceil() as usize;
        
        for r in 0..rows {
            for c in 0..cols {
                let color = if (r + c) % 2 == 0 {
                    egui::Color32::from_rgb(60, 60, 60)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
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
    }
}