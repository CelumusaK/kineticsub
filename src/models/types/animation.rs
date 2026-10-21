use super::subtitle::Subtitle;
use serde::{Deserialize, Serialize};

// ── Loop Mode ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LoopMode {
    #[default]
    None,
    Loop,
    PingPong,
}

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
    Custom([f32; 4]),
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
            Easing::Custom(_) => "Custom Bezier",
        }
    }
    pub fn all() -> Vec<Easing> {
        vec![
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::Bounce,
            Easing::Elastic,
            Easing::Back,
            Easing::Custom([0.25, 0.1, 0.25, 1.0]),
        ]
    }
}

// ── IN Animation Presets ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InPreset {
    FadeIn,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    BounceIn,
    ZoomIn,
}

impl InPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FadeIn => "Fade In",
            Self::SlideUp => "Slide In Up",
            Self::SlideDown => "Slide In Down",
            Self::SlideLeft => "Slide In Left",
            Self::SlideRight => "Slide In Right",
            Self::BounceIn => "Bounce In",
            Self::ZoomIn => "Zoom In",
        }
    }

    pub fn all() -> &'static [InPreset] {
        &[
            Self::FadeIn,
            Self::SlideUp,
            Self::SlideDown,
            Self::SlideLeft,
            Self::SlideRight,
            Self::BounceIn,
            Self::ZoomIn,
        ]
    }

    pub fn generate_keyframes(&self, sub: &Subtitle) -> Vec<Keyframe> {
        let dur = sub.duration();
        let in_dur = (dur * 0.25).clamp(0.12, 0.6).min(dur * 0.45);
        let slide_dist = 80.0_f32;

        let b = Keyframe {
            id: String::new(),
            time_offset: 0.0,
            x: sub.x,
            y: sub.y,
            scale: sub.scale,
            rotation: sub.rotation,
            opacity: sub.opacity,
            mask_center: sub.mask_center,
            mask_size: sub.mask_size,
            mask_rotation: sub.mask_rotation,
            mask_feather: sub.mask_feather,
            easing: Easing::Linear,
        };

        match self {
            InPreset::FadeIn => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.opacity = 0.0;
                    k.easing = Easing::EaseOut;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::SlideUp => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.y = sub.y + slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseOut;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.y = sub.y;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::SlideDown => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.y = sub.y - slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseOut;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.y = sub.y;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::SlideLeft => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.x = sub.x + slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseOut;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.x = sub.x;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::SlideRight => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.x = sub.x - slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseOut;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.x = sub.x;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::BounceIn => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.y = sub.y + slide_dist;
                    k.scale = sub.scale * 0.3;
                    k.opacity = 0.0;
                    k.easing = Easing::Bounce;
                }),
                in_kf(in_dur * 0.6, b.clone(), |k| {
                    k.y = sub.y - 10.0;
                    k.scale = sub.scale * 1.1;
                    k.opacity = sub.opacity;
                    k.easing = Easing::Bounce;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.y = sub.y;
                    k.scale = sub.scale;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseOut;
                }),
            ],
            InPreset::ZoomIn => vec![
                in_kf(0.0, b.clone(), |k| {
                    k.scale = 0.0;
                    k.opacity = 0.0;
                    k.easing = Easing::Back;
                }),
                in_kf(in_dur, b.clone(), |k| {
                    k.scale = sub.scale;
                    k.opacity = sub.opacity;
                    k.easing = Easing::Back;
                }),
            ],
        }
    }
}

// ── OUT Animation Presets ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutPreset {
    FadeOut,
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    ZoomOut,
}

