struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = i32(vertex_index) / 2;
    let y = i32(vertex_index) & 1;
    let tc = vec2<f32>(
        f32(x) * 2.0,
        f32(y) * 2.0
    );
    out.position = vec4<f32>(
        tc.x * 2.0 - 1.0,
        1.0 - tc.y * 2.0,
        0.0, 1.0
    );
    out.tex_coords = tc;
    return out;
}

@group(#OUTLINES_BIND_GROUP)
@binding(0)
var r_color: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(in.tex_coords * vec2<f32>(textureDimensions(r_color)));
    let center_color = textureLoad(r_color, p, 0);
    let center = center_color.a > 0.5;
    let size = 3;
    let top = textureLoad(r_color, p + vec2<i32>(0, -size), 0).a > 0.5;
    let bottom = textureLoad(r_color, p + vec2<i32>(0, size), 0).a > 0.5;
    let left = textureLoad(r_color, p + vec2<i32>(-size, 0), 0).a > 0.5;
    let right = textureLoad(r_color, p + vec2<i32>(size, 0), 0).a > 0.5;
    if (!(center && (!top || !bottom || !left || !right))) {
        discard;
    }

    return vec4<f32>(center_color.rgb, 1.);
}
