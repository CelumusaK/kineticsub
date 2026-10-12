use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use std::panic;
use std::collections::HashMap;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    pub sinks: HashMap<String, Sink>,
    pub active_paths: HashMap<String, PathBuf>,
    pub has_error: bool,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                log::warn!("AudioPlayer: could not open output stream gracefully: {e}");
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle: handle,
            sinks: HashMap::new(),
            active_paths: HashMap::new(),
            has_error: false,
        }
    }

    pub fn play(&mut self, id: &str, path: impl Into<PathBuf>, time_secs: f64, volume: f32) {
        if self.has_error { return; }
        
        let path = path.into();
        
        // If already playing this id, just update volume
        if self.sinks.contains_key(id) {
            if let Some(sink) = self.sinks.get(id) {
                sink.set_volume(volume);
            }
            return;
        }

        let handle = match &self.stream_handle {
            Some(h) => h.clone(),
            None => return, 
        };

        if !path.exists() {
            return;
        }

        let path_clone = path.clone();
        let decoder_result = panic::catch_unwind(|| {
            let f = File::open(&path_clone).map_err(|e| e.to_string())?;
            Decoder::new(BufReader::new(f)).map_err(|e| e.to_string())
        });

        let decoder = match decoder_result {
            Ok(Ok(d)) => d,
            Ok(Err(e)) => { 
                log::error!("AudioPlayer: cannot decode {:?}: {}", path, e); 
                return; 
            }
            Err(_) => { 
                log::error!("AudioPlayer: Symphonia internal panic caught parsing file {:?}", path); 
                return; 
            }
        };

        let seekable = decoder.skip_duration(Duration::from_secs_f64(time_secs.max(0.0)));

        match Sink::try_new(&handle) {
            Ok(sink) => {
                sink.set_volume(volume);
                sink.append(seekable);
                sink.play();
                self.sinks.insert(id.to_string(), sink);
                self.active_paths.insert(id.to_string(), path);
            }
            Err(e) => {
                log::error!("AudioPlayer: sink creation failed: {}", e);
            }
        }
    }

    pub fn set_volume(&mut self, id: &str, volume: f32) {
        if let Some(sink) = self.sinks.get(id) {
            sink.set_volume(volume);
        }
    }
    
    pub fn set_speed(&mut self, id: &str, speed: f32) {
        if let Some(sink) = self.sinks.get(id) {
            sink.set_speed(speed);
        }
    }

    pub fn stop(&mut self, id: &str) {
        if let Some(sink) = self.sinks.remove(id) {
            sink.stop();
        }
        self.active_paths.remove(id);
    }

    pub fn stop_all(&mut self) {
        for sink in self.sinks.values() {
            sink.stop();
        }
        self.sinks.clear();
        self.active_paths.clear();
    }
    
    pub fn pause_all(&mut self) {
        for sink in self.sinks.values() {
            sink.pause();
        }
    }
    
    pub fn resume_all(&mut self) {
        for sink in self.sinks.values() {
            sink.play();
        }
    }

    pub fn is_playing(&self, id: &str) -> bool {
        self.sinks.contains_key(id)
    }

    pub fn load(&mut self, _path: impl Into<PathBuf>) {
        self.stop_all();
    }

    // Compatibility method
    pub fn play_from(&mut self, path: impl Into<PathBuf>, time_secs: f64) {
        self.stop_all();
        self.play("preview", path, time_secs, 1.0);
    }
}

impl Default for AudioPlayer {
    fn default() -> Self { Self::new() }
}