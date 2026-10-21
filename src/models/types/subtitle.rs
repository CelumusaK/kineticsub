use super::animation::{
    ease_out_cubic, Easing, InPreset, InterpolatedState, Keyframe, LoopMode, OutPreset,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BloomSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub radius: f32,
    pub color: [f32; 4],
}
impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 1.0,
            radius: 20.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrokeProps {
    pub enabled: bool,
    pub width: f32,
    pub color: [f32; 4],
}
impl Default for StrokeProps {
    fn default() -> Self {
        Self {
            enabled: true,
            width: 4.0,
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

fn default_bg_corner_radius() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.0]
}

// ── Mask Data ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MaskType {
    #[default]
    None,
    Straight,
    Rectangle,
    Circle,
}

fn def_mask_size() -> [f32; 2] {
    [200.0, 200.0]
}

// ── Word Animation Data (Karaoke) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum WordAnimation {
    #[default]
    None,
    KaraokeHighlight {
        color: [f32; 4],
    },
    KaraokePop {
        scale: f32,
    },
    CascadeFade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleWord {
    pub text: String,
    pub start: f64,
    pub end: f64,
    #[serde(default)]
    pub custom_color: Option<[f32; 4]>,
}

// ── TextAlignment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Center
    }
}

fn default_font_family() -> String {
    "Arial".to_string()
}

// ── Subtitle ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtitle {
    pub id: String,
    pub media_id: Option<String>,
    #[serde(default = "default_track_index")]
    pub track_index: usize,
    pub text: String,
    pub timeline_start: f64,
    pub timeline_end: f64,

    // Parenting & Grouping
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,

    // Transform
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,

    // Masks
    #[serde(default)]
    pub mask_type: MaskType,
    #[serde(default)]
    pub mask_invert: bool,
    #[serde(default)]
    pub mask_center: [f32; 2],
    #[serde(default = "def_mask_size")]
    pub mask_size: [f32; 2],
    #[serde(default)]
    pub mask_rotation: f32,
    #[serde(default)]
    pub mask_feather: f32,

    // Word Animation
    #[serde(default)]
    pub words: Vec<SubtitleWord>,
    #[serde(default)]
    pub word_animation: WordAnimation,

    // Text style
    #[serde(default = "default_font_family")]
    pub font_family: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub color: [f32; 4],

    // Stroke / outline
    #[serde(default)]
    pub stroke_enabled: bool,
    #[serde(default = "default_stroke_width")]
    pub stroke_width: f32,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: [f32; 4],

    // Multiple Strokes
    #[serde(default)]
    pub additional_strokes_enabled: bool,
    #[serde(default)]
    pub additional_strokes: Vec<StrokeProps>,

    // Gradient
    #[serde(default)]
    pub gradient_enabled: bool,
    #[serde(default = "default_gradient_color")]
    pub gradient_color: [f32; 4],

    // Shadow & Bloom
    #[serde(default)]
    pub shadow_enabled: bool,
    #[serde(default = "default_shadow_offset")]
    pub shadow_offset: [f32; 2],
    #[serde(default = "default_shadow_blur")]
    pub shadow_blur: f32,
    #[serde(default = "default_shadow_color")]
    pub shadow_color: [f32; 4],
    #[serde(default)]
    pub bloom: BloomSettings,

    // Background box
    #[serde(default)]
    pub bg_box_enabled: bool,
    #[serde(default = "default_bg_box_color")]
    pub bg_box_color: [f32; 4],
    #[serde(default)]
    pub bg_box_padding: f32,
    #[serde(default = "default_bg_corner_radius")]
    pub bg_box_radius: [f32; 4],

    // Advanced Mechanics
    #[serde(default)]
    pub loop_mode: LoopMode,

    // Letter spacing / line height
    #[serde(default)]
    pub letter_spacing: f32,
    // Alignment
    #[serde(default)]
    pub text_align: TextAlign,
    // Effects
    #[serde(default)]
    pub motion_blur: f32,

    // In & Out Animation Presets
    #[serde(default)]
    pub in_preset: Option<InPreset>,
    #[serde(default)]
    pub out_preset: Option<OutPreset>,

    // Keyframes
    pub keyframes: Vec<Keyframe>,
    #[serde(skip)]
    pub has_collision: bool,
}

