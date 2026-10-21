use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{
    BG_BASE, BG_PANEL, BG_SELECTED, BORDER, TEXT_BRIGHT, TEXT_DIM, TEXT_MUTED,
};
use egui::{Context, Frame, Margin, RichText, Rounding, ScrollArea, Separator, SidePanel, Stroke};

pub mod animate_tab;
pub mod effects_tab;
pub mod text_tab;
pub mod transform_tab;
pub mod widgets;
pub mod words_tab;

use animate_tab::draw_animate;
use effects_tab::draw_effects;
use text_tab::draw_text_props;
use transform_tab::draw_transform;
use words_tab::draw_words;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InspectorTab {
    #[default]
    Transform,
    Text,
    Words,
    Animate,
    Effects,
}

pub fn draw(ctx: &Context, vm: &mut EditorViewModel, tab: &mut InspectorTab) {
    SidePanel::right("inspector_panel")
        .default_width(320.0)
        .width_range(280.0..=500.0)
        .resizable(true)
        .frame(Frame {
            fill: BG_PANEL,
            stroke: Stroke::new(1.0_f32, BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            draw_tabs(ui, tab);
            ui.add(Separator::default().spacing(0.0));
            ui.add_space(4.0);

            let mut disable_scroll = false;
            if *tab == InspectorTab::Animate {
                ui.input(|i| {
                    if i.modifiers.alt {
                        disable_scroll = true;
                    }
                });
            }

            ScrollArea::vertical()
                .enable_scrolling(!disable_scroll)
                .show(ui, |ui| {
                    if vm.selected_id.is_none() {
                        ui.add_space(32.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No clip selected")
                                    .color(TEXT_MUTED)
                                    .size(12.0),
                            );
                            ui.label(
                                RichText::new(
                                    "Select an element on the timeline to edit properties",
                                )
                                .color(TEXT_DIM)
                                .size(10.5),
                            );
                        });
                    } else {
                        match *tab {
                            InspectorTab::Transform => draw_transform(ui, vm, ctx),
                            InspectorTab::Text => draw_text_props(ui, vm),
                            InspectorTab::Words => draw_words(ui, vm),
                            InspectorTab::Animate => draw_animate(ui, vm),
                            InspectorTab::Effects => draw_effects(ui, vm),
                        }
                    }
                });
        });
}

fn draw_tabs(ui: &mut egui::Ui, tab: &mut InspectorTab) {
    Frame::none()
        .fill(BG_BASE)
        .inner_margin(Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.set_min_height(32.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                tab_btn(
                    ui,
                    "Transform",
                    *tab == InspectorTab::Transform,
                    tab,
                    InspectorTab::Transform,
                );
                tab_btn(
                    ui,
                    "Text",
                    *tab == InspectorTab::Text,
                    tab,
                    InspectorTab::Text,
                );
                tab_btn(
                    ui,
                    "Words",
                    *tab == InspectorTab::Words,
                    tab,
                    InspectorTab::Words,
                );
                tab_btn(
                    ui,
                    "Animate",
                    *tab == InspectorTab::Animate,
                    tab,
                    InspectorTab::Animate,
                );
                tab_btn(
                    ui,
                    "Effects",
                    *tab == InspectorTab::Effects,
                    tab,
                    InspectorTab::Effects,
                );
            });
        });
}

fn tab_btn(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    tab: &mut InspectorTab,
    target: InspectorTab,
) {
    let color = if active { TEXT_BRIGHT } else { TEXT_DIM };
    let bg_color = if active {
        BG_SELECTED
    } else {
        egui::Color32::TRANSPARENT
    };
    let resp = ui.add(
        egui::Button::new(RichText::new(label).color(color).size(11.0).strong())
            .fill(bg_color)
            .rounding(Rounding::same(4.0))
            .stroke(Stroke::NONE),
    );
    if resp.clicked() {
        *tab = target;
    }
}
