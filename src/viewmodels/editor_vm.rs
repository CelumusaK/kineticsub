use std::collections::HashSet;
use std::sync::mpsc;
use crate::models::{
    sync_engine::SyncEngine,
    types::{MediaFile, Project, Subtitle},
};
use crate::services::whisper_service::{spawn_transcription, WhisperMessage};
use crate::services::audio_player::AudioPlayer;

// ── Keyframe record mode ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeMode {
    Off,
    Record,
}

// ── ViewModel ─────────────────────────────────────────────────────────────────

pub struct EditorViewModel {
    pub project: Project,
    pub filepath: Option<std::path::PathBuf>,
    pub sync: SyncEngine,

    /// Primary selected subtitle id
    pub selected_id: Option<String>,
    /// Multi-selected subtitle ids
    pub selected_ids: HashSet<String>,

    pub timeline_zoom: f32,
    pub timeline_scroll: f64,
    pub new_sub_text: String,

    whisper_rx: Option<mpsc::Receiver<WhisperMessage>>,
    pub whisper_status: String,
    pub transcribing_media_id: Option<String>,

    next_id: u32,

    pub keyframe_mode: KeyframeMode,

    pub box_select_start: Option<egui::Pos2>,
    pub box_select_end:   Option<egui::Pos2>,

    pub audio_player: AudioPlayer,
}

impl EditorViewModel {
    pub fn new() -> Self {
        let duration = 10.0;
        Self {
            project: Project { name: "Untitled".into(), media_files: vec![], duration, subtitles: vec![] },
            filepath: None,
            sync: SyncEngine::new(duration),
            selected_id: None,
            selected_ids: HashSet::new(),
            timeline_zoom: 100.0,
            timeline_scroll: 0.0,
            new_sub_text: String::new(),
            whisper_rx: None,
            whisper_status: String::new(),
            transcribing_media_id: None,
            next_id: 0,
            keyframe_mode: KeyframeMode::Off,
            box_select_start: None,
            box_select_end: None,
            audio_player: AudioPlayer::new(),
        }
    }

    // ── Duration ──────────────────────────────────────────────────────────────
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

    // ── Media ─────────────────────────────────────────────────────────────────
    pub fn import_audio(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio", &["wav", "mp3", "m4a", "ogg"])
            .pick_file()
        {
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let id   = format!("media_{}", uuid::Uuid::new_v4());
            self.project.media_files.push(MediaFile {
                id,
                path: path.to_string_lossy().to_string(),
                name,
                timeline_offset: 0.0,
                duration: 60.0,
                on_timeline: false,
            });
            self.update_duration();
        }
    }

    pub fn toggle_media_timeline(&mut self, id: &str) {
        if let Some(media) = self.project.media_files.iter_mut().find(|m| m.id == id) {
            media.on_timeline = !media.on_timeline;
        }
        self.update_duration();
    }

    /// Move a media block by `delta_secs` and shift all subtitles linked to it
    /// by the same delta so they stay in sync.
    pub fn move_media(&mut self, index: usize, delta_secs: f64) {
        let (media_id, old_offset) = match self.project.media_files.get(index) {
            Some(m) => (m.id.clone(), m.timeline_offset),
            None    => return,
        };

        let new_offset = (old_offset + delta_secs).max(0.0);
        let actual_delta = new_offset - old_offset; // may differ if clamped

        if let Some(m) = self.project.media_files.get_mut(index) {
            m.timeline_offset = new_offset;
        }

        // Shift every subtitle that belongs to this media file
        for sub in self.project.subtitles.iter_mut() {
            if sub.media_id.as_deref() == Some(&media_id) {
                sub.timeline_start = (sub.timeline_start + actual_delta).max(0.0);
                sub.timeline_end   = (sub.timeline_end   + actual_delta).max(sub.timeline_start + 0.05);
            }
        }

        self.update_duration();
    }

    pub fn move_subtitle_idx(&mut self, index: usize, delta_secs: f64) {
        if let Some(sub) = self.project.subtitles.get_mut(index) {
            sub.timeline_start = (sub.timeline_start + delta_secs).max(0.0);
            sub.timeline_end   = (sub.timeline_end   + delta_secs).max(sub.timeline_start + 0.05);
        }
        self.update_duration();
    }

