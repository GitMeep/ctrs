struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) ray_origin: vec3<f32>,
};

struct CameraUniform {
    bases: mat3x3<f32>,
    sampling_interval: f32,
    sample_radius: f32,
    sample_height: f32,
    threshold: f32,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) index : u32
) -> VertexOutput {
    var vertices = array(
        vec2f(-1.0, -1.0),
        vec2f(-1.0,  3.0),
        vec2f( 3.0, -1.0),
    );

    let xy = vertices[index];

    let pos = vec4f(xy, 0.0, 1.0);
    let ray_origin = mat2x3(camera.bases[1], camera.bases[0]) * xy;

    return VertexOutput(pos, ray_origin);
}

// -----------------------
// Fragment shader:
// -----------------------

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

    for (var i: u32 = 0; i < n_projections; i++) {
        let point_proj = project_point(point_world, i);
        if point_proj.z < 0 {
            continue;
        }

        let point_texture = projection_to_texture(point_proj.xy, i);

        if (point_texture.x >= 0. && point_texture.x <= 1. &&
            point_texture.y >= 0. && point_texture.y <= 1.)
        {
            let dist = point_proj.z - camera.sample_radius;
            sample_value += textureSample(
                projection_textures,
                projections_sampler,
                point_texture,
                i,
            ).x;
        }
    }

    return sample_value/f32(n_projections);
}

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.,0.,0.), vec3(1.,1.,1.)), c.y);
}

fn opacity_map(sample: f32) -> f32 {
    return pow(sample,10.);
}

fn colormap(sample: f32) -> vec3<f32> {
    let low_color = vec3(1., 0.79, 1.);
    let high_color = vec3(0.56, 0.79, 1.);
    return mix(low_color, high_color, sample);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let n_projections: u32 = arrayLength(&projections);

    let sr_sq = pow(camera.sample_radius, 2.);
    let half_height = camera.sample_height/2.;

    let ray_direction = camera.bases[2];
    let step = ray_direction*camera.sampling_interval;

    var dist: f32 = 0;
    var accu: f32 = 1;
    var sample_pos = -ray_direction * vec3(camera.sample_radius) + in.ray_origin;
    while (dist < camera.sample_radius*2.) {
        if (dot(sample_pos.xy, sample_pos.xy) < sr_sq && abs(sample_pos.z) < half_height) {
            let sample = sample_volume(sample_pos, n_projections);
            if sample > camera.threshold {
                accu -= accu * sample;
            }
        }
        dist += camera.sampling_interval;
        sample_pos += step;
    }

    if accu == 1. {
        return vec4(0., 0., 0., 1.);
    }

    return vec4(hsv2rgb(colormap(1.-accu)), 1.);
}
