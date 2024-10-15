struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) cam_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) cam_coords: vec2<f32>,
};

@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    out.pos = vec4f(input.pos, 0.0, 1.0);
    out.cam_coords = input.cam_coords;

    return out;
}

// -----------------------------------------------------------------------

struct CameraUniform {
    position: vec3<f32>,
    bases: mat2x3<f32>,
    dimensions: vec2<f32>,
    sampling_interval: f32,
    threshold: f32,
}

struct Projection {
    translate: vec3<f32>,
    transform: mat3x3<f32>,

    texture_transform: mat3x2<f32>,
    sdd: f32, // Source to Detector Distance
}

@group(0) @binding(0)
var projection_textures: texture_2d_array<f32>;

@group(0) @binding(1)
var projections_sampler: sampler; 

@group(0) @binding(2)
var<storage, read> projections: array<Projection>;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// project point in world onto a projection plane as defined by an
// index in the projections array
fn project_point(point_world: vec3<f32>, index: u32) -> vec3<f32> {
    let projection = projections[index];

    let transformed = projection.transform * (point_world + projection.translate);

    // The x and z coordinates of the transformed point corresponds
    // to the projection plane x and y coordinates. The y coordinate
    // of the transformed point is the depth, which is used along with
    // the source-detector-distance to apply perspective.
    let projected = transformed.xz * projection.sdd / (projection.sdd - transformed.y);

    return vec3(projected, transformed.y);
}

// TODO: support non-square textures
fn projection_to_texture(point_proj: vec2<f32>, index: u32) -> vec2<f32> {
    let projection = projections[index];

    return (projection.texture_transform * vec3(point_proj, 1.)).xy;
}

fn sample_volume(point_world: vec3<f32>, n_projections: u32) -> f32 {
    var sample_value: f32 = 0.;
    var hits: u32 = 0;
    for (var i: u32 = 0; i < n_projections; i++) {
        let point_proj = project_point(point_world, i);
        if point_proj.z > 0 {
            let point_texture = projection_to_texture(point_proj.xy, i);

            if (point_texture.x >= -1. & point_texture.x <= 1. &
                point_texture.y >= -1. & point_texture.y <= 1.)
            {
                sample_value += textureSample(
                    projection_textures,
                    projections_sampler,
                    point_texture,
                    i,
                ).x;
                hits++;
            } else {
                sample_value += 0.;
            }
        }
    }

    if hits == n_projections {
        return sample_value/f32(n_projections);
    }

    return 0.;
    //return sample_value/f32(n_projections);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n_projections: u32 = arrayLength(&projections);
    var pixel_value: f32 = 0.;

    let pixel = in.cam_coords * camera.dimensions/2.;
    var sample_pos = camera.position + camera.bases * pixel;
    let ray_direction = cross(camera.bases[1], camera.bases[0]);

    var dist: f32 = 0;
    var n_samples: f32 = 0;
    while (dot(sample_pos, sample_pos) < pow(50., 2.)) {
        if (dot(sample_pos, sample_pos) < pow(30., 2.)) {
            let sample = sample_volume(sample_pos, n_projections);
            if sample > camera.threshold {
                pixel_value += sample;
            }
            n_samples += 1.;
        }
        sample_pos += ray_direction*camera.sampling_interval;
    }

    pixel_value /= n_samples;
    return vec4(pixel_value, pixel_value, pixel_value, 1.0);
}