    /// Move all selected subtitles together
    pub fn move_selected_subtitles(&mut self, delta_secs: f64) {
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                sub.timeline_start = (sub.timeline_start + delta_secs).max(0.0);
                sub.timeline_end   = (sub.timeline_end   + delta_secs).max(sub.timeline_start + 0.05);
            }
        }
        self.update_duration();
    }

    // ── Save / Load ───────────────────────────────────────────────────────────
    pub fn save_project(&mut self) {
        if let Some(path) = &self.filepath {
            let json = serde_json::to_string_pretty(&self.project).unwrap();
            std::fs::write(path, json).unwrap();
            self.project.name = path.file_stem().unwrap().to_string_lossy().to_string();
        } else {
            self.save_project_as();
        }
    }

    pub fn save_project_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KineticSub Project", &["ksub"])
            .save_file()
        {
            self.filepath = Some(path.clone());
            self.save_project();
        }
    }

    pub fn load_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KineticSub Project", &["ksub"])
            .pick_file()
        {
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(proj) = serde_json::from_str::<Project>(&json) {
                    self.project = proj;
                    self.filepath = Some(path);
                    self.selected_id = None;
                    self.selected_ids.clear();
                    self.update_duration();
                    self.sync.stop();
                }
            }
        }
    }

    // ── Whisper ───────────────────────────────────────────────────────────────

    /// Start transcribing a media file. The file MUST already be on the
    /// timeline (`on_timeline == true`). Returns false if the guard fails.
    pub fn start_auto_transcription(&mut self, media_id: String) -> bool {
        // Guard: must be on timeline
        let audio_path = match self.project.media_files.iter().find(|m| m.id == media_id) {
            Some(m) if m.on_timeline => m.path.clone(),
            _ => return false,
        };

        // Remove any existing subtitles linked to this media so a re-transcribe
        // produces a clean result
        self.project.subtitles.retain(|s| s.media_id.as_deref() != Some(&media_id));

        let (tx, rx) = mpsc::channel();
        self.whisper_rx = Some(rx);
        self.transcribing_media_id = Some(media_id);
        self.whisper_status = "Initializing Whisper...".into();
        spawn_transcription(audio_path, tx);
        true
    }

    pub fn poll_whisper(&mut self) {
        if let Some(rx) = &self.whisper_rx {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    WhisperMessage::DownloadProgress(cur, total) => {
                        let pct = if total > 0 {
                            (cur as f32 / total as f32 * 100.0) as u32
                        } else { 0 };
                        self.whisper_status = format!("Downloading Model... {}%", pct);
                    }
                    WhisperMessage::Transcribing => {
                        self.whisper_status = "Transcribing Audio...".into();
                    }
                    WhisperMessage::Done(words, duration) => {
                        let media_id = self.transcribing_media_id.clone().unwrap();

                        // Update exact media duration and capture offset
                        let offset = {
                            match self.project.media_files.iter_mut().find(|m| m.id == media_id) {
                                Some(m) => { m.duration = duration; m.timeline_offset }
                                None    => 0.0,
                            }
                        };

                        for w in words {
                            // Place subtitle relative to where the audio starts on
                            // the timeline — works correctly even if offset > 0
                            let abs_start = offset + w.start;
                            let abs_end   = offset + w.end;
                            let mut sub = Subtitle::new(
                                &self.next_id_str(),
                                &w.text,
                                abs_start,
                                abs_end,
                            );
                            sub.media_id = Some(media_id.clone());
                            self.project.subtitles.push(sub);
                        }

                        self.sort_subtitles();
                        self.update_duration();
                        self.whisper_status = "Done!".into();
                        self.whisper_rx = None;
                        self.transcribing_media_id = None;
                    }
                    WhisperMessage::Error(err) => {
                        self.whisper_status = format!("Error: {}", err);
                        self.whisper_rx = None;
                        self.transcribing_media_id = None;
                    }
                }
            }
        }
    }

    pub fn whisper_is_running(&self) -> bool { self.whisper_rx.is_some() }

    // ── Playback ──────────────────────────────────────────────────────────────
    pub fn tick(&mut self) {
        self.sync.tick();
        let t = self.sync.current_time;

        if self.sync.is_playing() {
            if let Some(m) = self.project.media_files.iter().find(|m| m.on_timeline) {
                if t >= m.timeline_offset && t < m.timeline_offset + m.duration {
                    if !self.audio_player.is_playing() {
                        self.audio_player.load(m.path.clone());
                        self.audio_player.play_from(t - m.timeline_offset);
                    }
                } else {
                    if self.audio_player.is_playing() {
                        self.audio_player.pause();
                    }
                }
            }
        } else {
            if self.audio_player.is_playing() {
                self.audio_player.pause();
            }
        }
    }

    pub fn toggle_play(&mut self) {
        self.sync.toggle_play_pause();
        if !self.sync.is_playing() {
            self.audio_player.pause();
        } else {
            self.audio_player.pause(); // Force resync on tick
        }
    }

    pub fn skip(&mut self, delta: f64) {
        self.sync.skip(delta);
        self.audio_player.pause();
    }

    pub fn seek_to(&mut self, t: f64) {
        self.sync.seek(t);
        self.audio_player.pause();
    }
    
    pub fn current_time(&self) -> f64  { self.sync.current_time }
    pub fn is_playing(&self) -> bool   { self.sync.is_playing() }

    // ── Subtitle CRUD ─────────────────────────────────────────────────────────
    fn next_id_str(&mut self) -> String {
        let s = format!("sub_{}", self.next_id);
        self.next_id += 1;
        s
    }

    pub fn add_subtitle_at(&mut self, text: &str, start: f64, end: f64) {
        let id  = self.next_id_str();
        let sub = Subtitle::new(&id, text, start, end);
        self.project.subtitles.push(sub);
        self.sort_subtitles();
        self.update_duration();
    }

    pub fn insert_subtitle_at_playhead(&mut self) {
        let text = self.new_sub_text.trim().to_string();
        if text.is_empty() { return; }
        let start = self.sync.current_time;
        let end   = (start + 3.0).min(self.project.duration);
        self.add_subtitle_at(&text, start, end);
        self.new_sub_text.clear();
    }

    pub fn delete_subtitle(&mut self, id: &str) {
        self.project.subtitles.retain(|s| s.id != id);
        if self.selected_id.as_deref() == Some(id) { self.selected_id = None; }
        self.selected_ids.remove(id);
        self.update_duration();
    }

    pub fn delete_selected_subtitles(&mut self) {
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        for id in &ids { self.project.subtitles.retain(|s| &s.id != id); }
        if let Some(sel) = &self.selected_id.clone() {
            if ids.contains(sel) { self.selected_id = None; }
        }
        self.selected_ids.clear();
        self.update_duration();
    }

    pub fn select_subtitle(&mut self, id: Option<String>) {
        self.selected_id = id.clone();
        self.selected_ids.clear();
        if let Some(id) = id { self.selected_ids.insert(id); }
    }

    pub fn toggle_select(&mut self, id: &str) {
        if self.selected_ids.contains(id) {
            self.selected_ids.remove(id);
            if self.selected_id.as_deref() == Some(id) {
                self.selected_id = self.selected_ids.iter().next().cloned();
            }
        } else {
            self.selected_ids.insert(id.to_string());
            if self.selected_id.is_none() { self.selected_id = Some(id.to_string()); }
        }
    }

    pub fn selected_subtitle(&self) -> Option<&Subtitle> {
        self.selected_id.as_ref()
            .and_then(|id| self.project.subtitles.iter().find(|s| &s.id == id))
    }

    pub fn selected_subtitle_mut(&mut self) -> Option<&mut Subtitle> {
        self.selected_id.clone()
            .and_then(|id| self.project.subtitles.iter_mut().find(|s| s.id == id))
    }

    pub fn active_subtitle(&self) -> Option<&Subtitle> {
        let t = self.sync.current_time;
        if let Some(sel) = self.selected_subtitle() {
            if t >= sel.timeline_start && t < sel.timeline_end { return Some(sel); }
        }
        self.project.subtitles.iter()
            .find(|s| t >= s.timeline_start && t < s.timeline_end)
    }

    pub fn sort_subtitles(&mut self) {
        self.project.subtitles
            .sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
    }

    // ── Keyframe helpers ──────────────────────────────────────────────────────

    pub fn write_keyframe_now(&mut self) {
        let local_time = {
            let sub = match self.selected_subtitle() { Some(s) => s, None => return };
            let lt = self.sync.current_time - sub.timeline_start;
            if lt < 0.0 || lt > sub.duration() { return; }
            lt
        };

        let sub = match self.selected_subtitle_mut() { Some(s) => s, None => return };
        sub.keyframes.retain(|k| (k.time_offset - local_time).abs() >= 0.02);

        let kf = crate::models::types::Keyframe {
            id: format!("kf_{}", uuid::Uuid::new_v4()),
            time_offset: local_time,
            x: sub.x, y: sub.y, scale: sub.scale,
            rotation: sub.rotation, opacity: sub.opacity,
            skew_x: sub.skew_x,
            skew_y: sub.skew_y,
            easing: crate::models::types::Easing::EaseOut,
        };
        sub.keyframes.push(kf);
        sub.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
    }

    pub fn maybe_autorecord_keyframe(&mut self) {
        if self.keyframe_mode == KeyframeMode::Record {
            self.write_keyframe_now();
        }
    }

    // ── Coordinate mapping ────────────────────────────────────────────────────
    pub fn px_to_time(&self, px: f32) -> f64 {
        self.timeline_scroll + (px / self.timeline_zoom) as f64
    }
    pub fn time_to_px(&self, t: f64) -> f32 {
        ((t - self.timeline_scroll) * self.timeline_zoom as f64) as f32
    }
}

impl Default for EditorViewModel { fn default() -> Self { Self::new() } }

use egui;