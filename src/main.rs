use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler, event::*, event_loop::ActiveEventLoop,
    event_loop::ControlFlow, event_loop::EventLoop, window::Window, window::WindowAttributes,
    window::WindowId,
};

// Uniform buffer for transformation and colors
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
    color_bg: [f32; 4],
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    num_vertices: u32,
}

impl<'a> State<'a> {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create fake data
        let points = generate_fake_data(800, 600, 1000, 5.0);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create uniform buffer and bind group
        let uniforms = Uniforms {
            matrix: create_orthographic_matrix(0.0, 800.0, 0.0, 600.0),
            color: [0.0, 0.5, 0.0, 0.8],
            color_bg: [0.0, 0.0, 0.0, 0.0],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the render pipeline and bind group
        let (pipeline, uniform_bind_group) =
            create_render_pipeline(&device, &config, &uniform_buffer);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            num_vertices: (points.len() / 2) as u32,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
    point_size: f32,
}

fn generate_fake_data(width: u32, height: u32, count: usize, volatility: f32) -> Vec<Vertex> {
    let mut points = Vec::with_capacity(count);
    let mut x = rand::random::<f32>() * width as f32;
    let mut y = rand::random::<f32>() * height as f32;
    let mut speed = 1.0 + rand::random::<f32>() * volatility;
    let mut dx = speed * (rand::random::<f32>() * 2.0 - 1.0);
    let mut dy = speed * (rand::random::<f32>() * 2.0 - 1.0);

    for _ in 0..count {
        let r = rand::random::<f32>();

        if r < 0.01 {
            x = rand::random::<f32>() * width as f32;
            y = rand::random::<f32>() * height as f32;
        } else if r < 0.1 / speed {
            speed = 1.0 + rand::random::<f32>() * volatility;
        } else if r < 0.2 / speed {
            dx = speed * (rand::random::<f32>() * 2.0 - 1.0);
            dy = speed * (rand::random::<f32>() * 2.0 - 1.0);
        } else {
            x += dx;
            y += dy;
        }

        if x < 0.0 || x > width as f32 || y < 0.0 || y > height as f32 {
            x = rand::random::<f32>() * width as f32;
            y = rand::random::<f32>() * height as f32;
        }

        points.push(Vertex {
            position: [x, y, 0.0, 1.0],
            point_size: 6.0, // Adjust this value as needed
        });
    }

    points
}

fn create_orthographic_matrix(left: f32, right: f32, bottom: f32, top: f32) -> [[f32; 4]; 4] {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);

    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, 0.0, 1.0],
    ]
}

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<State<'a>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop
            .create_window(Window::default_attributes())
                .unwrap());
            self.window = Some(window.clone());

            let state = pollster::block_on(State::new(window.clone()));
            self.state = Some(state);

            println!("State created");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                // self.window.as_ref().unwrap().request_redraw();

                if let Some(state) = self.state.as_mut() {
                    println!("Rendering Requested");
                    match state.render() {
                        Ok(_) => {
                            println!("Rendering Success");
                        }
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                println!("Window Resized Requested");
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size);
                }
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    // let window = WindowBuilder::new()
    //     .with_title("WGPU Points")
    //     .build(&event_loop)
    //     .unwrap();

    // let window = event_loop
    //     .create_window(WindowAttributes::default().with_title("WGPU Points"))
    //     .unwrap();

    // let mut state = pollster::block_on(State::new(&window));

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();

    let _ = event_loop.run_app(&mut app);

    // let _ = event_loop.run_app(move |event, app| {
    //     match event {
    //         Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
    //             match state.render() {
    //                 Ok(_) => {}
    //                 Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
    //                 Err(e) => eprintln!("{:?}", e),
    //             }
    //         }
    //         Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
    //             app.exit();
    //         }
    //         Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
    //             state.resize(new_size);
    //         }
    //         Event::AboutToWait => {
    //             window.request_redraw();
    //         }
    //         _ => {}
    //     }
    // });
}

fn create_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    uniform_buffer: &wgpu::Buffer,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("renderer/vertex.wgsl"))),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("renderer/fragment.wgsl"))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress, // position (4) + point_size (1)
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // Change to TriangleList
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    (pipeline, bind_group)
}
