use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::io::{BufRead, BufReader};
use std::fs::File;
use crate::models::types::{Project, Subtitle, InterpolatedState, RenderMode};

pub enum RenderMessage {
    Progress(f32, String),
    Done,
    Error(String),
}

pub fn run_render(
    project: Project,
    out_path: PathBuf,
    mode: RenderMode,
    include_audio: bool,
    transparent_bg: bool,
    tx: Sender<RenderMessage>
) {
    let fps = project.fps as f64;
    let res_w = project.resolution.0;
    let res_h = project.resolution.1;

    let _ = tx.send(RenderMessage::Progress(0.02, "Baking frame-by-frame animations...".into()));
    
    // Step 1: Generate Frame-Baked ASS file
    let ass_content = generate_ass_baked(&project, fps, res_w, res_h);
    
    let temp_dir = std::env::temp_dir().join("kineticsub_render");
    let _ = std::fs::remove_dir_all(&temp_dir); 
    let _ = std::fs::create_dir_all(&temp_dir);
    let ass_path = temp_dir.join("subs.ass");
    
    if let Err(e) = std::fs::write(&ass_path, ass_content) {
        let _ = tx.send(RenderMessage::Error(format!("Failed to write ASS file: {}", e)));
        return;
    }

    let _ = tx.send(RenderMessage::Progress(0.05, "Rendering frames...".into()));
    
    // Step 2: 1-Pass FFMPEG Magic!
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y"); 

    // Find active Background & Audio tracks
    let bg_media = project.media_files.iter().find(|m| m.on_timeline && m.is_video_track);
    let audio_media = project.media_files.iter().find(|m| m.on_timeline && !m.is_video_track);
    
    let is_transparent_seq = mode == RenderMode::ImageSequence && transparent_bg;

    // --- SETUP INPUTS ---
    if is_transparent_seq {
        // Pure Transparency (Disregards Background) using standard safe syntax
        cmd.arg("-f").arg("lavfi")
           .arg("-i").arg(format!("color=c=black@0.0:s={}x{}:r={}:d={:.3}", res_w, res_h, fps, project.duration));
    } else {
        // Use Background Track (or Solid Black if none exists)
        if let Some(bg) = bg_media {
            if let Some(color) = bg.color {
                let hex = color_to_ffmpeg_hex(&color);
                cmd.arg("-f").arg("lavfi")
                   .arg("-i").arg(format!("color=c={}:s={}x{}:r={}:d={:.3}", hex, res_w, res_h, fps, project.duration));
            } else {
                cmd.arg("-i").arg(&bg.path);
            }
        } else {
            cmd.arg("-f").arg("lavfi")
               .arg("-i").arg(format!("color=c=black:s={}x{}:r={}:d={:.3}", res_w, res_h, fps, project.duration));
        }
    }

    // Audio Input (only for Video)
    let mut has_audio = false;
    if mode == RenderMode::Video && include_audio {
        if let Some(audio) = audio_media {
            cmd.arg("-i").arg(&audio.path);
            has_audio = true;
        }
    }

    // --- SETUP FILTERS ---
    cmd.current_dir(&temp_dir);
    if is_transparent_seq {
        // Convert stream to RGBA before applying subtitles to maintain the alpha channel
        cmd.arg("-vf").arg("format=rgba,ass=subs.ass");
    } else {
        cmd.arg("-vf").arg("ass=subs.ass");
    }

    // --- SETUP OUTPUTS ---
    if mode == RenderMode::ImageSequence {
        cmd.arg("-c:v").arg("png");
        if is_transparent_seq {
            cmd.arg("-pix_fmt").arg("rgba"); // CRITICAL for transparent PNGs!
        }
        cmd.arg("frame_%04d.png");
    } else {
        if has_audio {
            cmd.arg("-map").arg("0:v").arg("-map").arg("1:a");
            cmd.arg("-c:a").arg("aac").arg("-b:a").arg("192k");
        } else {
            cmd.arg("-an");
        }
        cmd.arg("-c:v").arg("libx264").arg("-preset").arg("fast").arg("-crf").arg("22");
        cmd.arg(&out_path);
    }
        
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());
    
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(RenderMessage::Error(format!("Failed to start FFMPEG: {}", e)));
            return;
        }
    };
    
    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);
    let mut last_error = String::new();
    
    for line in reader.lines() {
        if let Ok(l) = line {
            if let Some(time_idx) = l.find("time=") {
                let time_str = &l[time_idx + 5..];
                if time_str.len() >= 11 {
                    let ts = &time_str[..11];
                    let parts: Vec<&str> = ts.split(':').collect();
                    if parts.len() == 3 {
                        if let (Ok(h), Ok(m), Ok(s)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                            let current_sec = h * 3600.0 + m * 60.0 + s;
                            let p = (current_sec / project.duration).clamp(0.0, 1.0) as f32;
                            // Progress bar 5% to 50%
                            let _ = tx.send(RenderMessage::Progress(0.05 + p * 0.45, format!("Rendering Text: {}%", (p*100.0) as u32)));
                        }
                    }
                }
            } else {
                // Dynamically capture the actual FFMPEG error instead of the generic locked file text!
                let lower = l.to_lowercase();
                if lower.contains("error") || lower.contains("invalid") || lower.contains("unrecognized") {
                    last_error = l.clone();
                } else if last_error.is_empty() && !lower.starts_with("frame=") && l.trim().len() > 0 {
                    last_error = l.clone();
                }
            }
        }
    }
    
    let status = child.wait().unwrap();
    if !status.success() {
        let err_msg = if last_error.is_empty() {
            "FFMPEG crashed unexpectedly. Ensure ffmpeg is installed properly.".to_string()
        } else {
            format!("FFMPEG: {}", last_error)
        };
        let _ = tx.send(RenderMessage::Error(err_msg));
        return;
    }

    // Step 3: Zip Sequence
    if mode == RenderMode::ImageSequence {
        let _ = tx.send(RenderMessage::Progress(0.5, "Zipping sequence...".into()));
        
        let zip_file = match File::create(&out_path) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(RenderMessage::Error(format!("Failed to create Zip file: {}", e)));
                return;
            }
        };

        let mut zip = zip::ZipWriter::new(zip_file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        let files: Vec<_> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("png"))
            .collect();
            
        let total_files = files.len() as f32;

        for (idx, entry) in files.iter().enumerate() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            
            if zip.start_file(name, options).is_ok() {
                if let Ok(mut f) = File::open(&path) {
                    let _ = std::io::copy(&mut f, &mut zip);
                }
            }
            
            if idx % 10 == 0 {
                let pct = 0.5 + (idx as f32 / total_files) * 0.48;
                let _ = tx.send(RenderMessage::Progress(pct, format!("Zipping frame {}/{}", idx+1, total_files)));
            }
        }
        let _ = zip.finish();
    }
    
    let _ = tx.send(RenderMessage::Done);
}

