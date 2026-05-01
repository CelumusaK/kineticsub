use super::EditorViewModel;

impl EditorViewModel {
    // ── Playback ──────────────────────────────────────────────────────────────
    pub fn tick(&mut self) {
        let dt = self.sync.tick();
        
        // Calculate the real-time playback/render FPS
        if dt > 0.0 {
            let instant_fps = 1.0 / dt;
            if self.current_fps == 0.0 {
                self.current_fps = instant_fps;
            } else {
                // Smooth the value slightly so the text doesn't flicker unreadably
                self.current_fps = self.current_fps * 0.9 + instant_fps * 0.1;
            }
        }

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
}