fn default_track_index() -> usize {
    1
}
fn default_stroke_width() -> f32 {
    2.0
}
fn default_stroke_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}
fn default_gradient_color() -> [f32; 4] {
    [1.0, 0.8, 0.0, 1.0]
}
fn default_shadow_offset() -> [f32; 2] {
    [3.0, 3.0]
}
fn default_shadow_blur() -> f32 {
    6.0
}
fn default_shadow_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.7]
}
fn default_bg_box_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 0.6]
}

impl Subtitle {
    pub fn new(id: &str, text: &str, start: f64, end: f64) -> Self {
        let mut s = Self {
            id: id.to_string(),
            media_id: None,
            track_index: 1,
            parent_id: None,
            group_id: None,
            text: text.to_string(),
            timeline_start: start,
            timeline_end: end,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rotation: 0.0,
            opacity: 1.0,
            mask_type: MaskType::None,
            mask_invert: false,
            mask_center: [0.0, 0.0],
            mask_size: [200.0, 200.0],
            mask_rotation: 0.0,
            mask_feather: 0.0,
            words: Vec::new(),
            word_animation: WordAnimation::None,
            font_family: "Arial".to_string(),
            font_size: 36.0,
            bold: false,
            italic: false,
            color: [1.0, 1.0, 1.0, 1.0],
            stroke_enabled: false,
            stroke_width: 2.0,
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            additional_strokes_enabled: false,
            additional_strokes: Vec::new(),
            gradient_enabled: false,
            gradient_color: [1.0, 0.8, 0.0, 1.0],
            shadow_enabled: true,
            shadow_offset: [3.0, 3.0],
            shadow_blur: 6.0,
            shadow_color: [0.0, 0.0, 0.0, 0.7],
            bloom: BloomSettings::default(),
            bg_box_enabled: false,
            bg_box_color: [0.0, 0.0, 0.0, 0.6],
            bg_box_padding: 8.0,
            bg_box_radius: [0.0, 0.0, 0.0, 0.0],
            loop_mode: LoopMode::None,
            letter_spacing: 0.0,
            text_align: TextAlign::Center,
            motion_blur: 0.0,
            in_preset: None,
            out_preset: None,
            keyframes: Vec::new(),
            has_collision: false,
        };
        s.sync_words_from_text();
        s
    }

    pub fn duration(&self) -> f64 {
        self.timeline_end - self.timeline_start
    }

    pub fn sync_words_from_text(&mut self) {
        let text_words: Vec<&str> = self.text.split_whitespace().collect();
        if text_words.is_empty() {
            self.words.clear();
            return;
        }
        let total = text_words.len();
        let dur = (self.timeline_end - self.timeline_start).max(0.05);
        let slot = dur / total as f64;
        self.words = text_words
            .into_iter()
            .enumerate()
            .map(|(i, w)| SubtitleWord {
                text: w.to_string(),
                start: self.timeline_start + (i as f64 * slot),
                end: self.timeline_start + ((i as f64 + 1.0) * slot),
                custom_color: None,
            })
            .collect();
    }

    pub fn sync_words_text(&mut self) {
        let text_words: Vec<&str> = self.text.split_whitespace().collect();
        if text_words.is_empty() {
            self.words.clear();
            return;
        }
        if self.words.is_empty() {
            self.sync_words_from_text();
            return;
        }
        if self.words.len() == text_words.len() {
            for (w, tw) in self.words.iter_mut().zip(text_words.iter()) {
                w.text = tw.to_string();
            }
        } else {
            let start = self
                .words
                .first()
                .map(|w| w.start)
                .unwrap_or(self.timeline_start);
            let end = self
                .words
                .last()
                .map(|w| w.end)
                .unwrap_or(self.timeline_end);
            let dur = (end - start).max(0.05);
            let slot = dur / text_words.len() as f64;
            self.words = text_words
                .into_iter()
                .enumerate()
                .map(|(i, tw)| SubtitleWord {
                    text: tw.to_string(),
                    start: start + (i as f64 * slot),
                    end: start + ((i as f64 + 1.0) * slot),
                    custom_color: None,
                })
                .collect();
        }
    }

