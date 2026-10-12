use egui::{Context, Rect, SidePanel, Vec2};
use std::path::PathBuf;
use crate::models::types::MediaType;
use crate::models::types::animation::Easing;
use crate::viewmodels::editor_vm::{EditorViewModel, GraphChannel};
use super::theme::{BG_PANEL, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER, BG_HOVER, BG_BASE};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    SidePanel::left("left_panel")
        .default_width(260.0) 
        .width_range(200.0..=500.0)
        .resizable(true)
        .frame(egui::Frame {
            fill: BG_PANEL,
            stroke: egui::Stroke::new(1.0_f32, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) {
                vm.hovered_panel = crate::viewmodels::editor_vm::HoverPanel::MediaBin;
            }

            let total_h = ui.available_height();
            let top_h = (total_h * vm.left_panel_split).max(150.0).min(total_h - 150.0);

            egui::TopBottomPanel::top("left_media_bin")
                .exact_height(top_h)
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    draw_media_bin(ui, vm, ctx);
                });

            let sep_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 8.0));
            let sep_resp = ui.interact(sep_rect, ui.id().with("left_sep"), egui::Sense::drag());
            if sep_resp.dragged() {
                vm.left_panel_split += sep_resp.drag_delta().y / total_h;
                vm.left_panel_split = vm.left_panel_split.clamp(0.1, 0.9);
            }
            if sep_resp.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
            }
            ui.painter().rect_filled(sep_rect, 0.0, BG_PANEL);
            ui.painter().line_segment(
                [egui::pos2(sep_rect.min.x, sep_rect.center().y), egui::pos2(sep_rect.max.x, sep_rect.center().y)], 
                egui::Stroke::new(1.0_f32, BORDER)
            );
            ui.allocate_space(Vec2::new(ui.available_width(), 8.0));

            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            draw_subtitle_list(ui, vm);
                            ui.add_space(8.0);
                            ui.add(egui::Separator::default().spacing(0.0));
                            ui.add_space(8.0);
                            
                            draw_animation_graph(ui, vm);
                        });
                });
        });

    draw_modals(ctx, vm);
}

fn draw_media_bin(ui: &mut egui::Ui, vm: &mut EditorViewModel, ctx: &Context) {
    section_header(ui, "MEDIA BIN");
    
    let mut min_rect = ui.available_rect_before_wrap();
    min_rect.max.y = min_rect.max.y.max(min_rect.min.y + 100.0);
    
    let bg_resp = ui.interact(min_rect, ui.id().with("bin_bg"), egui::Sense::click());
    
    bg_resp.context_menu(|ui| {
        if ui.button("Import Media").clicked() { vm.import_media(None); ui.close_menu(); }
        if ui.button("Create Color Background").clicked() { vm.creating_color_bg = true; ui.close_menu(); }
    });

    let has_hovered_files = !ctx.input(|i| i.raw.hovered_files.is_empty());
    if has_hovered_files {
        ui.painter().rect_filled(min_rect, 0.0, ACCENT_CYAN.linear_multiply(0.1));
        ui.painter().rect_stroke(min_rect, 0.0, egui::Stroke::new(2.0_f32, ACCENT_CYAN));
        ui.painter().text(min_rect.center(), egui::Align2::CENTER_CENTER, "Drop Files Here", egui::FontId::proportional(16.0), TEXT_BRIGHT);
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.indent("media_indent", |ui| {
                ui.add_space(4.0);

                if vm.project.media_files.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label(egui::RichText::new("Right-click to Add Media").color(TEXT_DIM).size(11.0));
                        ui.label(egui::RichText::new("or drop files directly here").color(TEXT_NORM).size(10.0));
                    });
                } else {
                    let folders = vm.project.folders.clone();
                    for f in &folders {
                        let member_indices: Vec<usize> = vm.project.media_files.iter().enumerate()
                            .filter(|(_, m)| m.folder_id.as_deref() == Some(&f.id))
                            .map(|(i, _)| i)
                            .collect();

                        if member_indices.is_empty() { continue; }

                        let mut is_open = f.is_open;
                        let icon = if is_open { "📂" } else { "📁" };
                        
                        ui.horizontal(|ui| {
                            if ui.button(format!("{} {} ({})", icon, f.name, member_indices.len())).clicked() {
                                is_open = !is_open;
                            }
                        });

                        if is_open != f.is_open {
                            if let Some(folder_ref) = vm.project.folders.iter_mut().find(|folder| folder.id == f.id) {
                                folder_ref.is_open = is_open;
                            }
                        }

                        if is_open {
                            ui.indent(format!("f_{}", f.id), |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                                    for i in member_indices {
                                        draw_media_grid_item(ui, vm, ctx, i);
                                    }
                                });
                            });
                        }
                        ui.add_space(4.0);
                    }

                    let root_indices: Vec<usize> = vm.project.media_files.iter().enumerate()
                        .filter(|(_, m)| m.folder_id.is_none())
                        .map(|(i, _)| i)
                        .collect();

                    if !root_indices.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                            for i in root_indices {
                                draw_media_grid_item(ui, vm, ctx, i);
                            }
                        });
                    }
                }
            });

            if vm.whisper_is_running() {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("⟳  {}", vm.whisper_status))
                        .color(ACCENT_CYAN)
                        .size(10.0)
                        .monospace(),
                );
            }
        });
}

