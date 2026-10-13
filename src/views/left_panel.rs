use std::path::PathBuf;

use egui::{Align2, Color32, Context, FontId, Frame, Margin, Rect, RichText, Rounding, ScrollArea, SidePanel, Stroke, TopBottomPanel, Vec2, pos2, vec2};
use crate::models::types::MediaType;
use crate::viewmodels::editor_vm::EditorViewModel;
use super::theme::{
    ACCENT_AMBER, ACCENT_CYAN, BG_BASE, BG_HOVER, BG_PANEL, BG_PANEL_ALT, BORDER,
    TEXT_BRIGHT, TEXT_DIM, TEXT_MUTED, TEXT_NORM,
};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    SidePanel::left("left_panel")
        .default_width(260.0) 
        .width_range(200.0..=500.0)
        .resizable(true)
        .frame(Frame {
            fill: BG_PANEL,
            stroke: Stroke::new(1.0, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) {
                vm.hovered_panel = crate::viewmodels::editor_vm::HoverPanel::MediaBin;
            }

            let total_h = ui.available_height();
            let top_h = (total_h * vm.left_panel_split).max(150.0).min(total_h - 150.0);

            TopBottomPanel::top("left_media_bin")
                .exact_height(top_h)
                .frame(Frame::none())
                .show_inside(ui, |ui| {
                    draw_media_bin(ui, vm, ctx);
                });

            let sep_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 6.0));
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
                [pos2(sep_rect.min.x, sep_rect.center().y), pos2(sep_rect.max.x, sep_rect.center().y)], 
                Stroke::new(1.0, BORDER)
            );
            ui.allocate_space(Vec2::new(ui.available_width(), 6.0));

            egui::CentralPanel::default()
                .frame(Frame::none())
                .show_inside(ui, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            draw_subtitle_list(ui, vm);
                        });
                });
        });

    draw_modals(ctx, vm);
}

