use super::widgets::*;
use crate::models::types::subtitle::WordAnimation;
use crate::viewmodels::editor_vm::EditorViewModel;
use crate::views::theme::{
    ACCENT_AMBER, ACCENT_CYAN, BG_BASE, BG_HOVER, BG_SELECTED, BORDER, TEXT_BRIGHT, TEXT_DIM,
    TEXT_MUTED, TEXT_NORM,
};
use egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

pub fn draw_words(ui: &mut egui::Ui, vm: &mut EditorViewModel) {
    let current_time = vm.current_time();

    // Ensure words exist for any text clip so words editor & word animation are directly accessible
    if let Some(sub) = vm.selected_subtitle_mut() {
        if sub.words.is_empty() && !sub.text.trim().is_empty() {
            sub.sync_words_from_text();
        }
    }

    let sub_info = match vm.selected_subtitle() {
        Some(s) => Some((
            s.id.clone(),
            s.timeline_start,
            s.timeline_end,
            s.word_animation.clone(),
            s.words.len(),
        )),
        None => None,
    };

    let (_sub_id, sub_start, sub_end, mut word_anim, word_count) = match sub_info {
        Some(info) => info,
        None => {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Word controls are only available for subtitles.")
                        .color(TEXT_DIM)
                        .size(11.0),
                );
            });
            return;
        }
    };

    let sub_dur = (sub_end - sub_start).max(0.01);
    let num_selected = vm.selected_ids.len();

    // ── ANIMATION STYLE (KARAOKE & CASCADE FADE) ──────────────────────────────
    ui.horizontal(|ui| {
        section_label(ui, "WORD ANIMATION STYLE");
        if num_selected > 1 {
            ui.label(
                egui::RichText::new(format!("({} clips selected)", num_selected))
                    .color(ACCENT_CYAN)
                    .size(9.5),
            );
        }
    });

    let mut anim_changed = false;

    let cur_mode = match &word_anim {
        WordAnimation::None => 0,
        WordAnimation::KaraokeHighlight { .. } => 1,
        WordAnimation::KaraokePop { .. } => 2,
        WordAnimation::CascadeFade => 3,
    };

    let mode_labels = [
        "None (Static Text)",
        "Karaoke Highlight",
        "Karaoke Pop & Bounce",
        "Cascade Fade",
    ];

    ui.horizontal_wrapped(|ui| {
        for (i, label) in mode_labels.iter().enumerate() {
            let is_sel = cur_mode == i;
            let btn = egui::Button::new(egui::RichText::new(*label).size(10.5).color(if is_sel {
                TEXT_BRIGHT
            } else {
                TEXT_NORM
            }))
            .fill(if is_sel { BG_SELECTED } else { BG_HOVER })
            .stroke(egui::Stroke::new(
                1.0_f32,
                if is_sel { ACCENT_CYAN } else { BORDER },
            ));

            if ui.add(btn).clicked() {
                word_anim = match i {
                    0 => WordAnimation::None,
                    1 => WordAnimation::KaraokeHighlight {
                        color: [1.0, 0.8, 0.0, 1.0],
                    },
                    2 => WordAnimation::KaraokePop { scale: 1.25 },
                    3 => WordAnimation::CascadeFade,
                    _ => WordAnimation::None,
                };
                anim_changed = true;
            }
        }
    });

    ui.add_space(4.0);

    egui::ComboBox::from_id_salt("word_anim_mode_combo")
        .selected_text(mode_labels[cur_mode])
        .width(ui.available_width() - 8.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(cur_mode == 0, mode_labels[0]).clicked() {
                word_anim = WordAnimation::None;
                anim_changed = true;
            }
            if ui.selectable_label(cur_mode == 1, mode_labels[1]).clicked() {
                word_anim = WordAnimation::KaraokeHighlight {
                    color: [1.0, 0.8, 0.0, 1.0],
                };
                anim_changed = true;
            }
            if ui.selectable_label(cur_mode == 2, mode_labels[2]).clicked() {
                word_anim = WordAnimation::KaraokePop { scale: 1.25 };
                anim_changed = true;
            }
            if ui.selectable_label(cur_mode == 3, mode_labels[3]).clicked() {
                word_anim = WordAnimation::CascadeFade;
                anim_changed = true;
            }
        });

    ui.add_space(4.0);
    match &mut word_anim {
        WordAnimation::KaraokeHighlight { color } => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Highlight Color")
                        .color(TEXT_NORM)
                        .size(10.5),
                );
                if ui.color_edit_button_rgba_unmultiplied(color).changed() {
                    anim_changed = true;
                }
            });
        }
        WordAnimation::KaraokePop { scale } => {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Pop Scale").color(TEXT_NORM).size(10.5));
                if ui
                    .add(egui::Slider::new(scale, 1.05..=2.0).text("x"))
                    .changed()
                {
                    anim_changed = true;
                }
            });
        }
        WordAnimation::CascadeFade => {
            ui.label(
                egui::RichText::new(
                    "Each word smoothly fades in sequentially according to its start/end window.",
                )
                .color(TEXT_DIM)
                .size(10.0),
            );
        }
        WordAnimation::None => {}
    }

    if anim_changed {
        let ids: Vec<String> = if !vm.selected_ids.is_empty() {
            vm.selected_ids.iter().cloned().collect()
        } else if let Some(sel) = &vm.selected_id {
            vec![sel.clone()]
        } else {
            vec![]
        };

        for sub in vm.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                sub.word_animation = word_anim.clone();
                // Ensure words exist so the animation style actually takes effect
                if sub.words.is_empty() && !sub.text.trim().is_empty() {
                    sub.sync_words_from_text();
                }
                // Override existing word styling overrides so the new animation takes over
                for w in &mut sub.words {
                    w.custom_color = None;
                }
            }
        }
        vm.mark_modified();
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(6.0);

    // ── WORD LIST AND TIMING ──────────────────────────────────────────────────
    if word_count == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("No words detected in this clip.")
                    .color(TEXT_MUTED)
                    .size(11.0),
            );
            if ui.button("✨ Generate Words from Text").clicked() {
                if let Some(sub) = vm.selected_subtitle_mut() {
                    sub.sync_words_from_text();
                }
                vm.mark_modified();
            }
        });
        return;
    }

    section_label(ui, "WORDS & TIMELINE CUES");

    let selected_idx_slot = ui.id().with("selected_word_idx");
    let mut selected_idx: usize = ui.data(|d| d.get_temp(selected_idx_slot).unwrap_or(0));
    if selected_idx >= word_count {
        selected_idx = 0;
    }

    // ── Visual Timeline Cue Bar ───────────────────────────────────────────────
    let bar_height = 20.0f32;
    let (bar_rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width() - 8.0, bar_height),
        egui::Sense::hover(),
    );
    let p = ui.painter_at(bar_rect);

    p.rect_filled(bar_rect, 3.0, BG_BASE);
    p.rect_stroke(bar_rect, 3.0, Stroke::new(1.0_f32, BORDER));

    let words_snapshot = match vm.selected_subtitle() {
        Some(s) => s.words.clone(),
        None => vec![],
    };

    let mut clicked_bar_word = None;
    for (i, word) in words_snapshot.iter().enumerate() {
        let frac_start = ((word.start - sub_start) / sub_dur).clamp(0.0, 1.0) as f32;
        let frac_end = ((word.end - sub_start) / sub_dur).clamp(0.0, 1.0) as f32;

        let w_x1 = bar_rect.min.x + frac_start * bar_rect.width();
        let w_x2 = (bar_rect.min.x + frac_end * bar_rect.width()).max(w_x1 + 2.0);

        let w_box = Rect::from_min_max(
            Pos2::new(w_x1, bar_rect.min.y + 2.0),
            Pos2::new(w_x2, bar_rect.max.y - 2.0),
        );
        let is_sel = selected_idx == i;
        let is_playing_word = current_time >= word.start && current_time <= word.end;

        let col = if is_sel {
            ACCENT_CYAN
        } else if is_playing_word {
            ACCENT_AMBER
        } else {
            Color32::from_rgba_unmultiplied(45, 185, 240, 75)
        };

        p.rect_filled(w_box, 2.0, col);

        let w_resp = ui.interact(w_box, ui.id().with(("w_bar", i)), egui::Sense::click());
        if w_resp.clicked() {
            clicked_bar_word = Some(i);
        }
    }

    // Draw playhead tick on visual cue bar
    if current_time >= sub_start && current_time <= sub_end {
        let ph_frac = ((current_time - sub_start) / sub_dur).clamp(0.0, 1.0) as f32;
        let ph_x = bar_rect.min.x + ph_frac * bar_rect.width();
        p.line_segment(
            [
                Pos2::new(ph_x, bar_rect.min.y),
                Pos2::new(ph_x, bar_rect.max.y),
            ],
            Stroke::new(1.5_f32, ACCENT_AMBER),
        );
    }

    if let Some(idx) = clicked_bar_word {
        selected_idx = idx;
        ui.data_mut(|d| d.insert_temp(selected_idx_slot, selected_idx));
    }

    ui.add_space(6.0);

    // ── Word Selector Chips ───────────────────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
        for (i, word) in words_snapshot.iter().enumerate() {
            let is_sel = selected_idx == i;
            let is_active_now = current_time >= word.start && current_time <= word.end;

            let text_color = if is_sel {
                BG_BASE
            } else if is_active_now {
                ACCENT_AMBER
            } else if word.custom_color.is_some() {
                ACCENT_CYAN
            } else {
                TEXT_BRIGHT
            };

            let bg_color = if is_sel {
                ACCENT_CYAN
            } else if is_active_now {
                Color32::from_rgba_unmultiplied(245, 158, 11, 40)
            } else {
                BG_HOVER
            };

            let resp = ui.add(
                egui::Button::new(egui::RichText::new(&word.text).color(text_color).size(10.5))
                    .fill(bg_color)
                    .stroke(Stroke::new(
                        1.0_f32,
                        if is_sel { ACCENT_CYAN } else { BORDER },
                    )),
            );

            if resp.clicked() {
                selected_idx = i;
                ui.data_mut(|d| d.insert_temp(selected_idx_slot, selected_idx));
            }
        }
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    // ── Selected Word Detail Editor ───────────────────────────────────────────
    let mut words_modified = false;
    let mut re_distribute_all = false;

    if let Some(sub) = vm.selected_subtitle_mut() {
        let total_words = sub.words.len();
        if selected_idx < total_words {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("WORD #{}:", selected_idx + 1))
                        .color(ACCENT_CYAN)
                        .size(10.5)
                        .strong(),
                );
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut sub.words[selected_idx].text)
                            .desired_width(120.0),
                    )
                    .changed()
                {
                    words_modified = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if selected_idx + 1 < total_words {
                        if ui.button("Next ⏭").clicked() {
                            selected_idx += 1;
                            ui.data_mut(|d| d.insert_temp(selected_idx_slot, selected_idx));
                        }
                    }
                    if selected_idx > 0 {
                        if ui.button("⏮ Prev").clicked() {
                            selected_idx -= 1;
                            ui.data_mut(|d| d.insert_temp(selected_idx_slot, selected_idx));
                        }
                    }
                });
            });

            ui.add_space(6.0);

            let word = &mut sub.words[selected_idx];
            let mut rel_start = (word.start - sub_start).clamp(0.0, sub_dur);
            let mut rel_end = (word.end - sub_start).clamp(0.0, sub_dur);

            let mut c1 = false;
            let mut c2 = false;
            ui.columns(2, |cols| {
                cols[0].label(
                    egui::RichText::new("Start Offset")
                        .color(TEXT_DIM)
                        .size(10.5),
                );
                if cols[0]
                    .add(
                        egui::DragValue::new(&mut rel_start)
                            .speed(0.02)
                            .suffix("s")
                            .range(0.0..=sub_dur),
                    )
                    .changed()
                {
                    c1 = true;
                }
                cols[1].label(egui::RichText::new("End Offset").color(TEXT_DIM).size(10.5));
                if cols[1]
                    .add(
                        egui::DragValue::new(&mut rel_end)
                            .speed(0.02)
                            .suffix("s")
                            .range(0.0..=sub_dur),
                    )
                    .changed()
                {
                    c2 = true;
                }
            });

            if c1 || c2 {
                if rel_start > rel_end {
                    rel_end = (rel_start + 0.05).min(sub_dur);
                }
                word.start = sub_start + rel_start;
                word.end = sub_start + rel_end;
                words_modified = true;
            }

            ui.add_space(4.0);

            // Timeline sync buttons
            ui.horizontal(|ui| {
                let is_inside_sub = current_time >= sub_start && current_time <= sub_end;

                if ui
                    .add_enabled(is_inside_sub, egui::Button::new("⏱ Set Start to Playhead"))
                    .clicked()
                {
                    word.start = current_time;
                    if word.end < word.start {
                        word.end = (word.start + 0.15).min(sub_end);
                    }
                    words_modified = true;
                }

                if ui
                    .add_enabled(is_inside_sub, egui::Button::new("⏱ Set End to Playhead"))
                    .clicked()
                {
                    word.end = current_time;
                    if word.start > word.end {
                        word.start = (word.end - 0.15).max(sub_start);
                    }
                    words_modified = true;
                }
            });

            ui.add_space(2.0);

            // Nudge controls
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Nudge:").color(TEXT_MUTED).size(10.0));
                if ui.button("◄ -50ms").clicked() {
                    word.start = (word.start - 0.05).max(sub_start);
                    word.end = (word.end - 0.05).max(word.start + 0.02);
                    words_modified = true;
                }
                if ui.button("+50ms ►").clicked() {
                    word.end = (word.end + 0.05).min(sub_end);
                    word.start = (word.start + 0.05).min(word.end - 0.02);
                    words_modified = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Custom color per word
            ui.horizontal(|ui| {
                let mut has_color = word.custom_color.is_some();
                if ui
                    .checkbox(
                        &mut has_color,
                        egui::RichText::new("Custom Color Override").size(10.5),
                    )
                    .changed()
                {
                    if has_color {
                        word.custom_color = Some([1.0, 0.8, 0.0, 1.0]);
                    } else {
                        word.custom_color = None;
                    }
                    words_modified = true;
                }

                if let Some(ref mut c) = word.custom_color {
                    if ui.color_edit_button_rgba_unmultiplied(c).changed() {
                        words_modified = true;
                    }
                }
            });
        }
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    // Global distribute helper
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("⚖ Distribute All Words Evenly").size(10.5))
            .clicked()
        {
            re_distribute_all = true;
        }
    });

    if re_distribute_all {
        if let Some(sub) = vm.selected_subtitle_mut() {
            let n = sub.words.len();
            if n > 0 {
                let slot = sub.duration() / n as f64;
                for (i, w) in sub.words.iter_mut().enumerate() {
                    w.start = sub.timeline_start + (i as f64 * slot);
                    w.end = w.start + slot * 0.95;
                }
                words_modified = true;
            }
        }
    }

    if words_modified {
        vm.mark_modified();
    }
}
