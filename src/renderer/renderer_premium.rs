use crate::log_print;
use crate::renderer::TileRendererBasic;
use crate::renderer::TileRendererTrait;
use crate::utils::TileSize;
use crate::FogMap;
use image::GenericImage;
use image::Rgba;
use image::RgbaImage;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::task;
use wasm_bindgen_futures::spawn_local;
use wgpu::{self, BindGroup, Buffer, ComputePipeline, Device, Queue};

const WORKGROUP_SIZE: (u32, u32) = (16, 16);

pub struct TileRendererPremium {
    device: Arc<Device>,
    queue: Arc<Queue>,
    compute_pipeline: ComputePipeline,
    bind_group: BindGroup,
    input_buffer: Buffer,
    output_buffer: Buffer,
    dimensions_buffer: Buffer,
    width: u32,
    height: u32,
    renderer: TileRendererBasic,
}

impl TileRendererTrait for TileRendererPremium {
    fn get_tile_size(&self) -> TileSize {
        self.renderer.get_tile_size()
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
        // check the image size
        let tile_size = self.get_tile_size().size();
        debug_assert!(image.width() >= start_x + self.get_tile_size().size());
        debug_assert!(image.height() >= start_y + self.get_tile_size().size());

        // currently the gpu shading cannot be applied in-place
        let temp_image = self
            .renderer
            .render_image(fogmap, view_x, view_y, zoom, bg_color, fg_color);
        let rt = Runtime::new().unwrap();

        let processed_image = RgbaImage::from_raw(
            tile_size,
            tile_size,
            rt.block_on(self.process_frame_async(temp_image.as_raw())),
        )
        .unwrap();

        let _ = image.copy_from(&processed_image, start_x, start_y);
    }
}

impl TileRendererPremium {
    pub fn new(tile_size: TileSize) -> Self {
        Runtime::new()
            .unwrap()
            .block_on(async move { Self::new_async(tile_size).await })
    }

    pub async fn new_async(tile_size: TileSize) -> Self {
        let width = tile_size.size();
        let height = tile_size.size();

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

        log_print!("Adapter created: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                // None,
                Some(std::path::Path::new("trace")),
            )
            .await
            .unwrap();

        log_print!(
            "Device created: {:?}, {:?}",
            device.features(),
            device.limits()
        );

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fog Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fog_shader.wgsl").into()),
        });

        log_print!("creating buffer size: {}", width * height * 4);
        let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Buffer"),
            size: (width * height * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (width * height * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dimensions Buffer"),
            size: std::mem::size_of::<[u32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: dimensions_buffer.as_entire_binding(),
                },
            ],
            label: Some("Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            cache: None,
            compilation_options: Default::default(),
        });

        Self {
            device,
            queue,
            compute_pipeline,
            bind_group,
            input_buffer,
            output_buffer,
            dimensions_buffer,
            width,
            height,
            renderer: TileRendererBasic::new(tile_size),
        }
    }

    pub async fn process_frame_async(&self, input: &[u8]) -> Vec<u8> {
        log_print!("Starting process_frame, input length: {}", input.len());
        self.queue.write_buffer(
            &self.dimensions_buffer,
            0,
            bytemuck::cast_slice(&[self.width, self.height]),
        );
        self.queue.write_buffer(&self.input_buffer, 0, input);
        log_print!("Input buffer written");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.width / WORKGROUP_SIZE.0,
                self.height / WORKGROUP_SIZE.1,
                1,
            );
        }
        log_print!("Compute pass encoded");

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (self.width * self.height * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buffer_arc = Arc::new(staging_buffer);

        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &staging_buffer_arc,
            0,
            (self.width * self.height * 4) as u64,
        );
        log_print!("Buffer copy encoded");

        self.queue.submit(Some(encoder.finish()));
        log_print!("Commands submitted");

        let staging_buffer_arc2 = staging_buffer_arc.clone();
        let buffer_slice = staging_buffer_arc2.slice(..);

        log_print!("buffer_slice created");

        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            println!("buffer slice completed!!!");
            let _ = tx.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);
        let _ = rx.await.unwrap();
        let buffer_slice3 = staging_buffer_arc2.slice(..);

        let mapped_range = buffer_slice3.get_mapped_range();
        // callback(mapped_range.to_vec());
        println!("mapped_range length: {}", mapped_range.len());
        mapped_range.to_vec()
    }

    pub fn process_frame(&self, input: &[u8], callback: Box<dyn Fn(Vec<u8>)>) {
        log_print!("Starting process_frame, input length: {}", input.len());
        self.queue.write_buffer(
            &self.dimensions_buffer,
            0,
            bytemuck::cast_slice(&[self.width, self.height]),
        );
        self.queue.write_buffer(&self.input_buffer, 0, input);
        log_print!("Input buffer written");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(
                self.width / WORKGROUP_SIZE.0,
                self.height / WORKGROUP_SIZE.1,
                1,
            );
        }
        log_print!("Compute pass encoded");

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (self.width * self.height * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buffer_arc = Arc::new(staging_buffer);

        encoder.copy_buffer_to_buffer(
            &self.output_buffer,
            0,
            &staging_buffer_arc,
            0,
            (self.width * self.height * 4) as u64,
        );
        log_print!("Buffer copy encoded");

        self.queue.submit(Some(encoder.finish()));
        log_print!("Commands submitted");

        let staging_buffer_arc2 = staging_buffer_arc.clone();
        let buffer_slice = staging_buffer_arc2.slice(..);

        log_print!("buffer_slice created");

        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // task::spawn_local(async move {
        //     let _ = rx.await.unwrap();
        //     let buffer_slice3 = staging_buffer_arc2.slice(..);

        //     // Handle the mapped data here
        //     let mapped_range = buffer_slice3.get_mapped_range();
        //     log_print!("mapped data is of length: {}", mapped_range.len());
        //     callback(mapped_range.to_vec());
        // });

        // if target is wasm, then we need to use spawn_local
        // if target is native, then we need to use task::spawn_local
        if cfg!(target_arch = "wasm32") {
            spawn_local(async move {
                let _ = rx.await.unwrap();
                let buffer_slice3 = staging_buffer_arc2.slice(..);

                let mapped_range = buffer_slice3.get_mapped_range();
                callback(mapped_range.to_vec());
            });
        } else {
            task::spawn_local(async move {
                let _ = rx.await.unwrap();
                let buffer_slice3 = staging_buffer_arc2.slice(..);

                let mapped_range = buffer_slice3.get_mapped_range();
                callback(mapped_range.to_vec());
            });
        }

        log_print!("process_frame finished");
    }
}
