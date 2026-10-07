use std::sync::Arc;

use iced::{Rectangle, wgpu, widget::shader::{self, Viewport}};

use crate::ctrs::{scene::pipeline::{Pipeline, uniforms::{Camera, Projection}}};

#[derive(Debug)]
pub struct Primitive {
    name: String,
    projections: Arc<[Projection]>,
    projections_texture: Arc<[f32]>,
    projections_dims: (u32, u32),
    camera_uniform: Camera,
    rerender: bool,
}

impl Primitive {
    pub fn new(
        name: String,
        projections: Arc<[Projection]>,
        projections_texture: Arc<[f32]>,
        projections_dims: (u32, u32),
        azimuth: f32,
        inclination: f32,
        threshold: f32,
        dimensions: (f32,f32),
        sampling_interval: f32,
        sample_radius: f32,
        sample_height: f32,
        rerender: bool
    ) -> Self {
        Self {
            name,
            projections,
            projections_texture,
            projections_dims,
            camera_uniform: Camera::new(
                azimuth,
                inclination,
                dimensions,
                sampling_interval,
                sample_radius,
                sample_height,
                threshold
            ),
            rerender
        }
    }
}

impl shader::Primitive for Box<Primitive> {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        if !self.rerender {
            return;
        }

        pipeline.prepare_projections(
            device,
            queue,
            &self.name,
            &self.projections_texture,
            &self.projections,
            (
                self.projections_dims.0,
                self.projections_dims.1,
                self.projections.len() as u32
            )
        );

        pipeline.prepare_render_buffer(device, bounds);

        pipeline.update_camera(queue, &self.camera_uniform);
    }

    fn draw(
        &self,
        pipeline: &Self::Pipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        if !self.rerender {
            pipeline.draw(render_pass);
            true
        } else {
            false
        }
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // render to target and buffer texture
        pipeline.render(target, encoder, clip_bounds, &self.name);
    }
}