fn draw_media_bin(ui: &mut egui::Ui, vm: &mut EditorViewModel, ctx: &Context) {
    section_header(ui, "MEDIA ASSETS");
    
    let mut min_rect = ui.available_rect_before_wrap();
    min_rect.max.y = min_rect.max.y.max(min_rect.min.y + 100.0);
    
    let bg_resp = ui.interact(min_rect, ui.id().with("bin_bg"), egui::Sense::click());
    
    bg_resp.context_menu(|ui| {
        if ui.button("Import Media...").clicked() { vm.import_media(None); ui.close_menu(); }
        if ui.button("New Solid Background...").clicked() { vm.creating_color_bg = true; ui.close_menu(); }
    });

    let has_hovered_files = !ctx.input(|i| i.raw.hovered_files.is_empty());
    if has_hovered_files {
        ui.painter().rect_filled(min_rect, 4.0, Color32::from_rgba_unmultiplied(45, 185, 240, 25));
        ui.painter().rect_stroke(min_rect, 4.0, Stroke::new(1.5, ACCENT_CYAN));
        ui.painter().text(min_rect.center(), Align2::CENTER_CENTER, "Drop Files to Import", FontId::proportional(13.0), TEXT_BRIGHT);
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.indent("media_indent", |ui| {
                ui.add_space(2.0);

                if vm.project.media_files.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(32.0);
                        ui.label(RichText::new("Bin is Empty").color(TEXT_MUTED).size(11.0));
                        ui.label(RichText::new("Drag files here or right click to import").color(TEXT_DIM).size(10.0));
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
                        let icon = if is_open { "▾" } else { "▸" };
                        
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new(format!("{} {} ({})", icon, f.name, member_indices.len())).color(TEXT_NORM).size(10.5)).clicked() {
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
                    RichText::new(format!("⟳  {}", vm.whisper_status))
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

    let item_size = Vec2::new(72.0, 78.0);
    let (rect, resp) = ui.allocate_exact_size(item_size, egui::Sense::click_and_drag());
    
    if resp.drag_started() {
        vm.dragging_media_id = Some(id.clone());
    }

    let is_hovered = resp.hovered();
    let is_dragging = vm.dragging_media_id.as_deref() == Some(id.as_str());
    let is_selected = vm.selected_ids.contains(&id);

    let bg_color = if is_dragging || is_selected {
        Color32::from_rgba_unmultiplied(45, 185, 240, 35)
    } else if is_hovered {
        BG_HOVER
    } else {
        BG_PANEL_ALT
    };

    let stroke_color = if is_dragging || is_selected { ACCENT_CYAN } else if is_hovered { TEXT_MUTED } else { BORDER };

    ui.painter().rect_filled(rect, 5.0, bg_color);
    ui.painter().rect_stroke(rect, 5.0, Stroke::new(1.0, stroke_color));

    let thumb_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 44.0));
    ui.painter().rect_filled(
        thumb_rect, 
        Rounding { nw: 5.0, ne: 5.0, sw: 0.0, se: 0.0 }, 
        Color32::from_black_alpha(80)
    );

    if media_type == MediaType::Image {
        if let Some(tex) = vm.get_or_load_texture(ctx, &path) {
            ui.painter().image(
                tex.id(),
                thumb_rect.shrink(2.0),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(thumb_rect.center(), Align2::CENTER_CENTER, "IMG", FontId::proportional(11.0), TEXT_MUTED);
        }
    } else if media_type == MediaType::Video {
        let thumb_path = std::env::temp_dir().join("kineticsub_cache").join(&id).join("frame_0001.jpg");
        if let Some(tex) = vm.get_or_load_texture(ctx, thumb_path.to_str().unwrap_or("")) {
            ui.painter().image(
                tex.id(),
                thumb_rect.shrink(2.0),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(thumb_rect.center(), Align2::CENTER_CENTER, "VID", FontId::proportional(11.0), TEXT_NORM);
        }
    } else {
        ui.painter().text(thumb_rect.center(), Align2::CENTER_CENTER, "AUD", FontId::proportional(11.0), ACCENT_CYAN);
    }

    if let Some(fps) = source_fps {
        if fps > (vm.project.fps as f64 + 1.0) {
            ui.painter().circle_filled(rect.right_top() + vec2(-7.0, 7.0), 3.5, ACCENT_AMBER);
        }
    }

    let mut short_name = name.clone();
    if short_name.len() > 10 {
        short_name.truncate(9);
        short_name.push('…');
    }
    
    ui.painter().text(
        pos2(rect.center().x, rect.max.y - 18.0),
        Align2::CENTER_CENTER,
        short_name,
        FontId::proportional(10.0),
        if is_hovered { TEXT_BRIGHT } else { TEXT_NORM },
    );
    
    ui.painter().text(
        pos2(rect.center().x, rect.max.y - 6.0),
        Align2::CENTER_CENTER,
        format!("{:.1}s", duration),
        FontId::proportional(8.5),
        TEXT_MUTED,
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
        ui.label(RichText::new(&name).color(TEXT_BRIGHT).strong());
        ui.separator();
        
        if ui.button("Preview Clip").clicked() {
            preview_clicked = true;
            ui.close_menu();
        }

        ui.separator();
        if ui.button(RichText::new("Delete").color(ACCENT_AMBER)).clicked() {
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
        egui::Window::new("New Solid Plate")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .frame(Frame { fill: BG_PANEL, stroke: Stroke::new(1.0, BORDER), inner_margin: Margin::same(12.0), rounding: Rounding::same(8.0), ..Default::default() })
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
            egui::Window::new(format!("Preview — {}", m.name))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .frame(Frame { fill: BG_PANEL, stroke: Stroke::new(1.0, BORDER), inner_margin: Margin::same(14.0), rounding: Rounding::same(8.0), ..Default::default() })
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
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied((c[0]*255.) as u8, (c[1]*255.) as u8, (c[2]*255.) as u8, 255));
                        }
                        
                        if let Some(id) = tex_id {
                            ui.image(egui::load::SizedTexture::new(id, Vec2::new(w, h)));
                        } else if m.media_type == MediaType::Audio {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 70.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 4.0, BG_BASE);
                            ui.painter().text(rect.center(), Align2::CENTER_CENTER, "Audio Source", FontId::proportional(14.0), ACCENT_CYAN);
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

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            if ui.button("⏮ -5s").clicked() {
                                vm.preview_time = (vm.preview_time - 5.0).max(0.0);
                                vm.audio_player.stop("preview");
                                if vm.preview_playing && m.media_type != MediaType::Image {
                                    let audio_path = if m.media_type == MediaType::Video { std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav") } else { PathBuf::from(&m.path) };
                                    vm.audio_player.play("preview", audio_path, vm.preview_time, 1.0);
                                }
                            }
                            if ui.button(if vm.preview_playing { "Pause" } else { "Play" }).clicked() {
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
    section_header(ui, "SUBTITLE BLOCKS");
    if vm.project.subtitles.is_empty() {
        ui.indent("subs_empty", |ui| { ui.label(RichText::new("No subtitles generated yet.").color(TEXT_MUTED).size(10.5)); });
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
            Color32::from_rgba_unmultiplied(45, 185, 240, 24)
        } else if is_selected {
            Color32::from_rgba_unmultiplied(45, 185, 240, 10)
        } else {
            Color32::TRANSPARENT
        };
        let resp = Frame::none()
            .fill(row_fill)
            .inner_margin(Margin::symmetric(8.0, 3.5))
            .rounding(Rounding::same(4.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&time_label).color(TEXT_MUTED).size(10.0).monospace());
                    ui.add_space(2.0);
                    if has_link { ui.label(RichText::new("🔗").size(9.0).color(ACCENT_AMBER)); }
                    ui.add_space(2.0);
                    let label_color = if is_primary { ACCENT_CYAN } else { TEXT_NORM };
                    ui.label(RichText::new(&text_preview).color(label_color).size(11.0));
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
    ui.label(RichText::new(title).color(TEXT_MUTED).size(10.0).strong());
    ui.add_space(2.0);
}