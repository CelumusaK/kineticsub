use super::{EditorViewModel, KeyframeMode};
use crate::models::types::animation::{Keyframe, Easing};
use crate::models::types::subtitle::Subtitle;
use crate::models::types::project::MediaFolder;

impl EditorViewModel {
    pub fn move_subtitle_idx(&mut self, index: usize, delta_secs: f64) {
        let sub = match self.project.subtitles.get(index) {
            Some(s) => s.clone(),
            None => return,
        };

        let target_start = (sub.timeline_start + delta_secs).max(0.0);

        if let Some(sub_mut) = self.project.subtitles.get_mut(index) {
            let dur = sub_mut.duration();
            sub_mut.timeline_start = target_start;
            sub_mut.timeline_end = target_start + dur;
        }
        
        self.resolve_all_overlaps();
        self.mark_modified();
        self.update_duration();
    }

    pub fn resize_subtitle_clip(&mut self, id: &str, mode: u8, dt: f64) {
        if dt == 0.0 || (mode != 1 && mode != 2) { return; }

        let sub = match self.project.subtitles.iter().find(|s| s.id == id) {
            Some(s) => s.clone(),
            None => return,
        };

        let group_id = sub.group_id.clone();

        let mut left_bound = 0.0_f64;
        let mut right_bound = f64::INFINITY;

        for other in &self.project.subtitles {
            if other.id != sub.id {
                if other.timeline_end <= sub.timeline_start + 0.001 {
                    if other.timeline_end > left_bound { left_bound = other.timeline_end; }
                }
                if other.timeline_start >= sub.timeline_end - 0.001 {
                    if other.timeline_start < right_bound { right_bound = other.timeline_start; }
                }
            }
        }

        let effective_dt = if mode == 1 {
            let min_dt = left_bound - sub.timeline_start;
            let max_dt = sub.duration() - 0.1;
            dt.clamp(min_dt, max_dt)
        } else {
            let min_dt = 0.1 - sub.duration();
            let max_dt = right_bound - sub.timeline_end;
            dt.clamp(min_dt, max_dt)
        };

        if effective_dt.abs() < 1e-6 { return; }

        if let Some(s) = self.project.subtitles.iter_mut().find(|s| s.id == id) {
            if mode == 1 {
                s.timeline_start += effective_dt;
            } else {
                s.timeline_end += effective_dt;
            }
        }

        if let Some(ref gid) = group_id {
            for other in self.project.subtitles.iter_mut() {
                if other.id != sub.id && other.group_id.as_deref() == Some(gid) {
                    if mode == 1 {
                        other.timeline_start += effective_dt;
                    } else {
                        other.timeline_end += effective_dt;
                    }
                }
            }
            for m in self.project.media_files.iter_mut() {
                if m.group_id.as_deref() == Some(gid) {
                    let speed = m.playback_speed.max(0.001);
                    if mode == 1 {
                        m.timeline_offset += effective_dt;
                        m.duration = (m.duration - effective_dt).max(0.1);
                        m.trim_start = (m.trim_start + effective_dt * speed).max(0.0);
                    } else {
                        m.duration = (m.duration + effective_dt).max(0.1);
                    }
                }
            }
        }

        self.mark_modified();
        self.update_duration();
    }

