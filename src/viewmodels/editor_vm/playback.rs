use super::EditorViewModel;
use crate::models::types::MediaType;
use std::path::PathBuf;

impl EditorViewModel {
    // ── Playback ──────────────────────────────────────────────────────────────
    pub fn tick(&mut self) {
        let dt = self.sync.tick();
        
        if dt > 0.0 {
            let instant_fps = 1.0 / dt;
            if self.current_fps == 0.0 {
                self.current_fps = instant_fps;
            } else {
                self.current_fps = self.current_fps * 0.9 + instant_fps * 0.1;
            }
        }

        let t = self.sync.current_time;

        // Preview Sync System (Independent of Timeline state)
        if let Some(ref prev_id) = self.previewing_media_id {
            if self.preview_playing {
                self.preview_time += dt;
                if let Some(m) = self.project.media_files.iter().find(|m| m.id == *prev_id) {
                    if self.preview_time >= m.duration {
                        self.preview_time = m.duration;
                        self.preview_playing = false;
                        self.audio_player.stop("preview");
                    } else if m.media_type != MediaType::Image {
                        if !self.audio_player.is_playing("preview") {
                            // Extract audio safely for BOTH Audio and Video clips
                            let audio_path = if m.media_type == MediaType::Video {
                                std::env::temp_dir().join("kineticsub_cache").join(&m.id).join("audio.wav")
                            } else {
                                PathBuf::from(&m.path)
                            };
                            
                            if audio_path.exists() {
                                self.audio_player.play("preview", audio_path, self.preview_time, 1.0);
                            }
                        }
                    }
                }
            } else {
                self.audio_player.stop("preview");
            }
            
            // Mute background timeline while previewing
            for id in self.audio_player.sinks.keys().cloned().collect::<Vec<_>>() {
                if id != "preview" { self.audio_player.stop(&id); }
            }
            return;
        }

        // Standard Timeline Playback
        if self.sync.is_playing() {
            let mut expected_playing = std::collections::HashSet::new();

            for m in &self.project.media_files {
                if m.on_timeline && m.media_type == MediaType::Audio {
                    if t >= m.timeline_offset && t < m.timeline_offset + m.duration {
                        expected_playing.insert(m.id.clone());
                        if !self.audio_player.is_playing(&m.id) {
                            let play_pos = m.trim_start + (t - m.timeline_offset) * m.playback_speed;
                            self.audio_player.play(&m.id, PathBuf::from(&m.path), play_pos, m.volume);
                            self.audio_player.set_speed(&m.id, m.playback_speed as f32);
                        } else {
                            self.audio_player.set_volume(&m.id, m.volume);
                            self.audio_player.set_speed(&m.id, m.playback_speed as f32);
                        }
                    }
                }
            }

            let active_ids: Vec<String> = self.audio_player.sinks.keys().cloned().collect();
            for id in active_ids {
                if !expected_playing.contains(&id) && id != "preview" {
                    self.audio_player.stop(&id);
                }
            }
        } else {
            self.audio_player.stop_all();
        }
    }

    pub fn toggle_play(&mut self) {
        if self.sync.is_playing() {
            self.preview_playing = false;
            self.audio_player.stop("preview");
        }
        
        self.sync.toggle_play_pause();
        if !self.sync.is_playing() {
            self.audio_player.stop_all();
        }
    }

    pub fn skip(&mut self, delta: f64) {
        self.sync.skip(delta);
        self.audio_player.stop_all();
    }

    pub fn seek_to(&mut self, t: f64) {
        self.sync.seek(t);
        self.audio_player.stop_all();
    }

    pub fn current_time(&self) -> f64  { self.sync.current_time }
    pub fn is_playing(&self) -> bool   { self.sync.is_playing() }
}