impl OutPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FadeOut => "Fade Out",
            Self::SlideUp => "Slide Out Up",
            Self::SlideDown => "Slide Out Down",
            Self::SlideLeft => "Slide Out Left",
            Self::SlideRight => "Slide Out Right",
            Self::ZoomOut => "Zoom Out",
        }
    }

    pub fn all() -> &'static [OutPreset] {
        &[
            Self::FadeOut,
            Self::SlideUp,
            Self::SlideDown,
            Self::SlideLeft,
            Self::SlideRight,
            Self::ZoomOut,
        ]
    }

    pub fn generate_keyframes(&self, sub: &Subtitle) -> Vec<Keyframe> {
        let dur = sub.duration();
        let in_dur = (dur * 0.25).clamp(0.12, 0.6).min(dur * 0.45);
        let out_dur = (dur * 0.25).clamp(0.12, 0.6).min(dur * 0.45);
        let out_start = (dur - out_dur).max(in_dur);
        let slide_dist = 80.0_f32;

        let b = Keyframe {
            id: String::new(),
            time_offset: 0.0,
            x: sub.x,
            y: sub.y,
            scale: sub.scale,
            rotation: sub.rotation,
            opacity: sub.opacity,
            mask_center: sub.mask_center,
            mask_size: sub.mask_size,
            mask_rotation: sub.mask_rotation,
            mask_feather: sub.mask_feather,
            easing: Easing::Linear,
        };

        match self {
            OutPreset::FadeOut => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
            OutPreset::SlideUp => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.y = sub.y;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.y = sub.y - slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
            OutPreset::SlideDown => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.y = sub.y;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.y = sub.y + slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
            OutPreset::SlideLeft => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.x = sub.x;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.x = sub.x - slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
            OutPreset::SlideRight => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.x = sub.x;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.x = sub.x + slide_dist;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
            OutPreset::ZoomOut => vec![
                out_kf(out_start, b.clone(), |k| {
                    k.scale = sub.scale;
                    k.opacity = sub.opacity;
                    k.easing = Easing::EaseIn;
                }),
                out_kf(dur, b.clone(), |k| {
                    k.scale = 0.0;
                    k.opacity = 0.0;
                    k.easing = Easing::EaseIn;
                }),
            ],
        }
    }
}

pub fn in_kf<F: FnOnce(&mut Keyframe)>(t: f64, mut base: Keyframe, apply: F) -> Keyframe {
    base.id = format!("in_kf_{}", uuid::Uuid::new_v4());
    base.time_offset = t;
    apply(&mut base);
    base
}

pub fn out_kf<F: FnOnce(&mut Keyframe)>(t: f64, mut base: Keyframe, apply: F) -> Keyframe {
    base.id = format!("out_kf_{}", uuid::Uuid::new_v4());
    base.time_offset = t;
    apply(&mut base);
    base
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
    pub mask_center: [f32; 2],
    #[serde(default = "def_mask_size")]
    pub mask_size: [f32; 2],
    #[serde(default)]
    pub mask_rotation: f32,
    #[serde(default)]
    pub mask_feather: f32,
    pub easing: Easing,
}

fn def_mask_size() -> [f32; 2] {
    [200.0, 200.0]
}

#[derive(Debug, Clone)]
pub struct InterpolatedState {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    pub mask_center: [f32; 2],
    pub mask_size: [f32; 2],
    pub mask_rotation: f32,
    pub mask_feather: f32,
}

// ── Unified Generic Interpolator ──────────────────────────────────────────────

