use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

/// Wraps rodio so the rest of the app can seek, play, pause, and stop
/// an audio file in sync with the timeline playhead.
pub struct AudioPlayer {
    _stream:        OutputStream,
    _stream_handle: OutputStreamHandle,
    sink:           Arc<Mutex<Option<Sink>>>,
    path:           Option<PathBuf>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self {
                _stream:        stream,
                _stream_handle: handle,
                sink:           Arc::new(Mutex::new(None)),
                path:           None,
            },
            Err(e) => {
                log::warn!("AudioPlayer: could not open output stream: {e}");
                let (stream, handle) =
                    OutputStream::try_default().unwrap_or_else(|_| {
                        panic!("No audio output device available")
                    });
                Self {
                    _stream:        stream,
                    _stream_handle: handle,
                    sink:           Arc::new(Mutex::new(None)),
                    path:           None,
                }
            }
        }
    }

    pub fn load(&mut self, path: impl Into<PathBuf>) {
        let new_path = path.into();
        if self.path.as_ref() != Some(&new_path) {
            self.path = Some(new_path);
            if let Ok(mut guard) = self.sink.lock() {
                *guard = None;
            }
        }
    }

    pub fn unload(&mut self) {
        self.path = None;
        if let Ok(mut guard) = self.sink.lock() {
            *guard = None;
        }
    }

    pub fn play_from(&mut self, time_secs: f64) {
        let path = match &self.path {
            Some(p) => p.clone(),
            None    => return,
        };

        if let Ok(mut guard) = self.sink.lock() {
            if let Some(s) = guard.take() {
                s.stop();
            }
        }

        let file = match File::open(&path) {
            Ok(f)  => f,
            Err(e) => { log::error!("AudioPlayer: cannot open {:?}: {}", path, e); return; }
        };

        let decoder = match Decoder::new(BufReader::new(file)) {
            Ok(d)  => d,
            Err(e) => { log::error!("AudioPlayer: cannot decode {:?}: {}", path, e); return; }
        };

        let seekable = decoder.skip_duration(Duration::from_secs_f64(time_secs.max(0.0)));

        let handle = self._stream_handle.clone();
        match Sink::try_new(&handle) {
            Ok(sink) => {
                sink.append(seekable);
                sink.play();
                if let Ok(mut guard) = self.sink.lock() {
                    *guard = Some(sink);
                }
            }
            Err(e) => log::error!("AudioPlayer: sink creation failed: {}", e),
        }
    }

    pub fn pause(&self) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(s) = &*guard { s.pause(); }
        }
    }

    pub fn resume(&self) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(s) = &*guard { s.play(); }
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.sink.lock() {
            if let Some(s) = guard.take() { s.stop(); }
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.lock().ok()
            .and_then(|g| g.as_ref().map(|s| s.is_paused()))
            .unwrap_or(true)
    }

    pub fn is_playing(&self) -> bool {
        self.sink.lock().ok()
            .and_then(|g| g.as_ref().map(|s| !s.is_paused() && !s.empty()))
            .unwrap_or(false)
    }
}

impl Default for AudioPlayer {
    fn default() -> Self { Self::new() }
}