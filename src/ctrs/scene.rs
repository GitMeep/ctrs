mod pipeline;

use std::{f32::consts::PI, sync::{Arc, RwLock}};

use iced::{mouse, widget::shader};
use iced_wgpu::wgpu;
use pipeline::{uniforms::{Camera, Projection}, Pipeline};

use super::scan::CtScan;

#[derive(Debug)]
pub struct Primitive {
    scan: Arc<CtScan>,
    projections: Arc<[Projection]>,
    camera_uniform: Camera,
    new_scene: bool,
}

impl Primitive {
    fn new(
        scan: Arc<CtScan>,
        projections: Arc<[Projection]>,
        inclination: f32,
        threshold: f32,
        new_scene: bool
    ) -> Self {
        Self {
            scan,
            projections,
            new_scene,
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
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        // (re)create the pipeline if it doesn't exist or we have switched to a new scene
        if !storage.has::<Pipeline>() || self.new_scene {
            log::info!("Creating pipeline!");
            let pipeline = Pipeline::new(
                device,
                &format,
                queue,
                &self.scan.projection_images,
                (500,500,256), // TODO: don't have constant dimensions here
                &self.projections,
            );

            storage.store(pipeline);
        }

        let pipeline = storage.get::<Pipeline>().unwrap();

        pipeline.update_camera(queue, &self.camera_uniform);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        storage.get::<Pipeline>().unwrap().render(target, encoder, clip_bounds);
    }
}

pub struct Scene {
    scan: Arc<CtScan>,
    projections: Arc<[Projection]>,
    inclination: f32,
    threshold: f32,
    new_scene: RwLock<bool>
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
            new_scene: RwLock::from(true),
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
        let mut new_scene = self.new_scene.write().unwrap();

        let primitive = Primitive::new(
            self.scan.clone(),
            self.projections.clone(),
            self.inclination,
            self.threshold,
            *new_scene,
        );

        *new_scene = false;

        primitive
    }
}