fn draw_media_grid_item(ui: &mut egui::Ui, vm: &mut EditorViewModel, ctx: &Context, index: usize) {
    let (id, name, duration, media_type, path, source_fps) = {
        let m = &vm.project.media_files[index];
        (m.id.clone(), m.name.clone(), m.duration, m.media_type.clone(), m.path.clone(), m.source_fps)
    };

    let item_size = Vec2::new(70.0, 76.0);
    let (rect, resp) = ui.allocate_exact_size(item_size, egui::Sense::click_and_drag());
    
    if resp.drag_started() {
        vm.dragging_media_id = Some(id.clone());
    }

    let is_hovered = resp.hovered();
    let is_dragging = vm.dragging_media_id.as_deref() == Some(id.as_str());
    let is_selected = vm.selected_ids.contains(&id);

    let bg_color = if is_dragging || is_selected {
        ACCENT_CYAN.linear_multiply(0.2)
    } else if is_hovered {
        BG_HOVER
    } else {
        egui::Color32::from_black_alpha(50)
    };

    let stroke_color = if is_dragging || is_selected { ACCENT_CYAN } else if is_hovered { TEXT_NORM } else { BORDER };

    ui.painter().rect_filled(rect, 4.0, bg_color);
    ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, stroke_color));

    let thumb_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 44.0));
    ui.painter().rect_filled(
        thumb_rect, 
        egui::Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 }, 
        egui::Color32::from_black_alpha(100)
    );

    if media_type == MediaType::Image {
        if let Some(tex) = vm.get_or_load_texture(ctx, &path) {
            ui.painter().image(
                tex.id(),
                thumb_rect.shrink(2.0),
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, "🖼", egui::FontId::proportional(20.0), TEXT_DIM);
        }
    } else if media_type == MediaType::Video {
        let thumb_path = std::env::temp_dir().join("kineticsub_cache").join(&id).join("frame_0001.jpg");
        if let Some(tex) = vm.get_or_load_texture(ctx, thumb_path.to_str().unwrap_or("")) {
            ui.painter().image(
                tex.id(),
                thumb_rect.shrink(2.0),
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, "🎞", egui::FontId::proportional(20.0), TEXT_NORM);
        }
    } else {
        ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(20.0), ACCENT_CYAN);
    }

    if let Some(fps) = source_fps {
        if fps > (vm.project.fps as f64 + 1.0) {
            ui.painter().circle_filled(rect.right_top() + egui::vec2(-8.0, 8.0), 5.0, ACCENT_AMBER);
            if is_hovered {
                resp.clone().on_hover_ui(|ui| {
                    ui.label(egui::RichText::new(format!("⚠ Video is {:.1} fps (Project is {} fps).\nConsider raising project FPS to match.", fps, vm.project.fps)).color(ACCENT_AMBER));
                    if ui.button("Match Project to Video").clicked() {
                        vm.project.fps = fps.round() as u32;
                    }
                });
            }
        }
    }

    let mut short_name = name.clone();
    if short_name.len() > 10 {
        short_name.truncate(9);
        short_name.push('…');
    }
    
    ui.painter().text(
        egui::pos2(rect.center().x, rect.max.y - 18.0),
        egui::Align2::CENTER_CENTER,
        short_name,
        egui::FontId::proportional(10.5),
        if is_hovered { TEXT_BRIGHT } else { TEXT_NORM },
    );
    
    ui.painter().text(
        egui::pos2(rect.center().x, rect.max.y - 6.0),
        egui::Align2::CENTER_CENTER,
        format!("{:.1}s", duration),
        egui::FontId::proportional(9.0),
        TEXT_DIM,
    );

    let mut delete_clicked = false;
    let mut preview_clicked = false;

    if resp.clicked() {
        if ctx.input(|i| i.modifiers.shift) {
            vm.toggle_select(&id);
        } else {
            vm.select_with_group(&id);
        }
    }

    resp.context_menu(|ui| {
        ui.label(egui::RichText::new(&name).color(TEXT_BRIGHT).strong());
        ui.separator();
        
        if ui.button("▶ Preview").clicked() {
            preview_clicked = true;
            ui.close_menu();
        }

        ui.separator();
        if ui.button(egui::RichText::new("🗑 Delete Everywhere").color(ACCENT_AMBER)).clicked() {
            delete_clicked = true;
            ui.close_menu();
        }
    });

    if preview_clicked {
        vm.sync.pause(); 
        vm.audio_player.stop_all();
        vm.previewing_media_id = Some(id.clone());
        vm.preview_time = 0.0;
        vm.preview_playing = true;
    }

    if delete_clicked {
        vm.project.media_files.remove(index);
        vm.project.subtitles.retain(|s| s.media_id.as_deref() != Some(id.as_str()));
        vm.update_duration();
        vm.snapshot();
    }
}

