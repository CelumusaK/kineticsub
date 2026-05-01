pub fn color_to_ffmpeg_hex(c: &[f32; 4]) -> String {
    let r = (c[0] * 255.0).clamp(0.0, 255.0) as u8;
    let g = (c[1] * 255.0).clamp(0.0, 255.0) as u8;
    let b = (c[2] * 255.0).clamp(0.0, 255.0) as u8;
    format!("0x{:02X}{:02X}{:02X}", r, g, b)
}

pub fn format_time(secs: f64) -> String {
    let h = (secs / 3600.0) as u32;
    let m = ((secs % 3600.0) / 60.0) as u32;
    let s = (secs % 60.0) as f64;
    format!("{}:{:02}:{:05.2}", h, m, s)
}

pub fn color_to_ass(c: &[f32; 4]) -> String {
    let r = (c[0] * 255.0).clamp(0.0, 255.0) as u8;
    let g = (c[1] * 255.0).clamp(0.0, 255.0) as u8;
    let b = (c[2] * 255.0).clamp(0.0, 255.0) as u8;
    format!("&H00{:02X}{:02X}{:02X}&", b, g, r)
}

pub fn alpha_to_ass(opacity: f32) -> String {
    let a = (255.0 - (opacity * 255.0).clamp(0.0, 255.0)) as u8;
    format!("&H{:02X}&", a)
}