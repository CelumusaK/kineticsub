🎬 KineticSub-RS

A High-Performance, GPU-Accelerated Kinetic Typography & Subtitle Editor

![alt text](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)


![alt text](https://img.shields.io/badge/wgpu-Render%20Engine-blue.svg)


![alt text](https://img.shields.io/badge/UI-egui-yellow.svg)


![alt text](https://img.shields.io/badge/AI-Whisper.cpp-purple.svg)

</div>
<br>

    💡 Project Notice:
    KineticSub-RS is an exploratory side project developed with the assistance of AI. It was built to experiment with native GUI rendering, audio processing, and local AI transcription in Rust.

    Currently, this application is a text-only engine. It focuses entirely on subtitle generation, kinetic text animation, and typography previewing, and does not yet handle video rendering or composition.

⚡ Overview

KineticSub-RS is a desktop application designed to bring advanced, non-linear editing paradigms to subtitle creation and kinetic typography. Built from the ground up in Rust, it leverages wgpu for hardware-accelerated rendering and egui for a highly responsive, immediate-mode user interface.

Whether you are auto-transcribing audio using local AI models or hand-crafting intricate text animations with custom easing curves, KineticSub-RS provides a fluid, real-time workspace.
✨ Key Features

    🤖 Local AI Transcription: Integrated whisper-rs allows for automatic, on-device audio transcription. Just drop in an audio file, and the app automatically generates time-aligned subtitle blocks.

    ⏱️ Non-Linear Timeline Editor: A fully featured, zoomable, and scrollable timeline track system. Includes block dragging, box selection, multi-clip shifting, and playhead scrubbing.

    🎢 Kinetic Keyframing System: Animate text dynamically. Add keyframes to text scale, rotation, opacity, and X/Y positioning. Interpolate smoothly using a robust set of easing functions (Bounce, Elastic, Back, Ease In/Out).

    🎨 Advanced Typography Inspector: Customize text down to the pixel. Features include drop shadows with variable blur, text strokes, dynamic gradient fills, and customizable background bounding boxes.

    🔊 Audio Playback Engine: Powered by rodio and symphonia, ensuring frame-accurate audio syncing as you scrub the timeline.

    💾 Project Serialization: Save and load your workspaces seamlessly using the custom .ksub JSON project format.

🛠️ Technology Stack

KineticSub-RS is built on a modern Rust ecosystem:

    Rendering: wgpu (WebGPU API for native platforms)

    User Interface: egui & egui-wgpu

    Audio Decoding: symphonia & hound

    Audio Playback: rodio

    AI Transcription: whisper-rs (Rust bindings for whisper.cpp)

    Serialization: serde & serde_json

🚀 Getting Started
Prerequisites

Because this project utilizes whisper.cpp under the hood, you will need a working C/C++ compiler toolchain installed on your machine (e.g., build-essential on Linux, Visual Studio Build Tools on Windows, or Xcode Command Line Tools on macOS).
Installation

    Clone the repository:
    code Bash

    git clone https://github.com/yourusername/kineticsub-rs.git
    cd kineticsub-rs

    Run the application:
    (It is highly recommended to run in release mode, as audio processing and Whisper AI inference are heavily computationally bound).
    code Bash

    cargo run --release

🎮 Usage Guide

    Import Audio: Click Import Audio in the Media Bin (left panel) to load a .wav, .mp3, or .m4a file.

    Add to Timeline: Click Add to Timeline on the media card.

    Transcribe: Right-click the newly created audio block on the timeline and select Transcribe. The application will download the Whisper base model (if not already cached) and generate your subtitles.

    Animate: Select a subtitle block. In the right-hand Inspector panel, switch to the Animate tab. Click ⏺ Record to auto-keyframe your changes as you scrub the timeline, or use the pre-built Animation Presets (e.g., Bounce In, Typewriter, Zoom Out).

    Save: Hit Ctrl+S to save your .ksub project file.

🚧 Roadmap & Limitations

As a passion project, KineticSub-RS is a work in progress. Current limitations include:

    Text Only: The canvas renders a 16:9 preview of the text, but video file decoding/playback is not supported.

    No Exporter: Currently, there is no render pipeline to export the canvas to an .mp4 or .mov file.

    Memory Optimization: Heavy audio files may take a moment to decode into PCM data for Whisper.

📄 License

This project is open-source and available under the MIT License. Feel free to fork, study the code, or use it as a boilerplate for your own Rust GUI applications.