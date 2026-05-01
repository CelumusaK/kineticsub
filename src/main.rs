use kineticsub::viewmodels::editor_vm::EditorViewModel;
use kineticsub::views::{self, inspector::InspectorTab};

use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
use wgpu::*;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    gpu: Option<GpuContext>,
    vm: EditorViewModel,
    inspector_tab: InspectorTab,
    modifiers: winit::keyboard::ModifiersState,
}

struct GpuContext {
    window: std::sync::Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: EguiWinitState,
    egui_renderer: egui_wgpu::Renderer,
}

impl GpuContext {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("KineticSub-RS")
                        .with_inner_size(PhysicalSize::new(1280u32, 720u32))
                        .with_min_inner_size(PhysicalSize::new(900u32, 600u32)),
                )
                .expect("Failed to create window"),
        );

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create wgpu surface");

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("KineticSub device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::default(),
            },
            None,
        ))
        .expect("Failed to create GPU device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let egui_ctx = egui::Context::default();
        egui_ctx.style_mut(kineticsub::views::theme::apply);

        let egui_state = EguiWinitState::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        Self { window, surface, device, queue, config, egui_ctx, egui_state, egui_renderer }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, vm: &mut EditorViewModel, inspector_tab: &mut InspectorTab) {
        vm.poll_whisper();
        vm.poll_render(); 
        vm.tick();

        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                self.resize(self.window.inner_size());
                return;
            }
            Err(e) => { eprintln!("Surface error: {e}"); return; }
        };
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let raw_input = self.egui_state.take_egui_input(&self.window);
        
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            views::top_bar::draw(ctx, vm);
            views::timeline::draw(ctx, vm);
            views::left_panel::draw(ctx, vm);
            views::inspector::draw(ctx, vm, inspector_tab);
            views::canvas::draw(ctx, vm);

            // ── Auto-Commit History Snapshots when drag ends
            let pointer_down = ctx.input(|i| i.pointer.any_down());
            vm.maybe_snapshot(pointer_down);
        });

        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let prims = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, delta);
        }

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &prims, &screen);

        {
            let mut rpass = encoder
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("main pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(wgpu::Color { r: 0.051, g: 0.059, b: 0.075, a: 1.0 }),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                })
                .forget_lifetime();
            self.egui_renderer.render(&mut rpass, &prims, &screen);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        if vm.is_playing() || vm.whisper_is_running() || vm.show_fps {
            self.window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            self.gpu = Some(GpuContext::new(event_loop));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let gpu = match &mut self.gpu { Some(g) => g, None => return };

        let response = gpu.egui_state.on_window_event(&gpu.window, &event);
        if response.repaint {
            gpu.window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                gpu.resize(size);
                gpu.window.request_redraw();
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }

            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state: ElementState::Pressed, .. }, .. } => {
                let ctrl = self.modifiers.control_key();
                let shift = self.modifiers.shift_key();
                
                match code {
                    KeyCode::Space => self.vm.toggle_play(),
                    KeyCode::KeyJ => self.vm.skip(-5.0),
                    KeyCode::KeyL => self.vm.skip(5.0),
                    KeyCode::ArrowLeft => self.vm.skip(-1.0 / 30.0),
                    KeyCode::ArrowRight => self.vm.skip(1.0 / 30.0),
                    KeyCode::Escape => self.vm.select_subtitle(None),
                    KeyCode::KeyS if ctrl => self.vm.save_project(),
                    // ── Undo / Redo Keybinds
                    KeyCode::KeyZ if ctrl && shift => self.vm.redo(),
                    KeyCode::KeyZ if ctrl && !shift => self.vm.undo(),
                    KeyCode::KeyY if ctrl => self.vm.redo(),
                    _ => {}
                }
                gpu.window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                gpu.render(&mut self.vm, &mut self.inspector_tab);
                if self.vm.is_playing() || self.vm.whisper_is_running() || self.vm.show_fps {
                    gpu.window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu {
            if self.vm.is_playing() || self.vm.whisper_is_running() || self.vm.show_fps {
                gpu.window.request_redraw();
            }
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App {
        gpu: None,
        vm: EditorViewModel::new(),
        inspector_tab: InspectorTab::default(),
        modifiers: winit::keyboard::ModifiersState::empty(),
    };
    event_loop.run_app(&mut app).expect("Event loop failed");
}