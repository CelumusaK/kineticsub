// File: src/views/canvas/paths.rs
// ─────────────────────────────────────────────────────────────────────────────
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
            // Create an immutable clone so we can look at siblings without conflicting mutable borrows
            let old_path = modified_path.clone();

            for (i, node) in modified_path.iter_mut().enumerate() {
                // ── Node screen position ──────────────────────────────────────
                let cx = preview_rect.center().x + (sub.x + node.x * sub.path_scale_x) * scale_factor;
                let cy = preview_rect.center().y + (sub.y + node.y * sub.path_scale_y) * scale_factor;
                let screen_pos = Pos2::new(cx, cy);

                // ── Compute tangent direction for arrowhead handles ────────────
                let n = old_path.len();
                let prev_pos = if i > 0 {
                    let pn = &old_path[i - 1];
                    let px = preview_rect.center().x + (sub.x + pn.x * sub.path_scale_x) * scale_factor;
                    let py = preview_rect.center().y + (sub.y + pn.y * sub.path_scale_y) * scale_factor;
                    Some(Pos2::new(px, py))
                } else { None };

                let next_pos = if i + 1 < n {
                    let nn = &old_path[i + 1];
                    let nx = preview_rect.center().x + (sub.x + nn.x * sub.path_scale_x) * scale_factor;
                    let ny = preview_rect.center().y + (sub.y + nn.y * sub.path_scale_y) * scale_factor;
                    Some(Pos2::new(nx, ny))
                } else { None };

                // Tangent: average of prev->cur and cur->next directions
                let tangent = compute_tangent(screen_pos, prev_pos, next_pos);
                let _normal = Vec2::new(-tangent.y, tangent.x); // perpendicular

                // ── Draw the arrowhead handle on each side of the node ────────
                let handle_len = 18.0;
                let arrow_size = 7.0;

                // Tangent handle tip (ahead)
                let tip_fwd  = screen_pos + tangent * handle_len;
                let tip_back = screen_pos - tangent * handle_len;

                // Draw handle lines
                canvas_painter.line_segment([screen_pos, tip_fwd],  Stroke::new(1.0, ACCENT_AMBER.linear_multiply(0.5)));
                canvas_painter.line_segment([screen_pos, tip_back], Stroke::new(1.0, ACCENT_AMBER.linear_multiply(0.5)));

                // Draw arrowheads
                draw_arrowhead(canvas_painter, tip_fwd,  tangent,  arrow_size, node.smooth);
                draw_arrowhead(canvas_painter, tip_back, -tangent, arrow_size, node.smooth);

                // ── Interact with arrowhead handles for smooth/sharp toggle ────
                let fwd_rect  = Rect::from_center_size(tip_fwd,  Vec2::splat(16.0));
                let back_rect = Rect::from_center_size(tip_back, Vec2::splat(16.0));

                let fwd_resp  = ui.interact(fwd_rect,  ui.id().with(("arrowfwd",  &sub.id, i)), egui::Sense::click_and_drag());
                let back_resp = ui.interact(back_rect, ui.id().with(("arrowback", &sub.id, i)), egui::Sense::click_and_drag());

                // Dragging a handle adjusts the node position along the tangent
                if fwd_resp.dragged() {
                    let delta = fwd_resp.drag_delta();
                    node.x += delta.x / scale_factor / sub.path_scale_x.max(0.01);
                    node.y += delta.y / scale_factor / sub.path_scale_y.max(0.01);
                    clicked_node_idx = Some(i);
                    path_changed = true;
                }
                if back_resp.dragged() {
                    let delta = back_resp.drag_delta();
                    node.x += delta.x / scale_factor / sub.path_scale_x.max(0.01);
                    node.y += delta.y / scale_factor / sub.path_scale_y.max(0.01);
                    clicked_node_idx = Some(i);
                    path_changed = true;
                }

                // Clicking an arrowhead toggles smooth
                if fwd_resp.clicked() || back_resp.clicked() {
                    node.smooth = !node.smooth;
                    clicked_node_idx = Some(i);
                    path_changed = true;
                }

                // ── Main node drag ────────────────────────────────────────────
                let rect = Rect::from_center_size(screen_pos, Vec2::splat(18.0));
                let id = ui.id().with(("path_node", sub.id.clone(), i));
                let resp = ui.interact(rect, id, egui::Sense::click_and_drag());

                if resp.clicked() {
                    clicked_node_idx = Some(i);
                }
                if resp.dragged() {
                    let new_pos = screen_pos + resp.drag_delta();
                    node.x = ((new_pos.x - preview_rect.center().x) / scale_factor - sub.x) / sub.path_scale_x.max(0.01);
                    node.y = ((new_pos.y - preview_rect.center().y) / scale_factor - sub.y) / sub.path_scale_y.max(0.01);
                    clicked_node_idx = Some(i);
                    path_changed = true;
                }

                let is_selected = vm.selected_path_node == Some(i);
                let node_color = if is_selected { ACCENT_AMBER } else { ACCENT_CYAN };

                // Draw node dot — square for sharp, circle for smooth
                if node.smooth {
                    canvas_painter.circle_filled(screen_pos, 5.5, node_color);
                    canvas_painter.circle_stroke(screen_pos, 7.0, Stroke::new(1.5, BORDER));
                } else {
                    // Sharp node: diamond shape
                    let d = 5.5f32;
                    canvas_painter.add(egui::Shape::convex_polygon(
                        vec![
                            Pos2::new(screen_pos.x,     screen_pos.y - d),
                            Pos2::new(screen_pos.x + d, screen_pos.y),
                            Pos2::new(screen_pos.x,     screen_pos.y + d),
                            Pos2::new(screen_pos.x - d, screen_pos.y),
                        ],
                        node_color,
                        Stroke::new(1.5, BORDER),
                    ));
                }

                // Node tooltip popup
                if is_selected {
                    egui::Window::new(format!("Node {}", i))
                        .fixed_pos(screen_pos + Vec2::new(14.0, 14.0))
                        .title_bar(false)
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                let smooth_label = if node.smooth { "● Smooth" } else { "◆ Sharp" };
                                if ui.button(smooth_label).clicked() {
                                    node.smooth = !node.smooth;
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
                    let nx = ((pos.x - preview_rect.center().x) / scale_factor - sub.x) / sub.path_scale_x.max(0.01);
                    let ny = ((pos.y - preview_rect.center().y) / scale_factor - sub.y) / sub.path_scale_y.max(0.01);
                    modified_path.push(PathNode { x: nx, y: ny, smooth: true });
                    clicked_node_idx = Some(modified_path.len() - 1);
                    path_changed = true;
                }
            }

            // Delete via keyboard
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

    // Apply mutations outside immutable borrow
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute unit tangent at a node given optional prev/next screen positions.
fn compute_tangent(pos: Pos2, prev: Option<Pos2>, next: Option<Pos2>) -> Vec2 {
    let dir = match (prev, next) {
        (Some(p), Some(n)) => n - p,
        (None, Some(n))    => n - pos,
        (Some(p), None)    => pos - p,
        (None, None)       => Vec2::new(1.0, 0.0),
    };
    let len = dir.length();
    if len < 0.001 { Vec2::new(1.0, 0.0) } else { dir / len }
}

/// Draw an arrowhead at `tip` pointing in direction `dir`.
/// If `smooth` is true draws a round cap; if false draws a sharp triangle.
fn draw_arrowhead(painter: &egui::Painter, tip: Pos2, dir: Vec2, size: f32, smooth: bool) {
    let perp = Vec2::new(-dir.y, dir.x);
    let base = tip - dir * size;

    if smooth {
        // Round: filled circle
        painter.circle_filled(tip, size * 0.45, ACCENT_AMBER.linear_multiply(0.85));
    } else {
        // Sharp: triangle
        let pts = vec![
            tip,
            base + perp * size * 0.5,
            base - perp * size * 0.5,
        ];
        painter.add(egui::Shape::convex_polygon(
            pts,
            ACCENT_AMBER.linear_multiply(0.85),
            Stroke::NONE,
        ));
    }
}