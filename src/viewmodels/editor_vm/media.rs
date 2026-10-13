use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::process::Command;
use super::{EditorViewModel, TranscribeMode, PreviewMessage};
use crate::models::types::{MediaFile, MediaType, Subtitle};
use crate::services::whisper_service::{
    is_model_downloaded, spawn_transcription, WhisperMessage,
};

fn probe_media_duration(path: &Path) -> Option<f64> {
    let mut cmd = Command::new("ffprobe");
    cmd.args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(path);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let out = cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<f64>().ok()
}

impl EditorViewModel {
    pub fn import_media(&mut self, path_opt: Option<PathBuf>) {
        let paths = if let Some(p) = path_opt {
            vec![p]
        } else {
            if let Some(files) = rfd::FileDialog::new()
                .add_filter("Media Files", &["mp4", "mkv", "mov", "avi", "png", "jpg", "jpeg", "webp", "wav", "mp3", "m4a", "ogg"])
                .pick_files() {
                files
            } else {
                vec![]
            }
        };

        for path in paths {
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            
            let media_type = match ext.as_str() {
                "mp4" | "mkv" | "mov" | "avi" => MediaType::Video,
                "png" | "jpg" | "jpeg" | "webp" => MediaType::Image,
                _ => MediaType::Audio,
            };

            let id = format!("media_{}", uuid::Uuid::new_v4());
            
            let probed_duration = if media_type != MediaType::Image {
                probe_media_duration(&path)
            } else {
                Some(5.0)
            };

            let duration = if media_type == MediaType::Image { 
                5.0 
            } else { 
                probed_duration.unwrap_or(60.0) 
            }; 

            let path_str = path.to_string_lossy().to_string();
            let mut stored_path = path_str.clone();

            if media_type == MediaType::Audio {
                let extracted_wav = std::env::temp_dir().join("kineticsub_cache").join(&id).join("audio.wav");
                stored_path = extracted_wav.to_string_lossy().to_string();
            }

            let mut source_width = None;
            let mut source_height = None;
            let mut scale = 1.0;

            if media_type == MediaType::Image {
                if let Ok((w, h)) = image::image_dimensions(&path) {
                    source_width = Some(w);
                    source_height = Some(h);
                    let scale_w = self.project.resolution.0 as f32 / w as f32;
                    let scale_h = self.project.resolution.1 as f32 / h as f32;
                    scale = 1.0f32.min(scale_w.min(scale_h)); 
                }
            }

            self.project.media_files.push(MediaFile {
                id: id.clone(),
                path: stored_path, 
                name,
                timeline_offset: 0.0,
                duration,
                on_timeline: false,
                media_type: media_type.clone(),
                track_index: 1, 
                color: None,
                source_fps: None,
                source_duration: probed_duration,
                source_width,
                source_height,
                preview_dir: None,
                preview_ready: false,
                x: 0.0,
                y: 0.0,
                scale,
                rotation: 0.0,
                opacity: 1.0,
                volume: 1.0,
                trim_start: 0.0,
                playback_speed: 1.0,
                keyframes: Vec::new(),
                loop_mode: Default::default(),
                group_id: None,
                folder_id: None,
                has_collision: false,
            });

            if media_type != MediaType::Image {
                self.spawn_media_extractor(id, path_str, media_type == MediaType::Video, self.project.fps);
            }
        }
        self.update_duration();
        self.snapshot();
    }