// ── Helpers ───────────────────────────────────────────────────

fn color_to_ffmpeg_hex(c: &[f32; 4]) -> String {
    let r = (c[0] * 255.0).clamp(0.0, 255.0) as u8;
    let g = (c[1] * 255.0).clamp(0.0, 255.0) as u8;
    let b = (c[2] * 255.0).clamp(0.0, 255.0) as u8;
    format!("0x{:02X}{:02X}{:02X}", r, g, b)
}

fn format_time(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as f64;
    format!("{}:{:02}:{:05.2}", h, m, s)
}

fn color_to_ass(c: &[f32; 4]) -> String {
    let r = (c[0] * 255.0).clamp(0.0, 255.0) as u8;
    let g = (c[1] * 255.0).clamp(0.0, 255.0) as u8;
    let b = (c[2] * 255.0).clamp(0.0, 255.0) as u8;
    format!("&H00{:02X}{:02X}{:02X}&", b, g, r)
}

fn alpha_to_ass(opacity: f32) -> String {
    let a = (255.0 - (opacity * 255.0).clamp(0.0, 255.0)) as u8;
    format!("&H{:02X}&", a)
}

pub fn generate_ass_baked(project: &Project, fps: f64, res_w: u32, res_h: u32) -> String {
    let mut out = String::new();
    out.push_str("[Script Info]\n");
    out.push_str("ScriptType: v4.00+\n");
    out.push_str(&format!("PlayResX: {}\n", res_w));
    out.push_str(&format!("PlayResY: {}\n", res_h));
    out.push_str("WrapStyle: 0\n");
    out.push_str("ScaledBorderAndShadow: yes\n\n");
    
    out.push_str("[V4+ Styles]\n");
    out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    out.push_str("Style: Default,Arial,36,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,5,0,0,0,1\n\n");
    
    out.push_str("[Events]\n");
    out.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    
    let dt = 1.0 / fps;
    let total_frames = (project.duration * fps).ceil() as usize;

    for frame in 0..=total_frames {
        let t_start = frame as f64 * dt;
        let t_end   = t_start + dt + 0.01; // Tiny overlap ensures FFMPEG never drops a text frame

        for sub in &project.subtitles {
            if t_end > sub.timeline_start && t_start < sub.timeline_end {
                let state = sub.get_interpolated_state(t_start);
                
                if state.opacity > 0.01 {
                    if sub.motion_blur > 0.0 {
                        for b_step in 1..=3 {
                            let blur_t = t_start - (b_step as f64 * 0.02 * sub.motion_blur as f64);
                            if blur_t < sub.timeline_start { continue; }
                            
                            let mut b_state = sub.get_interpolated_state(blur_t);
                            b_state.opacity *= 0.25; 
                            
                            if b_state.opacity > 0.01 {
                                let b_tags = build_static_ass_tags(sub, &b_state, res_w, res_h);
                                let b_text = sub.text.replace("\n", "\\N");
                                out.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", 
                                    format_time(t_start), format_time(t_end), b_tags, b_text));
                            }
                        }
                    }

                    let tags = build_static_ass_tags(sub, &state, res_w, res_h);
                    let text = sub.text.replace("\n", "\\N");
                    out.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{{{}}}{}\n", 
                        format_time(t_start), format_time(t_end), tags, text));
                }
            }
        }
    }
    out
}

