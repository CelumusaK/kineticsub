# 🎬 KineticSub-RS

A bespoke, GPU-accelerated non-linear video editor and kinetic typography workstation built in Rust.

![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)
![Render Engine](https://img.shields.io/badge/wgpu-Hardware%20Accelerated-blue.svg)
![UI](https://img.shields.io/badge/UI-egui-yellow.svg)
![AI](https://img.shields.io/badge/AI-Whisper.cpp-purple.svg)
![FFMPEG](https://img.shields.io/badge/Export-FFMPEG-green.svg)

---

## ⚡ Overview

**KineticSub-RS** is a personal desktop video editor built from scratch in Rust. Designed as a focused, lightweight alternative to heavyweight NLEs for dynamic subtitle animations, kinetic typography, and multi-track video editing, it pairs hardware-accelerated rendering with on-device local AI transcription.

Everything runs locally with zero cloud dependencies: speech recognition is powered by native Whisper models, rendering uses native `wgpu` pipelines, and final assembly is handled via high-performance FFMPEG encoding.

---

## ✨ Features

- **🎞️ Non-Linear Video Editing:**
  - Multi-track timeline supporting visual layers (video, image plates, solid colors) and audio tracks.
  - Interactive clip trimming, slip-editing, precision splitting (`K`), and multi-clip grouping.
  - Video transition blocks (Crossfades, Dip-to-Black, Dip-to-White, Wipes, and Slides).

- **🤖 Local AI Speech Recognition:**
  - Embedded Whisper engine with on-demand model downloading (`tiny`, `base`, `small`, `medium`, `large-v3`).
  - Automatic phrase and word-level time alignment directly onto the timeline tracks.

- **🎢 Kinetic Typography & Animation Engine:**
  - Full transform animability (Position, Scale, Rotation, Opacity, 3D Pitch/Yaw, Skew).
  - Visual Bezier graph editor with configurable cubic easing curves.
  - Deterministic physics and floor-bounce dynamics.
  - Code-driven motion expressions (`wiggle(freq, amp)`, `time * x`).
  - Motion path deformation (Circle, Star, and interactive on-canvas custom Bezier paths).
  - Masking engine (Rectangle, Circle, Split) with edge feathering and inversion.

- **🎨 Styling & Pro VFX:**
  - Glow and cinematic bloom engine.
  - Multi-layer outline strokes and soft drop shadows.
  - Glitch VFX with real-time RGB chromatic aberration and CRT scanline overlays.
  - Text deformers (Wave, Arc, Bulge, Flag) and gradient fills.
  - Karaoke-style word animation highlights and cascade fades.

- **🔊 Audio Engine & Playback:**
  - Resampled, multi-track audio playback synced to the visual playhead via `rodio` and `symphonia`.
  - Frame-accurate timeline scrubbing and playback speed compensation.

- **🚀 Flexible Export:**
  - Direct video muxing to MP4 / MKV with auto audio mixdown.
  - Transparent PNG image sequence export (bundled as a `.zip` archive) for DaVinci Resolve, Premiere Pro, and Final Cut overlay workflows.

---

## 🛠️ Tech Stack

- **GUI Framework:** `egui` (Immediate mode UI)
- **Graphics Pipeline:** `wgpu` (Native WebGPU backend)
- **Audio Decoding & Playback:** `symphonia`, `hound`, `rodio`
- **Transcription Engine:** `whisper-rs` (Rust bindings for `whisper.cpp`)
- **Video Encoding:** `FFMPEG` CLI subprocess pipeline
- **Project Format:** `.ksub` JSON project format

---

## 🚀 Getting Started

### Prerequisites

1. **Rust Toolchain:** Install Rust 1.80+ via [rustup.rs](https://rustup.rs/).
2. **C/C++ Compiler:** Required to compile the native `whisper.cpp` bindings (e.g. `build-essential` on Linux, Xcode command line tools on macOS, Visual Studio Build Tools on Windows).
3. **FFMPEG:** Ensure `ffmpeg` and `ffprobe` are installed and reachable in your system's `PATH`.

### Build & Run

Clone the repository and compile in release mode:

```bash
git clone https://github.com/CelumusaK/kineticsub.git
cd kineticsub
cargo run --release