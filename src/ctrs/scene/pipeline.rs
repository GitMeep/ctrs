pub mod uniforms;
pub mod vertex;

use iced::Rectangle;
use iced_wgpu::wgpu::{self, util::DeviceExt};
use uniforms::{Camera, Projection};
use vertex::Vertex;

use crate::ctrs::scan::ScanImage;

const VERTICES: &[Vertex; 4] = &[
    Vertex { position: [-1.0,  1.0], cam_coords: [-1.0,  1.0] }, // top left
    Vertex { position: [-1.0, -1.0], cam_coords: [-1.0, -1.0] }, // bottom left
    Vertex { position: [ 1.0, -1.0], cam_coords: [ 1.0, -1.0] }, // bottom right
    Vertex { position: [ 1.0,  1.0], cam_coords: [ 1.0,  1.0] }, // top right
];

const INDICES: &[u16] = &[
    0,1,2, // bottom left triangle
    2,3,0, // top right triangle
];

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    camera_uniform_buffer: wgpu::Buffer,

    camera_bind_group: wgpu::BindGroup,
    projections_bind_group: wgpu::BindGroup,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device, 
        texture_format: &wgpu::TextureFormat,
        queue: &wgpu::Queue,
        projection_images: &[ScanImage],
        proj_extent: (u32,u32,u32),
        projections: &[Projection],
    ) -> Self {
        let buf_size: usize = projection_images.iter().map(|img| img.len()).sum();

        let mut transformed_texture_data: Vec<f32> = Vec::with_capacity(buf_size);
        for proj in projection_images {
            transformed_texture_data.extend(proj.iter().map(|sample| -sample.ln()));
        }

        let max: f32 = transformed_texture_data.iter().copied().reduce(|prev, cur| prev.max(cur)).unwrap();
        let normalized_texture_data: Vec<f32> = transformed_texture_data.iter().map(|sample| sample/max).collect();

        // TODO: handle differing image sizes (maybe not here, but in CtScan::load_images)
        let projections_extent = wgpu::Extent3d {
            width: proj_extent.0,
            height: proj_extent.1,
            depth_or_array_layers: proj_extent.2,
        };

        let projections_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Projections texture"),
                size: projections_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&normalized_texture_data)

        );

        let projections_view = projections_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let projections_sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            label: Some("Projections texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            // border_color: Some(wgpu::SamplerBorderColor::OpaqueBlack),
            ..Default::default()
        });

        let projections_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projections storage buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&projections) //&projections_wgsl,
        });

        let projections_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Projections texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let projections_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projections texture bind group"),
            layout: &projections_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&projections_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&projections_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: projections_buffer.as_entire_binding(),
                }
            ],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera buffer"),
            size: std::mem::size_of::<Camera>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ]
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                }
            ]
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Camera bind group layout"),
            push_constant_ranges: &[],
            bind_group_layouts: &[
                &projections_bind_group_layout,
                &camera_bind_group_layout
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Volume rendering pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        }
                    ]
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            camera_uniform_buffer,
            camera_bind_group,
            projections_bind_group,
        }
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        queue.write_buffer(&self.camera_uniform_buffer, 0, bytemuck::cast_slice(&[*camera]));
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("CTRS render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0
        );

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.projections_bind_group, &[]);
        pass.set_bind_group(1, &self.camera_bind_group, &[]);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        pass.draw_indexed(0..6, 0, 0..1);
    }
}