fn draw_modals(ctx: &Context, vm: &mut EditorViewModel) {
    if vm.creating_color_bg {
        let mut open = true;
        egui::Window::new("Create Solid Background")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .frame(egui::Frame { fill: BG_PANEL, stroke: egui::Stroke::new(1.0_f32, BORDER), inner_margin: egui::Margin::same(12.0), ..Default::default() })
            .show(ctx, |ui| {
            ui.color_edit_button_rgba_unmultiplied(&mut vm.new_bg_color);
            ui.add_space(8.0);
            if ui.button("Add to Media Bin").clicked() {
                vm.add_solid_bg(vm.new_bg_color);
                vm.creating_color_bg = false;
            }
        });
        if !open { vm.creating_color_bg = false; }
    }

    if let Some(prev_id) = vm.previewing_media_id.clone() {
        let mut open = true;
        let mut media_copy = None;
        if let Some(m) = vm.project.media_files.iter().find(|m| m.id == prev_id) {
            media_copy = Some(m.clone());
        }

        if let Some(m) = media_copy {
            egui::Window::new(format!("Preview: {}", m.name))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .frame(egui::Frame { fill: BG_PANEL, stroke: egui::Stroke::new(1.0_f32, BORDER), inner_margin: egui::Margin::same(12.0), ..Default::default() })
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        let w = 400.0;
                        let h = 225.0;
                        let mut tex_id = None;
                        
                        if m.media_type == MediaType::Image {
                            if let Some(tex) = vm.get_or_load_texture(ctx, &m.path) { tex_id = Some(tex.id()); }
                        } else if m.media_type == MediaType::Video && m.color.is_none() {
                            let frame_num = (vm.preview_time * vm.project.fps as f64).floor() as usize + 1;
                            let frame_path = std::env::temp_dir().join("kineticsub_cache").join(&m.id).join(format!("frame_{:04}.jpg", frame_num));
                            if let Some(tex) = vm.get_or_load_texture(ctx, frame_path.to_str().unwrap_or("")) { tex_id = Some(tex.id()); }
                        } else if let Some(c) = m.color {
                            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(w, h), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied((c[0]*255.) as u8, (c[1]*255.) as u8, (c[2]*255.) as u8, 255));
                        }
                        
                        if let Some(id) = tex_id {
                            ui.image(egui::load::SizedTexture::new(id, egui::Vec2::new(w, h)));
                        } else if m.media_type == MediaType::Audio {
                            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(w, 80.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 4.0, BG_BASE);
                            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🎵 Audio Playing...", egui::FontId::proportional(24.0), ACCENT_CYAN);
                        }

                        ui.add_space(8.0);
                        
                        if ui.add(egui::Slider::new(&mut vm.preview_time, 0.0..=m.duration).text("Time")).changed() {
                            vm.audio_player.stop("preview");
                            if vm.preview_playing && m.media_type != MediaType::Image {
                                let audio_path = if m.media_type == MediaType::Video {
                                    std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav")
                                } else { PathBuf::from(&m.path) };
                                vm.audio_player.play("preview", audio_path, vm.preview_time, 1.0);
                            }
                        }

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button("⏮ -5s").clicked() {
                                vm.preview_time = (vm.preview_time - 5.0).max(0.0);
                                vm.audio_player.stop("preview");
                                if vm.preview_playing && m.media_type != MediaType::Image {
                                    let audio_path = if m.media_type == MediaType::Video { std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav") } else { PathBuf::from(&m.path) };
                                    vm.audio_player.play("preview", audio_path, vm.preview_time, 1.0);
                                }
                            }
                            if ui.button(if vm.preview_playing { "⏸ Pause" } else { "▶ Play" }).clicked() {
                                vm.preview_playing = !vm.preview_playing;
                                if vm.preview_playing {
                                    if m.media_type != MediaType::Image {
                                        let audio_path = if m.media_type == MediaType::Video { std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav") } else { PathBuf::from(&m.path) };
                                        vm.audio_player.play("preview", audio_path, vm.preview_time, 1.0);
                                    }
                                } else {
                                    vm.audio_player.stop("preview");
                                }
                            }
                            if ui.button("+5s ⏭").clicked() {
                                vm.preview_time = (vm.preview_time + 5.0).min(m.duration);
                                vm.audio_player.stop("preview");
                                if vm.preview_playing && m.media_type != MediaType::Image {
                                    let audio_path = if m.media_type == MediaType::Video { std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav") } else { PathBuf::from(&m.path) };
                                    vm.audio_player.play("preview", audio_path, vm.preview_time, 1.0);
                                }
                            }
                        });
                    });
                });
        }
        
        if !open {
            vm.previewing_media_id = None;
            vm.preview_playing = false;
            vm.audio_player.stop("preview");
        }
    }
}

