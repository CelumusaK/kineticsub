use serde::{Deserialize, Serialize};
use super::subtitle::Subtitle;

// ── MediaFile ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub timeline_offset: f64,
    pub duration: f64,
    pub on_timeline: bool,
    
    // Video/Background properties
    #[serde(default)]
    pub is_video_track: bool,
    #[serde(default)]
    pub color: Option<[f32; 4]>,
}

// ── RenderMode ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RenderMode {
    #[default]
    ImageSequence,
    Video,
}

// ── Project ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub media_files: Vec<MediaFile>,
    pub subtitles: Vec<Subtitle>,
    pub duration: f64,
    #[serde(default = "default_resolution")]
    pub resolution: (u32, u32),
    #[serde(default = "default_fps")]
    pub fps: u32,
}

fn default_resolution() -> (u32, u32) { (1920, 1080) }
fn default_fps() -> u32 { 30 }

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled".into(),
            media_files: vec![],
            subtitles: vec![],
            duration: 10.0,
            resolution: (1920, 1080),
            fps: 30,
        }
    }
}