use std::f32::consts::PI;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub bases: [[f32; 4]; 3], // 3x3 matrix. The 4 is for alignment with WGSL
    pub sampling_interval: f32,
    pub sample_radius: f32,
    pub sample_height: f32,
    pub threshold: f32,
}

impl Camera {
    pub fn new(
        azimuth: f32,
        inclination: f32,
        dimensions: (f32,f32),
        sampling_interval: f32,
        sample_radius: f32,
        sample_height: f32,
        threshold: f32
    ) -> Self {
        let bases = [
            // camera x-basis vector (out)
            [
                dimensions.1*inclination.sin()*azimuth.cos()/2.,
                dimensions.1*inclination.sin()*azimuth.sin()/2.,
                dimensions.1*inclination.cos()/2.,
                0.,
            ],
            // camera y-basis vector (up)
            [
                -dimensions.0*azimuth.sin()/2.,
                dimensions.0*azimuth.cos()/2.,
                0.,
                0.,
            ],
            // camera z-basis vector (out)
            [
                inclination.cos()*azimuth.cos(),
                inclination.cos()*azimuth.sin(),
                -inclination.sin(),
                0.,
            ],
        ];

        Self {
            bases,
            sampling_interval,
            sample_radius,
            sample_height,
            threshold,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Projection {
    // transform from world to sensor-plane coordinates
    pub translate: [f32; 3],
    _padding0: u32,
    pub transform: [[f32; 4]; 3], // 3x3 matrix, the 4 is for alignment with WGSL
    // transformation from projection coordinates to texture coordinates
    // this is a 3x2 matrix where the right-most column is a translation
    // to apply after the transformation (in texture space). The input should
    // be [proj.x, proj.y, 1]. The translation will be scaled by the last element.
    pub texture_transform: [[f32; 2]; 3],

    // sensor to detector distance
    pub sdd: f32,
    _padding1: u32,
}

impl Projection {
    pub fn new(world_angle: f32, sod: f32, sdd: f32, detector_dimensions: (f32, f32)) -> Self {
        
        let translate = {
            // distance from the world origin to the sensor center
            let radius = sdd - sod;

            // the vector that the world will need to be translated by to
            // move the sensor center to the world origin
            [
                -radius * world_angle.cos(),
                -radius * world_angle.sin(),
                0., // assume that sensor center and source are at same height
            ]
        };

        let transform = {
            // the size of the angle between the detector plane and the negative x-axis
            // the world will need to be rotated by this amout to make the plane face
            // towards positive y
            let detector_angle = 3.*PI/2. - world_angle;

            // this is a simple 2D rotation matrix leaving the z-coordinate unchanged
            // this will be applied after the translation in the shader.
            // WGSL is column-major.
            [
                [ detector_angle.cos(), detector_angle.sin(), 0., 0.],
                [-detector_angle.sin(), detector_angle.cos(), 0., 0.],
                [0.,                    0.,                   1., 0.],
            ]
        };

        // Matrix for taking a point on a projection plane and
        // transforming it to the texture coordinates.
        // Scales the plane down, flips the y-axis and adds 0.5
        // to each axis in texture space to align the origins.
        // WGSL is column-major.
        let texture_transform = [
            [0.5/detector_dimensions.0, 0.],
            [0., -0.5/detector_dimensions.1],
            [0.5, 0.5]
        ];

        Self {
            translate,
            transform,

            texture_transform,
            sdd,

            _padding0: 0,
            _padding1: 0,
        }
    }
}