fn draw_subtitle_list(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_header(ui, "SUBTITLES");
    if vm.project.subtitles.is_empty() {
        ui.indent("subs_empty", |ui| { ui.label(egui::RichText::new("No subtitles yet.").color(TEXT_DIM).size(11.0)); });
        return;
    }
    let mut clicked_id = None;
    for i in 0..vm.project.subtitles.len() {
        let sub = &vm.project.subtitles[i];
        let mut text_preview: String = sub.text.chars().take(22).collect();
        if sub.text.len() > text_preview.len() { text_preview.push('…'); }
        let time_label = format!("{:02}:{:04.1}", (sub.timeline_start / 60.0) as i32, sub.timeline_start % 60.0);
        let is_selected = vm.selected_ids.contains(&sub.id);
        let is_primary  = vm.selected_id.as_deref() == Some(sub.id.as_str());
        let has_link = sub.media_id.is_some();
        let row_fill = if is_primary {
            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 28)
        } else if is_selected {
            egui::Color32::from_rgba_unmultiplied(34, 211, 238, 12)
        } else {
            egui::Color32::TRANSPARENT
        };
        let resp = egui::Frame::none()
            .fill(row_fill)
            .inner_margin(egui::Margin::symmetric(8.0, 3.0))
            .rounding(egui::Rounding::same(3.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&time_label).color(TEXT_DIM).size(10.5).monospace());
                    ui.add_space(2.0);
                    if has_link { ui.label(egui::RichText::new("⛓").size(9.0).color(ACCENT_AMBER)); }
                    ui.add_space(2.0);
                    let label_color = if is_primary { ACCENT_CYAN } else { TEXT_NORM };
                    ui.label(egui::RichText::new(&text_preview).color(label_color).size(11.5));
                });
            })
            .response;
        if resp.interact(egui::Sense::click()).clicked() {
            clicked_id = Some(sub.id.clone());
        }
    }
    if let Some(id) = clicked_id {
        vm.selected_id = Some(id.clone());
        vm.selected_ids.clear();
        vm.selected_ids.insert(id);
    }
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(title).color(TEXT_DIM).size(11.0).strong());
    ui.add_space(2.0);
}

fn easing_to_handles(e: &Easing) -> [f32; 4] {
    match e {
        Easing::Linear => [0.25, 0.25, 0.75, 0.75],
        Easing::EaseIn => [0.42, 0.0, 1.0, 1.0],
        Easing::EaseOut => [0.0, 0.0, 0.58, 1.0],
        Easing::EaseInOut => [0.42, 0.0, 0.58, 1.0],
        Easing::Bounce => [0.3, 0.1, 0.7, 1.0],
        Easing::Elastic => [0.2, 0.0, 0.8, 1.2],
        Easing::Back => [0.35, -0.2, 0.65, 1.2],
        Easing::Custom(h) => *h,
    }
}

