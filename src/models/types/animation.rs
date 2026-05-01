use serde::{Deserialize, Serialize};
use super::subtitle::Subtitle;

// ── Easing ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Easing {
    Linear, EaseIn, EaseOut, EaseInOut, Bounce, Elastic, Back,
    Custom([f32; 4]),
}

impl Easing {
    pub fn label(&self) -> &str {
        match self {
            Easing::Linear => "Linear", Easing::EaseIn => "Ease In", Easing::EaseOut => "Ease Out",
            Easing::EaseInOut => "Ease In/Out", Easing::Bounce => "Bounce", Easing::Elastic => "Elastic",
            Easing::Back => "Back", Easing::Custom(_) => "Custom Bezier",
        }
    }
    pub fn all() -> Vec<Easing> {
        vec![
            Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut, 
            Easing::Bounce, Easing::Elastic, Easing::Back, 
            Easing::Custom([0.25, 0.1, 0.25, 1.0])
        ]
    }
}

// ── Animation Presets ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPreset {
    FadeIn, FadeOut, SlideUp, SlideDown, SlideLeft, SlideRight, BounceIn, ZoomIn, ZoomOut, TypeWriter,
}

impl AnimationPreset {
    pub fn label(&self) -> &str {
        match self {
            AnimationPreset::FadeIn => "Fade In", AnimationPreset::FadeOut => "Fade Out",
            AnimationPreset::SlideUp => "Slide Up", AnimationPreset::SlideDown => "Slide Down",
            AnimationPreset::SlideLeft => "Slide Left", AnimationPreset::SlideRight => "Slide Right",
            AnimationPreset::BounceIn => "Bounce In", AnimationPreset::ZoomIn => "Zoom In",
            AnimationPreset::ZoomOut => "Zoom Out", AnimationPreset::TypeWriter => "Typewriter",
        }
    }
    
