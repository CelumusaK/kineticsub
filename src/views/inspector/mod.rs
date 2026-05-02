// File: src/views/inspector/mod.rs
// ─────────────────────────────────────────────────────────────────────────────
use egui::{Context, SidePanel};
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_PANEL, BG_BASE, BORDER, TEXT_DIM, ACCENT_CYAN};

pub mod animate_tab;
pub mod text_tab;
pub mod transform_tab;
pub mod render_tab;
pub mod effects_tab;
pub mod words_tab;
pub mod widgets;

use animate_tab::draw_animate;
use text_tab::draw_text_props;
use transform_tab::draw_transform;
use render_tab::draw_render;
use effects_tab::draw_effects;
use words_tab::draw_words;

// ── Tab ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InspectorTab {
    #[default]
    Transform,
    Text,
    Words,
    Animate,
    Effects,
    Render,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn draw(ctx: &Context, vm: &mut EditorViewModel, tab: &mut InspectorTab) {
    SidePanel::right("inspector_panel")
        .default_width(320.0)
        .width_range(280.0..=500.0)
        .resizable(true)
        .frame(egui::Frame {
            fill: BG_PANEL,
            stroke: egui::Stroke::new(1.0, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            draw_tabs(ui, tab);
            ui.add(egui::Separator::default().spacing(0.0));
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                match *tab {
                    InspectorTab::Render => {
                        draw_render(ui, vm);
                    }
                    _ => {
                        if vm.selected_id.is_none() {
                            ui.add_space(24.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("No subtitle selected").color(TEXT_DIM).size(12.0));
                                ui.label(egui::RichText::new("Click one on the timeline").color(TEXT_DIM).size(11.0));
                            });
                        } else {
                            match *tab {
                                InspectorTab::Transform => draw_transform(ui, vm, ctx),
                                InspectorTab::Text      => draw_text_props(ui, vm),
                                InspectorTab::Words     => draw_words(ui, vm),
                                InspectorTab::Animate   => draw_animate(ui, vm),
                                InspectorTab::Effects   => draw_effects(ui, vm),
                                _ => {}
                            }
                        }
                    }
                }
            });
        });
}

// ── Tab bar ───────────────────────────────────────────────────────────────────

fn draw_tabs(ui: &mut egui::Ui, tab: &mut InspectorTab) {
    egui::Frame::none()
        .fill(BG_BASE)
        .inner_margin(egui::Margin::symmetric(10.0, 0.0))
        .show(ui, |ui| {
            ui.set_min_height(28.0);
            
            ui.horizontal(|ui| {
                tab_btn(ui, "TRN", *tab == InspectorTab::Transform, tab, InspectorTab::Transform);
                ui.add_space(2.0);
                tab_btn(ui, "TXT", *tab == InspectorTab::Text, tab, InspectorTab::Text);
                ui.add_space(2.0);
                tab_btn(ui, "WRD", *tab == InspectorTab::Words, tab, InspectorTab::Words);
                ui.add_space(2.0);
                tab_btn(ui, "ANI", *tab == InspectorTab::Animate, tab, InspectorTab::Animate);
                ui.add_space(2.0);
                tab_btn(ui, "FX", *tab == InspectorTab::Effects, tab, InspectorTab::Effects);
                ui.add_space(2.0);
                tab_btn(ui, "OUT", *tab == InspectorTab::Render, tab, InspectorTab::Render);
            });
        });
}

fn tab_btn(ui: &mut egui::Ui, label: &str, active: bool, tab: &mut InspectorTab, target: InspectorTab) {
    let color = if active { ACCENT_CYAN } else { TEXT_DIM };
    let resp = ui.add(egui::Label::new(egui::RichText::new(label).color(color).size(10.0).strong()).sense(egui::Sense::click()));
    if active {
        let r = resp.rect;
        ui.painter().line_segment([egui::pos2(r.min.x, r.max.y + 1.0), egui::pos2(r.max.x, r.max.y + 1.0)],
            egui::Stroke::new(2.0, ACCENT_CYAN),
        );
    }
    if resp.clicked() { *tab = target; }
}