    pub fn poll_preview_extraction(&mut self) {
        let mut incoming = Vec::new();
        if let Some(rx) = &self.preview_rx {
            while let Ok(msg) = rx.try_recv() {
                incoming.push(msg);
            }
        }

        let mut needs_duration_update = false;
        
        for msg in incoming {
            match msg {
                PreviewMessage::MetadataReady { id, duration, fps, width, height } => {
                    if let Some(m) = self.project.media_files.iter_mut().find(|m| m.id == id) {
                        m.source_duration = Some(duration);
                        if m.media_type == MediaType::Image {
                            m.duration = 5.0;
                        } else if !m.on_timeline {
                            m.duration = duration;
                        }
                        if fps > 0.0 { m.source_fps = Some(fps); }
                        if width > 0 && height > 0 {
                            m.source_width = Some(width);
                            m.source_height = Some(height);
                            if m.scale == 1.0 {
                                let scale_w = self.project.resolution.0 as f32 / width as f32;
                                let scale_h = self.project.resolution.1 as f32 / height as f32;
                                m.scale = 1.0f32.min(scale_w.min(scale_h));
                            }
                        }
                    }

                    for m in self.project.media_files.iter_mut() {
                        if m.media_type == MediaType::Audio && m.path.contains(&id) {
                            m.source_duration = Some(duration);
                            if !m.on_timeline {
                                m.duration = duration;
                            }
                        }
                    }

                    needs_duration_update = true;
                }
                PreviewMessage::ExtractionComplete { id } => {
                    if let Some(m) = self.project.media_files.iter_mut().find(|m| m.id == id) {
                        let temp_dir = std::env::temp_dir().join("kineticsub_cache").join(&id);
                        m.preview_dir = Some(temp_dir.to_string_lossy().to_string());
                        m.preview_ready = true;
                    }
                }
                PreviewMessage::Error(_) => {}
            }
        }
        
        if needs_duration_update {
            self.update_duration();
        }
    }

    pub fn add_solid_bg(&mut self, color: [f32; 4]) {
        let id = format!("bg_{}", uuid::Uuid::new_v4());
        self.project.media_files.push(MediaFile {
            id,
            name: "Solid Background".into(),
            path: "".into(),
            timeline_offset: 0.0,
            duration: 10.0,
            on_timeline: false, 
            media_type: MediaType::Video,
            track_index: 1, 
            color: Some(color),
            source_fps: None,
            source_duration: None,
            source_width: None,
            source_height: None,
            preview_dir: None,
            preview_ready: true,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rotation: 0.0,
            opacity: 1.0,
            volume: 1.0,
            trim_start: 0.0,
            playback_speed: 1.0,
            keyframes: Vec::new(),
            loop_mode: Default::default(),
            group_id: None,
            folder_id: None,
            has_collision: false,
        });
        self.snapshot();
    }

    pub fn spawn_media_extractor(&mut self, id: String, path: String, is_video: bool, project_fps: u32) {
        if self.preview_rx.is_none() {
            let (tx, rx) = mpsc::channel();
            self.preview_rx = Some(rx);
            let tx_clone = tx.clone();
            std::thread::spawn(move || Self::run_extraction(id, path, is_video, project_fps, tx_clone));
        } else if let Some(rx) = self.preview_rx.take() {
            let (tx, new_rx) = mpsc::channel();
            self.preview_rx = Some(new_rx);
            let tx_clone = tx.clone();
            std::thread::spawn(move || Self::run_extraction(id, path, is_video, project_fps, tx_clone));
            std::thread::spawn(move || {
                for msg in rx { let _ = tx.send(msg); }
            });
        }
    }

    fn run_extraction(id: String, path: String, is_video: bool, project_fps: u32, tx: mpsc::Sender<PreviewMessage>) {
        let dur_output = Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", &path])
            .output();
        
        let mut duration = 60.0;
        if let Ok(out) = dur_output {
            if let Ok(d_str) = String::from_utf8(out.stdout) {
                if let Ok(d) = d_str.trim().parse::<f64>() {
                    duration = d;
                }
            }
        }

        let mut source_fps = 0.0;
        let mut width = 0;
        let mut height = 0;

        let temp_dir = std::env::temp_dir().join("kineticsub_cache").join(&id);
        let _ = std::fs::create_dir_all(&temp_dir);

        let _cmd_audio = Command::new("ffmpeg")
            .args([
                "-i", &path,
                "-vn", 
                "-acodec", "pcm_s16le",
                "-ar", "44100",
                "-ac", "2",
                "-y",
                temp_dir.join("audio.wav").to_str().unwrap()
            ])
            .output();

        if is_video {
            let fps_output = Command::new("ffprobe")
                .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=r_frame_rate", "-of", "default=noprint_wrappers=1:nokey=1", &path])
                .output();
            if let Ok(out) = fps_output {
                if let Ok(f_str) = String::from_utf8(out.stdout) {
                    let parts: Vec<&str> = f_str.trim().split('/').collect();
                    if parts.len() == 2 {
                        if let (Ok(num), Ok(den)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                            if den > 0.0 { source_fps = num / den; }
                        }
                    } else if parts.len() == 1 {
                        if let Ok(f) = parts[0].parse::<f64>() { source_fps = f; }
                    }
                }
            }

            let size_output = Command::new("ffprobe")
                .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0", &path])
                .output();
            if let Ok(out) = size_output {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let parts: Vec<&str> = s.trim().split(',').collect();
                    if parts.len() >= 2 {
                        width = parts[0].parse().unwrap_or(0);
                        height = parts[1].parse().unwrap_or(0);
                    }
                }
            }

            let _cmd_video = Command::new("ffmpeg")
                .args([
                    "-i", &path, 
                    "-vf", "scale=-1:360", 
                    "-r", &project_fps.to_string(), 
                    "-q:v", "5", 
                    "-y",
                    temp_dir.join("frame_%04d.jpg").to_str().unwrap()
                ])
                .output();
        }