pub fn interpolate_keyframes(
    current_time: f64,
    start_time: f64,
    keyframes: &[Keyframe],
    loop_mode: &LoopMode,
    mut base: InterpolatedState,
) -> InterpolatedState {
    let local_time = current_time - start_time;
    let mut eval_time = local_time;

    if !keyframes.is_empty() {
        let mut sorted = keyframes.to_vec();
        sorted.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());

        let first_t = sorted.first().unwrap().time_offset;
        let last_t = sorted.last().unwrap().time_offset;
        let kf_dur = last_t - first_t;

        if kf_dur > 0.0 && local_time > last_t {
            match loop_mode {
                LoopMode::None => eval_time = last_t,
                LoopMode::Loop => eval_time = first_t + ((local_time - first_t) % kf_dur),
                LoopMode::PingPong => {
                    let cycle = ((local_time - first_t) / kf_dur).floor() as i32;
                    let rem = (local_time - first_t) % kf_dur;
                    if cycle % 2 == 0 {
                        eval_time = first_t + rem;
                    } else {
                        eval_time = last_t - rem;
                    }
                }
            }
        } else if local_time < first_t {
            eval_time = first_t;
        }

        let mut applied_kf = false;
        for i in 0..sorted.len() - 1 {
            let k1 = &sorted[i];
            let k2 = &sorted[i + 1];
            if eval_time >= k1.time_offset && eval_time <= k2.time_offset {
                let dur = k2.time_offset - k1.time_offset;
                let raw = if dur > 0.0 {
                    (eval_time - k1.time_offset) / dur
                } else {
                    1.0
                };
                let e = apply_ease(raw, &k2.easing) as f32;

                base.x = k1.x + (k2.x - k1.x) * e;
                base.y = k1.y + (k2.y - k1.y) * e;
                base.scale = k1.scale + (k2.scale - k1.scale) * e;
                base.rotation = k1.rotation + (k2.rotation - k1.rotation) * e;
                base.opacity = k1.opacity + (k2.opacity - k1.opacity) * e;
                base.mask_center = [
                    k1.mask_center[0] + (k2.mask_center[0] - k1.mask_center[0]) * e,
                    k1.mask_center[1] + (k2.mask_center[1] - k1.mask_center[1]) * e,
                ];
                base.mask_size = [
                    k1.mask_size[0] + (k2.mask_size[0] - k1.mask_size[0]) * e,
                    k1.mask_size[1] + (k2.mask_size[1] - k1.mask_size[1]) * e,
                ];
                base.mask_rotation = k1.mask_rotation + (k2.mask_rotation - k1.mask_rotation) * e;
                base.mask_feather = k1.mask_feather + (k2.mask_feather - k1.mask_feather) * e;

                applied_kf = true;
                break;
            }
        }
        if !applied_kf {
            if eval_time <= first_t {
                let k = sorted.first().unwrap();
                base.x = k.x;
                base.y = k.y;
                base.scale = k.scale;
                base.rotation = k.rotation;
                base.opacity = k.opacity;
            } else if eval_time >= last_t {
                let k = sorted.last().unwrap();
                base.x = k.x;
                base.y = k.y;
                base.scale = k.scale;
                base.rotation = k.rotation;
                base.opacity = k.opacity;
            }
        }
    }
    base
}

// ── Easing functions ──────────────────────────────────────────────────────────

pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}
pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}
pub fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}
pub fn ease_bounce(t: f64) -> f64 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}
pub fn ease_elastic(t: f64) -> f64 {
    if t == 0.0 || t == 1.0 {
        return t;
    }
    let c4 = std::f64::consts::TAU / 3.0;
    -(2.0_f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
}
pub fn ease_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

pub fn solve_cubic_bezier(p: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    let mut t = p;
    for _ in 0..8 {
        let f = cubic_bezier(t, x1, x2) - p;
        if f.abs() < 0.001 {
            break;
        }
        let d = cubic_bezier_deriv(t, x1, x2);
        if d.abs() < 1e-6 {
            break;
        }
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
        Easing::Linear => c,
        Easing::EaseIn => ease_in_cubic(c),
        Easing::EaseOut => ease_out_cubic(c),
        Easing::EaseInOut => ease_in_out(c),
        Easing::Bounce => ease_bounce(c),
        Easing::Elastic => 1.0 - ease_elastic(1.0 - c),
        Easing::Back => {
            let o = ease_back(c);
            if o < 0.0 {
                0.0
            } else {
                o
            }
        }
        Easing::Custom(h) => {
            solve_cubic_bezier(c, h[0] as f64, h[1] as f64, h[2] as f64, h[3] as f64)
        }
    }
}
