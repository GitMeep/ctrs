use iced::wgpu;

pub struct PostPipeline {
    pipeline: wgpu::RenderPipeline,

    input_texture_bind_group_layout: wgpu::BindGroupLayout,
    input_texture_bind_group: Option<wgpu::BindGroup>,
}

impl PostPipeline {
    pub fn new(
        device: &wgpu::Device, 
        format: wgpu::TextureFormat
    ) -> Self {
        let input_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("../../shaders/post.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post processing pipeline layout"),
            push_constant_ranges: &[],
            bind_group_layouts: &[
                &input_texture_bind_group_layout,
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post processing pipeline"),
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

        Self {
            pipeline,
            input_texture_bind_group_layout,
            input_texture_bind_group: None,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        input: &wgpu::TextureView,
    ) {
        let input_sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            label: Some("Post processing input texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post processing input texture bind group"),
            layout: &self.input_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&input_sampler),
                },
            ]
        });

        self.input_texture_bind_group = Some(bind_group);
    }

    pub fn apply(
        &self,
        pass: &mut wgpu::RenderPass<'_>
    ) {
        if let Some(bind_group) = &self.input_texture_bind_group {
            pass.set_pipeline(&self.pipeline);

            pass.set_bind_group(0, bind_group, &[]);
            
            pass.draw(0..3, 0..1);
        } else {
            log::warn!("Attempted to apply post-processing without input texture being prepared!");
        }
    }
}
