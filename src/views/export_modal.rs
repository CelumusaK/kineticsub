use egui::{Align2, Context, Frame, Margin, RichText, Rounding, Stroke, Window};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::models::types::RenderMode;
use crate::views::theme::{ACCENT_AMBER, ACCENT_CYAN, ACCENT_EMERALD, BG_BASE, BG_PANEL, BORDER, TEXT_BRIGHT, TEXT_DIM, TEXT_MUTED};
use crate::views::inspector::widgets::{prop_row, section_label, two_col_row};

pub fn draw(ctx: &Context, vm: &mut EditorViewModel) {
    if !vm.show_export_modal {
        return;
    }

    let mut open = vm.show_export_modal;
    let mut close_requested = false;

    let center_pos = ctx.screen_rect().center();

    Window::new("Export Media")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(center_pos)
        .default_width(390.0)
        .frame(Frame {
            fill: BG_PANEL,
            stroke: Stroke::new(1.0, BORDER),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::same(18.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.add_enabled_ui(!vm.is_rendering, |ui| {
                section_label(ui, "CANVAS OUTPUT");
                
                ui.add_space(2.0);
                two_col_row(ui, |ui| {
                    ui.label(RichText::new("Width").color(TEXT_DIM).size(11.0));
                    ui.add(egui::DragValue::new(&mut vm.project.resolution.0).speed(10.0).range(100..=7680).suffix(" px"));
                }, |ui| {
                    ui.label(RichText::new("Height").color(TEXT_DIM).size(11.0));
                    ui.add(egui::DragValue::new(&mut vm.project.resolution.1).speed(10.0).range(100..=7680).suffix(" px"));
                });
                
                ui.add_space(4.0);
                prop_row(ui, "Frame Rate", |ui| {
                    ui.add(egui::DragValue::new(&mut vm.project.fps).speed(1.0).range(12..=120).suffix(" fps"));
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                section_label(ui, "FORMAT & ENCODING");

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut vm.render_mode, RenderMode::Video, "Standalone MP4 Video");
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut vm.render_mode, RenderMode::ImageSequence, "PNG Sequence (.zip archive)");
                });

                ui.add_space(10.0);

                ui.add_enabled_ui(vm.render_mode == RenderMode::ImageSequence, |ui| {
                    ui.checkbox(&mut vm.render_transparent_bg, "Preserve Alpha Transparency");
                });

                ui.add_space(2.0);

                ui.add_enabled_ui(vm.render_mode == RenderMode::Video, |ui| {
                    ui.checkbox(&mut vm.render_include_audio, "Mux Mixed Audio Channels");
                });
            });

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(14.0);

            let btn_text = if vm.is_rendering { "Rendering Video..." } else { "Export Project" };

            ui.vertical_centered(|ui| {
                if ui.add_sized(
                    [ui.available_width(), 34.0],
                    egui::Button::new(RichText::new(btn_text).strong().color(BG_BASE).size(12.0))
                        .fill(ACCENT_CYAN)
                        .rounding(5.0)
                ).clicked() && !vm.is_rendering {
                    vm.start_render();
                }
                
                if vm.is_rendering || vm.render_status.contains("Complete") || vm.render_status.contains("Error") {
                    ui.add_space(12.0);
                    
                    let color = if vm.render_status.contains("Error") {
                        ACCENT_AMBER
                    } else if vm.render_status.contains("Complete") {
                        ACCENT_EMERALD
                    } else {
                        TEXT_BRIGHT
                    };

                    ui.label(RichText::new(&vm.render_status).color(color).size(11.5));
                    
                    ui.add_space(6.0);
                    
                    let progress_bar = egui::ProgressBar::new(vm.render_progress)
                        .fill(ACCENT_CYAN)
                        .animate(vm.is_rendering);
                    ui.add(progress_bar);

                    if !vm.is_rendering {
                        ui.add_space(10.0);
                        if ui.button("Close").clicked() {
                            vm.render_status.clear();
                            close_requested = true;
                        }
                    }
                }
            });
        });

    if close_requested {
        open = false;
    }
    vm.show_export_modal = open;
}