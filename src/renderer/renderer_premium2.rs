use crate::renderer::TileRendererTrait;
use crate::renderer::TileShader2;
use crate::utils::TileSize;
use crate::utils::{DEFAULT_BG_COLOR2, DEFAULT_FG_COLOR2};
use crate::FogMap;
use image::GenericImage;
use image::Pixel;
use image::Rgba;
use image::RgbaImage;
use std::iter;
use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

const DEFAULT_DOT_SIZE: f32 = 3.0;

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

pub struct TileRendererPremium2 {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_format: TextureFormat,
    dot_size: f32,
    tile_size: TileSize,
}

impl TileRendererTrait for TileRendererPremium2 {
    fn get_tile_size(&self) -> TileSize {
        self.tile_size
    }

    fn render_on_image(
        &self,
        image: &mut RgbaImage,
        start_x: u32,
        start_y: u32,
        fogmap: &FogMap,
        view_x: i64,
        view_y: i64,
        zoom: i16,
        bg_color: Rgba<u8>,
        fg_color: Rgba<u8>,
    ) {
        let rendered_image = pollster::block_on(async move {
            self.render_image_async(fogmap, view_x, view_y, zoom, bg_color, fg_color)
                .await
        });
        let _ = image.copy_from(&rendered_image, start_x, start_y);
    }
}

impl TileRendererPremium2 {
    pub fn new(tile_size: TileSize) -> Self {
        pollster::block_on(Self::new_async(tile_size))
    }

    pub async fn new_async(tile_size: TileSize) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await;

        let adapter = match adapter {
            Some(adapter) => adapter,
            None => {
                // If no adapter is found, try again with force_fallback_adapter set to true
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: None,
                        force_fallback_adapter: true,
                    })
                    .await
                    .expect("Failed to find an appropriate adapter")
            }
        };

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

        // Use a common texture format for offscreen rendering
        let texture_format = TextureFormat::Rgba8UnormSrgb;

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
                    format: texture_format,
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

        let bg_color = DEFAULT_BG_COLOR2;
        let fg_color = DEFAULT_FG_COLOR2;

        // Create uniform buffer and bind group
        let uniforms = Uniforms {
            matrix: create_orthographic_matrix(
                0.0,
                tile_size.size() as f32,
                tile_size.size() as f32,
                0.0,
            ),
            size: [tile_size.size() as f32, tile_size.size() as f32, 0.0, 0.0],
            dot_size: [DEFAULT_DOT_SIZE, DEFAULT_DOT_SIZE, 0.0, 0.0],
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
            device,
            queue,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_format,
            dot_size: DEFAULT_DOT_SIZE,
            tile_size,
        }
    }

    fn update_uniforms(&self, start_x: u32, start_y: u32, bg_color: Rgba<u8>, fg_color: Rgba<u8>) {
        let uniforms = Uniforms {
            // Update matrix for the current tile position
            matrix: create_orthographic_matrix(
                start_x as f32,
                (start_x + self.tile_size.size()) as f32,
                (start_y + self.tile_size.size()) as f32,
                start_y as f32,
            ),
            size: [
                self.tile_size.size() as f32,
                self.tile_size.size() as f32,
                0.0,
                0.0,
            ],
            dot_size: [self.dot_size, self.dot_size, 0.0, 0.0],
            color: rgba_to_vec4(fg_color),
            color_bg: rgba_to_vec4(bg_color),
        };

        // Update the uniform buffer with new values
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub async fn render_image_async(
        &self,
        fogmap: &FogMap,
        view_x: i64,
        view_y: i64,
        zoom: i16,
        bg_color: Rgba<u8>,
        fg_color: Rgba<u8>,
    ) -> RgbaImage {
        let mut image = RgbaImage::new(
            self.get_tile_size().size() as u32,
            self.get_tile_size().size() as u32,
        );
        let start_x = 0;
        let start_y = 0;
        // Update uniforms for this specific tile
        self.update_uniforms(start_x, start_y, bg_color, fg_color);

        // check the image size
        let tile_size = self.get_tile_size().size();
        debug_assert!(image.width() >= start_x + self.get_tile_size().size() as u32);
        debug_assert!(image.height() >= start_y + self.get_tile_size().size() as u32);

        // currently the gpu shading cannot be applied in-place
        let mut pixels_coordinates = TileShader2::get_pixels_coordinates(
            start_x,
            start_y,
            fogmap,
            view_x,
            view_y,
            zoom,
            self.tile_size.power(),
        );

        if pixels_coordinates.len() == 0 {
            // FIXME: this is a hack to avoid the shader from crashing when there are no pixels to render
            pixels_coordinates.push(-1.0);
            pixels_coordinates.push(-1.0);
        }

        let num_pixels = pixels_coordinates.len() / 2;

        // Create texture for offscreen rendering
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Texture"),
            size: wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create output buffer for reading pixels
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (tile_size * tile_size * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Create vertex buffer
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&pixels_coordinates),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: bg_color.channels()[0] as f64 / 255.0,
                            g: bg_color.channels()[1] as f64 / 255.0,
                            b: bg_color.channels()[2] as f64 / 255.0,
                            a: bg_color.channels()[3] as f64 / 255.0,
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
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..num_pixels as u32);
        }

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(tile_size * 4),
                    rows_per_image: Some(tile_size),
                },
            },
            wgpu::Extent3d {
                width: tile_size,
                height: tile_size,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(iter::once(encoder.finish()));

        // Read the buffer - modified to handle lifetimes correctly
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        receiver.recv_async().await.unwrap().unwrap();

        // Create the vector before dropping the buffer
        let data = buffer_slice.get_mapped_range().to_vec();

        let rendered_image = RgbaImage::from_raw(tile_size, tile_size, data).unwrap();

        let _ = image.copy_from(&rendered_image, start_x, start_y);
        image
    }
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
