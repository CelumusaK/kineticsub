pub mod media;
pub mod playback;
pub mod project;
pub mod subtitle;

use std::collections::HashSet;
use std::sync::mpsc;
use crate::models::{
    sync_engine::SyncEngine,
    types::{Project, RenderMode},
};
use crate::services::whisper_service::WhisperMessage;
use crate::services::audio_player::AudioPlayer;
use crate::services::render_service::RenderMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeMode {
    Off,
    Record,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TranscribeMode {
    #[default]
    Phrase,
    Word,
}

pub struct EditorViewModel {
    pub project: Project,
    pub filepath: Option<std::path::PathBuf>,
    pub sync: SyncEngine,

    // ── History & Undo ─────────────────────────────────────────────
    pub history: Vec<Project>,
    pub history_index: usize,
    pub pending_snapshot: bool,

    pub selected_id: Option<String>,
    pub selected_ids: HashSet<String>,
    pub selected_path_node: Option<usize>,

    pub timeline_zoom: f32,
    pub timeline_scroll: f64,
    pub new_sub_text: String,

    pub whisper_rx: Option<mpsc::Receiver<WhisperMessage>>,
    pub whisper_status: String,
    pub transcribing_media_id: Option<String>,
    pub transcribe_mode: TranscribeMode,

    pub next_id: u32,
    pub keyframe_mode: KeyframeMode,

    pub box_select_start: Option<egui::Pos2>,
    pub box_select_end:   Option<egui::Pos2>,
    pub audio_player: AudioPlayer,

    pub show_fps: bool,
    pub current_fps: f64,
    pub render_mode: RenderMode,
    pub render_include_audio: bool,
    pub render_transparent_bg: bool,
    
    // Rendering State
    pub is_rendering: bool,
    pub render_progress: f32,
    pub render_status: String,
    pub render_rx: Option<mpsc::Receiver<RenderMessage>>,
}

impl EditorViewModel {
    pub fn new() -> Self {
        let duration = 10.0;
        let mut vm = Self {
            project: Project { 
                name: "Untitled".into(), 
                media_files: vec![], 
                duration, 
                subtitles: vec![],
                resolution: (1920, 1080),
                fps: 30,
            },
            filepath: None,
            sync: SyncEngine::new(duration),
            
            history: Vec::new(),
            history_index: 0,
            pending_snapshot: false,

            selected_id: None,
            selected_ids: HashSet::new(),
            selected_path_node: None,
            timeline_zoom: 100.0,
            timeline_scroll: 0.0,
            new_sub_text: String::new(),
            
            whisper_rx: None,
            whisper_status: String::new(),
            transcribing_media_id: None,
            transcribe_mode: TranscribeMode::default(),
            
            next_id: 0,
            keyframe_mode: KeyframeMode::Off,
            box_select_start: None,
            box_select_end: None,
            audio_player: AudioPlayer::new(),
            show_fps: false,
            current_fps: 0.0,
            render_mode: RenderMode::default(),
            render_include_audio: true,
            render_transparent_bg: true,
            is_rendering: false,
            render_progress: 0.0,
            render_status: String::new(),
            render_rx: None,
        };

        // Initialize history with empty project
        vm.history.push(vm.project.clone());
        vm
    }

    pub fn update_duration(&mut self) {
        let mut max_end = 5.0;
        for m in &self.project.media_files {
            if m.on_timeline { let e = m.timeline_offset + m.duration; if e > max_end { max_end = e; } }
        }
        for s in &self.project.subtitles {
            if s.timeline_end > max_end { max_end = s.timeline_end; }
        }
        self.project.duration = max_end;
        self.sync.duration = max_end;
        if self.sync.current_time > max_end { self.sync.seek(max_end); }
    }

    pub fn px_to_time(&self, px: f32) -> f64 {
        self.timeline_scroll + (px / self.timeline_zoom) as f64
    }
    pub fn time_to_px(&self, t: f64) -> f32 {
        ((t - self.timeline_scroll) * self.timeline_zoom as f64) as f32
    }
}

impl Default for EditorViewModel { fn default() -> Self { Self::new() } }