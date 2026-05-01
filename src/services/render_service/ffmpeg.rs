use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::io::{BufRead, BufReader};
use std::fs::File;
use crate::models::types::{Project, RenderMode};
use super::{RenderMessage, ass_generator::generate_ass_baked, helpers::color_to_ffmpeg_hex};

pub fn run_render(
    project: Project, out_path: PathBuf, mode: RenderMode,
    include_audio: bool, transparent_bg: bool, tx: Sender<RenderMessage>
) {
    let fps = project.fps as f64;
    let res_w = project.resolution.0;
    let res_h = project.resolution.1;

    let _ = tx.send(RenderMessage::Progress(0.02, "Baking frame-by-frame animations...".into()));
    
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
    
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y"); 

    let bg_media = project.media_files.iter().find(|m| m.on_timeline && m.is_video_track);
    let audio_media = project.media_files.iter().find(|m| m.on_timeline && !m.is_video_track);
    
    let is_transparent_seq = mode == RenderMode::ImageSequence && transparent_bg;

    if is_transparent_seq {
        cmd.arg("-f").arg("lavfi")
           .arg("-i").arg(format!("color=c=black@0.0:s={}x{}:r={}:d={:.3}", res_w, res_h, fps, project.duration));
    } else {
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

    let mut has_audio = false;
    if mode == RenderMode::Video && include_audio {
        if let Some(audio) = audio_media {
            cmd.arg("-i").arg(&audio.path);
            has_audio = true;
        }
    }

    cmd.current_dir(&temp_dir);
    if is_transparent_seq {
        cmd.arg("-vf").arg("format=rgba,ass=subs.ass");
    } else {
        cmd.arg("-vf").arg("ass=subs.ass");
    }

    if mode == RenderMode::ImageSequence {
        cmd.arg("-c:v").arg("png");
        if is_transparent_seq {
            cmd.arg("-pix_fmt").arg("rgba");
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
                            let _ = tx.send(RenderMessage::Progress(0.05 + p * 0.45, format!("Rendering Text: {}%", (p*100.0) as u32)));
                        }
                    }
                }
            } else {
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