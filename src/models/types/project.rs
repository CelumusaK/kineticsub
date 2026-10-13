use serde::{Deserialize, Serialize};
use super::animation::{Keyframe, InterpolatedState, LoopMode, Easing};
use super::subtitle::Subtitle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    Subtitle,
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MediaType {
    #[default]
    Audio,
    Video,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaFolder {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub is_open: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TransitionType {
    #[default]
    Crossfade,
    DipToBlack,
    DipToWhite,
    WipeLeft,
    WipeRight,
    SlideLeft,
}

impl TransitionType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Crossfade => "Crossfade",
            Self::DipToBlack => "Dip to Black",
            Self::DipToWhite => "Dip to White",
            Self::WipeLeft => "Wipe Left",
            Self::WipeRight => "Wipe Right",
            Self::SlideLeft => "Slide Left",
        }
    }
    pub fn all() -> &'static [TransitionType] {
        &[
            Self::Crossfade,
            Self::DipToBlack,
            Self::DipToWhite,
            Self::WipeLeft,
            Self::WipeRight,
            Self::SlideLeft,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionBlock {
    pub id: String,
    pub name: String,
    pub timeline_start: f64,
    pub duration: f64,
    #[serde(default)]
    pub track_index: usize,
    #[serde(default)]
    pub transition_type: TransitionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub timeline_offset: f64,
    pub duration: f64,
    pub on_timeline: bool,
    
    #[serde(default)]
    pub media_type: MediaType,
    #[serde(default)]
    pub track_index: usize,
    
    #[serde(default)]
    pub color: Option<[f32; 4]>,

    #[serde(default)] pub source_fps: Option<f64>,
    #[serde(default)] pub source_duration: Option<f64>,
    #[serde(default)] pub source_width: Option<u32>,
    #[serde(default)] pub source_height: Option<u32>,
    #[serde(default)] pub preview_dir: Option<String>,
    #[serde(default)] pub preview_ready: bool,

    #[serde(default = "default_zero")] pub x: f32,
    #[serde(default = "default_zero")] pub y: f32,
    #[serde(default = "default_one")] pub scale: f32,
    #[serde(default = "default_zero")] pub rotation: f32,
    #[serde(default = "default_one")] pub opacity: f32,
    #[serde(default = "default_one")] pub volume: f32,
    
    #[serde(default)] pub trim_start: f64,
    #[serde(default = "default_one_f64")] pub playback_speed: f64,

    #[serde(default)] pub keyframes: Vec<Keyframe>,
    #[serde(default)] pub loop_mode: LoopMode,

    #[serde(default)] pub group_id: Option<String>,
    #[serde(default)] pub folder_id: Option<String>,
    
    #[serde(skip)] pub has_collision: bool,
}

fn default_zero() -> f32 { 0.0 }
fn default_one() -> f32 { 1.0 }
fn default_one_f64() -> f64 { 1.0 }

impl MediaFile {
    pub fn get_interpolated_state(&self, current_time: f64) -> InterpolatedState {
        let base = InterpolatedState {
            x: self.x, y: self.y, scale: self.scale,
            rotation: self.rotation, opacity: self.opacity,
            skew_x: 0.0, skew_y: 0.0, yaw: 0.0, pitch: 0.0, path_progress: 0.0,
            mask_center: [0.0, 0.0], mask_size: [0.0, 0.0], mask_rotation: 0.0, mask_feather: 0.0,
        };
        crate::models::types::animation::interpolate_keyframes(
            current_time, self.timeline_offset, &self.keyframes, &self.loop_mode, base
        )
    }

    pub fn apply_state_auto_key(&mut self, local_t: f64, record_mode: bool, state: &InterpolatedState) {
        if !self.keyframes.is_empty() || record_mode {
            if let Some(kf) = self.keyframes.iter_mut().find(|k| (k.time_offset - local_t).abs() < 0.02) {
                kf.x = state.x; kf.y = state.y; kf.scale = state.scale; kf.rotation = state.rotation;
                kf.opacity = state.opacity;
            } else {
                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_t,
                    x: state.x, y: state.y, scale: state.scale, rotation: state.rotation, opacity: state.opacity,
                    skew_x: 0.0, skew_y: 0.0, yaw: 0.0, pitch: 0.0, path_progress: 0.0,
                    mask_center: [0.0, 0.0], mask_size: [0.0, 0.0], mask_rotation: 0.0, mask_feather: 0.0,
                    easing: Easing::Linear,
                };
                self.keyframes.push(kf);
                self.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        } else {
            self.x = state.x; self.y = state.y; self.scale = state.scale; self.rotation = state.rotation; self.opacity = state.opacity;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RenderMode {
    #[default]
    ImageSequence,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub media_files: Vec<MediaFile>,
    #[serde(default)]
    pub folders: Vec<MediaFolder>,
    #[serde(default)]
    pub transitions: Vec<TransitionBlock>,
    pub subtitles: Vec<Subtitle>,
    pub duration: f64,
    
    #[serde(default = "default_resolution")]
    pub resolution: (u32, u32),
    #[serde(default = "default_fps")]
    pub fps: u32,

    #[serde(default = "default_tracks")]
    pub subtitle_track_order: Vec<usize>,
    #[serde(default = "default_tracks")]
    pub video_track_order: Vec<usize>,
    #[serde(default = "default_tracks")]
    pub audio_track_order: Vec<usize>,

    #[serde(default = "default_next_id")]
    pub next_subtitle_id: usize,
    #[serde(default = "default_next_id")]
    pub next_video_id: usize,
    #[serde(default = "default_next_id")]
    pub next_audio_id: usize,

    #[serde(default)]
    pub export_range: Option<(f64, f64)>,
}

fn default_resolution() -> (u32, u32) { (1920, 1080) }
fn default_fps() -> u32 { 25 }
fn default_tracks() -> Vec<usize> { vec![1] }
fn default_next_id() -> usize { 2 }

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled".into(),
            media_files: vec![],
            folders: vec![],
            transitions: vec![],
            subtitles: vec![],
            duration: 10.0,
            resolution: (1920, 1080),
            fps: 25,
            subtitle_track_order: vec![1],
            video_track_order: vec![1],
            audio_track_order: vec![1],
            next_subtitle_id: 2,
            next_video_id: 2,
            next_audio_id: 2,
            export_range: None,
        }
    }
}

impl Project {
    pub fn ensure_tracks_initialized(&mut self) {
        if self.subtitle_track_order.is_empty() {
            self.subtitle_track_order = vec![1];
            self.next_subtitle_id = 2;
        }
        if self.video_track_order.is_empty() {
            self.video_track_order = vec![1];
            self.next_video_id = 2;
        }
        if self.audio_track_order.is_empty() {
            self.audio_track_order = vec![1];
            self.next_audio_id = 2;
        }
    }
}
