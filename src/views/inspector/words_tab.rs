use egui;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{BG_HOVER, BORDER, TEXT_DIM, TEXT_NORM, TEXT_BRIGHT, ACCENT_CYAN, ACCENT_AMBER, BG_BASE};
use super::widgets::*;

pub fn draw_words(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    section_label(ui, "WORD EMPHASIS");
    
    let sub = match vm.selected_subtitle_mut() {
        Some(s) => s,
        None => return,
    };

    if sub.words.is_empty() {
        ui.label(egui::RichText::new("No words in this subtitle. Generate via Whisper first!").color(TEXT_DIM).size(10.5));
        return;
    }

    let selected_idx_id = ui.id().with("selected_word_idx");
    let mut selected_idx: Option<usize> = ui.data(|d| d.get_temp(selected_idx_id).unwrap_or(None));

    ui.label(egui::RichText::new("Select a word to apply custom styling.").color(TEXT_DIM).size(10.0));
    ui.add_space(4.0);

    // Draw Flow Layout of Words
    ui.horizontal_wrapped(|ui| {
        for (i, word) in sub.words.iter().enumerate() {
            let is_selected = selected_idx == Some(i);
            
            // Show custom color indicator if it has one
            let has_custom = word.custom_color.is_some();
            let text_col = if is_selected { BG_BASE } else if has_custom { ACCENT_AMBER } else { TEXT_BRIGHT };
            let bg_col = if is_selected { ACCENT_CYAN } else { BG_HOVER };

            let resp = ui.add(egui::Button::new(
                egui::RichText::new(&word.text).color(text_col).size(11.0)
            ).fill(bg_col).stroke(egui::Stroke::new(1.0, if is_selected { ACCENT_CYAN } else { BORDER })));

            if resp.clicked() {
                selected_idx = Some(i);
                ui.data_mut(|d| d.insert_temp(selected_idx_id, selected_idx));
            }
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    if let Some(idx) = selected_idx {
        if idx < sub.words.len() {
            let mut word_modified = false;
            let word = &mut sub.words[idx];

            section_label(ui, &format!("EDITING: \"{}\"", word.text));
            
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Custom Color").color(TEXT_NORM).size(10.5));
                
                let mut has_color = word.custom_color.is_some();
                if ui.checkbox(&mut has_color, "Enable").changed() {
                    if has_color {
                        word.custom_color = Some([1.0, 0.8, 0.0, 1.0]); // Default yellow
                    } else {
                        word.custom_color = None;
                    }
                    word_modified = true;
                }
            });

            if let Some(ref mut c) = word.custom_color {
                ui.add_space(4.0);
                if ui.color_edit_button_rgba_unmultiplied(c).changed() {
                    word_modified = true;
                }
            }

            if word_modified {
                vm.mark_modified();
            }
        }
    }
}