pub fn draw_animation_graph(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    if vm.is_playing() {
        let active_info = vm.active_subtitle().map(|s| (s.id.clone(), !s.keyframes.is_empty()));
        if let Some((sub_id, has_kfs)) = active_info {
            if vm.selected_id.as_deref() != Some(&sub_id) && has_kfs {
                vm.selected_id = Some(sub_id.clone());
                vm.selected_ids.clear();
                vm.selected_ids.insert(sub_id);
                vm.selected_keyframe_id = None;
            }
        }
    }

    let (has_kfs, dur, start, local_time, keyframes) = if let Some(sub) = vm.selected_subtitle() {
        (sub.keyframes.len() >= 2, sub.duration(), sub.timeline_start, vm.current_time() - sub.timeline_start, sub.keyframes.clone())
    } else if let Some(m) = vm.selected_media() {
        (m.keyframes.len() >= 2, m.duration, m.timeline_offset, vm.current_time() - m.timeline_offset, m.keyframes.clone())
    } else {
        (false, 0.0, 0.0, 0.0, vec![])
    };

    if !has_kfs || dur <= 0.0 {
        section_header(ui, "ANIMATION GRAPH");
        ui.indent("graph_empty", |ui| {
            ui.label(egui::RichText::new("Requires at least 2 keyframes.").color(TEXT_DIM).size(11.0));
        });
        return;
    }

    section_header(ui, "ANIMATION GRAPH");
    
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("graph_channel")
            .selected_text(match vm.graph_channel {
                GraphChannel::X => "Position X",
                GraphChannel::Y => "Position Y",
                GraphChannel::Scale => "Scale",
                GraphChannel::Rotation => "Rotation",
                GraphChannel::Opacity => "Opacity",
            })
            .width(120.0)
            .show_ui(ui, |ui| {
                let mut switched = false;
                if ui.selectable_value(&mut vm.graph_channel, GraphChannel::X, "Position X").clicked() { switched = true; }
                if ui.selectable_value(&mut vm.graph_channel, GraphChannel::Y, "Position Y").clicked() { switched = true; }
                if ui.selectable_value(&mut vm.graph_channel, GraphChannel::Scale, "Scale").clicked() { switched = true; }
                if ui.selectable_value(&mut vm.graph_channel, GraphChannel::Rotation, "Rotation").clicked() { switched = true; }
                if ui.selectable_value(&mut vm.graph_channel, GraphChannel::Opacity, "Opacity").clicked() { switched = true; }
                
                if switched {
                    match vm.graph_channel {
                        GraphChannel::X | GraphChannel::Y => { vm.graph_zoom_y = 0.1; vm.graph_offset_y = 0.0; },
                        GraphChannel::Scale => { vm.graph_zoom_y = 50.0; vm.graph_offset_y = 1.0; },
                        GraphChannel::Rotation => { vm.graph_zoom_y = 0.5; vm.graph_offset_y = 0.0; },
                        GraphChannel::Opacity => { vm.graph_zoom_y = 100.0; vm.graph_offset_y = 0.5; },
                    }
                }
            });
            
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Slider::new(&mut vm.graph_zoom_x, 10.0..=1000.0).text("Zoom").show_value(false));
        });
    });
    
    ui.add_space(4.0);

    let content_width = (dur * vm.graph_zoom_x as f64).max((ui.available_width() - 8.0).into()) as f32;

    let mut seek_target = None;
    let mut modified_kf: Option<(String, f64, f32)> = None;
    let mut changed_easing: Option<(String, Easing)> = None;
    let mut modified_handles: Option<(String, [f32; 4])> = None;

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let graph_size = egui::Vec2::new(content_width, 160.0);
        let (resp, painter) = ui.allocate_painter(graph_size, egui::Sense::click_and_drag());
        let rect = resp.rect;

        painter.rect_filled(rect, 3.0, BG_BASE);
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, BORDER));

        for i in 1..4 {
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5_f32, egui::Color32::from_white_alpha(10)),
            );
        }

        let steps = 120usize;
        let mut prev_pt: Option<egui::Pos2> = None;

        let get_val = |state: &crate::models::types::animation::InterpolatedState| -> f32 {
            match vm.graph_channel {
                GraphChannel::X => state.x,
                GraphChannel::Y => state.y,
                GraphChannel::Scale => state.scale,
                GraphChannel::Rotation => state.rotation,
                GraphChannel::Opacity => state.opacity,
            }
        };
        
        let get_kf_val = |kf: &crate::models::types::animation::Keyframe| -> f32 {
            match vm.graph_channel {
                GraphChannel::X => kf.x,
                GraphChannel::Y => kf.y,
                GraphChannel::Scale => kf.scale,
                GraphChannel::Rotation => kf.rotation,
                GraphChannel::Opacity => kf.opacity,
            }
        };

        for step in 0..=steps {
            let t = dur * step as f64 / steps as f64;
            
            let val = if let Some(sub) = vm.selected_subtitle() {
                get_val(&sub.get_interpolated_state(start + t, &vm.project.subtitles, 0))
            } else if let Some(m) = vm.selected_media() {
                get_val(&m.get_interpolated_state(start + t))
            } else { Default::default() };

            let x = rect.min.x + (t / dur) as f32 * rect.width();
            let normalized_y = (val - vm.graph_offset_y) * vm.graph_zoom_y;
            let draw_y = rect.center().y - normalized_y;
            let clamped_y = draw_y.clamp(rect.min.y, rect.max.y);

            let pt = egui::Pos2::new(x, clamped_y);
            if let Some(prev) = prev_pt { painter.line_segment([prev, pt], egui::Stroke::new(1.5_f32, ACCENT_CYAN)); }
            prev_pt = Some(pt);
        }

        let mut sorted = keyframes;
        sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());

        let mut kf_interacted = false;

        if let Some(ref sel_id) = vm.selected_keyframe_id.clone() {
            if let Some(sel_idx) = sorted.iter().position(|k| &k.id == sel_id) {
                let curr_kf = &sorted[sel_idx];
                let curr_val = get_kf_val(curr_kf);
                let curr_x = rect.min.x + (curr_kf.time_offset / dur) as f32 * rect.width();
                let curr_y = (rect.center().y - (curr_val - vm.graph_offset_y) * vm.graph_zoom_y).clamp(rect.min.y + 6.0, rect.max.y - 6.0);
                let curr_pos = egui::pos2(curr_x, curr_y);

                if sel_idx > 0 {
                    let prev_kf = &sorted[sel_idx - 1];
                    let prev_val = get_kf_val(prev_kf);
                    let seg_dur = (curr_kf.time_offset - prev_kf.time_offset).max(0.001);
                    let val_diff = curr_val - prev_val;
                    let mut handles = easing_to_handles(&curr_kf.easing);

                    let h2_time = prev_kf.time_offset + handles[2] as f64 * seg_dur;
                    let h2_val = if val_diff.abs() > 0.001 {
                        prev_val + handles[3] * val_diff
                    } else {
                        curr_val + (handles[3] - 0.75) * (40.0 / vm.graph_zoom_y.max(0.01))
                    };
                    let h2_x = rect.min.x + (h2_time / dur) as f32 * rect.width();
                    let h2_y = (rect.center().y - (h2_val - vm.graph_offset_y) * vm.graph_zoom_y).clamp(rect.min.y + 6.0, rect.max.y - 6.0);
                    let h2_pos = egui::pos2(h2_x, h2_y);

                    painter.line_segment([curr_pos, h2_pos], egui::Stroke::new(1.2_f32, ACCENT_AMBER.linear_multiply(0.7)));

                    let h2_rect = Rect::from_center_size(h2_pos, Vec2::splat(14.0));
                    let h2_resp = ui.interact(h2_rect, ui.id().with(("h_in_left", &curr_kf.id)), egui::Sense::drag());
                    if h2_resp.hovered() || h2_resp.dragged() { kf_interacted = true; }

                    let h_col = if h2_resp.hovered() || h2_resp.dragged() { egui::Color32::WHITE } else { ACCENT_AMBER };
                    painter.circle_filled(h2_pos, 4.0, h_col);
                    painter.circle_stroke(h2_pos, 5.0, egui::Stroke::new(1.0_f32, BG_BASE));

                    if h2_resp.dragged() {
                        let d = h2_resp.drag_delta();
                        let dt = (d.x / rect.width() as f32) as f64 * dur;
                        handles[2] = (handles[2] + (dt / seg_dur) as f32).clamp(0.0, 1.0);
                        let d_val = -d.y / vm.graph_zoom_y.max(0.01);
                        let dh = if val_diff.abs() > 0.001 { d_val / val_diff } else { d_val * vm.graph_zoom_y.max(0.01) / 40.0 };
                        handles[3] = (handles[3] + dh).clamp(-0.5, 1.5);
                        modified_handles = Some((curr_kf.id.clone(), handles));
                    }
                }

                if sel_idx + 1 < sorted.len() {
                    let next_kf = &sorted[sel_idx + 1];
                    let next_val = get_kf_val(next_kf);
                    let seg_dur = (next_kf.time_offset - curr_kf.time_offset).max(0.001);
                    let val_diff = next_val - curr_val;
                    let mut handles = easing_to_handles(&next_kf.easing);

                    let h1_time = curr_kf.time_offset + handles[0] as f64 * seg_dur;
                    let h1_val = if val_diff.abs() > 0.001 {
                        curr_val + handles[1] * val_diff
                    } else {
                        curr_val + (handles[1] - 0.25) * (40.0 / vm.graph_zoom_y.max(0.01))
                    };
                    let h1_x = rect.min.x + (h1_time / dur) as f32 * rect.width();
                    let h1_y = (rect.center().y - (h1_val - vm.graph_offset_y) * vm.graph_zoom_y).clamp(rect.min.y + 6.0, rect.max.y - 6.0);
                    let h1_pos = egui::pos2(h1_x, h1_y);

                    painter.line_segment([curr_pos, h1_pos], egui::Stroke::new(1.2_f32, ACCENT_AMBER.linear_multiply(0.7)));

                    let h1_rect = Rect::from_center_size(h1_pos, Vec2::splat(14.0));
                    let h1_resp = ui.interact(h1_rect, ui.id().with(("h_out_left", &next_kf.id)), egui::Sense::drag());
                    if h1_resp.hovered() || h1_resp.dragged() { kf_interacted = true; }

                    let h_col = if h1_resp.hovered() || h1_resp.dragged() { egui::Color32::WHITE } else { ACCENT_AMBER };
                    painter.circle_filled(h1_pos, 4.0, h_col);
                    painter.circle_stroke(h1_pos, 5.0, egui::Stroke::new(1.0_f32, BG_BASE));

                    if h1_resp.dragged() {
                        let d = h1_resp.drag_delta();
                        let dt = (d.x / rect.width() as f32) as f64 * dur;
                        handles[0] = (handles[0] + (dt / seg_dur) as f32).clamp(0.0, 1.0);
                        let d_val = -d.y / vm.graph_zoom_y.max(0.01);
                        let dh = if val_diff.abs() > 0.001 { d_val / val_diff } else { d_val * vm.graph_zoom_y.max(0.01) / 40.0 };
                        handles[1] = (handles[1] + dh).clamp(-0.5, 1.5);
                        modified_handles = Some((next_kf.id.clone(), handles));
                    }
                }
            }
        }

        for kf in &sorted {
            let kx = rect.min.x + (kf.time_offset / dur) as f32 * rect.width();
            let is_selected_kf = vm.selected_keyframe_id.as_deref() == Some(&kf.id);

            let val = get_kf_val(kf);
            let normalized_y = (val - vm.graph_offset_y) * vm.graph_zoom_y;
            let ky = rect.center().y - normalized_y;
            let clamped_ky = ky.clamp(rect.min.y + 6.0, rect.max.y - 6.0);
            
            let pos = egui::Pos2::new(kx, clamped_ky);
            let kf_rect = Rect::from_center_size(pos, Vec2::splat(14.0));
            let kf_resp = ui.interact(kf_rect, ui.id().with(("kf_graph", &kf.id)), egui::Sense::click_and_drag());

            if kf_resp.hovered() || kf_resp.dragged() || kf_resp.secondary_clicked() {
                kf_interacted = true;
            }

            if kf_resp.clicked() {
                vm.selected_keyframe_id = Some(kf.id.clone());
            }

            let color = if is_selected_kf {
                ACCENT_AMBER
            } else if kf_resp.hovered() {
                egui::Color32::WHITE
            } else {
                ACCENT_CYAN
            };
            
            painter.circle_filled(pos, if is_selected_kf { 5.5 } else { 4.5 }, color);
            painter.circle_stroke(pos, if is_selected_kf { 7.0 } else { 5.5 }, egui::Stroke::new(1.0_f32, BG_BASE));

            if kf_resp.dragged() {
                let d = kf_resp.drag_delta();
                let dt = (d.x / rect.width() as f32) as f64 * dur;
                let dy = -d.y / vm.graph_zoom_y;
                
                let new_t = (kf.time_offset + dt).clamp(0.0, dur);
                let new_v = val + dy;
                modified_kf = Some((kf.id.clone(), new_t, new_v));
            }

            kf_resp.context_menu(|ui| {
                ui.label(egui::RichText::new("Keyframe Easing / Smoothing").strong());
                ui.separator();
                if ui.button("Linear").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::Linear));
                    ui.close_menu();
                }
                if ui.button("Cubic Out (Ease Out)").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::EaseOut));
                    ui.close_menu();
                }
                if ui.button("Cubic In (Ease In)").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::EaseIn));
                    ui.close_menu();
                }
                if ui.button("Cubic In/Out (Smooth)").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::EaseInOut));
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Bounce").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::Bounce));
                    ui.close_menu();
                }
                if ui.button("Elastic").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::Elastic));
                    ui.close_menu();
                }
                if ui.button("Back").clicked() {
                    changed_easing = Some((kf.id.clone(), Easing::Back));
                    ui.close_menu();
                }
            });
        }

        let ph_x = rect.min.x + (local_time / dur).clamp(0.0, 1.0) as f32 * rect.width();
        painter.line_segment(
            [egui::Pos2::new(ph_x, rect.min.y), egui::Pos2::new(ph_x, rect.max.y)],
            egui::Stroke::new(1.5_f32, ACCENT_AMBER),
        );
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(ph_x - 5.0, rect.min.y),
                egui::pos2(ph_x + 5.0, rect.min.y),
                egui::pos2(ph_x, rect.min.y + 8.0),
            ],
            ACCENT_AMBER,
            egui::Stroke::NONE,
        ));

        if vm.is_playing() && local_time >= 0.0 && local_time <= dur {
            ui.scroll_to_rect(
                Rect::from_center_size(egui::pos2(ph_x, rect.center().y), Vec2::new(80.0, 80.0)),
                None,
            );
        }

        let ph_head_rect = Rect::from_min_size(egui::pos2(ph_x - 8.0, rect.min.y), Vec2::new(16.0, 16.0));
        let ph_head_resp = ui.interact(ph_head_rect, ui.id().with("graph_ph_head_left"), egui::Sense::click_and_drag());
        if ph_head_resp.dragged() {
            if let Some(pos) = ph_head_resp.interact_pointer_pos() {
                let frac = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0) as f64;
                seek_target = Some(start + (dur * frac));
            }
        }

        if !kf_interacted && !ph_head_resp.dragged() && (resp.dragged() || resp.clicked()) {
            if let Some(pos) = resp.interact_pointer_pos() {
                let frac = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0) as f64;
                seek_target = Some(start + (dur * frac));
            }
        }

        if resp.hovered() {
            ui.input(|i| {
                let scroll = i.raw_scroll_delta;
                if i.modifiers.alt && scroll.y != 0.0 {
                    vm.graph_zoom_x = (vm.graph_zoom_x + scroll.y * 0.35).clamp(10.0, 1000.0);
                }
            });
            if resp.dragged_by(egui::PointerButton::Middle) {
                vm.graph_offset_y += resp.drag_delta().y / vm.graph_zoom_y;
            }
        }
    });

    if let Some((id, handles)) = modified_handles {
        if let Some(sub) = vm.selected_subtitle_mut() {
            if let Some(kf) = sub.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = Easing::Custom(handles);
            }
        }
        if let Some(m) = vm.selected_media_mut() {
            if let Some(kf) = m.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = Easing::Custom(handles);
            }
        }
        vm.mark_modified();
    }

    if let Some((id, t, v)) = modified_kf {
        let ch = vm.graph_channel;
        if let Some(sub) = vm.selected_subtitle_mut() {
            if let Some(kf) = sub.keyframes.iter_mut().find(|k| k.id == id) {
                kf.time_offset = t;
                match ch {
                    GraphChannel::X => kf.x = v,
                    GraphChannel::Y => kf.y = v,
                    GraphChannel::Scale => kf.scale = v,
                    GraphChannel::Rotation => kf.rotation = v,
                    GraphChannel::Opacity => kf.opacity = v,
                }
            }
            sub.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }
        if let Some(m) = vm.selected_media_mut() {
            if let Some(kf) = m.keyframes.iter_mut().find(|k| k.id == id) {
                kf.time_offset = t;
                match ch {
                    GraphChannel::X => kf.x = v,
                    GraphChannel::Y => kf.y = v,
                    GraphChannel::Scale => kf.scale = v,
                    GraphChannel::Rotation => kf.rotation = v,
                    GraphChannel::Opacity => kf.opacity = v,
                }
            }
            m.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }
        vm.mark_modified();
    }
    
    if let Some((id, new_easing)) = changed_easing {
        if let Some(sub) = vm.selected_subtitle_mut() {
            if let Some(kf) = sub.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = new_easing.clone();
            }
        }
        if let Some(m) = vm.selected_media_mut() {
            if let Some(kf) = m.keyframes.iter_mut().find(|k| k.id == id) {
                kf.easing = new_easing;
            }
        }
        vm.mark_modified();
    }

    if let Some(t) = seek_target {
        vm.seek_to(t);
    }
}