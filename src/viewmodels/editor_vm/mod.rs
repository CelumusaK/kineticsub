pub mod media;
pub mod playback;
pub mod project;
pub mod subtitle;

use std::collections::{HashSet, HashMap};
use std::sync::mpsc;
use crate::models::{
    sync_engine::SyncEngine,
    types::{Project, RenderMode, MediaType, TransitionBlock, TransitionType},
};
use crate::services::whisper_service::{
    spawn_download_model, ModelDownloadMessage, WhisperMessage, WhisperModel,
};
use crate::services::audio_player::AudioPlayer;
use crate::services::render_service::RenderMessage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeMode { Off, Record }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TranscribeMode { #[default] Phrase, Word }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HoverPanel { #[default] None, Timeline, MediaBin }

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GraphChannel { #[default] X, Y, Scale, Rotation, Opacity }

pub enum PreviewMessage {
    MetadataReady { id: String, duration: f64, fps: f64, width: u32, height: u32 },
    ExtractionComplete { id: String },
    Error(String),
}

pub struct EditorViewModel {
    pub project: Project,
    pub filepath: Option<std::path::PathBuf>,
    pub sync: SyncEngine,

    pub history: Vec<Project>,
    pub history_index: usize,
    pub pending_snapshot: bool,

    pub hovered_panel: HoverPanel,
    pub selected_id: Option<String>,
    pub selected_ids: HashSet<String>,
    pub selected_path_node: Option<usize>,
    pub selected_track: usize,
    pub selected_keyframe_ids: HashSet<String>,

    pub timeline_zoom: f32,
    pub timeline_scroll: f64,
    pub timeline_vertical_scroll: f32,
    pub timeline_zoom_to_mouse: bool,
    pub is_scrubbing: bool,
    
    // Graph Editor State
    pub graph_channel: GraphChannel,
    pub graph_zoom_x: f32,
    pub graph_zoom_y: f32,
    pub graph_offset_y: f32,
    pub graph_scroll: f64,
    
    pub new_sub_text: String,

    pub whisper_rx: Option<mpsc::Receiver<WhisperMessage>>,
    pub whisper_status: String,
    pub transcribing_media_id: Option<String>,
    pub transcribe_mode: TranscribeMode,
    pub selected_whisper_model: WhisperModel,

    pub model_download_rx: Option<mpsc::Receiver<ModelDownloadMessage>>,
    pub downloading_model: Option<WhisperModel>,
    pub model_download_progress: f32,
    pub model_download_status: String,

    pub preview_rx: Option<mpsc::Receiver<PreviewMessage>>,
    pub show_settings_modal: bool,
    pub show_export_modal: bool,
    pub next_id: u32,
    pub keyframe_mode: KeyframeMode,
    pub box_select_start: Option<egui::Pos2>,
    pub box_select_end:   Option<egui::Pos2>,
    
    pub audio_player: AudioPlayer,

    pub previewing_media_id: Option<String>,
    pub preview_time: f64,
    pub preview_playing: bool,
    pub creating_color_bg: bool,
    pub new_bg_color: [f32; 4],

    pub show_fps: bool,
    pub current_fps: f64,
    pub render_mode: RenderMode,
    pub render_include_audio: bool,
    pub render_transparent_bg: bool,
    
    pub is_rendering: bool,
    pub render_progress: f32,
    pub render_status: String,
    pub render_rx: Option<mpsc::Receiver<RenderMessage>>,

    pub dragging_media_id: Option<String>,
    pub texture_cache: HashMap<String, egui::TextureHandle>,
    pub left_panel_split: f32,
}

impl EditorViewModel {
    pub fn new() -> Self {
        let duration = 10.0;
        let mut vm = Self {
            project: Project::default(),
            filepath: None,
            sync: SyncEngine::new(duration),
            history: Vec::new(),
            history_index: 0,
            pending_snapshot: false,
            hovered_panel: HoverPanel::default(),
            selected_id: None,
            selected_ids: HashSet::new(),
            selected_path_node: None,
            selected_track: 0,
            selected_keyframe_ids: HashSet::new(),
            timeline_zoom: 100.0,
            timeline_scroll: 0.0,
            timeline_vertical_scroll: 0.0,
            timeline_zoom_to_mouse: true,
            is_scrubbing: false,
            
            graph_channel: GraphChannel::X,
            graph_zoom_x: 200.0,
            graph_zoom_y: 0.1,
            graph_offset_y: 0.0,
            graph_scroll: 0.0,
            
            new_sub_text: String::new(),
            whisper_rx: None,
            whisper_status: String::new(),
            transcribing_media_id: None,
            transcribe_mode: TranscribeMode::default(),
            selected_whisper_model: WhisperModel::default(),
            model_download_rx: None,
            downloading_model: None,
            model_download_progress: 0.0,
            model_download_status: String::new(),
            show_settings_modal: false,
            show_export_modal: false,
            preview_rx: None,
            next_id: 0,
            keyframe_mode: KeyframeMode::Off,
            box_select_start: None,
            box_select_end: None,
            audio_player: AudioPlayer::new(),
            previewing_media_id: None,
            preview_time: 0.0,
            preview_playing: false,
            creating_color_bg: false,
            new_bg_color: [0.05, 0.05, 0.05, 1.0],
            show_fps: false,
            current_fps: 0.0,
            render_mode: RenderMode::default(),
            render_include_audio: true,
            render_transparent_bg: true,
            is_rendering: false,
            render_progress: 0.0,
            render_status: String::new(),
            render_rx: None,
            dragging_media_id: None,
            texture_cache: HashMap::new(),
            left_panel_split: 0.5,
        };
        vm.project.ensure_tracks_initialized();
        vm.history.push(vm.project.clone());
        vm
    }

    pub fn resolve_all_overlaps(&mut self) {
        let mut tracks_to_resolve = std::collections::HashSet::new();
        for m in &self.project.media_files {
            if m.on_timeline {
                tracks_to_resolve.insert((m.track_index, m.media_type == MediaType::Audio));
            }
        }
        
        let mut iterations = 0;
        while !tracks_to_resolve.is_empty() && iterations < 5 {
            let mut next_tracks = std::collections::HashSet::new();
            
            for (track_index, is_audio) in tracks_to_resolve {
                let mut indices: Vec<usize> = self.project.media_files.iter().enumerate()
                    .filter(|(_, m)| m.on_timeline && m.track_index == track_index && (m.media_type == MediaType::Audio) == is_audio)
                    .map(|(i, _)| i)
                    .collect();
                    
                indices.sort_by(|&a, &b| {
                    self.project.media_files[a].timeline_offset.partial_cmp(&self.project.media_files[b].timeline_offset).unwrap()
                });

                let mut cursor = 0.0;
                let mut group_pushes: Vec<(String, f64)> = Vec::new();

                for &idx in &indices {
                    let m = &mut self.project.media_files[idx];
                    if m.timeline_offset < cursor - 0.001 {
                        let push_delta = cursor - m.timeline_offset;
                        m.timeline_offset = cursor;
                        m.has_collision = true;
                        if let Some(ref gid) = m.group_id {
                            group_pushes.push((gid.clone(), push_delta));
                        }
                    }
                    cursor = m.timeline_offset + m.duration;
                }

                for (gid, delta) in group_pushes {
                    for m in &mut self.project.media_files {
                        if m.group_id.as_ref() == Some(&gid) {
                            if m.track_index == track_index && (m.media_type == MediaType::Audio) == is_audio { continue; }
                            m.timeline_offset += delta;
                            next_tracks.insert((m.track_index, m.media_type == MediaType::Audio));
                        }
                    }
                    for s in &mut self.project.subtitles {
                        if s.group_id.as_ref() == Some(&gid) {
                            s.timeline_start += delta;
                            s.timeline_end += delta;
                            s.has_collision = true;
                        }
                    }
                }
            }
            tracks_to_resolve = next_tracks;
            iterations += 1;
        }

        let mut sub_indices: Vec<usize> = (0..self.project.subtitles.len()).collect();
        sub_indices.sort_by(|&a, &b| {
            self.project.subtitles[a].timeline_start.partial_cmp(&self.project.subtitles[b].timeline_start).unwrap()
        });

        let mut sub_cursor = 0.0;
        for &idx in &sub_indices {
            let s = &mut self.project.subtitles[idx];
            if s.timeline_start < sub_cursor - 0.001 {
                let push_delta = sub_cursor - s.timeline_start;
                s.timeline_start = sub_cursor;
                s.timeline_end += push_delta;
                s.has_collision = true;
            }
            sub_cursor = s.timeline_end;
        }
    }

    pub fn total_tracks(&self) -> usize {
        1 + self.project.video_track_order.len() + self.project.audio_track_order.len()
    }

    pub fn select_all_on_track(&mut self, row: usize) {
        self.selected_ids.clear();
        self.selected_id = None;
        if row == 0 {
            for sub in &self.project.subtitles {
                self.selected_ids.insert(sub.id.clone());
            }
        } else {
            let (track_id, is_audio) = self.get_row_track(row);
            for m in &self.project.media_files {
                if m.on_timeline && m.track_index == track_id && (m.media_type == MediaType::Audio) == is_audio {
                    self.selected_ids.insert(m.id.clone());
                }
            }
        }
        self.selected_id = self.selected_ids.iter().next().cloned();
        self.selected_track = row;
    }

    pub fn select_with_group(&mut self, id: &str) {
        self.selected_ids.clear();
        self.selected_id = Some(id.to_string());
        self.selected_ids.insert(id.to_string());

        let target_group = self.project.media_files.iter()
            .find(|m| m.id == id)
            .and_then(|m| m.group_id.clone())
            .or_else(|| {
                self.project.subtitles.iter()
                    .find(|s| s.id == id)
                    .and_then(|s| s.group_id.clone())
            });

        if let Some(gid) = target_group {
            for m in &self.project.media_files {
                if m.on_timeline && m.group_id.as_deref() == Some(&gid) {
                    self.selected_ids.insert(m.id.clone());
                }
            }
            for s in &self.project.subtitles {
                if s.group_id.as_deref() == Some(&gid) {
                    self.selected_ids.insert(s.id.clone());
                }
            }
        }
    }

    pub fn group_selected(&mut self) {
        if self.selected_ids.len() < 2 { return; }
        let gid = format!("group_{}", uuid::Uuid::new_v4());
        let ids = self.selected_ids.clone();

        for m in &mut self.project.media_files {
            if ids.contains(&m.id) {
                m.group_id = Some(gid.clone());
            }
        }
        for s in &mut self.project.subtitles {
            if ids.contains(&s.id) {
                s.group_id = Some(gid.clone());
            }
        }
        self.snapshot();
    }

    pub fn ungroup_selected(&mut self) {
        let ids = self.selected_ids.clone();
        for m in &mut self.project.media_files {
            if ids.contains(&m.id) {
                m.group_id = None;
            }
        }
        for s in &mut self.project.subtitles {
            if ids.contains(&s.id) {
                s.group_id = None;
            }
        }
        self.snapshot();
    }

    pub fn is_transition_valid(&self, trans: &TransitionBlock) -> bool {
        let t_start = trans.timeline_start;
        let t_end = t_start + trans.duration;
        for m in &self.project.media_files {
            if m.on_timeline && m.media_type != MediaType::Audio {
                if trans.track_index == 0 || m.track_index == trans.track_index {
                    let m_start = m.timeline_offset;
                    let m_end = m.timeline_offset + m.duration;
                    if t_start < m_end && t_end > m_start {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_transition_at(&mut self, time: f64, track_index: usize, trans_type: TransitionType) {
        let default_dur = 1.0;
        let mut align_center = time;

        let mut best_cut_point = None;
        let mut best_diff = 1.5;

        for m in &self.project.media_files {
            if m.on_timeline && m.media_type != MediaType::Audio {
                let end = m.timeline_offset + m.duration;
                let diff = (end - time).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_cut_point = Some((end, m.track_index));
                }
            }
        }

        let actual_track = if let Some((cut_pt, trk)) = best_cut_point {
            align_center = cut_pt;
            for m in &mut self.project.media_files {
                if m.on_timeline && m.media_type != MediaType::Audio && m.track_index == trk {
                    let gap = m.timeline_offset - cut_pt;
                    if gap > 0.0 && gap < 0.6 {
                        m.timeline_offset = cut_pt;
                    }
                }
            }
            trk
        } else {
            track_index
        };

        let start = (align_center - (default_dur * 0.5)).max(0.0);
        let block = TransitionBlock {
            id: format!("trans_{}", uuid::Uuid::new_v4()),
            name: trans_type.label().to_string(),
            timeline_start: start,
            duration: default_dur,
            track_index: actual_track,
            transition_type: trans_type,
        };

        self.project.transitions.push(block);
        self.update_duration();
        self.snapshot();
    }

    pub fn shift_selected_tracks(&mut self, dragged_id: &str, row_delta: isize) {
        if row_delta == 0 { return; }

        let dragged_is_audio = self.project.media_files.iter()
            .find(|m| m.id == dragged_id)
            .map(|m| m.media_type == MediaType::Audio)
            .unwrap_or(false);

        let selected_media_ids: Vec<String> = self.project.media_files.iter()
            .filter(|m| self.selected_ids.contains(&m.id))
            .map(|m| m.id.clone())
            .collect();

        for m_id in selected_media_ids {
            let (is_audio, curr_track) = match self.project.media_files.iter().find(|m| m.id == m_id) {
                Some(m) => (m.media_type == MediaType::Audio, m.track_index),
                None => continue,
            };

            let effective_delta = if is_audio == dragged_is_audio {
                row_delta
            } else {
                -row_delta
            };

            let current_row = self.get_track_row(curr_track, is_audio) as isize;
            let mut target_row = current_row + effective_delta;

            if !is_audio {
                let v_len = self.project.video_track_order.len() as isize;
                if target_row < 1 {
                    self.add_track_row(1, true, Some(false));
                    target_row = 1;
                } else if target_row > v_len {
                    self.add_track_row(v_len as usize, false, Some(false));
                    target_row = v_len + 1;
                }
                let (new_track_id, _) = self.get_row_track(target_row as usize);
                if let Some(m) = self.project.media_files.iter_mut().find(|m| m.id == m_id) {
                    m.track_index = new_track_id;
                }
            } else {
                let v_len = self.project.video_track_order.len() as isize;
                let a_len = self.project.audio_track_order.len() as isize;
                let min_audio_row = v_len + 1;
                let max_audio_row = v_len + a_len;

                if target_row < min_audio_row {
                    self.add_track_row(min_audio_row as usize, true, Some(true));
                    target_row = min_audio_row;
                } else if target_row > max_audio_row {
                    self.add_track_row(max_audio_row as usize, false, Some(true));
                    target_row = max_audio_row + 1;
                }
                let (new_track_id, _) = self.get_row_track(target_row as usize);
                if let Some(m) = self.project.media_files.iter_mut().find(|m| m.id == m_id) {
                    m.track_index = new_track_id;
                }
            }
        }

        self.mark_modified();
    }

    pub fn apply_preset(&mut self, category: &str, preset: &str) {
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) && category == "Text" {
                match preset {
                    "Default"     => { sub.y = 0.0;    sub.x = 0.0;    sub.font_size = 36.0; }
                    "Lower Third" => { sub.y = 380.0;  sub.x = -600.0; sub.font_size = 30.0; }
                    "Title Card"  => { sub.y = 0.0;    sub.x = 0.0;    sub.font_size = 72.0; }
                    "Caption"     => { sub.y = 420.0;  sub.x = 0.0;    sub.font_size = 28.0; }
                    _ => {}
                }
            }
        }
        for m in self.project.media_files.iter_mut() {
            if ids.contains(&m.id) {
                if category == "Video" {
                    match preset {
                        "Fit to Screen" => { m.scale = 1.0; m.x = 0.0; m.y = 0.0; }
                        "Picture in Picture" => { m.scale = 0.3; m.x = 600.0; m.y = 300.0; }
                        _ => {}
                    }
                }
            }
        }
        self.snapshot();
    }

    pub fn get_track_row(&self, track_index: usize, is_audio: bool) -> usize {
        if track_index == 0 { return 0; }
        if !is_audio {
            if let Some(pos) = self.project.video_track_order.iter().position(|&id| id == track_index) {
                return pos + 1;
            }
            1
        } else {
            if let Some(pos) = self.project.audio_track_order.iter().position(|&id| id == track_index) {
                return self.project.video_track_order.len() + 1 + pos;
            }
            self.project.video_track_order.len() + 1
        }
    }

    pub fn get_row_track(&self, row: usize) -> (usize, bool) {
        if row == 0 { return (0, false); }
        let v_len = self.project.video_track_order.len();
        if row <= v_len {
            (self.project.video_track_order[row - 1], false)
        } else {
            let a_idx = row - 1 - v_len;
            if a_idx < self.project.audio_track_order.len() {
                (self.project.audio_track_order[a_idx], true)
            } else {
                (0, false)
            }
        }
    }

    pub fn get_track_name(&self, row: usize) -> String {
        let (id, is_audio) = self.get_row_track(row);
        if id == 0 { return "SUBTITLES".into(); }
        if is_audio {
            format!("AUDIO {}", id)
        } else {
            format!("VIDEO {}", id)
        }
    }

    pub fn add_track_row(&mut self, row: usize, above: bool, force_audio: Option<bool>) {
        let (_, mut is_audio) = self.get_row_track(row);
        if let Some(fa) = force_audio { is_audio = fa; }
        
        let new_id = if is_audio {
            let id = self.project.next_audio_id;
            self.project.next_audio_id += 1;
            id
        } else {
            let id = self.project.next_video_id;
            self.project.next_video_id += 1;
            id
        };

        if is_audio {
            let v_len = self.project.video_track_order.len();
            let mut insert_pos = if row <= v_len { 0 } else { row - 1 - v_len };
            if !above && row > v_len { insert_pos += 1; }
            self.project.audio_track_order.insert(insert_pos.min(self.project.audio_track_order.len()), new_id);
        } else {
            let mut insert_pos = if row == 0 { 0 } else { row - 1 };
            if !above && row > 0 { insert_pos += 1; }
            self.project.video_track_order.insert(insert_pos.min(self.project.video_track_order.len()), new_id);
        }
        self.mark_modified();
    }

    pub fn move_track_row(&mut self, from_row: usize, to_row: usize) {
        if from_row == 0 || to_row == 0 || from_row == to_row { return; }
        
        let (from_id, from_is_audio) = self.get_row_track(from_row);
        let (_, to_is_audio) = self.get_row_track(to_row);
        
        if from_is_audio != to_is_audio { return; }
        
        if from_is_audio {
            let v_len = self.project.video_track_order.len();
            let from_pos = from_row - 1 - v_len;
            let to_pos = to_row - 1 - v_len;
            self.project.audio_track_order.remove(from_pos);
            self.project.audio_track_order.insert(to_pos.min(self.project.audio_track_order.len()), from_id);
        } else {
            let from_pos = from_row - 1;
            let to_pos = to_row - 1;
            self.project.video_track_order.remove(from_pos);
            self.project.video_track_order.insert(to_pos.min(self.project.video_track_order.len()), from_id);
        }
        self.mark_modified();
    }

    pub fn delete_track(&mut self, row: usize) {
        let (id, is_audio) = self.get_row_track(row);
        if id == 0 { return; } 
        
        if is_audio {
            if self.project.audio_track_order.len() > 1 {
                self.project.media_files.retain(|m| !(m.track_index == id && m.media_type == MediaType::Audio));
                if let Some(pos) = self.project.audio_track_order.iter().position(|&x| x == id) {
                    self.project.audio_track_order.remove(pos);
                }
            }
        } else {
            if self.project.video_track_order.len() > 1 {
                self.project.media_files.retain(|m| !(m.track_index == id && m.media_type != MediaType::Audio));
                if let Some(pos) = self.project.video_track_order.iter().position(|&x| x == id) {
                    self.project.video_track_order.remove(pos);
                }
            }
        }
        self.mark_modified();
    }

    pub fn find_free_audio_track(&mut self, start: f64, duration: f64) -> usize {
        let ids = self.project.audio_track_order.clone();
        for &track_id in &ids {
            if self.can_place_media(None, track_id, true, start, duration) {
                return track_id;
            }
        }
        let new_id = self.project.next_audio_id;
        self.project.next_audio_id += 1;
        self.project.audio_track_order.push(new_id);
        new_id
    }

    pub fn can_place_media(&self, ignore_id: Option<&str>, track_id: usize, is_audio: bool, start: f64, duration: f64) -> bool {
        let end = start + duration;
        for m in &self.project.media_files {
            if m.on_timeline && m.track_index == track_id && (m.media_type == MediaType::Audio) == is_audio {
                if let Some(ig) = ignore_id {
                    if m.id == ig { continue; }
                }
                let m_s = m.timeline_offset;
                let m_e = m_s + m.duration;
                if start < m_e - 0.001 && end > m_s + 0.001 {
                    return false;
                }
            }
        }
        true
    }

    pub fn place_media_on_timeline(&mut self, drag_id: String, mut target_time: f64, mut target_row: usize) {
        target_time = target_time.max(0.0);
        if target_row == 0 { target_row = 1; }

        let (mut target_track_id, target_is_audio) = self.get_row_track(target_row);

        let base_media = match self.project.media_files.iter().find(|m| m.id == drag_id) {
            Some(m) => m.clone(),
            None => return,
        };

        let is_audio_media = base_media.media_type == MediaType::Audio;

        if is_audio_media != target_is_audio {
            target_track_id = if is_audio_media {
                self.project.audio_track_order.first().copied().unwrap_or(1)
            } else {
                self.project.video_track_order.first().copied().unwrap_or(1)
            };
        }

        let is_already_cut = base_media.trim_start > 0.0 || base_media.name.contains("(Cut)");

        let duration_to_place = if base_media.media_type == MediaType::Image {
            5.0
        } else if is_already_cut {
            base_media.duration
        } else if let Some(src_dur) = base_media.source_duration {
            src_dur
        } else {
            base_media.duration
        };

        let pair_group_id = if base_media.media_type == MediaType::Video {
            Some(format!("group_{}", uuid::Uuid::new_v4()))
        } else {
            None
        };

        if let Some(media) = self.project.media_files.iter_mut().find(|m| m.id == drag_id) {
            media.on_timeline = true;
            media.timeline_offset = target_time;
            media.track_index = target_track_id;
            media.duration = duration_to_place;
            if !is_already_cut {
                media.trim_start = 0.0;
            }
            if pair_group_id.is_some() {
                media.group_id = pair_group_id.clone();
            }
        }

        if base_media.media_type == MediaType::Video {
            let audio_track_id = self.find_free_audio_track(target_time, duration_to_place);
            let mut audio_clone = base_media.clone();
            let audio_id = format!("media_audio_{}", uuid::Uuid::new_v4());
            audio_clone.id = audio_id.clone();
            audio_clone.name = format!("{} (Audio)", base_media.name);
            audio_clone.media_type = MediaType::Audio;
            audio_clone.on_timeline = true;
            audio_clone.timeline_offset = target_time;
            audio_clone.duration = duration_to_place;
            audio_clone.source_duration = base_media.source_duration.or(Some(duration_to_place));
            if !is_already_cut {
                audio_clone.trim_start = 0.0;
            }
            audio_clone.track_index = audio_track_id;
            audio_clone.group_id = pair_group_id;
            
            let extracted_wav = std::env::temp_dir().join("kineticsub_cache").join(&base_media.id).join("audio.wav");
            audio_clone.path = extracted_wav.to_string_lossy().to_string();

            self.project.media_files.push(audio_clone);

            self.selected_ids.clear();
            self.selected_ids.insert(drag_id.clone());
            self.selected_ids.insert(audio_id);
            self.selected_id = Some(drag_id);
        } else {
            self.selected_ids.clear();
            self.selected_ids.insert(drag_id.clone());
            self.selected_id = Some(drag_id);
        }

        self.resolve_all_overlaps();
        self.update_duration();
        self.snapshot();
    }

    pub fn get_or_load_texture(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if path.is_empty() { return None; }
        if let Some(tex) = self.texture_cache.get(path) {
            return Some(tex.clone());
        }

        if let Ok(mut image) = image::open(path) {
            let max_dim = 2048;
            if image.width() > max_dim || image.height() > max_dim {
                image = image.resize(max_dim, max_dim, image::imageops::FilterType::Triangle);
            }
            
            let image_rgba = image.to_rgba8();
            let dimensions = image_rgba.dimensions();
            let size = [dimensions.0 as usize, dimensions.1 as usize];
            let pixels = image_rgba.into_raw();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let handle = ctx.load_texture(path, color_image, egui::TextureOptions::LINEAR);
            self.texture_cache.insert(path.to_string(), handle.clone());
            return Some(handle);
        }

        None
    }

    pub fn start_model_download(&mut self, model: WhisperModel) {
        if self.downloading_model.is_some() { return; }
        let (tx, rx) = mpsc::channel();
        self.model_download_rx = Some(rx);
        self.downloading_model = Some(model);
        self.model_download_progress = 0.0;
        self.model_download_status = format!("Connecting to download {}...", model.id());
        spawn_download_model(model, tx);
    }

    pub fn poll_model_download(&mut self) {
        let mut finished = false;
        if let Some(rx) = &self.model_download_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ModelDownloadMessage::Progress { current, total } => {
                        if total > 0 {
                            self.model_download_progress = current as f32 / total as f32;
                            let mb_curr = current as f64 / 1_048_576.0;
                            let mb_tot = total as f64 / 1_048_576.0;
                            self.model_download_status = format!(
                                "{:.1} MB / {:.1} MB ({:.0}%)",
                                mb_curr, mb_tot, self.model_download_progress * 100.0
                            );
                        } else {
                            let mb_curr = current as f64 / 1_048_576.0;
                            self.model_download_status = format!("{:.1} MB downloaded", mb_curr);
                        }
                    }
                    ModelDownloadMessage::Done => {
                        self.model_download_progress = 1.0;
                        self.model_download_status = "Download complete!".into();
                        finished = true;
                    }
                    ModelDownloadMessage::Error(e) => {
                        self.model_download_status = format!("Error: {}", e);
                        finished = true;
                    }
                }
            }
        }
        if finished {
            self.downloading_model = None;
            self.model_download_rx = None;
        }
    }

    pub fn is_downloading_model(&self) -> bool {
        self.downloading_model.is_some()
    }

    pub fn update_duration(&mut self) {
        let mut max_end = 10.0; 
        for m in &self.project.media_files {
            if m.on_timeline { let e = m.timeline_offset + m.duration; if e > max_end { max_end = e; } }
        }
        for s in &self.project.subtitles {
            if s.timeline_end > max_end { max_end = s.timeline_end; }
        }
        for t in &self.project.transitions {
            let e = t.timeline_start + t.duration;
            if e > max_end { max_end = e; }
        }
        self.project.duration = max_end;
        self.sync.duration = max_end;
    }

    pub fn px_to_time(&self, px: f32) -> f64 {
        self.timeline_scroll + (px / self.timeline_zoom) as f64
    }
    pub fn time_to_px(&self, t: f64) -> f32 {
        ((t - self.timeline_scroll) * self.timeline_zoom as f64) as f32
    }
}

impl Default for EditorViewModel { fn default() -> Self { Self::new() } }