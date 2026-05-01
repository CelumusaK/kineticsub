use super::{EditorViewModel, KeyframeMode};
use crate::models::types::{Subtitle, Keyframe, Easing};

impl EditorViewModel {
pub fn move_subtitle_idx(&mut self, index: usize, delta_secs: f64) {
if let Some(sub) = self.project.subtitles.get_mut(index) {
sub.timeline_start = (sub.timeline_start + delta_secs).max(0.0);
sub.timeline_end = (sub.timeline_end + delta_secs).max(sub.timeline_start + 0.05);
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

// ── Subtitle CRUD ─────────────────────────────────────────────────────────
pub fn next_id_str(&mut self) -> String {
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
        
        // Linear scan but breaks aggressively as soon as time overflows
        for s in &self.project.subtitles {
            if s.timeline_start > t { break; } 
            if t < s.timeline_end { return Some(s); }
        }
        None
    }

pub fn sort_subtitles(&mut self) {
    self.project.subtitles
        .sort_by(|a, b| a.timeline_start.partial_cmp(&b.timeline_start).unwrap());
}

// ── Keyframe helpers ──────────────────────────────────────────────────────

pub fn write_keyframe_now(&mut self) {
        let current_t = self.sync.current_time;
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                let local_time = current_t - sub.timeline_start;
                // Only place keyframe if playhead is currently over THIS specific subtitle block
                if local_time < 0.0 || local_time > sub.duration() { continue; }

                sub.keyframes.retain(|k| (k.time_offset - local_time).abs() >= 0.02);

                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_time,
                    x: sub.x, y: sub.y, scale: sub.scale,
                    rotation: sub.rotation, opacity: sub.opacity,
                    skew_x: sub.skew_x,
                    skew_y: sub.skew_y,
                    easing: Easing::EaseOut,
                };
                sub.keyframes.push(kf);
                sub.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        }
    }

    pub fn maybe_autorecord_keyframe(&mut self) {
        if self.keyframe_mode == KeyframeMode::Record {
            self.write_keyframe_now();
        }
    }

}