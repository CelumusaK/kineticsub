// File: src/viewmodels/editor_vm/project.rs
use super::EditorViewModel;
use crate::models::types::{Project, RenderMode};
use crate::services::render_service::{run_render, RenderMessage};
use std::sync::mpsc;

impl EditorViewModel {
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

    // ── Export / Render ───────────────────────────────────────────────────────────

    pub fn start_render(&mut self) {
        let mode = self.render_mode.clone();
        
        let path = match mode {
            RenderMode::Video => {
                rfd::FileDialog::new()
                    .add_filter("Video file", &["mp4", "mkv"])
                    .set_file_name("output.mp4")
                    .save_file()
            }
            RenderMode::ImageSequence => {
                rfd::FileDialog::new()
                    .add_filter("Zip Archive", &["zip"])
                    .set_file_name("image_sequence.zip")
                    .save_file()
            }
        };
        
        if let Some(out_path) = path {
            self.is_rendering = true;
            self.render_progress = 0.0;
            self.render_status = "Preparing to export...".into();
            
            let (tx, rx) = mpsc::channel();
            self.render_rx = Some(rx);
            
            let project = self.project.clone();
            let include_audio = self.render_include_audio;
            let transparent_bg = self.render_transparent_bg;
            
            std::thread::spawn(move || {
                run_render(project, out_path, mode, include_audio, transparent_bg, tx);
            });
        }
    }

    pub fn poll_render(&mut self) {
        let mut done_or_error = false;
        
        if let Some(rx) = &self.render_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    RenderMessage::Progress(p, status) => {
                        self.render_progress = p;
                        self.render_status = status;
                    }
                    RenderMessage::Done => {
                        self.render_progress = 1.0;
                        self.render_status = "Export Complete!".into();
                        done_or_error = true;
                        
                        std::thread::spawn(|| { std::thread::sleep(std::time::Duration::from_secs(4)); });
                    }
                    RenderMessage::Error(e) => {
                        self.render_status = format!("Error: {}", e);
                        done_or_error = true;
                    }
                }
            }
        }
        
        if done_or_error {
            self.render_rx = None;
        }
    }
}