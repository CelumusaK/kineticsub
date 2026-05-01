pub mod helpers;
pub mod ass_generator;
pub mod ffmpeg;

// Re-export core items so the rest of the app doesn't need to change its imports
pub use ffmpeg::run_render;
pub use ass_generator::generate_ass_baked;

pub enum RenderMessage {
    Progress(f32, String),
    Done,
    Error(String),
}