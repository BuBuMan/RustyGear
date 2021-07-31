struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] index: u32) -> VertexOutput {
    var positions : array<vec3<f32>, 3>;
    positions[2] = vec3<f32>(1.0, -1.0, 0.0);
    positions[1] = vec3<f32>(-1.0, -1.0, 0.0);
    positions[0] = vec3<f32>(0.0, 1.0, 0.0);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(positions[index], 1.0);
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}