    pub fn apply_in_preset(&mut self, preset: Option<InPreset>) {
        self.keyframes.retain(|k| !k.id.starts_with("in_kf_"));
        if let Some(p) = &preset {
            let kfs = p.generate_keyframes(self);
            self.keyframes.extend(kfs);
            self.keyframes
                .sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }
        self.in_preset = preset;
    }

    pub fn apply_out_preset(&mut self, preset: Option<OutPreset>) {
        self.keyframes.retain(|k| !k.id.starts_with("out_kf_"));
        if let Some(p) = &preset {
            let kfs = p.generate_keyframes(self);
            self.keyframes.extend(kfs);
            self.keyframes
                .sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        }
        self.out_preset = preset;
    }

    pub fn apply_state_auto_key(
        &mut self,
        local_t: f64,
        record_mode: bool,
        state: &InterpolatedState,
    ) {
        if !self.keyframes.is_empty() || record_mode {
            if let Some(kf) = self
                .keyframes
                .iter_mut()
                .find(|k| (k.time_offset - local_t).abs() < 0.02)
            {
                kf.x = state.x;
                kf.y = state.y;
                kf.scale = state.scale;
                kf.rotation = state.rotation;
                kf.opacity = state.opacity;
            } else {
                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_t,
                    x: state.x,
                    y: state.y,
                    scale: state.scale,
                    rotation: state.rotation,
                    opacity: state.opacity,
                    mask_center: state.mask_center,
                    mask_size: state.mask_size,
                    mask_rotation: state.mask_rotation,
                    mask_feather: state.mask_feather,
                    easing: Easing::Linear,
                };
                self.keyframes.push(kf);
                self.keyframes
                    .sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        } else {
            self.x = state.x;
            self.y = state.y;
            self.scale = state.scale;
            self.rotation = state.rotation;
            self.opacity = state.opacity;
        }
    }

    pub fn get_interpolated_state(
        &self,
        current_time: f64,
        all_subs: &[Subtitle],
        depth: usize,
    ) -> InterpolatedState {
        let mut base = InterpolatedState {
            x: self.x,
            y: self.y,
            scale: self.scale,
            rotation: self.rotation,
            opacity: self.opacity,
            mask_center: self.mask_center,
            mask_size: self.mask_size,
            mask_rotation: self.mask_rotation,
            mask_feather: self.mask_feather,
        };

        let local_time = current_time - self.timeline_start;

        base = crate::models::types::animation::interpolate_keyframes(
            current_time,
            self.timeline_start,
            &self.keyframes,
            &self.loop_mode,
            base,
        );

        if self.keyframes.is_empty() {
            let anim_dur = 0.08;
            if local_time >= 0.0 && local_time < anim_dur {
                let p = ease_out_cubic(local_time / anim_dur) as f32;
                base.scale *= 0.4 + p * 0.6;
                base.opacity = p;
            } else if local_time < 0.0 {
                base.opacity = 0.0;
            }
        }

        if depth < 5 {
            if let Some(pid) = &self.parent_id {
                if let Some(parent) = all_subs.iter().find(|s| &s.id == pid) {
                    let p_state = parent.get_interpolated_state(current_time, all_subs, depth + 1);
                    base.x += p_state.x;
                    base.y += p_state.y;
                    base.scale *= p_state.scale;
                    base.rotation += p_state.rotation;
                }
            }
        }

        base
    }

    pub fn keyframe_at(&self, local_time: f64) -> Option<&Keyframe> {
        self.keyframes
            .iter()
            .find(|k| (k.time_offset - local_time).abs() < 0.02)
    }

    pub fn has_keyframe_nearby(&self, local_time: f64) -> bool {
        self.keyframes
            .iter()
            .any(|k| (k.time_offset - local_time).abs() < 0.05)
    }

    pub fn prev_keyframe_time(&self, local_time: f64) -> Option<f64> {
        let mut sorted: Vec<f64> = self.keyframes.iter().map(|k| k.time_offset).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted
            .into_iter()
            .filter(|&t| t <= local_time - 0.02)
            .last()
    }

    pub fn next_keyframe_time(&self, local_time: f64) -> Option<f64> {
        let mut sorted: Vec<f64> = self.keyframes.iter().map(|k| k.time_offset).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.into_iter().find(|&t| t > local_time + 0.02)
    }
}
