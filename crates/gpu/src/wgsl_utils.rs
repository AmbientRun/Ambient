pub fn wgsl_interpolate() -> String {
    fn interpolate(name: &str, xt: &str, yt: &str) -> String {
        format!(
            "fn interpolate_{name}(x: {xt}, x0: {xt}, x1: {xt}, y0: {yt}, y1: {yt}) -> {yt} {{
            let p = (x - x0) / (x1 - x0);
            return mix(y0, y1, {yt}(p));
          }}"
        )
    }
    fn interpolate_clamped(name: &str, xt: &str, yt: &str) -> String {
        format!(
            "fn interpolate_clamped_{name}(x: {xt}, x0: {xt}, x1: {xt}, y0: {yt}, y1: {yt}) -> {yt} {{
            let p = clamp((x - x0) / (x1 - x0), {xt}(0.), {xt}(1.));
            return mix(y0, y1, {yt}(p));
          }}"
        )
    }
    [
        interpolate("1_1", "f32", "f32"),
        interpolate("1_3", "f32", "vec3<f32>"),
        interpolate("2_2", "vec2<f32>", "vec2<f32>"),
        interpolate("3_3", "vec3<f32>", "vec3<f32>"),
        interpolate("4_4", "vec4<f32>", "vec4<f32>"),
        interpolate_clamped("1_1", "f32", "f32"),
        interpolate_clamped("1_2", "f32", "vec2<f32>"),
        interpolate_clamped("1_3", "f32", "vec3<f32>"),
        interpolate_clamped("2_2", "vec2<f32>", "vec2<f32>"),
        interpolate_clamped("3_3", "vec3<f32>", "vec3<f32>"),
        interpolate_clamped("4_4", "vec4<f32>", "vec4<f32>"),
    ]
    .join("\n")
}
