mod camera;
mod util;
mod text;

use std::time::{Duration, Instant};

use camera::Camera;
use pollster::block_on;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const FPS: f64 = 60.0;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("SDF Text Renderer Prototype")
        .build(&event_loop)
        .unwrap();

    let mut gfx = block_on(Graphics::new(&window)).unwrap();
    let camera = Camera::new(&gfx);

    let pipeline1 = util::pipeline1(&gfx);

    let target_framerate = Duration::from_secs_f64(1.0 / FPS);
    let mut time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(inner_size)
                | winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut inner_size,
                    ..
                } => gfx.resize(inner_size),
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            winit::event::Event::DeviceEvent { event, .. } => (),
            winit::event::Event::MainEventsCleared => {
                let elapsed = time.elapsed();
                if elapsed >= target_framerate {
                    window.request_redraw();
                    time = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_framerate - elapsed,
                    );
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                let frame = gfx.surface.get_current_texture().unwrap();
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = gfx.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Command Encoder"),
                    },
                );

                {
                    let mut rpass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(
                                wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(
                                            wgpu::Color {
                                                r: 0.1,
                                                g: 0.2,
                                                b: 0.3,
                                                a: 1.,
                                            },
                                        ),
                                        store: true,
                                    },
                                },
                            )],
                            depth_stencil_attachment: None,
                        },
                    );

                    rpass.set_pipeline(&pipeline1);
                    rpass.draw(0..4, 0..todo!());
                }

                gfx.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    });
}
pub struct Graphics {
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl Graphics {
    async fn new(
        window: &winit::window::Window,
    ) -> Result<Self, wgpu::RequestDeviceError> {
        let backends = wgpu::util::backend_bits_from_env()
            .unwrap_or_else(wgpu::Backends::all);
        let instance = wgpu::Instance::new(backends);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(&window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("No adapters found!");

        let adapter_info = adapter.get_info();
        println!(
            "Adapter info: Name: {}, backend: {:?}, device: {:?}",
            adapter_info.name, adapter_info.backend, adapter_info.device_type
        );

        let required_features = wgpu::Features::empty();
        let adapter_features = adapter.features();
        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: adapter_features & required_features,
                    limits: required_limits,
                },
                None,
            )
            .await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            surface,
            adapter,
            queue,
            config,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}
