struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) instance_pos: vec2<f32>,
    @location(1) instance_color: vec4<f32>,
) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    var out: VertexOutput;
    let scale = 5.0; // particle size
    let pos = positions[vi] * scale + instance_pos;
    out.clip_position = vec4<f32>((pos / vec2<f32>(400.0, 300.0)) - vec2<f32>(1.0, 1.0), 0.0, 1.0);
    out.clip_position.y = -out.clip_position.y; // flip y
    out.color = instance_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