        let _ = tx.send(PreviewMessage::MetadataReady { id: id.clone(), duration, fps: source_fps, width, height });
        let _ = tx.send(PreviewMessage::ExtractionComplete { id });
    }

    pub fn toggle_media_timeline(&mut self, id: &str) {
        if let Some(media) = self.project.media_files.iter_mut().find(|m| m.id == id) {
            media.on_timeline = !media.on_timeline;
            if media.on_timeline {
                media.track_index = 1; 
                if media.media_type == MediaType::Image {
                    media.duration = 5.0;
                } else if media.trim_start == 0.0 && !media.name.contains("(Cut)") {
                    if let Some(src_dur) = media.source_duration {
                        media.duration = src_dur;
                    }
                }
            }
        }
        self.update_duration();
        self.snapshot();
    }

    pub fn move_media(&mut self, index: usize, delta_secs: f64, mut target_track: usize) {
        let media = match self.project.media_files.get(index) {
            Some(m) => m.clone(),
            None    => return,
        };
        
        if target_track == 0 { target_track = 1; }
        let is_audio = media.media_type == MediaType::Audio;

        let current_row = self.get_track_row(media.track_index, is_audio);
        let target_row  = self.get_track_row(target_track, is_audio);

        let mut final_track = media.track_index;
        if target_track != media.track_index {
            if self.can_place_media(Some(&media.id), target_track, is_audio, media.timeline_offset, media.duration) {
                final_track = target_track;
            } else {
                let v_len = self.project.video_track_order.len();
                let a_len = self.project.audio_track_order.len();
                let (min_r, max_r) = if !is_audio {
                    (1, v_len)
                } else {
                    (v_len + 1, v_len + a_len)
                };

                if target_row > current_row {
                    for r in target_row..=max_r {
                        let (cand_track, cand_is_audio) = self.get_row_track(r);
                        if cand_is_audio == is_audio && self.can_place_media(Some(&media.id), cand_track, is_audio, media.timeline_offset, media.duration) {
                            final_track = cand_track;
                            break;
                        }
                    }
                } else if target_row < current_row {
                    for r in (min_r..=target_row).rev() {
                        let (cand_track, cand_is_audio) = self.get_row_track(r);
                        if cand_is_audio == is_audio && self.can_place_media(Some(&media.id), cand_track, is_audio, media.timeline_offset, media.duration) {
                            final_track = cand_track;
                            break;
                        }
                    }
                }
            }
        }

        let target_offset = (media.timeline_offset + delta_secs).max(0.0);
        let actual_delta = target_offset - media.timeline_offset;

        if let Some(m) = self.project.media_files.get_mut(index) {
            m.timeline_offset = target_offset;
            m.track_index = final_track;
        }

        if let Some(ref gid) = media.group_id {
            for (i, m) in self.project.media_files.iter_mut().enumerate() {
                if i != index && m.on_timeline && m.group_id.as_deref() == Some(gid) {
                    m.timeline_offset = (m.timeline_offset + actual_delta).max(0.0);
                }
            }
            for sub in self.project.subtitles.iter_mut() {
                if sub.group_id.as_deref() == Some(gid) || sub.media_id.as_deref() == Some(&media.id) {
                    sub.timeline_start = (sub.timeline_start + actual_delta).max(0.0);
                    sub.timeline_end = (sub.timeline_end + actual_delta).max(sub.timeline_start + 0.05);
                }
            }
        } else {
            for sub in self.project.subtitles.iter_mut() {
                if sub.media_id.as_deref() == Some(&media.id) {
                    sub.timeline_start = (sub.timeline_start + actual_delta).max(0.0);
                    sub.timeline_end   = (sub.timeline_end   + actual_delta).max(sub.timeline_start + 0.05);
                }
            }
        }

        self.resolve_all_overlaps();
        self.mark_modified();
        self.update_duration();
    }

    pub fn resize_media_clip(&mut self, id: &str, mode: u8, dt: f64) {
        if dt == 0.0 || (mode != 1 && mode != 2) { return; }

        let target_media = match self.project.media_files.iter().find(|m| m.id == id) {
            Some(m) => m.clone(),
            None => return,
        };

        let group_id = target_media.group_id.clone();
        let is_grouped = group_id.is_some();

        let member_ids: Vec<String> = if let Some(ref gid) = group_id {
            self.project.media_files.iter()
                .filter(|m| m.on_timeline && m.group_id.as_deref() == Some(gid))
                .map(|m| m.id.clone())
                .collect()
        } else {
            vec![id.to_string()]
        };

        let mut allowed_min_dt: f64 = -1e9_f64;
        let mut allowed_max_dt: f64 = 1e9_f64;

        for m_id in &member_ids {
            if let Some(m) = self.project.media_files.iter().find(|m| &m.id == m_id) {
                let speed = m.playback_speed.max(0.001);
                let is_audio = m.media_type == MediaType::Audio;

                let mut left_neighbor_end = 0.0_f64;
                let mut right_neighbor_start = f64::INFINITY;

                for other in &self.project.media_files {
                    if other.id != m.id && other.on_timeline && other.track_index == m.track_index && (other.media_type == MediaType::Audio) == is_audio {
                        let o_end = other.timeline_offset + other.duration;
                        if o_end <= m.timeline_offset + 0.001 {
                            if o_end > left_neighbor_end { left_neighbor_end = o_end; }
                        }
                        if other.timeline_offset >= m.timeline_offset + m.duration - 0.001 {
                            if other.timeline_offset < right_neighbor_start { right_neighbor_start = other.timeline_offset; }
                        }
                    }
                }

                if mode == 1 {
                    let min_dt_timeline = left_neighbor_end - m.timeline_offset;
                    let min_dt_src = if m.media_type == MediaType::Image || m.color.is_some() {
                        -1e9_f64
                    } else {
                        -m.trim_start / speed
                    };
                    let member_min = min_dt_timeline.max(min_dt_src);
                    let member_max = m.duration - 0.1;

                    allowed_min_dt = allowed_min_dt.max(member_min);
                    allowed_max_dt = allowed_max_dt.min(member_max);
                } else if mode == 2 {
                    let member_min = 0.1 - m.duration;
                    let max_dt_timeline = right_neighbor_start - (m.timeline_offset + m.duration);
                    let max_dt_src = if let Some(src_dur) = m.source_duration {
                        if m.color.is_none() && m.media_type != MediaType::Image {
                            ((src_dur - m.trim_start) / speed - m.duration).max(0.0)
                        } else {
                            1e9_f64
                        }
                    } else {
                        1e9_f64
                    };
                    let member_max = max_dt_timeline.min(max_dt_src);

                    allowed_min_dt = allowed_min_dt.max(member_min);
                    allowed_max_dt = allowed_max_dt.min(member_max);
                }
            }
        }

        if allowed_min_dt > allowed_max_dt { return; }
        let effective_dt = dt.clamp(allowed_min_dt, allowed_max_dt);
        if effective_dt.abs() < 1e-6 { return; }

        for m_id in &member_ids {
            if let Some(m) = self.project.media_files.iter_mut().find(|m| &m.id == m_id) {
                let speed = m.playback_speed.max(0.001);
                if mode == 1 {
                    m.timeline_offset += effective_dt;
                    m.duration = (m.duration - effective_dt).max(0.1);
                    m.trim_start = (m.trim_start + effective_dt * speed).max(0.0);
                } else if mode == 2 {
                    m.duration = (m.duration + effective_dt).max(0.1);
                }
            }
        }

        if is_grouped {
            if let Some(ref gid) = group_id {
                for sub in self.project.subtitles.iter_mut() {
                    if sub.group_id.as_deref() == Some(gid) {
                        if mode == 1 {
                            sub.timeline_start += effective_dt;
                        } else if mode == 2 {
                            sub.timeline_end += effective_dt;
                        }
                    }
                }
            }
        }

        self.mark_modified();
        self.update_duration();
    }

    pub fn start_auto_transcription(&mut self, media_id: String) -> bool {
        let audio_path = match self.project.media_files.iter().find(|m| m.id == media_id) {
            Some(m) if m.on_timeline && m.media_type != MediaType::Image => {
                if m.media_type == MediaType::Video {
                    let extracted_wav = std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav");
                    extracted_wav.to_string_lossy().to_string()
                } else {
                    m.path.clone()
                }
            }
            _ => return false,
        };

        if !is_model_downloaded(self.selected_whisper_model) {
            self.whisper_status = format!("Model '{}' not installed.", self.selected_whisper_model.id());
            self.show_settings_modal = true;
            return false;
        }

        self.project.subtitles.retain(|s| s.media_id.as_deref() != Some(&media_id));

        let (tx, rx) = mpsc::channel();
        self.whisper_rx = Some(rx);
        self.transcribing_media_id = Some(media_id);
        self.whisper_status = "Initializing Whisper...".into();
        spawn_transcription(audio_path, self.selected_whisper_model, tx);
        true
    }

    pub fn poll_whisper(&mut self) {
        if let Some(rx) = &self.whisper_rx {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    WhisperMessage::Transcribing => {
                        self.whisper_status = "Transcribing Audio...".into();
                    }
                    WhisperMessage::Done(words, _duration) => {
                        let media_id = self.transcribing_media_id.clone().unwrap();
                        let offset = {
                            match self.project.media_files.iter_mut().find(|m| m.id == media_id) {
                                Some(m) => { m.timeline_offset }
                                None    => 0.0,
                            }
                        };

                        let mut next_id = self.next_id;

                        match self.transcribe_mode {
                            TranscribeMode::Word => {
                                for w in words {
                                    let abs_start = offset + w.start;
                                    let abs_end   = offset + w.end;
                                    let mut sub = Subtitle::new(&format!("sub_{}", next_id), &w.text, abs_start, abs_end);
                                    next_id += 1;
                                    sub.media_id = Some(media_id.clone());
                                    self.project.subtitles.push(sub);
                                }
                            }
                            TranscribeMode::Phrase => {
                                let mut current_phrase: std::vec::Vec<crate::models::types::SubtitleWord> = Vec::new();
                                
                                let flush_phrase = |phrase: &mut Vec<crate::models::types::SubtitleWord>, proj: &mut crate::models::types::Project, nid: &mut u32| {
                                    if phrase.is_empty() { return; }
                                    let start = phrase.first().unwrap().start;
                                    let end = phrase.last().unwrap().end;
                                    let text = phrase.iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
                                    
                                    let mut sub = Subtitle::new(&format!("sub_{}", *nid), &text, start, end);
                                    *nid += 1;
                                    
                                    sub.words = phrase.clone();
                                    sub.media_id = Some(media_id.clone());
                                    proj.subtitles.push(sub);
                                    phrase.clear();
                                };

                                for w in words {
                                    let abs_start = offset + w.start;
                                    let abs_end   = offset + w.end;
                                    
                                    let is_end = w.text.ends_with('.') || w.text.ends_with('?') || w.text.ends_with('!') || w.text.ends_with(',');

                                    if !current_phrase.is_empty() {
                                        let last_end = current_phrase.last().unwrap().end;
                                        if abs_start - last_end > 0.4 || current_phrase.len() >= 6 {
                                            flush_phrase(&mut current_phrase, &mut self.project, &mut next_id);
                                        }
                                    }
                                    
                                    current_phrase.push(crate::models::types::SubtitleWord { 
                                        text: w.text.clone(), start: abs_start, end: abs_end, custom_color: None 
                                    });
                                    
                                    if is_end {
                                        flush_phrase(&mut current_phrase, &mut self.project, &mut next_id);
                                    }
                                }
                                flush_phrase(&mut current_phrase, &mut self.project, &mut next_id);
                            }
                        }

                        self.next_id = next_id;
                        self.sort_subtitles();
                        self.update_duration();
                        self.whisper_status = "Done!".into();
                        self.whisper_rx = None;
                        self.transcribing_media_id = None;
                        self.snapshot();
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
}