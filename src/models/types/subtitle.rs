use serde::{Deserialize, Serialize};
use super::animation::{Keyframe, InterpolatedState, ease_out_cubic, LoopMode, Easing};

// ── Text Advanced Features ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TextDeform {
    #[default]
    None,
    Arc,
    Bulge,
    Wave,
    Flag,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TextFillMode {
    #[default]
    Solid,
    Gradient,
    ImageTexture,
    VideoTexture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum BlendMode { #[default] Normal, Multiply, Screen, Overlay, ColorDodge }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TrackMatte { #[default] None, Alpha, AlphaInverted, Luma, LumaInverted }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlitchSettings { pub enabled: bool, pub rgb_split: f32, pub intensity: f32, pub scanlines: bool }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BloomSettings { pub enabled: bool, pub intensity: f32, pub radius: f32, pub color: [f32; 4] }
impl Default for BloomSettings { fn default() -> Self { Self { enabled: false, intensity: 1.0, radius: 20.0, color: [1.0, 1.0, 1.0, 1.0] } } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrokeProps { pub enabled: bool, pub width: f32, pub color: [f32; 4] }
impl Default for StrokeProps { fn default() -> Self { Self { enabled: true, width: 4.0, color: [0.0, 0.0, 0.0, 1.0] } } }

fn default_bg_corner_radius() -> [f32; 4] { [0.0, 0.0, 0.0, 0.0] }

// ── Motion Path Data ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PathType {
    #[default]
    None,
    Circle,
    Star,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PathNode {
    pub x: f32,
    pub y: f32,
    pub smooth: bool,
}

fn def_path_scale() -> f32 { 100.0 }

// ── Mask Data ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MaskType {
    #[default]
    None,
    Straight,
    Rectangle,
    Circle,
}

fn def_mask_size() -> [f32; 2] {[200.0, 200.0] }

// ── Word Animation Data (Karaoke) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum WordAnimation {
    #[default]
    None,
    KaraokeHighlight { color: [f32; 4] },
    KaraokePop { scale: f32 },
    CascadeFade,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleWord {
    pub text: String,
    pub start: f64,
    pub end: f64,
    #[serde(default)] pub custom_color: Option<[f32; 4]>,
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

// ── Physics & Expressions ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicsSettings {
    pub enabled: bool,
    pub gravity: f32,
    pub bounce: f32,
    pub floor_y: f32,
    pub initial_velocity_x: f32,
    pub initial_velocity_y: f32,
}
impl Default for PhysicsSettings {
    fn default() -> Self {
        Self { enabled: false, gravity: 2000.0, bounce: 0.6, floor_y: 400.0, initial_velocity_x: 0.0, initial_velocity_y: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Expressions {
    pub x: String,
    pub y: String,
    pub scale: String,
    pub rotation: String,
}

pub fn eval_expr(expr: &str, t: f64) -> f32 {
    let s = expr.replace(" ", "").to_lowercase();
    if s.is_empty() { return 0.0; }
    
    if s.starts_with("wiggle(") {
        if let Some(inner) = s.strip_prefix("wiggle(").and_then(|s| s.strip_suffix(")")) {
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 2 {
                let freq = parts[0].parse::<f32>().unwrap_or(0.0);
                let amp = parts[1].parse::<f32>().unwrap_or(0.0);
                let p1 = (t as f32 * freq * 6.28318).sin();
                let p2 = (t as f32 * freq * 4.123 + 2.0).cos();
                let p3 = (t as f32 * freq * 2.5 + 4.0).sin();
                return (p1 + p2 + p3) / 3.0 * amp;
            }
        }
    } else if s.starts_with("time*") {
        let mult = s.trim_start_matches("time*").parse::<f32>().unwrap_or(1.0);
        return t as f32 * mult;
    }
    0.0
}

// ── Subtitle ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subtitle {
    pub id: String,
    pub media_id: Option<String>,
    pub text: String,
    pub timeline_start: f64,
    pub timeline_end: f64,
    
    // Parenting & Grouping
    #[serde(default)] pub parent_id: Option<String>,
    #[serde(default)] pub group_id: Option<String>,
    
    // Transform
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub opacity: f32,
    #[serde(default)] pub skew_x: f32,
    #[serde(default)] pub skew_y: f32,
    #[serde(default)] pub yaw: f32,
    #[serde(default)] pub pitch: f32,
    
    // Motion Path
    #[serde(default)] pub path_type: PathType,
    #[serde(default = "def_path_scale")] pub path_scale_x: f32,
    #[serde(default = "def_path_scale")] pub path_scale_y: f32,
    #[serde(default)] pub path_orient: bool,
    #[serde(default)] pub path_align_words: bool,
    #[serde(default)] pub custom_path: Vec<PathNode>,
    #[serde(default)] pub path_progress: f32,

    // Masks
    #[serde(default)] pub mask_type: MaskType,
    #[serde(default)] pub mask_invert: bool,
    #[serde(default)] pub mask_center:[f32; 2],
    #[serde(default = "def_mask_size")] pub mask_size:[f32; 2],
    #[serde(default)] pub mask_rotation: f32,
    #[serde(default)] pub mask_feather: f32,

    // Word Animation
    #[serde(default)] pub words: Vec<SubtitleWord>,
    #[serde(default)] pub word_animation: WordAnimation,

    // Text style
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub color: [f32; 4],
    
    // Fill Mode / Clipping Masks
    #[serde(default)] pub text_fill_mode: TextFillMode,
    #[serde(default)] pub text_fill_path: Option<String>,

    // Stroke / outline
    #[serde(default)] pub stroke_enabled: bool,
    #[serde(default = "default_stroke_width")] pub stroke_width: f32,
    #[serde(default = "default_stroke_color")] pub stroke_color: [f32; 4],
    
    // Multiple Strokes
    #[serde(default)] pub additional_strokes_enabled: bool,
    #[serde(default)] pub additional_strokes: Vec<StrokeProps>,
    
    // Gradient
    #[serde(default)] pub gradient_enabled: bool,
    #[serde(default = "default_gradient_color")] pub gradient_color: [f32; 4],
    
    // Shadow & Bloom
    #[serde(default)] pub shadow_enabled: bool,
    #[serde(default = "default_shadow_offset")] pub shadow_offset:[f32; 2],
    #[serde(default = "default_shadow_blur")] pub shadow_blur: f32,
    #[serde(default = "default_shadow_color")] pub shadow_color:[f32; 4],
    #[serde(default)] pub bloom: BloomSettings,
    
    // Background box
    #[serde(default)] pub bg_box_enabled: bool,
    #[serde(default = "default_bg_box_color")] pub bg_box_color: [f32; 4],
    #[serde(default)] pub bg_box_padding: f32,
    #[serde(default = "default_bg_corner_radius")] pub bg_box_radius: [f32; 4],
    
    // Pro Effects
    #[serde(default)] pub text_deform: TextDeform,
    #[serde(default)] pub text_deform_amount: f32,
    #[serde(default)] pub blend_mode: BlendMode,
    #[serde(default)] pub track_matte: TrackMatte,
    #[serde(default)] pub glitch: GlitchSettings,
    
    // Advanced Mechanics
    #[serde(default)] pub loop_mode: LoopMode,
    #[serde(default)] pub physics: PhysicsSettings,
    #[serde(default)] pub expressions: Expressions,
    
    // Letter spacing / line height
    #[serde(default)] pub letter_spacing: f32,
    // Alignment
    #[serde(default)] pub text_align: TextAlign,
    // Effects
    #[serde(default)] pub motion_blur: f32,
    // Keyframes
    pub keyframes: Vec<Keyframe>,
}

fn default_stroke_width() -> f32 { 2.0 }
fn default_stroke_color() ->[f32; 4] {[0.0, 0.0, 0.0, 1.0] }
fn default_gradient_color() -> [f32; 4] {[1.0, 0.8, 0.0, 1.0] }
fn default_shadow_offset() ->[f32; 2] { [3.0, 3.0] }
fn default_shadow_blur() -> f32 { 6.0 }
fn default_shadow_color() -> [f32; 4] {[0.0, 0.0, 0.0, 0.7] }
fn default_bg_box_color() ->[f32; 4] {[0.0, 0.0, 0.0, 0.6] }

impl Subtitle {
    pub fn new(id: &str, text: &str, start: f64, end: f64) -> Self {
        Self {
            id: id.to_string(),
            media_id: None,
            parent_id: None,
            group_id: None,
            text: text.to_string(),
            timeline_start: start,
            timeline_end: end,
            x: 0.0, y: 0.0, scale: 1.0, rotation: 0.0, opacity: 1.0,
            skew_x: 0.0, skew_y: 0.0, yaw: 0.0, pitch: 0.0,
            path_type: PathType::None,
            path_scale_x: 100.0, path_scale_y: 100.0,
            path_orient: false,
            path_align_words: false,
            custom_path: Vec::new(),
            path_progress: 0.0,
            mask_type: MaskType::None,
            mask_invert: false,
            mask_center:[0.0, 0.0],
            mask_size: [200.0, 200.0],
            mask_rotation: 0.0,
            mask_feather: 0.0,
            words: Vec::new(),
            word_animation: WordAnimation::None,
            font_size: 36.0,
            bold: false, italic: false,
            color:[1.0, 1.0, 1.0, 1.0],
            text_fill_mode: TextFillMode::Solid,
            text_fill_path: None,
            stroke_enabled: false, stroke_width: 2.0, stroke_color:[0.0, 0.0, 0.0, 1.0],
            additional_strokes_enabled: false, additional_strokes: Vec::new(),
            gradient_enabled: false, gradient_color:[1.0, 0.8, 0.0, 1.0],
            shadow_enabled: true, shadow_offset: [3.0, 3.0], shadow_blur: 6.0, shadow_color:[0.0, 0.0, 0.0, 0.7],
            bloom: BloomSettings::default(),
            bg_box_enabled: false, bg_box_color:[0.0, 0.0, 0.0, 0.6], bg_box_padding: 8.0,
            bg_box_radius: [0.0, 0.0, 0.0, 0.0],
            text_deform: TextDeform::None, text_deform_amount: 0.0,
            blend_mode: BlendMode::Normal, track_matte: TrackMatte::None, glitch: GlitchSettings::default(),
            loop_mode: LoopMode::None, physics: PhysicsSettings::default(), expressions: Expressions::default(),
            letter_spacing: 0.0, text_align: TextAlign::Center, motion_blur: 0.0,
            keyframes: Vec::new(),
        }
    }

    pub fn duration(&self) -> f64 { self.timeline_end - self.timeline_start }

    pub fn apply_state_auto_key(&mut self, local_t: f64, record_mode: bool, state: &InterpolatedState) {
        if !self.keyframes.is_empty() || record_mode {
            if let Some(kf) = self.keyframes.iter_mut().find(|k| (k.time_offset - local_t).abs() < 0.02) {
                kf.x = state.x; kf.y = state.y; kf.scale = state.scale; kf.rotation = state.rotation;
                kf.opacity = state.opacity; kf.skew_x = state.skew_x; kf.skew_y = state.skew_y;
                kf.yaw = state.yaw; kf.pitch = state.pitch; kf.path_progress = state.path_progress;
            } else {
                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_t,
                    x: state.x, y: state.y, scale: state.scale, rotation: state.rotation,
                    opacity: state.opacity, skew_x: state.skew_x, skew_y: state.skew_y,
                    yaw: state.yaw, pitch: state.pitch, path_progress: state.path_progress,
                    mask_center: state.mask_center, mask_size: state.mask_size,
                    mask_rotation: state.mask_rotation, mask_feather: state.mask_feather,
                    easing: Easing::Linear,
                };
                self.keyframes.push(kf);
                self.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        } else {
            self.x = state.x; self.y = state.y; self.scale = state.scale; self.rotation = state.rotation;
            self.opacity = state.opacity; self.skew_x = state.skew_x; self.skew_y = state.skew_y;
            self.yaw = state.yaw; self.pitch = state.pitch; self.path_progress = state.path_progress;
        }
    }

    pub fn evaluate_path(&self, p: f32) -> (f32, f32, f32) {
        let get_xy = |p: f32| -> (f32, f32) {
            match self.path_type {
                PathType::None => (0.0, 0.0),
                PathType::Circle => {
                    let a = p * std::f32::consts::TAU;
                    (a.cos() * self.path_scale_x, a.sin() * self.path_scale_y)
                }
                PathType::Star => {
                    let n = 10;
                    let t = p * n as f32;
                    let idx = (t.floor() as usize).min(n - 1);
                    let frac = t - idx as f32;
                    let get_pt = |i: usize| {
                        let a = (i as f32 * std::f32::consts::TAU / n as f32) - std::f32::consts::FRAC_PI_2;
                        let r = if i % 2 == 0 { self.path_scale_x } else { self.path_scale_y };
                        (a.cos() * r, a.sin() * r)
                    };
                    let p1 = get_pt(idx);
                    let p2 = get_pt(idx + 1);
                    (p1.0 + (p2.0 - p1.0) * frac, p1.1 + (p2.1 - p1.1) * frac)
                }
                PathType::Custom => {
                    let pts = &self.custom_path;
                    if pts.is_empty() { return (0.0, 0.0); }
                    if pts.len() == 1 { return (pts[0].x * self.path_scale_x, pts[0].y * self.path_scale_y); }
                    
                    let n = pts.len() - 1;
                    let t = p * n as f32;
                    let idx = (t.floor() as usize).min(n - 1);
                    let frac = t - idx as f32;
                    
                    let p1 = &pts[idx];
                    let p2 = &pts[idx + 1];
                    
                    if p1.smooth {
                        let p0 = if idx > 0 { &pts[idx - 1] } else { p1 };
                        let p3 = if idx + 2 < pts.len() { &pts[idx + 2] } else { p2 };
                        
                        let t2 = frac * frac;
                        let t3 = t2 * frac;
                        let f0 = -0.5 * t3 + t2 - 0.5 * frac;
                        let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
                        let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * frac;
                        let f3 = 0.5 * t3 - 0.5 * t2;
                        
                        (
                            (p0.x * f0 + p1.x * f1 + p2.x * f2 + p3.x * f3) * self.path_scale_x,
                            (p0.y * f0 + p1.y * f1 + p2.y * f2 + p3.y * f3) * self.path_scale_y
                        )
                    } else {
                        (
                            (p1.x + (p2.x - p1.x) * frac) * self.path_scale_x,
                            (p1.y + (p2.y - p1.y) * frac) * self.path_scale_y
                        )
                    }
                }
            }
        };

        let (x, y) = get_xy(p);
        let dp = 0.01;
        let (dx, dy) = if p < 0.99 { get_xy(p + dp) } else { (x, y) };
        let (bx, by) = if p > 0.01 { get_xy(p - dp) } else { (x, y) };
        
        let angle = if p < 0.99 && p > 0.01 {
            (dy - by).atan2(dx - bx)
        } else if p < 0.99 {
            (dy - y).atan2(dx - x)
        } else {
            (y - by).atan2(x - bx)
        };

        (x, y, angle)
    }

    pub fn get_interpolated_state(&self, current_time: f64, all_subs: &[Subtitle], depth: usize) -> InterpolatedState {
        let mut base = InterpolatedState {
            x: self.x, y: self.y, scale: self.scale,
            rotation: self.rotation, opacity: self.opacity,
            skew_x: self.skew_x, skew_y: self.skew_y, yaw: self.yaw, pitch: self.pitch,
            path_progress: self.path_progress,
            mask_center: self.mask_center, mask_size: self.mask_size,
            mask_rotation: self.mask_rotation, mask_feather: self.mask_feather,
        };

        let local_time = current_time - self.timeline_start;
        
        base = crate::models::types::animation::interpolate_keyframes(
            current_time, self.timeline_start, &self.keyframes, &self.loop_mode, base
        );

        if self.keyframes.is_empty() {
            let anim_dur = 0.08;
            if local_time >= 0.0 && local_time < anim_dur {
                let p = ease_out_cubic(local_time / anim_dur) as f32;
                base.scale  *= 0.4 + p * 0.6;
                base.opacity = p;
            } else if local_time < 0.0 {
                base.opacity = 0.0;
            }
        }

        base.x += eval_expr(&self.expressions.x, local_time);
        base.y += eval_expr(&self.expressions.y, local_time);
        base.scale += eval_expr(&self.expressions.scale, local_time);
        base.rotation += eval_expr(&self.expressions.rotation, local_time);

        if self.physics.enabled && local_time > 0.0 {
            let dt: f64 = 1.0 / 60.0;
            let mut sim_t: f64 = 0.0;
            
            let mut px = 0.0_f32;
            let mut py = 0.0_f32;
            let mut vx = self.physics.initial_velocity_x;
            let mut vy = self.physics.initial_velocity_y;
            let g = self.physics.gravity;
            let bounce = self.physics.bounce;
            let floor = self.physics.floor_y;

            let dt_f32 = dt as f32;

            while sim_t < local_time {
                vy += g * dt_f32;
                px += vx * dt_f32;
                py += vy * dt_f32;
                if py > floor {
                    py = floor;
                    vy = -vy * bounce;
                    vx *= 0.95; 
                }
                sim_t += dt;
            }
            base.x += px;
            base.y += py;
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