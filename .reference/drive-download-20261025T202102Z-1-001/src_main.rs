// src/main.rs
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const MAX_DEFECTS: usize = 65536;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniforms {
    inv_view_proj: [f32; 16],
    light_dir: [f32; 4],
    screen_dim: [f32; 2],
    near_plane: f32,
    far_plane: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
struct DefectRecord {
    pixel_coord: [u32; 2],
    defect_type: u32,
    instance_id_a: u32,
    instance_id_b: u32,
    metric_val: f32,
    _pad0: u32,
    _pad1: u32,
}

struct GBufferDetector {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GBufferDetector {
    async fn init() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to locate GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("CI GBuffer Analyzer Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to init logical device");

        let shader_src = include_str!("../artifact_detector.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Artifact Detector Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_src)),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Analyzer Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Uint, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self { device, queue, pipeline, bind_group_layout }
    }

    fn run_pass(
        &self,
        depth_tex: &wgpu::Texture,
        normal_tex: &wgpu::Texture,
        inst_tex: &wgpu::Texture,
        shadow_tex: &wgpu::Texture,
        uniforms: CameraUniforms,
    ) -> (u32, Vec<DefectRecord>) {
        let uniform_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let buffer_size = (16 + (MAX_DEFECTS * std::mem::size_of::<DefectRecord>())) as u64;
        
        let storage_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Defect Storage Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.queue.write_buffer(&storage_buf, 0, bytemuck::bytes_of(&0u32));

        let staging_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&depth_tex.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&normal_tex.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&inst_tex.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&shadow_tex.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 4, resource: uniform_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: storage_buf.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups((WIDTH + 15) / 16, (HEIGHT + 15) / 16, 1);
        }

        encoder.copy_buffer_to_buffer(&storage_buf, 0, &staging_buf, 0, buffer_size);
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| tx.send(res).unwrap());
        
        while rx.try_recv().is_err() {
            self.device.poll(wgpu::Maintain::Wait);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let mapped = buffer_slice.get_mapped_range();
        let total_defects = *bytemuck::from_bytes::<u32>(&mapped[0..4]);
        let safe_count = (total_defects as usize).min(MAX_DEFECTS);

        let records_offset = 16;
        let records_byte_len = safe_count * std::mem::size_of::<DefectRecord>();
        
        let defect_slice: &[DefectRecord] = bytemuck::cast_slice(&mapped[records_offset..records_offset + records_byte_len]);
        let result_records = defect_slice.to_vec();
        
        drop(mapped);
        staging_buf.unmap();

        (total_defects, result_records)
    }
}

fn create_texture_2d(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, format: wgpu::TextureFormat, data: &[u8]) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        data,
    )
}

fn main() {
    env_logger::init(); 
    println!("[CI Pipeline] Initializing Apple/AAA-Tier G-Buffer Validator...");
    let detector = pollster::block_on(GBufferDetector::init());

    let mut depth_mock = vec![0.5f32; (WIDTH * HEIGHT) as usize];
    let mut normal_mock = vec![[0.0f32, 1.0f32, 0.0f32, 0.0f32]; (WIDTH * HEIGHT) as usize];
    let mut instance_mock = vec![101u32; (WIDTH * HEIGHT) as usize];
    let mut shadow_mock = vec![1.0f32; (WIDTH * HEIGHT) as usize];

    for y in 400..420 {
        for x in 400..420 {
            let idx = (y * WIDTH + x) as usize;
            depth_mock[idx] = 0.500001; 
            instance_mock[idx] = if (x + y) % 2 == 0 { 101 } else { 202 };
        }
    }

    for y in 600..620 {
        for x in 600..620 {
            let idx = (y * WIDTH + x) as usize;
            shadow_mock[idx] = if (x + y) % 2 == 0 { 0.0 } else { 1.0 };
        }
    }

    let depth_tex = create_texture_2d(&detector.device, &detector.queue, WIDTH, HEIGHT, wgpu::TextureFormat::R32Float, bytemuck::cast_slice(&depth_mock));
    let normal_tex = create_texture_2d(&detector.device, &detector.queue, WIDTH, HEIGHT, wgpu::TextureFormat::Rgba32Float, bytemuck::cast_slice(&normal_mock));
    let instance_tex = create_texture_2d(&detector.device, &detector.queue, WIDTH, HEIGHT, wgpu::TextureFormat::R32Uint, bytemuck::cast_slice(&instance_mock));
    let shadow_tex = create_texture_2d(&detector.device, &detector.queue, WIDTH, HEIGHT, wgpu::TextureFormat::R32Float, bytemuck::cast_slice(&shadow_mock));

    let view_proj = Mat4::perspective_infinite_reverse_rh(45.0f32.to_radians(), WIDTH as f32 / HEIGHT as f32, 0.1);
    
    let uniforms = CameraUniforms {
        inv_view_proj: view_proj.inverse().to_cols_array(),
        light_dir: [0.577, 0.577, 0.577, 0.0],
        screen_dim: [WIDTH as f32, HEIGHT as f32],
        near_plane: 0.1,
        far_plane: 1000.0,
    };

    println!("[CI Pipeline] Dispatching Compute Inspection...");
    let (defect_count, defects) = detector.run_pass(&depth_tex, &normal_tex, &instance_tex, &shadow_tex, uniforms);

    println!("[Verification Result] Total Detected Defects: {}", defect_count);
    let (mut z_fights, mut shadow_acne) = (0, 0);

    for defect in &defects {
        match defect.defect_type {
            1 => z_fights += 1,
            2 => shadow_acne += 1,
            _ => {}
        }
    }

    println!("  ├── Confirmed Z-Fighting Clusters: {}", z_fights);
    println!("  └── Confirmed Shadow Acne Anomalies: {}", shadow_acne);

    if defect_count > 0 {
        eprintln!("[FAILURE] Procedural generation output rejected by QA render pass!");
        let sample = &defects[0];
        eprintln!("  -> Critical Sample: Type {} at ({}, {}) between Mesh Instances {} & {}",
            sample.defect_type, sample.pixel_coord[0], sample.pixel_coord[1], sample.instance_id_a, sample.instance_id_b);
        std::process::exit(1);
    } else {
        println!("[SUCCESS] Render pass clear. Ready for PCG integration.");
    }
}