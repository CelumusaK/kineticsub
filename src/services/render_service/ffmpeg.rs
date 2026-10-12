use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use crate::models::types::{MediaFile, MediaType, Project, RenderMode};
use super::{RenderMessage, ass_generator::generate_ass_baked, helpers::color_to_ffmpeg_hex};

const MIN_AUDIO_FILE_BYTES: u64 = 64;

struct AudioSource {
    path: PathBuf,
    timeline_offset: f64,
    volume: f32,
    trim_start: f64,
    playback_speed: f64,
}

fn hide_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

fn ffmpeg_cmd() -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin");
    hide_window(&mut cmd);
    cmd
}

fn is_usable_file(p: &Path) -> bool {
    std::fs::metadata(p).map(|m| m.is_file() && m.len() > MIN_AUDIO_FILE_BYTES).unwrap_or(false)
}

fn has_audio_stream(path: &Path) -> bool {
    let mut cmd = Command::new("ffprobe");
    cmd.args(["-v", "error", "-select_streams", "a:0", "-show_entries", "stream=index", "-of", "csv=p=0"]).arg(path);
    hide_window(&mut cmd);
    match cmd.output() {
        Ok(out) => !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        Err(_) => true,
    }
}

fn for_each_line<R: Read>(mut reader: R, mut f: impl FnMut(&str)) {
    let mut buf = [0u8; 4096];
    let mut line: Vec<u8> = Vec::new();
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for &b in &buf[..n] {
                    if b == b'\n' || b == b'\r' {
                        if !line.is_empty() {
                            f(&String::from_utf8_lossy(&line));
                            line.clear();
                        }
                    } else {
                        line.push(b);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    if !line.is_empty() { f(&String::from_utf8_lossy(&line)); }
}

fn parse_ffmpeg_time(s: &str) -> Option<f64> {
    let token = s.split_whitespace().next()?;
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 { return None; }
    let h = parts[0].parse::<f64>().ok()?;
    let m = parts[1].parse::<f64>().ok()?;
    let sec = parts[2].parse::<f64>().ok()?;
    Some((h * 3600.0 + m * 60.0 + sec).max(0.0))
}

fn resolve_audio_path(project: &Project, aud: &MediaFile) -> Option<PathBuf> {
    let primary = PathBuf::from(&aud.path);
    if is_usable_file(&primary) { return Some(primary); }

    let source_id = primary.parent()?.file_name()?.to_str()?;
    let source = project.media_files.iter().find(|m| m.id == source_id && m.media_type == MediaType::Video && m.color.is_none() && !m.path.is_empty())?;

    let src_path = PathBuf::from(&source.path);
    if is_usable_file(&src_path) && has_audio_stream(&src_path) {
        Some(src_path)
    } else { None }
}

fn collect_audio_sources(project: &Project) -> (Vec<AudioSource>, Vec<String>) {
    let mut sources = Vec::new();
    let mut skipped = Vec::new();

    for aud in project.media_files.iter().filter(|m| m.on_timeline && m.media_type == MediaType::Audio) {
        match resolve_audio_path(project, aud) {
            Some(path) => sources.push(AudioSource {
                path,
                timeline_offset: aud.timeline_offset,
                volume: aud.volume,
                trim_start: aud.trim_start,
                playback_speed: aud.playback_speed,
            }),
            None => skipped.push(aud.name.clone()),
        }
    }
    (sources, skipped)
}

fn mix_audio(sources: &[AudioSource], out_wav: &Path, use_limiter: bool) -> Result<(), String> {
    let n = sources.len();
    if n == 0 { return Err("no audio sources".into()); }

    let mut cmd = ffmpeg_cmd();
    cmd.arg("-hide_banner").arg("-loglevel").arg("error");
    for s in sources { cmd.arg("-i").arg(&s.path); }

    let mut parts: Vec<String> = Vec::new();
    let mut labels = String::new();

    for (i, s) in sources.iter().enumerate() {
        let label = if n == 1 { "aout".to_string() } else { format!("a{}", i) };
        
        let mut effective_trim = s.trim_start;
        let mut effective_offset = s.timeline_offset;
        if effective_offset < 0.0 {
            effective_trim += (-effective_offset) * s.playback_speed;
            effective_offset = 0.0;
        }

        let mut chain = format!("[{}:a:0]aformat=sample_fmts=fltp:sample_rates=44100:channel_layouts=stereo", i);
        if effective_trim > 0.0 {
            chain.push_str(&format!(",atrim=start={:.3}", effective_trim));
        }
        if s.playback_speed != 1.0 {
            chain.push_str(&format!(",atempo={:.3}", s.playback_speed));
        }
        chain.push_str(",asetpts=PTS-STARTPTS");
        chain.push_str(&format!(",volume={:.2}", s.volume));
        
        if effective_offset > 0.0 {
            chain.push_str(&format!(",adelay={0}|{0}", (effective_offset * 1000.0).round() as u64));
        }
        
        chain.push_str(&format!("[{}]", label));
        parts.push(chain);
        labels.push_str(&format!("[{}]", label));
    }

    if n > 1 {
        let mut mix = format!("{}amix=inputs={}:duration=longest:normalize=0", labels, n);
        if use_limiter { mix.push_str(",alimiter=limit=0.95:level=0"); }
        mix.push_str("[aout]");
        parts.push(mix);
    }

    let graph = parts.join(";");

    cmd.arg("-filter_complex").arg(&graph)
        .arg("-map").arg("[aout]")
        .arg("-c:a").arg("pcm_s16le")
        .arg("-ar").arg("44100")
        .arg("-ac").arg("2")
        .arg(out_wav);

    let output = cmd.output().map_err(|e| format!("Failed to start FFMPEG: {}", e))?;

    if output.status.success() && is_usable_file(out_wav) { return Ok(()); }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let msg = stderr.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("unknown error").to_string();
    Err(msg)
}

pub fn run_render(
    project: Project, out_path: PathBuf, mode: RenderMode,
    include_audio: bool, transparent_bg: bool, tx: Sender<RenderMessage>
) {
    let fps = project.fps as f64;
    let res_w = project.resolution.0;
    let res_h = project.resolution.1;

    // Shift the project clone natively to start exactly at the requested 'Export Start' marker 
    let mut render_proj = project.clone();
    let (exp_start, exp_end) = render_proj.export_range.unwrap_or((0.0, render_proj.duration));
    let exp_dur = (exp_end - exp_start).max(0.1);
    
    render_proj.duration = exp_dur;

    for m in &mut render_proj.media_files {
        m.timeline_offset -= exp_start;
    }
    for s in &mut render_proj.subtitles {
        s.timeline_start -= exp_start;
        s.timeline_end -= exp_start;
    }

    let _ = tx.send(RenderMessage::Progress(0.02, "Baking frame-by-frame animations...".into()));

    let ass_content = generate_ass_baked(&render_proj, fps, res_w, res_h);

    let temp_dir = std::env::temp_dir().join("kineticsub_render");
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::create_dir_all(&temp_dir);
    let ass_path = temp_dir.join("subs.ass");

    if let Err(e) = std::fs::write(&ass_path, ass_content) {
        let _ = tx.send(RenderMessage::Error(format!("Failed to write ASS file: {}", e)));
        return;
    }

    let mut mixed_audio: Option<PathBuf> = None;

    if mode == RenderMode::Video && include_audio {
        let _ = tx.send(RenderMessage::Progress(0.03, "Collecting audio tracks...".into()));
        let (sources, skipped) = collect_audio_sources(&render_proj);

        if sources.is_empty() {
            if !skipped.is_empty() {
                let _ = tx.send(RenderMessage::Error(format!("Audio file(s) for [{}] could not be found.", skipped.join(", "))));
                return;
            }
        } else {
            let _ = tx.send(RenderMessage::Progress(0.04, format!("Mixing {} audio track(s)...", sources.len())));
            let mix_path = temp_dir.join("mix.wav");
            
            let first = mix_audio(&sources, &mix_path, sources.len() > 1);
            let result = if first.is_err() && sources.len() > 1 { mix_audio(&sources, &mix_path, false) } else { first };

            match result {
                Ok(()) => mixed_audio = Some(mix_path),
                Err(e) => { let _ = tx.send(RenderMessage::Error(format!("Audio mixdown failed: {}", e))); return; }
            }
        }
    }

    let _ = tx.send(RenderMessage::Progress(0.05, "Rendering video...".into()));

    let mut cmd = ffmpeg_cmd();
    cmd.arg("-hide_banner");

    let mut visual_tracks: Vec<_> = render_proj.media_files.iter()
        .filter(|m| m.on_timeline && m.media_type != MediaType::Audio)
        .collect();
    visual_tracks.sort_by(|a, b| a.track_index.cmp(&b.track_index));
    let bg_media = visual_tracks.first().copied();

    let is_transparent_seq = mode == RenderMode::ImageSequence && transparent_bg;
    let bg_is_image = bg_media.map_or(false, |m| m.media_type == MediaType::Image && m.color.is_none());

    // Input 0: Visual layer 
    if is_transparent_seq {
        cmd.arg("-f").arg("lavfi").arg("-i").arg(format!("color=c=black@0.0:s={}x{}:r={}:d={:.3}", res_w, res_h, fps, exp_dur));
    } else if let Some(bg) = bg_media {
        if let Some(color) = bg.color {
            let hex = color_to_ffmpeg_hex(&color);
            cmd.arg("-f").arg("lavfi").arg("-i").arg(format!("color=c={}:s={}x{}:r={}:d={:.3}", hex, res_w, res_h, fps, exp_dur));
        } else {
            if bg.media_type == MediaType::Image {
                cmd.arg("-loop").arg("1");
            } else {
                let mut trim = bg.trim_start;
                if bg.timeline_offset < 0.0 {
                    trim += -bg.timeline_offset * bg.playback_speed;
                }
                if trim > 0.0 {
                    cmd.arg("-ss").arg(format!("{:.3}", trim));
                }
            }
            cmd.arg("-i").arg(&bg.path);
        }
    } else {
        cmd.arg("-f").arg("lavfi").arg("-i").arg(format!("color=c=black:s={}x{}:r={}:d={:.3}", res_w, res_h, fps, exp_dur));
    }

    // Input 1: Pre-mixed audio
    if let Some(mix) = &mixed_audio {
        cmd.arg("-i").arg(mix);
    }

    cmd.current_dir(&temp_dir);

    let mut vf = String::new();
    if let Some(bg) = bg_media {
        if bg.media_type != MediaType::Image && bg.playback_speed != 1.0 && bg.color.is_none() {
            vf.push_str(&format!("setpts={:.4}*PTS,", 1.0 / bg.playback_speed));
        }
    }

    if is_transparent_seq {
        vf.push_str("format=rgba,ass=subs.ass");
    } else if bg_is_image {
        vf.push_str(&format!("trim=duration={:.3},ass=subs.ass", exp_dur));
    } else {
        vf.push_str("ass=subs.ass");
    }
    
    cmd.arg("-vf").arg(vf);

    // Explicit Muxing mapping for MKV/MP4 Video 
    if mode == RenderMode::Video {
        if mixed_audio.is_some() {
            cmd.arg("-map").arg("0:v")
               .arg("-map").arg("1:a")
               .arg("-c:v").arg("libx264").arg("-preset").arg("fast").arg("-crf").arg("22").arg("-pix_fmt").arg("yuv420p")
               .arg("-c:a").arg("aac").arg("-b:a").arg("192k");
        } else {
            cmd.arg("-map").arg("0:v")
               .arg("-c:v").arg("libx264").arg("-preset").arg("fast").arg("-crf").arg("22").arg("-pix_fmt").arg("yuv420p")
               .arg("-an");
        }
        cmd.arg("-t").arg(format!("{:.3}", exp_dur));
        cmd.arg(&out_path);
    } else if mode == RenderMode::ImageSequence {
        cmd.arg("-map").arg("0:v").arg("-c:v").arg("png");
        if is_transparent_seq { cmd.arg("-pix_fmt").arg("rgba"); }
        cmd.arg("-t").arg(format!("{:.3}", exp_dur));
        cmd.arg("frame_%04d.png");
    }

    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(RenderMessage::Error(format!("Failed to start FFMPEG: {}", e)));
            return;
        }
    };

    let stderr = child.stderr.take().unwrap();
    let mut last_error = String::new();
    let mut tail: VecDeque<String> = VecDeque::new();

    let render_weight = if mode == RenderMode::ImageSequence { 0.45 } else { 0.95 };
    let total_duration = exp_dur.max(0.001);

    for_each_line(stderr, |l| {
        if let Some(time_idx) = l.find("time=") {
            if let Some(current_sec) = parse_ffmpeg_time(&l[time_idx + 5..]) {
                let p = (current_sec / total_duration).clamp(0.0, 1.0) as f32;
                let _ = tx.send(RenderMessage::Progress(0.05 + p * render_weight, format!("Rendering: {}%", (p * 100.0) as u32)));
            }
        } else {
            let lower = l.to_lowercase();
            if lower.contains("error") || lower.contains("invalid") || lower.contains("unrecognized") {
                last_error = l.to_string();
            } else if last_error.is_empty() && !lower.starts_with("frame=") && !l.trim().is_empty() {
                last_error = l.to_string();
            }
            tail.push_back(l.to_string());
            if tail.len() > 20 { tail.pop_front(); }
        }
    });

    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => { let _ = tx.send(RenderMessage::Error(format!("FFMPEG wait failed: {}", e))); return; }
    };

    if let Some(m) = &mixed_audio { let _ = std::fs::remove_file(m); }

    if !status.success() {
        let err_msg = if last_error.is_empty() { "FFMPEG crashed unexpectedly.".to_string() } else { format!("FFMPEG: {}", last_error) };
        let _ = tx.send(RenderMessage::Error(err_msg));
        return;
    }

    if mode == RenderMode::ImageSequence {
        let _ = tx.send(RenderMessage::Progress(0.5, "Zipping sequence...".into()));
        let zip_file = File::create(&out_path).unwrap();
        let mut zip = zip::ZipWriter::new(zip_file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        let mut files: Vec<_> = std::fs::read_dir(&temp_dir).unwrap().filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("png")).collect();
        files.sort_by_key(|e| e.file_name());

        let total_files = files.len() as f32;
        for (idx, entry) in files.iter().enumerate() {
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            if zip.start_file(name, options).is_ok() {
                if let Ok(mut f) = File::open(&path) { let _ = std::io::copy(&mut f, &mut zip); }
            }
            if idx % 10 == 0 {
                let pct = 0.5 + (idx as f32 / total_files) * 0.48;
                let _ = tx.send(RenderMessage::Progress(pct, format!("Zipping frame {}/{}", idx + 1, total_files)));
            }
        }
        let _ = zip.finish();
    }

    let _ = tx.send(RenderMessage::Done);
}