    pub fn all() -> &'static [AnimationPreset] {
        &[
            AnimationPreset::FadeIn, AnimationPreset::FadeOut, AnimationPreset::SlideUp, AnimationPreset::SlideDown,
            AnimationPreset::SlideLeft, AnimationPreset::SlideRight, AnimationPreset::BounceIn, AnimationPreset::ZoomIn,
            AnimationPreset::ZoomOut,
        ]
    }

    pub fn generate_keyframes(&self, sub: &Subtitle) -> Vec<Keyframe> {
        let dur       = sub.duration();
        let in_dur    = (dur * 0.25).min(0.5);
        let out_start = (dur - (dur * 0.25).min(0.5)).max(in_dur);
        let slide_dist = 80.0_f32;

        let b = Keyframe {
            id: String::new(), time_offset: 0.0,
            x: sub.x, y: sub.y, scale: sub.scale, rotation: sub.rotation, opacity: sub.opacity,
            skew_x: sub.skew_x, skew_y: sub.skew_y, yaw: sub.yaw, pitch: sub.pitch,
            path_progress: sub.path_progress,
            mask_center: sub.mask_center, mask_size: sub.mask_size,
            mask_rotation: sub.mask_rotation, mask_feather: sub.mask_feather,
            easing: Easing::Linear,
        };

        match self {
            AnimationPreset::FadeIn => vec![
                kf(0.0,    b.clone(), |k| { k.opacity = 0.0; k.easing = Easing::EaseOut; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::FadeOut => vec![
                kf(out_start, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseIn; }),
                kf(dur,       b.clone(), |k| { k.opacity = 0.0; k.easing = Easing::EaseIn; }),
            ],
            AnimationPreset::SlideUp => vec![
                kf(0.0,    b.clone(), |k| { k.y += slide_dist; k.opacity = 0.0; k.easing = Easing::EaseOut; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::SlideDown => vec![
                kf(0.0,    b.clone(), |k| { k.y -= slide_dist; k.opacity = 0.0; k.easing = Easing::EaseOut; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::SlideLeft => vec![
                kf(0.0,    b.clone(), |k| { k.x += slide_dist; k.opacity = 0.0; k.easing = Easing::EaseOut; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::SlideRight => vec![
                kf(0.0,    b.clone(), |k| { k.x -= slide_dist; k.opacity = 0.0; k.easing = Easing::EaseOut; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::BounceIn => vec![
                kf(0.0,        b.clone(), |k| { k.y += slide_dist; k.scale = 0.3; k.opacity = 0.0; k.easing = Easing::Bounce; }),
                kf(in_dur*0.6, b.clone(), |k| { k.y -= 10.0;       k.scale = 1.1; k.opacity = 1.0; k.easing = Easing::Bounce; }),
                kf(in_dur,     b.clone(), |k| { k.scale = 1.0; k.opacity = 1.0; k.easing = Easing::EaseOut; }),
            ],
            AnimationPreset::ZoomIn => vec![
                kf(0.0,    b.clone(), |k| { k.scale = 0.0; k.opacity = 0.0; k.easing = Easing::Back; }),
                kf(in_dur, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::Back; }),
            ],
            AnimationPreset::ZoomOut => vec![
                kf(out_start, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::EaseIn; }),
                kf(dur,       b.clone(), |k| { k.scale = 0.0; k.opacity = 0.0; k.easing = Easing::EaseIn; }),
            ],
            AnimationPreset::TypeWriter => vec![
                kf(0.0, b.clone(), |k| { k.opacity = 1.0; k.easing = Easing::Linear; }),
            ],
        }
    }
}

fn kf<F: FnOnce(&mut Keyframe)>(t: f64, mut base: Keyframe, apply: F) -> Keyframe {
    base.id = format!("kf_{}", uuid::Uuid::new_v4());
    base.time_offset = t;
    apply(&mut base);
    base
}

// ── Keyframe ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    pub id: String,
    pub time_offset: f64,
    pub x: f32, pub y: f32, pub scale: f32, pub rotation: f32, pub opacity: f32,
    #[serde(default)] pub skew_x: f32,
    #[serde(default)] pub skew_y: f32,
    #[serde(default)] pub yaw: f32,
    #[serde(default)] pub pitch: f32,
    #[serde(default)] pub path_progress: f32,
    #[serde(default)] pub mask_center: [f32; 2],
    #[serde(default = "def_mask_size")] pub mask_size: [f32; 2],
    #[serde(default)] pub mask_rotation: f32,
    #[serde(default)] pub mask_feather: f32,
    pub easing: Easing,
}

fn def_mask_size() -> [f32; 2] { [200.0, 200.0] }

#[derive(Debug, Clone)]
pub struct InterpolatedState {
    pub x: f32, pub y: f32, pub scale: f32, pub rotation: f32, pub opacity: f32,
    pub skew_x: f32, pub skew_y: f32, pub yaw: f32, pub pitch: f32, pub path_progress: f32,
    pub mask_center: [f32; 2], pub mask_size: [f32; 2], pub mask_rotation: f32, pub mask_feather: f32,
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

pub fn solve_cubic_bezier(p: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if p <= 0.0 { return 0.0; }
    if p >= 1.0 { return 1.0; }
    let mut t = p;
    for _ in 0..8 {
        let f = cubic_bezier(t, x1, x2) - p;
        if f.abs() < 0.001 { break; }
        let d = cubic_bezier_deriv(t, x1, x2);
        if d.abs() < 1e-6 { break; }
        t -= f / d;
    }
    t = t.clamp(0.0, 1.0);
    cubic_bezier(t, y1, y2)
}

fn cubic_bezier(t: f64, p1: f64, p2: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
}

fn cubic_bezier_deriv(t: f64, p1: f64, p2: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * p1 + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

pub fn apply_ease(t: f64, easing: &Easing) -> f64 {
    let c = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => c, Easing::EaseIn => ease_in_cubic(c), Easing::EaseOut => ease_out_cubic(c),
        Easing::EaseInOut => ease_in_out(c), Easing::Bounce => ease_bounce(c),
        Easing::Elastic => 1.0 - ease_elastic(1.0 - c), Easing::Back => { let o = ease_back(c); if o < 0.0 { 0.0 } else { o } }
        Easing::Custom(h) => solve_cubic_bezier(c, h[0] as f64, h[1] as f64, h[2] as f64, h[3] as f64),
    }
}