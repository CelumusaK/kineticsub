use std::time::Instant;

/// Drives playback timing independently from the render loop.
pub struct SyncEngine {
    state: PlaybackState,
    last_tick: Instant,
    pub current_time: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

impl SyncEngine {
    pub fn new(duration: f64) -> Self {
        Self {
            state: PlaybackState::Stopped,
            last_tick: Instant::now(),
            current_time: 0.0,
            duration,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
        self.last_tick = Instant::now();
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_time = 0.0;
    }

    pub fn toggle_play_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            _ => self.play(),
        }
    }

    pub fn seek(&mut self, time: f64) {
        self.current_time = time.clamp(0.0, self.duration);
        self.last_tick = Instant::now();
    }

    pub fn skip(&mut self, delta: f64) {
        self.seek(self.current_time + delta);
    }

    /// Call once per frame; advances internal clock and returns elapsed seconds.
    pub fn tick(&mut self) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;

        if self.state == PlaybackState::Playing {
            self.current_time += dt;
            if self.current_time >= self.duration {
                self.current_time = self.duration;
                self.pause();
            }
        }
        dt
    }

    /// Progress 0..1 through the total duration.
    pub fn progress(&self) -> f64 {
        if self.duration > 0.0 {
            self.current_time / self.duration
        } else {
            0.0
        }
    }
}