    pub fn move_selected_items(&mut self, delta_secs: f64) {
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) && sub.media_id.is_none() { 
                sub.timeline_start = (sub.timeline_start + delta_secs).max(0.0);
                sub.timeline_end   = (sub.timeline_end   + delta_secs).max(sub.timeline_start + 0.05);
            }
        }
        
        for m in self.project.media_files.iter_mut() {
            if ids.contains(&m.id) {
                m.timeline_offset = (m.timeline_offset + delta_secs).max(0.0);
            }
        }
        
        let selected_media_ids: Vec<String> = self.project.media_files.iter().filter(|m| ids.contains(&m.id)).map(|m| m.id.clone()).collect();
        for sub in self.project.subtitles.iter_mut() {
            if let Some(m_id) = &sub.media_id {
                if selected_media_ids.contains(m_id) && !ids.contains(&sub.id) {
                    sub.timeline_start = (sub.timeline_start + delta_secs).max(0.0);
                    sub.timeline_end   = (sub.timeline_end   + delta_secs).max(sub.timeline_start + 0.05);
                }
            }
        }

        self.resolve_all_overlaps();
        self.mark_modified();
        self.update_duration();
    }

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
        self.snapshot();
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
        if self.selected_id.as_deref() == Some(id) { 
            self.selected_id = None; 
            self.selected_keyframe_ids.clear();
        }
        self.selected_ids.remove(id);
        self.update_duration();
        self.snapshot();
    }
    
    pub fn delete_selected(&mut self) {
        let timeline_only = self.hovered_panel == crate::viewmodels::editor_vm::HoverPanel::Timeline;
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        for id in &ids {
            if timeline_only {
                if let Some(m) = self.project.media_files.iter_mut().find(|m| &m.id == id) {
                    m.on_timeline = false;
                }
                self.project.subtitles.retain(|s| &s.id != id); 
                self.project.transitions.retain(|t| &t.id != id);
            } else {
                self.project.media_files.retain(|m| &m.id != id);
                self.project.subtitles.retain(|s| &s.id != id);
                self.project.transitions.retain(|t| &t.id != id);
            }
        }
        if let Some(sel) = &self.selected_id.clone() {
            if ids.contains(sel) { 
                self.selected_id = None; 
                self.selected_keyframe_ids.clear();
            }
        }
        self.selected_ids.clear();
        
        self.update_duration();
        self.snapshot();
    }

    pub fn split_at_playhead(&mut self) {
        let t = self.sync.current_time;
        let mut ids_to_split: std::collections::HashSet<String> = self.selected_ids.iter().cloned().collect();

        if ids_to_split.is_empty() {
            for m in &self.project.media_files {
                if m.on_timeline && t > m.timeline_offset + 0.05 && t < m.timeline_offset + m.duration - 0.05 {
                    ids_to_split.insert(m.id.clone());
                }
            }
            for s in &self.project.subtitles {
                if t > s.timeline_start + 0.05 && t < s.timeline_end - 0.05 {
                    ids_to_split.insert(s.id.clone());
                }
            }
        }

        let touched_groups: std::collections::HashSet<String> = self.project.media_files.iter()
            .filter(|m| ids_to_split.contains(&m.id))
            .filter_map(|m| m.group_id.clone())
            .chain(
                self.project.subtitles.iter()
                    .filter(|s| ids_to_split.contains(&s.id))
                    .filter_map(|s| s.group_id.clone())
            )
            .collect();

        for m in &self.project.media_files {
            if let Some(ref gid) = m.group_id {
                if touched_groups.contains(gid) {
                    ids_to_split.insert(m.id.clone());
                }
            }
        }
        for s in &self.project.subtitles {
            if let Some(ref gid) = s.group_id {
                if touched_groups.contains(gid) {
                    ids_to_split.insert(s.id.clone());
                }
            }
        }

        let mut group_mapping: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut new_subs = Vec::new();
        let mut new_media = Vec::new();

        let mut created_folders: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for m in &mut self.project.media_files {
            if m.on_timeline && ids_to_split.contains(&m.id) {
                if t > m.timeline_offset + 0.05 && t < m.timeline_offset + m.duration - 0.05 {
                    let cut_offset = t - m.timeline_offset;
                    let speed = m.playback_speed.max(0.001);
                    let original_dur = m.duration;

                    let root_name = m.name
                        .replace(" (Cut)", "")
                        .replace(" (Audio)", "")
                        .trim()
                        .to_string();

                    let folder_id = if let Some(ref fid) = m.folder_id {
                        fid.clone()
                    } else if let Some(fid) = created_folders.get(&root_name) {
                        fid.clone()
                    } else if let Some(existing_f) = self.project.folders.iter().find(|f| f.name == format!("Cuts: {}", root_name)) {
                        let fid = existing_f.id.clone();
                        created_folders.insert(root_name.clone(), fid.clone());
                        fid
                    } else {
                        let fid = format!("folder_{}", uuid::Uuid::new_v4());
                        self.project.folders.push(MediaFolder {
                            id: fid.clone(),
                            name: format!("Cuts: {}", root_name),
                            is_open: true,
                        });
                        created_folders.insert(root_name.clone(), fid.clone());
                        fid
                    };

                    m.folder_id = Some(folder_id.clone());

                    let mut m2 = m.clone();
                    m2.id = format!("media_{}", uuid::Uuid::new_v4());
                    m2.timeline_offset = t;
                    m2.duration = original_dur - cut_offset;
                    m2.trim_start = m.trim_start + cut_offset * speed;
                    m2.folder_id = Some(folder_id);

                    let original_cache_dir = m.preview_dir.clone().unwrap_or_else(|| {
                        std::env::temp_dir().join("kineticsub_cache").join(&m.id).to_string_lossy().to_string()
                    });
                    m.preview_dir = Some(original_cache_dir.clone());
                    m2.preview_dir = Some(original_cache_dir);
                    m2.preview_ready = m.preview_ready;

                    m2.keyframes = m.keyframes.iter()
                        .filter(|k| k.time_offset >= cut_offset)
                        .map(|k| {
                            let mut k2 = k.clone();
                            k2.time_offset -= cut_offset;
                            k2
                        })
                        .collect();

                    m.keyframes.retain(|k| k.time_offset < cut_offset);
                    m.duration = cut_offset;

                    if let Some(ref gid) = m.group_id {
                        let new_gid = group_mapping.entry(gid.clone()).or_insert_with(|| format!("group_{}", uuid::Uuid::new_v4())).clone();
                        m2.group_id = Some(new_gid);
                    }

                    new_media.push(m2);
                }
            }
        }

        let mut next_id = self.next_id;
        for sub in &mut self.project.subtitles {
            if ids_to_split.contains(&sub.id) {
                if t > sub.timeline_start + 0.05 && t < sub.timeline_end - 0.05 {
                    let mut sub2 = sub.clone();
                    sub2.id = format!("sub_{}", next_id);
                    next_id += 1;
                    sub2.timeline_start = t;

                    let cut_offset = t - sub.timeline_start;
                    sub2.keyframes = sub.keyframes.iter()
                        .filter(|k| k.time_offset >= cut_offset)
                        .map(|k| {
                            let mut k2 = k.clone();
                            k2.time_offset -= cut_offset;
                            k2
                        })
                        .collect();
                    sub.keyframes.retain(|k| k.time_offset < cut_offset);

                    sub.timeline_end = t;

                    if let Some(ref gid) = sub.group_id {
                        let new_gid = group_mapping.entry(gid.clone()).or_insert_with(|| format!("group_{}", uuid::Uuid::new_v4())).clone();
                        sub2.group_id = Some(new_gid);
                    }

                    new_subs.push(sub2);
                }
            }
        }
        self.next_id = next_id;

        if !new_media.is_empty() || !new_subs.is_empty() {
            self.project.media_files.extend(new_media);
            self.project.subtitles.extend(new_subs);
            self.snapshot();
            self.update_duration();
        }
    }

    pub fn select_subtitle(&mut self, id: Option<String>) {
        if self.selected_id != id {
            self.selected_keyframe_ids.clear();
        }
        self.selected_id = id.clone();
        self.selected_ids.clear();
        if let Some(id) = id { self.selected_ids.insert(id); }
    }

    pub fn toggle_select(&mut self, id: &str) {
        if self.selected_ids.contains(id) {
            self.selected_ids.remove(id);
            if self.selected_id.as_deref() == Some(id) {
                self.selected_id = self.selected_ids.iter().next().cloned();
                self.selected_keyframe_ids.clear();
            }
        } else {
            self.selected_ids.insert(id.to_string());
            if self.selected_id.is_none() { 
                self.selected_id = Some(id.to_string()); 
                self.selected_keyframe_ids.clear();
            }
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

    pub fn selected_media(&self) -> Option<&crate::models::types::MediaFile> {
        self.selected_id.as_ref()
            .and_then(|id| self.project.media_files.iter().find(|m| &m.id == id))
    }
    
    pub fn selected_media_mut(&mut self) -> Option<&mut crate::models::types::MediaFile> {
        self.selected_id.clone()
            .and_then(|id| self.project.media_files.iter_mut().find(|m| m.id == id))
    }

    pub fn active_subtitle(&self) -> Option<&Subtitle> {
        let t = self.sync.current_time;
        if let Some(sel) = self.selected_subtitle() {
            if t >= sel.timeline_start && t < sel.timeline_end { return Some(sel); }
        }
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

    pub fn write_keyframe_now(&mut self) {
        let current_t = self.sync.current_time;
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) {
                let local_time = current_t - sub.timeline_start;
                if local_time < 0.0 || local_time > sub.duration() { continue; }

                sub.keyframes.retain(|k| (k.time_offset - local_time).abs() >= 0.02);

                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_time,
                    x: sub.x, y: sub.y, scale: sub.scale,
                    rotation: sub.rotation, opacity: sub.opacity,
                    skew_x: sub.skew_x, skew_y: sub.skew_y,
                    yaw: sub.yaw, pitch: sub.pitch,
                    path_progress: sub.path_progress,
                    mask_center: sub.mask_center,
                    mask_size: sub.mask_size,
                    mask_rotation: sub.mask_rotation,
                    mask_feather: sub.mask_feather,
                    easing: Easing::EaseOut,
                };
                sub.keyframes.push(kf);
                sub.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        }
        
        for m in self.project.media_files.iter_mut() {
            if ids.contains(&m.id) {
                let local_time = current_t - m.timeline_offset;
                if local_time < 0.0 || local_time > m.duration { continue; }

                m.keyframes.retain(|k| (k.time_offset - local_time).abs() >= 0.02);
                let kf = Keyframe {
                    id: format!("kf_{}", uuid::Uuid::new_v4()),
                    time_offset: local_time,
                    x: m.x, y: m.y, scale: m.scale,
                    rotation: m.rotation, opacity: m.opacity,
                    skew_x: 0.0, skew_y: 0.0, yaw: 0.0, pitch: 0.0, path_progress: 0.0,
                    mask_center: [0.0, 0.0], mask_size: [0.0, 0.0], mask_rotation: 0.0, mask_feather: 0.0,
                    easing: Easing::EaseOut,
                };
                m.keyframes.push(kf);
                m.keyframes.sort_by(|a, b| a.time_offset.partial_cmp(&b.time_offset).unwrap());
            }
        }
        self.snapshot();
    }

    pub fn clear_keyframes(&mut self) {
        let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
        for sub in self.project.subtitles.iter_mut() {
            if ids.contains(&sub.id) { sub.keyframes.clear(); }
        }
        for m in self.project.media_files.iter_mut() {
            if ids.contains(&m.id) { m.keyframes.clear(); }
        }
        self.selected_keyframe_ids.clear();
        self.snapshot();
    }

    pub fn maybe_autorecord_keyframe(&mut self) {
        if self.keyframe_mode == KeyframeMode::Record {
            self.write_keyframe_now();
        }
    }
}