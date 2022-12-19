struct GpuIn {
    data: array<vec4<f32>>,
};

struct GpuOut {
    data: array<vec2<f32>>,
};

@group(0)
@binding(0)
var<storage, read>  gpu_in_buffer: GpuIn;
@group(0)
@binding(1)
var<storage, read_write> gpu_out_buffer: GpuOut;

fn run(input: vec4<f32>) -> vec2<f32> {
    return (input * 3.).xy;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    gpu_out_buffer.data[global_id.x] = run(gpu_in_buffer.data[global_id.x]);
}