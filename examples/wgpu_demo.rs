use image::{Pixel, Rgba};
use std::iter;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler, event::*, event_loop::ActiveEventLoop,
    event_loop::ControlFlow, event_loop::EventLoop, window::Window, window::WindowId,
};

const DEFAULT_DOT_SIZE: f32 = 3.0;
const DEFAULT_BG_COLOR2: Rgba<u8> = Rgba([255, 0, 255, 127]);
const DEFAULT_FG_COLOR2: Rgba<u8> = Rgba([0, 0, 0, 0]);

const PIXELS: &[f32] = &[
    256.0, 384.0, 128.0, 128.0, 384.0, 128.0, 100.0, 200.0, 200.0, 200.0, 300.0, 200.0, 400.0,
    200.0, 500.0, 200.0, 600.0, 200.0, 100.0, 300.0, 100.0, 400.0, 100.0, 500.0, 100.0, 600.0,
];

// Uniform buffer for transformation and colors
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    size: [f32; 4],
    dot_size: [f32; 4],
    color: [f32; 4],
    color_bg: [f32; 4],
}

// Shader code
const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) center: vec2<f32>,
};

struct Uniforms {
    matrix: mat4x4<f32>,
    size: vec4<f32>,
    dot_size: vec4<f32>,
    color: vec4<f32>,
    color_bg: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) position: vec2<f32>,
    @builtin(vertex_index) vNdx: u32
) -> VertexOutput {

    // for each point, we clip a square around it
    let points = array(
        vec2f(-1, -1),
        vec2f( 1, -1),
        vec2f(-1,  1),
        vec2f(-1,  1),
        vec2f( 1, -1),
        vec2f( 1,  1),
    );

    let pos = points[vNdx];
    
    var output: VertexOutput;

    // TODO: clamp all the positions

    output.position = uniforms.matrix * vec4<f32>(position + pos * uniforms.dot_size.x, 0.0, 1.0);
    output.center = (uniforms.matrix * vec4<f32>(position, 0.0, 1.0)).xy;
    
    return output;
}

@fragment
fn fs_main(vsOutput: VertexOutput) -> @location(0) vec4<f32> {
    var center = (vsOutput.center + vec2<f32>(1.0, 1.0)) * 0.5 * uniforms.size.xy;
    // TODO: this is a hack to flip the y axis, maybe its platform dependent
    center.y = uniforms.size.y - center.y;

    let dist = length(vsOutput.position.xy / vsOutput.position.w - center);
    let diff = vsOutput.position.xy / vsOutput.position.w - center;
    
    if (dist > uniforms.dot_size.x) {
        discard;
    }

    return uniforms.color;

    // Smooth the edges
    // let smoothing = 0.1;
    // let alpha = 1.0 - smoothstep(0.8, 1.0, dist);
    
    // return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    size: winit::dpi::PhysicalSize<u32>,
    dot_size: f32,
    bg_color: Rgba<u8>,
    fg_color: Rgba<u8>,
}

impl<'a> State<'a> {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let dot_size = DEFAULT_DOT_SIZE;
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
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(PIXELS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bg_color = DEFAULT_BG_COLOR2;
        let fg_color = DEFAULT_FG_COLOR2;

        // Create uniform buffer and bind group
        let uniforms = Uniforms {
            matrix: create_orthographic_matrix(0.0, size.width as f32, size.height as f32, 0.0),
            size: [size.width as f32, size.height as f32, 0.0, 0.0],
            dot_size: [dot_size, dot_size, 0.0, 0.0],
            color: rgba_to_vec4(fg_color),
            color_bg: rgba_to_vec4(bg_color),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            size,
            dot_size,
            bg_color,
            fg_color,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let uniforms = Uniforms {
                matrix: create_orthographic_matrix(
                    0.0,
                    new_size.width as f32,
                    new_size.height as f32,
                    0.0,
                ),
                size: [new_size.width as f32, new_size.height as f32, 0.0, 0.0],
                dot_size: [self.dot_size, self.dot_size, 0.0, 0.0],
                color: rgba_to_vec4(self.fg_color),
                color_bg: rgba_to_vec4(self.bg_color),
            };

            self.queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.bg_color.channels()[0] as f64 / 255.0,
                            g: self.bg_color.channels()[1] as f64 / 255.0,
                            b: self.bg_color.channels()[2] as f64 / 255.0,
                            a: self.bg_color.channels()[3] as f64 / 255.0,
                        }),
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
            render_pass.draw(0..6, 0..13);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// async fn run() {
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();
//     let mut state = State::new(&window).await;

//     event_loop.run(move |event, _, control_flow| {
//         match event {
//             Event::RedrawRequested(window_id) if window_id == window.id() => {
//                 match state.render() {
//                     Ok(_) => {}
//                     Err(wgpu::SurfaceError::Lost) => {},
//                     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                     Err(e) => eprintln!("{:?}", e),
//                 }
//             }
//             Event::MainEventsCleared => {
//                 window.request_redraw();
//             }
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == window.id() => match event {
//                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                 _ => {}
//             },
//             _ => {}
//         }
//     });
// }

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<State<'a>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
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
                if let Some(state) = self.state.as_mut() {
                    println!("Rendering Requested");
                    match state.render() {
                        Ok(_) => {
                            println!("Rendering Success");
                        }
                        // Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                println!("Window Resized Requested: {:?}", new_size);
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size);
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();

    let _ = event_loop.run_app(&mut app);
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

fn rgba_to_vec4(color: Rgba<u8>) -> [f32; 4] {
    [
        color.channels()[0] as f32 / 255.0,
        color.channels()[1] as f32 / 255.0,
        color.channels()[2] as f32 / 255.0,
        color.channels()[3] as f32 / 255.0,
    ]
}
