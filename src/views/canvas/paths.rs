use egui::{Context, Pos2, Rect, Stroke, Vec2};
use crate::viewmodels::editor_vm::{EditorViewModel};
use crate::models::types::subtitle::{PathType, PathNode};
use crate::views::theme::{ACCENT_CYAN, ACCENT_AMBER, BORDER};

pub fn draw_preview(painter: &egui::Painter, preview_rect: Rect, scale_factor: f32, vm: &EditorViewModel) {
    if let Some(sub) = vm.selected_subtitle() {
        if sub.path_type != PathType::None {
            let steps = 60;
            let mut path_points = vec![];
            for i in 0..=steps {
                let (px, py, _) = sub.evaluate_path(i as f32 / steps as f32);
                let cx = preview_rect.center().x + (sub.x + px) * scale_factor;
                let cy = preview_rect.center().y + (sub.y + py) * scale_factor;
                path_points.push(Pos2::new(cx, cy));
            }
            painter.add(egui::Shape::line(path_points, Stroke::new(1.5, ACCENT_AMBER.linear_multiply(0.4))));
        }
    }
}

pub fn handle_interaction(
    ctx: &Context,
    ui: &mut egui::Ui,
    canvas_response: &egui::Response,
    canvas_painter: &egui::Painter,
    preview_rect: Rect,
    scale_factor: f32,
    vm: &mut EditorViewModel
) {
    let mut custom_path_updated = None;
    let mut clicked_node_idx = None;
    let mut delete_node_idx = None;

    if let Some(sub) = vm.selected_subtitle() {
        if sub.path_type == PathType::Custom {
            let mut modified_path = sub.custom_path.clone();
            let mut path_changed = false;

            for (i, node) in modified_path.iter_mut().enumerate() {
                let cx = preview_rect.center().x + (sub.x + node.x * sub.path_scale_x) * scale_factor;
                let cy = preview_rect.center().y + (sub.y + node.y * sub.path_scale_y) * scale_factor;
                let screen_pos = Pos2::new(cx, cy);

                let rect = Rect::from_center_size(screen_pos, Vec2::splat(18.0));
                let id = ui.id().with(("path_node", sub.id.clone(), i));
                let resp = ui.interact(rect, id, egui::Sense::click_and_drag());

                if resp.clicked() {
                    clicked_node_idx = Some(i);
                }
                if resp.dragged() {
                    let new_pos = screen_pos + resp.drag_delta();
                    node.x = ((new_pos.x - preview_rect.center().x) / scale_factor - sub.x) / sub.path_scale_x;
                    node.y = ((new_pos.y - preview_rect.center().y) / scale_factor - sub.y) / sub.path_scale_y;
                    clicked_node_idx = Some(i);
                    path_changed = true;
                }

                let is_selected = vm.selected_path_node == Some(i);
                let color = if is_selected { ACCENT_AMBER } else { ACCENT_CYAN };
                
                canvas_painter.circle_filled(screen_pos, 5.0, color);
                canvas_painter.circle_stroke(screen_pos, 6.5, Stroke::new(1.5, BORDER));

                if is_selected {
                    egui::Window::new(format!("Node {}", i))
                        .fixed_pos(screen_pos + Vec2::new(12.0, 12.0))
                        .title_bar(false)
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut node.smooth, "Smooth").changed() {
                                    path_changed = true;
                                }
                                if ui.button("🗑").clicked() {
                                    delete_node_idx = Some(i);
                                }
                            });
                        });
                }
            }

            // Add Node via background click
            if canvas_response.clicked() && clicked_node_idx.is_none() && delete_node_idx.is_none() {
                if let Some(pos) = canvas_response.interact_pointer_pos() {
                    let nx = ((pos.x - preview_rect.center().x) / scale_factor - sub.x) / sub.path_scale_x;
                    let ny = ((pos.y - preview_rect.center().y) / scale_factor - sub.y) / sub.path_scale_y;
                    modified_path.push(PathNode { x: nx, y: ny, smooth: true });
                    clicked_node_idx = Some(modified_path.len() - 1);
                    path_changed = true;
                }
            }

            // Delete Node via Delete Key
            if let Some(idx) = vm.selected_path_node {
                if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                    delete_node_idx = Some(idx);
                }
            }

            if path_changed || delete_node_idx.is_some() {
                custom_path_updated = Some(modified_path);
            }
        }
    }

    // Apply Path edits back to VM cleanly
    if let Some(idx) = delete_node_idx {
        if let Some(sub) = vm.selected_subtitle_mut() {
            if idx < sub.custom_path.len() {
                sub.custom_path.remove(idx);
                vm.selected_path_node = None;
                vm.maybe_autorecord_keyframe();
                vm.snapshot();
            }
        }
    } else if let Some(path) = custom_path_updated {
        if let Some(sub) = vm.selected_subtitle_mut() {
            sub.custom_path = path;
        }
        if let Some(idx) = clicked_node_idx {
            vm.selected_path_node = Some(idx);
        }
        vm.maybe_autorecord_keyframe();
        vm.mark_modified();
    } else if let Some(idx) = clicked_node_idx {
        vm.selected_path_node = Some(idx);
    }
}