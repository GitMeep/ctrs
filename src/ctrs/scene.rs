mod pipeline;

use std::{f32::consts::PI, sync::Arc};

use iced::{Rectangle, mouse, wgpu, widget::shader::{self, Viewport}};
use pipeline::{uniforms::{Camera, Projection}, Pipeline};

use super::scan::CtScan;

#[derive(Debug)]
pub struct Primitive {
    scan: Arc<CtScan>,
    projections: Arc<[Projection]>,
    camera_uniform: Camera,
}

impl Primitive {
    fn new(
        scan: Arc<CtScan>,
        projections: Arc<[Projection]>,
        inclination: f32,
        threshold: f32
    ) -> Self {
        Self {
            scan,
            projections,
            camera_uniform: Camera::new(
                40.,
                inclination,
                (70., 70.),
                0.5,
                threshold
            ),
        }
    }
}

impl shader::Primitive for Primitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline.prepare(
            device,
            queue,
            &self.scan.projection_images,
            (500,500,256), // TODO: don't have constant dimensions here
            &self.projections,
        );

        pipeline.update_camera(queue, &self.camera_uniform);
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.render(target, encoder, clip_bounds);
    }
}

pub struct Scene {
    scan: Arc<CtScan>,
    projections: Arc<[Projection]>,
    inclination: f32,
    threshold: f32,
}

impl Scene {
    pub fn new(scan: Arc<CtScan>, threshold: f32) -> Self {
        let rot_dir = scan.direction.dir();

        let n_projections = scan.projection_images.len();
        let projections = (0..n_projections).into_iter()
            .map(|i| Projection::new(
                    rot_dir * (i as f32)*(scan.swept_angle*PI/180.)/(n_projections as f32),
                    scan.sod,
                    scan.sdd,
                    (500.*scan.pixel_size, 500.*scan.pixel_size)
                )
            )
            .collect();
        
        Self {
            scan,
            projections,
            inclination: 0.,
            threshold,
        }
    }

    pub fn rotate(&mut self, delta: f32) {
        self.inclination += delta;
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Primitive {
        let primitive = Primitive::new(
            self.scan.clone(),
            self.projections.clone(),
            self.inclination,
            self.threshold,
        );

        primitive
    }
}

impl shader::Pipeline for Pipeline {
    fn new(
        device: &iced::wgpu::Device,
        queue: &iced::wgpu::Queue,
        format: iced::wgpu::TextureFormat,
    ) -> Self
    where
        Self: Sized {
        Self::new(device, queue, format)
    }
}
