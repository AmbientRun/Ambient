
struct Brush {
    center: vec2<f32>,
    radius: f32,
    shape: i32,
    smoothness: f32,
    amplitude: f32,
    _padding: vec2<u32>,
};

// When smoothness = 0: |^^^^^^ (i.e it's basically outputing 1 everywhere)
// When smoothness = 1: A normal smoothstep function
fn smoothstep_power(e0: f32, e1: f32, x: f32, smoothness: f32) -> f32 {
    let r = 6.643856189774724; // Math.log(10/1000) / Math.log(0.5);
    let z = 1. + pow(1. - smoothness, r) * 1000.;
    return 1. - pow(1. - smoothstep(e0, e1, x), z);
}

fn get_brush_distance_from_edge(brush: Brush, world_position: vec2<f32>) -> f32 {
    if (brush.shape == 0) { // circle
        let d = length(brush.center - world_position);
        return min(d, brush.radius);
    } else { // rectangle
        let d = abs(world_position - brush.center);
        let t = max(d.x, d.y);
        return min(t, brush.radius);
    }
}

fn get_brush_strength(brush: Brush, world_position: vec2<f32>) -> f32 {
    let d = get_brush_distance_from_edge(brush, world_position);
    return smoothstep_power(brush.radius, 0., d, brush.smoothness) * brush.amplitude;
}
