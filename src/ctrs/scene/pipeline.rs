pub mod uniforms;
mod post_pipeline;

use std::collections::HashMap;

use iced::{Rectangle, wgpu::{self, util::DeviceExt}, widget::shader};
use uniforms::{Camera, Projection};

use crate::ctrs::scene::pipeline::post_pipeline::PostPipeline;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    post_pipeline: PostPipeline,

    camera_uniform_buffer: wgpu::Buffer,

    projections_bind_group_layout: wgpu::BindGroupLayout,
    projections_bind_group: HashMap<String, wgpu::BindGroup>,

    camera_bind_group: wgpu::BindGroup,
    
    render_texture: Option<wgpu::Texture>,
    render_texture_view: Option<wgpu::TextureView>,
    texture_format: wgpu::TextureFormat,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device, 
        format: wgpu::TextureFormat
    ) -> Self {
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
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            label: Some("Render pipeline layout"),
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
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
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
            cache: None,
        });

        let post_pipeline = PostPipeline::new(device, format);

        Self {
            pipeline,
            post_pipeline,
            camera_uniform_buffer,
            camera_bind_group,
            projections_bind_group_layout,
            projections_bind_group: HashMap::new(),
            render_texture: None,
            render_texture_view: None,
            texture_format: format,
        }
    }

    pub fn prepare_projections(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        projections_texture: &[f32],
        projections: &[Projection],
        extent: (u32, u32, u32)
    ) {
        if self.projections_bind_group.contains_key(name) {
            return;
        }

        let projections_extent = wgpu::Extent3d {
            width: extent.0,
            height: extent.1,
            depth_or_array_layers: extent.2,
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
            bytemuck::cast_slice(&projections_texture)
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
            ..Default::default()
        });

        let projections_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projections storage buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&projections),
        });

        let projections_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Projections texture bind group"),
            layout: &self.projections_bind_group_layout,
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

        self.projections_bind_group.insert(name.into(), projections_bind_group);
    }

    pub fn prepare_render_buffer(
        &mut self,
        device: &wgpu::Device,
        bounds: &Rectangle,
    ) {
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("CTRS render buffer texture"),
            size: wgpu::Extent3d {
                width: bounds.width as u32,
                height: bounds.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let render_texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.post_pipeline.prepare(device, &render_texture_view);

        self.render_texture = Some(render_texture);
        self.render_texture_view = Some(render_texture_view);
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        queue.write_buffer(&self.camera_uniform_buffer, 0, bytemuck::cast_slice(&[*camera]));
    }

    fn do_render(
        &self,
        target: &wgpu::TextureView,
        buffer_target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: &Rectangle<u32>,
        projections_bind_group: &wgpu::BindGroup
    ) {
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CTRS render pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: buffer_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);

            pass.set_bind_group(0, projections_bind_group, &[]);
            pass.set_bind_group(1, &self.camera_bind_group, &[]);
            
            pass.draw(0..3, 0..1);
        }

        let mut post_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("CTRS post processing pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        post_pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0
        );

        self.post_pipeline.apply(&mut post_pass);
    }

    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.post_pipeline.apply(pass);
    }

    pub fn render(
        &self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clip_bounds: &Rectangle<u32>,
        name: &str,
    ) {
        match (&self.render_texture_view, &self.projections_bind_group.get(name)) {
            (Some(view), Some(group)) => self.do_render(
                target,
                view,
                encoder,
                clip_bounds,
                group
            ),
            _ => log::warn!("Rerender attempted with unprepared render texture and/or projections bind group")
        }
    }
}

impl shader::Pipeline for Pipeline {
    fn new(
        device: &iced::wgpu::Device,
        _queue: &iced::wgpu::Queue,
        format: iced::wgpu::TextureFormat,
    ) -> Self {
        Self::new(device, format)
    }
}
