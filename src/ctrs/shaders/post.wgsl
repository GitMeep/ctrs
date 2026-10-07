struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index : u32,
) -> VertexOutput {
    var vertices = array(
        vec2f(-1.0, -1.0),
        vec2f(-1.0,  3.0),
        vec2f( 3.0, -1.0),
    );

    let xy = vertices[index];
    let pos = vec4f(xy, 0.0, 1.0);
    let tex_coords = xy * vec2f(0.5, -0.5) + vec2f(0.5);
    return VertexOutput(pos, tex_coords);
}

// -----------------------
// Fragment shader:
// -----------------------

@group(0) @binding(0)
var render_texture: texture_2d<f32>;

@group(0) @binding(1)
var post_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let color = textureSample(render_texture, post_sampler, in.texcoord);
    return vec4f(color);
}