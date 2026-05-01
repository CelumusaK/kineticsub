use egui::{Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::subtitle::MaskType;
use crate::views::theme::{BORDER};

pub fn draw_guides(painter: &egui::Painter, preview_rect: Rect, scale_factor: f32, vm: &EditorViewModel) {
    if let Some(sub) = vm.active_subtitle() {
        if sub.mask_type != MaskType::None && vm.selected_id.as_deref() == Some(&sub.id) {
            let offset_t = vm.current_time();
            // Pass all_subs to allow parents to carry masks
            let state = sub.get_interpolated_state(offset_t, &vm.project.subtitles, 0);
            
            let mc_x = preview_rect.center().x + state.mask_center[0] * scale_factor;
            let mc_y = preview_rect.center().y + state.mask_center[1] * scale_factor;
            let m_center = Pos2::new(mc_x, mc_y);
            
            let rot_rad = state.mask_rotation.to_radians();
            let color = egui::Color32::from_rgba_unmultiplied(240, 100, 240, 200);

            let rot_pt = |dx: f32, dy: f32| -> Pos2 {
                let rx = dx * rot_rad.cos() - dy * rot_rad.sin();
                let ry = dx * rot_rad.sin() + dy * rot_rad.cos();
                m_center + Vec2::new(rx, ry)
            };

            match sub.mask_type {
                MaskType::Rectangle => {
                    let hw = state.mask_size[0] * 0.5 * scale_factor;
                    let hh = state.mask_size[1] * 0.5 * scale_factor;
                    let pts = vec![
                        rot_pt(-hw, -hh), rot_pt(hw, -hh),
                        rot_pt(hw, hh), rot_pt(-hw, hh)
                    ];
                    painter.add(egui::Shape::closed_line(pts, Stroke::new(1.5, color)));
                }
                MaskType::Straight => {
                    let hw = 1500.0 * scale_factor; 
                    let pts = vec![ rot_pt(-hw, 0.0), rot_pt(hw, 0.0) ];
                    painter.add(egui::Shape::line_segment([pts[0], pts[1]], Stroke::new(2.0, color)));
                    
                    let arrow_tip = rot_pt(0.0, if sub.mask_invert { -20.0 } else { 20.0 });
                    painter.add(egui::Shape::line_segment([m_center, arrow_tip], Stroke::new(1.5, color)));
                }
                MaskType::Circle => {
                    let r = state.mask_size[0] * scale_factor;
                    painter.circle_stroke(m_center, r, Stroke::new(1.5, color));
                }
                _ => {}
            }
            
            painter.circle_filled(m_center, 4.0, color);
            painter.circle_stroke(m_center, 5.0, Stroke::new(1.0, BORDER));
        }
    }
}