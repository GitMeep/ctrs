mod pipeline;
mod primitive;

use std::{cell::RefCell, f32::consts::PI, sync::Arc};

use iced::{Event, Point, Rectangle, Vector, mouse::{self, Button}, widget::{Action, shader::{self}}};
use pipeline::uniforms::Projection;

use crate::ctrs::scene::primitive::Primitive;

use super::scan::CtScan;

pub struct Scene {
    name: String,
    projections_texture: Arc<[f32]>,
    projections: Arc<[Projection]>,
    threshold: f32,
    rerender: RefCell<bool>,
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

        let transformed: Vec<f32> = scan.projection_images.iter()
            .flat_map(|i| i.iter().map(|s| -s.log2()))
            .collect();

        let max = transformed.iter().reduce(|a,b| if a > b { a } else { b }).cloned().unwrap_or(1.);
        
        let projections_texture = transformed.iter().map(|s| s/max).collect();

        Self {
            name: scan.name.clone(),
            projections_texture,
            projections,
            threshold,
            rerender: RefCell::from(true),
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn set_rerender(&mut self) {
        *self.rerender.get_mut() = true;
    }

    pub fn try_set_rerender(&self) {
        match self.rerender.try_borrow_mut() {
            Ok(mut val) => {
                *val = true;
            }
            Err(e) => {
                log::warn!("Couldn't borrow rerender value! Error was: {e}");
            }
        };
    }
}

pub struct SceneState {
    pub pixel_size: f32,
    pub dragging: bool,
    pub drag_last: Point,
    pub azimuth: f32,
    pub inclination: f32,
    pub sampling_interval: f32,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            pixel_size: 0.1,
            dragging: false,
            drag_last: Point::new(0.,0.),
            azimuth: 0.,
            inclination: 0.,
            sampling_interval: 2.,
        }
    }
}

impl<Message> shader::Program<Message> for Scene {
    type State = SceneState;
    type Primitive = Box<Primitive>;

    fn draw(
        &self,
        state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: iced::Rectangle,
    ) -> Self::Primitive {
        let rerender = match self.rerender.try_borrow_mut() {
            Ok(mut val) => {
                let value = *val;
                *val = false; // don't re-render next time
                value
            }
            Err(e) => {
                log::warn!("Couldn't borrow rerender value! Will rerender. Error was: {e}");
                true
            }
        };

        Box::new(Primitive::new(
            self.name.clone(),
            self.projections.clone(),
            self.projections_texture.clone(),
            (500, 500),
            state.azimuth,
            state.inclination,
            self.threshold,
            (bounds.width*state.pixel_size, bounds.height*state.pixel_size),
            state.sampling_interval,
            30.,
            50.,
            rerender
        ))
    }
    
    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::default()
        }
    }
    
    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        let in_bounds = cursor.is_over(bounds);

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if state.dragging {
                        let delta: Vector = state.drag_last - *position;
                        state.drag_last = *position;

                        state.azimuth += delta.x/100.;
                        state.inclination += delta.y/100.;
                        
                        self.try_set_rerender();
                        state.sampling_interval = 2.;
                        Some(Action::request_redraw().and_capture())
                    } else {
                        None
                    }
                },
                mouse::Event::ButtonPressed(button) => {
                    match button {
                        Button::Left => {
                            if let Some(pos) = cursor.position() {
                                if in_bounds {
                                    state.dragging = true;
                                    state.drag_last = pos;
                                }
                            }
                            None
                        },
                        _ => None
                    }
                    
                }
                mouse::Event::ButtonReleased(button) => {
                    match button {
                        Button::Left => {
                            if let Some(_) = cursor.position() {
                                state.dragging = false;
                                
                                if state.sampling_interval != 0.5 {
                                    state.sampling_interval = 0.5;
                                    self.try_set_rerender();
                                }

                                Some(Action::request_redraw())
                            } else {
                                None
                            }
                        },
                        _ => None
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    if !in_bounds {
                        return None;
                    }
                    
                    let exponent: f32 = match delta {
                        // TODO: we probably need different scaling factors for lines (scroll wheel) and pixel (touch pad)
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y,
                    };
                    state.pixel_size *= f32::exp(-exponent/4.);

                    self.try_set_rerender();
                    Some(Action::request_redraw().and_capture())
                },
                _ => None
            },
            _ => None
        }
    }
}
