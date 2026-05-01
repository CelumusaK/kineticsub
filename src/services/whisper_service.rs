use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone)]
pub struct RawWord {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

pub enum WhisperMessage {
    DownloadProgress(u64, u64),
    Transcribing,
    Done(Vec<RawWord>, f64), // ── NEW: f64 is the exact audio duration
    Error(String),
}

pub fn spawn_transcription(audio_path: String, tx: mpsc::Sender<WhisperMessage>) {
    thread::spawn(move || {
        let model_name = "base.en";
        let cache_dir = std::env::temp_dir().join("models");
        let _ = std::fs::create_dir_all(&cache_dir);
        let model_path = cache_dir.join(format!("{}.bin", model_name));

        if !model_path.exists() {
            let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin", model_name);
            match reqwest::blocking::get(&url) {
                Ok(mut res) => {
                    let total_size = res.content_length().unwrap_or(0);
                    let mut file = File::create(&model_path).unwrap();
                    let mut downloaded: u64 = 0;
                    let mut buffer = [0; 8192];
                    
                    while let Ok(n) = std::io::Read::read(&mut res, &mut buffer) {
                        if n == 0 { break; }
                        file.write_all(&buffer[..n]).unwrap();
                        downloaded += n as u64;
                        let _ = tx.send(WhisperMessage::DownloadProgress(downloaded, total_size));
                    }
                }
                Err(e) => {
                    let _ = tx.send(WhisperMessage::Error(format!("Download Failed: {}", e)));
                    return;
                }
            }
        }

        let _ = tx.send(WhisperMessage::Transcribing);

        match run_whisper(&audio_path, model_path.to_str().unwrap()) {
            Ok((words, duration)) => { let _ = tx.send(WhisperMessage::Done(words, duration)); }
            Err(e) => { let _ = tx.send(WhisperMessage::Error(e)); }
        }
    });
}

fn run_whisper(audio_path: &str, model_path: &str) -> Result<(Vec<RawWord>, f64), String> {
    let pcm = decode_audio_to_pcm(audio_path)?;
    let audio_duration = pcm.len() as f64 / 16000.0; // Exact length at 16kHz

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load model: {e}"))?;
    let mut state = ctx.create_state().map_err(|e| e.to_string())?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_token_timestamps(true);
    params.set_max_len(1);
    params.set_split_on_word(true);

    state.full(params, &pcm).map_err(|e| format!("Whisper inference failed: {e}"))?;

    let mut words = Vec::new();
    for segment in state.as_iter() {
        let text = segment.to_string().trim().to_string();
        if text.starts_with('[') || text.is_empty() { continue; }

        let start_secs = segment.start_timestamp() as f64 / 100.0;
        let end_secs = segment.end_timestamp() as f64 / 100.0;

        if end_secs > start_secs {
            words.push(RawWord { text, start: start_secs, end: end_secs });
        }
    }

    if words.is_empty() {
        return Err("No words found. Is the audio silent?".into());
    }

    Ok((words, audio_duration))
}

fn decode_audio_to_pcm(audio_path: &str) -> Result<Vec<f32>, String> {
    if audio_path.to_lowercase().ends_with(".wav") {
        decode_wav(audio_path)
    } else {
        decode_with_symphonia(audio_path)
    }
}

fn decode_wav(path: &str) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();

    let samples_i16: Vec<i16> = reader.samples::<i16>().map(|s| s.map_err(|e| e.to_string())).collect::<Result<Vec<_>, _>>()?;
    let mut samples_f32: Vec<f32> = samples_i16.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

    if spec.channels == 2 {
        samples_f32 = samples_f32.chunks(2).map(|c| (c[0] + c[1]) / 2.0).collect();
    }
    if spec.sample_rate != 16000 {
        samples_f32 = resample_linear(samples_f32, spec.sample_rate as f64, 16000.0);
    }
    Ok(samples_f32)
}

fn decode_with_symphonia(path: &str) -> Result<Vec<f32>, String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100) as f64;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| e.to_string())?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };
        if packet.track_id() != track_id { continue; }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(buf.samples());
    }

    let mono: Vec<f32> = if channels > 1 {
        all_samples.chunks(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect()
    } else {
        all_samples
    };

    Ok(if sample_rate != 16000.0 {
        resample_linear(mono, sample_rate, 16000.0)
    } else {
        mono
    })
}

fn resample_linear(samples: Vec<f32>, from_rate: f64, to_rate: f64) -> Vec<f32> {
    let ratio = from_rate / to_rate;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len).map(|i| {
        let src = i as f64 * ratio;
        let lo = src as usize;
        let hi = (lo + 1).min(samples.len() - 1);
        let t = (src - lo as f64) as f32;
        samples[lo] * (1.0 - t) + samples[hi] * t
    }).collect()
}