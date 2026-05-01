use std::sync::mpsc;
use super::{EditorViewModel, TranscribeMode};
use crate::models::types::{MediaFile, Subtitle};
use crate::services::whisper_service::{spawn_transcription, WhisperMessage};

impl EditorViewModel {
    // ── Media ─────────────────────────────────────────────────────────────────
    pub fn import_audio(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Audio", &["wav", "mp3", "m4a", "ogg"])
            .pick_file()
        {
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let id = format!("media_{}", uuid::Uuid::new_v4());
            self.project.media_files.push(MediaFile {
                id,
                path: path.to_string_lossy().to_string(),
                name,
                timeline_offset: 0.0,
                duration: 60.0,
                on_timeline: false,
                is_video_track: false,
                color: None,
            });
            self.update_duration();
            self.snapshot();
        }
    }

    pub fn add_solid_bg(&mut self) {
        let id = format!("bg_{}", uuid::Uuid::new_v4());
        self.project.media_files.push(MediaFile {
            id,
            name: "Solid Background".into(),
            path: "".into(),
            timeline_offset: 0.0,
            duration: self.project.duration.max(10.0),
            on_timeline: true,
            is_video_track: true,
            color: Some([0.05, 0.05, 0.05, 1.0]), // Dark grey
        });
        self.update_duration();
        self.snapshot();
    }

    pub fn toggle_media_timeline(&mut self, id: &str) {
        if let Some(media) = self.project.media_files.iter_mut().find(|m| m.id == id) {
            media.on_timeline = !media.on_timeline;
        }
        self.update_duration();
        self.snapshot();
    }

    pub fn move_media(&mut self, index: usize, delta_secs: f64) {
        let (media_id, old_offset) = match self.project.media_files.get(index) {
            Some(m) => (m.id.clone(), m.timeline_offset),
            None    => return,
        };

        let new_offset = (old_offset + delta_secs).max(0.0);
        let actual_delta = new_offset - old_offset;

        if let Some(m) = self.project.media_files.get_mut(index) {
            m.timeline_offset = new_offset;
        }

        for sub in self.project.subtitles.iter_mut() {
            if sub.media_id.as_deref() == Some(&media_id) {
                sub.timeline_start = (sub.timeline_start + actual_delta).max(0.0);
                sub.timeline_end   = (sub.timeline_end   + actual_delta).max(sub.timeline_start + 0.05);
            }
        }
        self.mark_modified();
        self.update_duration();
    }

    // ── Whisper ───────────────────────────────────────────────────────────────

    pub fn start_auto_transcription(&mut self, media_id: String) -> bool {
        let audio_path = match self.project.media_files.iter().find(|m| m.id == media_id) {
            Some(m) if m.on_timeline && !m.is_video_track => m.path.clone(),
            _ => return false,
        };

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
                        let pct = if total > 0 { (cur as f32 / total as f32 * 100.0) as u32 } else { 0 };
                        self.whisper_status = format!("Downloading Model... {}%", pct);
                    }
                    WhisperMessage::Transcribing => { self.whisper_status = "Transcribing Audio...".into(); }
                    WhisperMessage::Done(words, duration) => {
                        let media_id = self.transcribing_media_id.clone().unwrap();
                        let offset = {
                            match self.project.media_files.iter_mut().find(|m| m.id == media_id) {
                                Some(m) => { m.duration = duration; m.timeline_offset }
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
                        self.snapshot(); // Commit to history
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