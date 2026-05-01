use serde::{Deserialize, Serialize};
use super::subtitle::Subtitle;

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
Easing::Linear => "Linear",
Easing::EaseIn => "Ease In",
Easing::EaseOut => "Ease Out",
Easing::EaseInOut => "Ease In/Out",
Easing::Bounce => "Bounce",
Easing::Elastic => "Elastic",
Easing::Back => "Back",
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
AnimationPreset::FadeIn => "Fade In",
AnimationPreset::FadeOut => "Fade Out",
AnimationPreset::SlideUp => "Slide Up",
AnimationPreset::SlideDown => "Slide Down",
AnimationPreset::SlideLeft => "Slide Left",
AnimationPreset::SlideRight => "Slide Right",
AnimationPreset::BounceIn => "Bounce In",
AnimationPreset::ZoomIn => "Zoom In",
AnimationPreset::ZoomOut => "Zoom Out",
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

// ── Easing functions ──────────────────────────────────────────────────────────

pub fn ease_out_cubic(t: f64) -> f64 { 1.0 - (1.0 - t).powi(3) }
pub fn ease_in_cubic(t: f64) -> f64 { t * t * t }
pub fn ease_in_out(t: f64) -> f64 {
if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}
pub fn ease_bounce(t: f64) -> f64 {
let n1 = 7.5625; let d1 = 2.75;
if t < 1.0/d1 { n1 * t * t }
else if t < 2.0/d1 { let t = t - 1.5/d1; n1 * t * t + 0.75 }
else if t < 2.5/d1 { let t = t - 2.25/d1; n1 * t * t + 0.9375 }
else { let t = t - 2.625/d1; n1 * t * t + 0.984375 }
}
pub fn ease_elastic(t: f64) -> f64 {
if t == 0.0 || t == 1.0 { return t; }
let c4 = std::f64::consts::TAU / 3.0;
-(2.0_f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
}
pub fn ease_back(t: f64) -> f64 {
let c1 = 1.70158; let c3 = c1 + 1.0;
c3 * t * t * t - c1 * t * t
}

pub fn apply_ease(t: f64, easing: &Easing) -> f64 {
let c = t.clamp(0.0, 1.0);
match easing {
Easing::Linear => c,
Easing::EaseIn => ease_in_cubic(c),
Easing::EaseOut => ease_out_cubic(c),
Easing::EaseInOut => ease_in_out(c),
Easing::Bounce => ease_bounce(c),
Easing::Elastic => 1.0 - ease_elastic(1.0 - c),
Easing::Back => { let o = ease_back(c); if o < 0.0 { 0.0 } else { o } }
}
}