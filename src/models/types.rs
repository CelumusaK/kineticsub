use serde::{Deserialize, Serialize};

// ── Easing ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

impl Easing {
    pub fn label(&self) -> &str {
        match self {
            Easing::Linear    => "Linear",
            Easing::EaseIn    => "Ease In",
            Easing::EaseOut   => "Ease Out",
            Easing::EaseInOut => "Ease In/Out",
            Easing::Bounce    => "Bounce",
            Easing::Elastic   => "Elastic",
            Easing::Back      => "Back",
        }
    }
    pub fn all() -> &'static [Easing] {
        &[Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut,
          Easing::Bounce, Easing::Elastic, Easing::Back]
    }
}

// ── Animation Presets ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPreset {
    FadeIn,
    FadeOut,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    BounceIn,
    ZoomIn,
    ZoomOut,
    TypeWriter,
}

impl AnimationPreset {
    pub fn label(&self) -> &str {
        match self {
            AnimationPreset::FadeIn     => "Fade In",
            AnimationPreset::FadeOut    => "Fade Out",
            AnimationPreset::SlideUp    => "Slide Up",
            AnimationPreset::SlideDown  => "Slide Down",
            AnimationPreset::SlideLeft  => "Slide Left",
            AnimationPreset::SlideRight => "Slide Right",
            AnimationPreset::BounceIn   => "Bounce In",
            AnimationPreset::ZoomIn     => "Zoom In",
            AnimationPreset::ZoomOut    => "Zoom Out",
            AnimationPreset::TypeWriter => "Typewriter",
        }
    }
    pub fn all() -> &'static [AnimationPreset] {
        &[
            AnimationPreset::FadeIn, AnimationPreset::FadeOut,
            AnimationPreset::SlideUp, AnimationPreset::SlideDown,
            AnimationPreset::SlideLeft, AnimationPreset::SlideRight,
            AnimationPreset::BounceIn, AnimationPreset::ZoomIn,
            AnimationPreset::ZoomOut,
        ]
    }

    pub fn generate_keyframes(&self, sub: &Subtitle) -> Vec<Keyframe> {
        let dur       = sub.duration();
        let in_dur    = (dur * 0.25).min(0.5);
        let out_start = (dur - (dur * 0.25).min(0.5)).max(in_dur);
        let base_x    = sub.x;
        let base_y    = sub.y;
        let slide_dist = 80.0_f32;

        match self {
            AnimationPreset::FadeIn => vec![
                kf(0.0,    base_x, base_y, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseOut),
                kf(in_dur, base_x, base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::FadeOut => vec![
                kf(out_start, base_x, base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseIn),
                kf(dur,       base_x, base_y, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseIn),
            ],
            AnimationPreset::SlideUp => vec![
                kf(0.0,    base_x, base_y + slide_dist, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseOut),
                kf(in_dur, base_x, base_y,              sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::SlideDown => vec![
                kf(0.0,    base_x, base_y - slide_dist, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseOut),
                kf(in_dur, base_x, base_y,              sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::SlideLeft => vec![
                kf(0.0,    base_x + slide_dist, base_y, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseOut),
                kf(in_dur, base_x,              base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::SlideRight => vec![
                kf(0.0,    base_x - slide_dist, base_y, sub.scale, sub.rotation, 0.0, 0.0, 0.0, Easing::EaseOut),
                kf(in_dur, base_x,              base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::BounceIn => vec![
                kf(0.0,        base_x, base_y + slide_dist, 0.3, sub.rotation, 0.0, 0.0, 0.0, Easing::Bounce),
                kf(in_dur*0.6, base_x, base_y - 10.0,       1.1, sub.rotation, 1.0, 0.0, 0.0, Easing::Bounce),
                kf(in_dur,     base_x, base_y,               1.0, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseOut),
            ],
            AnimationPreset::ZoomIn => vec![
                kf(0.0,    base_x, base_y, 0.0,       sub.rotation, 0.0, 0.0, 0.0, Easing::Back),
                kf(in_dur, base_x, base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::Back),
            ],
            AnimationPreset::ZoomOut => vec![
                kf(out_start, base_x, base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::EaseIn),
                kf(dur,       base_x, base_y, 0.0,       sub.rotation, 0.0, 0.0, 0.0, Easing::EaseIn),
            ],
            AnimationPreset::TypeWriter => vec![
                kf(0.0, base_x, base_y, sub.scale, sub.rotation, 1.0, 0.0, 0.0, Easing::Linear),
            ],
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn kf(t: f64, x: f32, y: f32, scale: f32, rotation: f32, opacity: f32,
      skew_x: f32, skew_y: f32, easing: Easing) -> Keyframe {
    Keyframe {
        id: format!("kf_{}", uuid::Uuid::new_v4()),
        time_offset: t,
        x, y, scale, rotation, opacity, skew_x, skew_y, easing,
    }
}

// ── Keyframe ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    pub id: String,
    pub time_offset: f64,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    #[serde(default)]
    pub skew_x: f32,
    #[serde(default)]
    pub skew_y: f32,
    pub easing: Easing,
}

#[derive(Debug, Clone)]
pub struct InterpolatedState {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    pub skew_x: f32,
    pub skew_y: f32,
}

// ── MediaFile ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub timeline_offset: f64,
    pub duration: f64,
    pub on_timeline: bool,
}

// ── TextAlignment ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self { TextAlign::Center }
}

// ── Subtitle ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtitle {
    pub id: String,
    pub media_id: Option<String>,
    pub text: String,
    pub timeline_start: f64,
    pub timeline_end: f64,
    // Transform
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    #[serde(default)]
    pub skew_x: f32,
    #[serde(default)]
    pub skew_y: f32,
    // Text style
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
    // Gradient
    #[serde(default)]
    pub gradient_enabled: bool,
    #[serde(default = "default_gradient_color")]
    pub gradient_color: [f32; 4],
    // Shadow
    #[serde(default)]
    pub shadow_enabled: bool,
    #[serde(default = "default_shadow_offset")]
    pub shadow_offset: [f32; 2],
    #[serde(default = "default_shadow_blur")]
    pub shadow_blur: f32,
    #[serde(default = "default_shadow_color")]
    pub shadow_color: [f32; 4],
    // Background box
    #[serde(default)]
    pub bg_box_enabled: bool,
    #[serde(default = "default_bg_box_color")]
    pub bg_box_color: [f32; 4],
    #[serde(default)]
    pub bg_box_padding: f32,
    // Letter spacing / line height
    #[serde(default)]
    pub letter_spacing: f32,
    // Alignment
    #[serde(default)]
    pub text_align: TextAlign,
    // Keyframes
    pub keyframes: Vec<Keyframe>,
}

fn default_stroke_width()   -> f32      { 2.0 }
fn default_stroke_color()   -> [f32; 4] { [0.0, 0.0, 0.0, 1.0] }
fn default_gradient_color() -> [f32; 4] { [1.0, 0.8, 0.0, 1.0] }
fn default_shadow_offset()  -> [f32; 2] { [3.0, 3.0] }
fn default_shadow_blur()    -> f32      { 6.0 }
fn default_shadow_color()   -> [f32; 4] { [0.0, 0.0, 0.0, 0.7] }
fn default_bg_box_color()   -> [f32; 4] { [0.0, 0.0, 0.0, 0.6] }

impl Subtitle {
    pub fn new(id: &str, text: &str, start: f64, end: f64) -> Self {
        Self {
            id: id.to_string(),
            media_id: None,
            text: text.to_string(),
            timeline_start: start,
            timeline_end: end,
            x: 0.0,
            y: 300.0,
            scale: 1.0,
            rotation: 0.0,
            opacity: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
            font_size: 36.0,
            bold: false,
            italic: false,
            color: [1.0, 1.0, 1.0, 1.0],
            stroke_enabled: false,
            stroke_width: 2.0,
            stroke_color: [0.0, 0.0, 0.0, 1.0],
            gradient_enabled: false,
            gradient_color: [1.0, 0.8, 0.0, 1.0],
            shadow_enabled: true,
            shadow_offset: [3.0, 3.0],
            shadow_blur: 6.0,
            shadow_color: [0.0, 0.0, 0.0, 0.7],
            bg_box_enabled: false,
            bg_box_color: [0.0, 0.0, 0.0, 0.6],
            bg_box_padding: 8.0,
            letter_spacing: 0.0,
            text_align: TextAlign::Center,
            keyframes: Vec::new(),
        }
    }

    pub fn duration(&self) -> f64 {
        self.timeline_end - self.timeline_start
    }

    pub fn get_interpolated_state(&self, current_time: f64) -> InterpolatedState {
        let mut base = InterpolatedState {
            x: self.x, y: self.y, scale: self.scale,
            rotation: self.rotation, opacity: self.opacity,
            skew_x: self.skew_x, skew_y: self.skew_y,
        };

        if self.keyframes.is_empty() {
            let local_time = current_time - self.timeline_start;
            let anim_dur = 0.08;
            if local_time >= 0.0 && local_time < anim_dur {
                let p = ease_out_cubic(local_time / anim_dur) as f32;
                base.scale  *= 0.4 + p * 0.6;
                base.opacity = p;
            } else if local_time < 0.0 {
                base.opacity = 0.0;
            }
            return base;
        }

        let mut sorted = self.keyframes.clone();
        sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
        let local_time = current_time - self.timeline_start;

        if local_time <= sorted.first().unwrap().time_offset {
            let k = sorted.first().unwrap();
            return InterpolatedState {
                x: k.x, y: k.y, scale: k.scale,
                rotation: k.rotation, opacity: k.opacity,
                skew_x: k.skew_x, skew_y: k.skew_y,
            };
        }
        if local_time >= sorted.last().unwrap().time_offset {
            let k = sorted.last().unwrap();
            return InterpolatedState {
                x: k.x, y: k.y, scale: k.scale,
                rotation: k.rotation, opacity: k.opacity,
                skew_x: k.skew_x, skew_y: k.skew_y,
            };
        }

        for i in 0..sorted.len() - 1 {
            let k1 = &sorted[i];
            let k2 = &sorted[i + 1];
            if local_time >= k1.time_offset && local_time <= k2.time_offset {
                let dur = k2.time_offset - k1.time_offset;
                let raw = if dur > 0.0 { (local_time - k1.time_offset) / dur } else { 1.0 };
                let e = apply_ease(raw, &k2.easing) as f32;
                return InterpolatedState {
                    x:        k1.x        + (k2.x        - k1.x)        * e,
                    y:        k1.y        + (k2.y        - k1.y)        * e,
                    scale:    k1.scale    + (k2.scale    - k1.scale)    * e,
                    rotation: k1.rotation + (k2.rotation - k1.rotation) * e,
                    opacity:  k1.opacity  + (k2.opacity  - k1.opacity)  * e,
                    skew_x:   k1.skew_x   + (k2.skew_x   - k1.skew_x)   * e,
                    skew_y:   k1.skew_y   + (k2.skew_y   - k1.skew_y)   * e,
                };
            }
        }
        base
    }

    pub fn keyframe_at(&self, local_time: f64) -> Option<&Keyframe> {
        self.keyframes.iter().find(|k| (k.time_offset - local_time).abs() < 0.02)
    }

    pub fn has_keyframe_nearby(&self, local_time: f64) -> bool {
        self.keyframes.iter().any(|k| (k.time_offset - local_time).abs() < 0.05)
    }

    pub fn prev_keyframe_time(&self, local_time: f64) -> Option<f64> {
        let mut sorted: Vec<f64> = self.keyframes.iter().map(|k| k.time_offset).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.into_iter().filter(|&t| t <= local_time - 0.02).last()
    }

    pub fn next_keyframe_time(&self, local_time: f64) -> Option<f64> {
        let mut sorted: Vec<f64> = self.keyframes.iter().map(|k| k.time_offset).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sorted.into_iter().find(|&t| t > local_time + 0.02)
    }
}

// ── Easing functions ──────────────────────────────────────────────────────────

pub fn ease_out_cubic(t: f64) -> f64 { 1.0 - (1.0 - t).powi(3) }
pub fn ease_in_cubic(t: f64) -> f64  { t * t * t }
pub fn ease_in_out(t: f64) -> f64    { if t < 0.5 { 4.0*t*t*t } else { 1.0 - (-2.0*t+2.0).powi(3)/2.0 } }
pub fn ease_bounce(t: f64) -> f64 {
    let n1 = 7.5625; let d1 = 2.75;
    if      t < 1.0/d1      { n1*t*t }
    else if t < 2.0/d1      { let t=t-1.5/d1;   n1*t*t + 0.75 }
    else if t < 2.5/d1      { let t=t-2.25/d1;  n1*t*t + 0.9375 }
    else                     { let t=t-2.625/d1; n1*t*t + 0.984375 }
}
pub fn ease_elastic(t: f64) -> f64 {
    if t == 0.0 || t == 1.0 { return t; }
    let c4 = std::f64::consts::TAU / 3.0;
    -(2.0_f64.powf(10.0*t - 10.0)) * ((t*10.0 - 10.75)*c4).sin()
}
pub fn ease_back(t: f64) -> f64 {
    let c1 = 1.70158; let c3 = c1 + 1.0;
    c3*t*t*t - c1*t*t
}

pub fn apply_ease(t: f64, easing: &Easing) -> f64 {
    let c = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear    => c,
        Easing::EaseIn    => ease_in_cubic(c),
        Easing::EaseOut   => ease_out_cubic(c),
        Easing::EaseInOut => ease_in_out(c),
        Easing::Bounce    => ease_bounce(c),
        Easing::Elastic   => 1.0 - ease_elastic(1.0 - c),
        Easing::Back      => { let o = ease_back(c); if o < 0.0 { 0.0 } else { o } }
    }
}

// ── Project ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    pub name: String,
    pub media_files: Vec<MediaFile>,
    pub subtitles: Vec<Subtitle>,
    pub duration: f64,
}