fn build_static_ass_tags(sub: &Subtitle, state: &InterpolatedState, res_w: u32, res_h: u32) -> String {
    let mut t = String::new();
    t.push_str("\\an5"); // Center alignment
    
    // Position relatively to user's selected resolution output
    let base_x = (res_w as f32) / 2.0 + state.x;
    let base_y = (res_h as f32) / 2.0 + state.y;
    t.push_str(&format!("\\pos({:.1},{:.1})", base_x, base_y));
    t.push_str(&format!("\\fs{:.1}", sub.font_size));
    
    t.push_str(&format!("\\fscx{:.1}\\fscy{:.1}", state.scale * 100.0, state.scale * 100.0));
    t.push_str(&format!("\\frz{:.1}", -state.rotation));
    t.push_str(&format!("\\alpha{}", alpha_to_ass(state.opacity)));

    if sub.bold { t.push_str("\\b1"); }
    if sub.italic { t.push_str("\\i1"); }
    t.push_str(&format!("\\c{}", color_to_ass(&sub.color)));
    
    if sub.stroke_enabled {
        t.push_str(&format!("\\bord{:.1}", sub.stroke_width * state.scale));
        t.push_str(&format!("\\3c{}", color_to_ass(&sub.stroke_color)));
    } else {
        t.push_str("\\bord0");
    }
    
    if sub.shadow_enabled {
        t.push_str(&format!("\\shad{:.1}", sub.shadow_offset[0].abs().max(sub.shadow_offset[1].abs()) * state.scale));
        t.push_str(&format!("\\4c{}", color_to_ass(&sub.shadow_color)));
    } else {
        t.push_str("\\shad0");
    }

    t
}